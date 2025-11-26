# ADR-004: Versionamento de Estratégias

## Status

Aceita

## Contexto

Estratégias de trading no RoboTrade evoluem ao longo do tempo:

1. **Parâmetros mudam**: Períodos de médias móveis, thresholds de sinais
2. **Lógica evolui**: Novos indicadores, condições de entrada/saída
3. **Backtests precisam ser reproduzíveis**: Comparar performance entre versões
4. **Auditoria é necessária**: Saber exatamente qual versão gerou cada ordem
5. **Rollback pode ser necessário**: Voltar para versão anterior se nova performar mal

Requisitos:

- Rastrear histórico completo de mudanças
- Associar ordens à versão específica da estratégia
- Permitir comparação de performance entre versões
- Não perder configurações antigas
- Suportar múltiplas versões ativas (A/B testing)

## Decisão

Implementamos um **sistema de versionamento semântico** para estratégias, inspirado em Git mas simplificado para nosso caso de uso.

### Modelo de Dados

```rust
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyVersion {
    pub id: Uuid,
    pub strategy_id: Uuid,
    pub version: SemanticVersion,
    pub config: StrategyConfig,
    pub created_at: DateTime<Utc>,
    pub created_by: String,  // "user" ou "backtest_optimizer"
    pub parent_version: Option<Uuid>,
    pub change_description: String,
    pub is_active: bool,
    pub performance_metrics: Option<PerformanceSnapshot>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SemanticVersion {
    pub major: u32,  // Breaking changes na lógica
    pub minor: u32,  // Novos indicadores/features
    pub patch: u32,  // Ajustes de parâmetros
}

impl std::fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub name: String,
    pub symbols: Vec<String>,
    pub timeframe: String,
    pub indicators: Vec<IndicatorConfig>,
    pub entry_rules: Vec<Rule>,
    pub exit_rules: Vec<Rule>,
    pub risk_params: RiskParameters,
    pub metadata: serde_json::Value,  // Extensível para configs específicas
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub total_return: Decimal,
    pub sharpe_ratio: Decimal,
    pub max_drawdown: Decimal,
    pub win_rate: Decimal,
    pub profit_factor: Decimal,
    pub total_trades: u32,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}
```

### Schema do Banco

```sql
CREATE TABLE strategy_versions (
    id TEXT PRIMARY KEY,
    strategy_id TEXT NOT NULL,
    version_major INTEGER NOT NULL,
    version_minor INTEGER NOT NULL,
    version_patch INTEGER NOT NULL,
    config_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    created_by TEXT NOT NULL,
    parent_version_id TEXT REFERENCES strategy_versions(id),
    change_description TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 0,
    performance_json TEXT,

    UNIQUE(strategy_id, version_major, version_minor, version_patch)
);

CREATE INDEX idx_strategy_versions_active
ON strategy_versions(strategy_id, is_active)
WHERE is_active = 1;

-- Associação ordem -> versão da estratégia
CREATE TABLE orders (
    id TEXT PRIMARY KEY,
    strategy_version_id TEXT NOT NULL REFERENCES strategy_versions(id),
    -- ... outros campos
);
```

### API de Versionamento

```rust
pub struct StrategyVersionManager {
    db: SqlitePool,
}

impl StrategyVersionManager {
    /// Cria nova versão a partir da atual
    pub async fn create_version(
        &self,
        strategy_id: Uuid,
        new_config: StrategyConfig,
        change_description: &str,
        version_bump: VersionBump,
    ) -> Result<StrategyVersion, Error> {
        // Buscar versão atual
        let current = self.get_active_version(strategy_id).await?;

        // Calcular nova versão
        let new_version = match version_bump {
            VersionBump::Major => SemanticVersion {
                major: current.version.major + 1,
                minor: 0,
                patch: 0,
            },
            VersionBump::Minor => SemanticVersion {
                major: current.version.major,
                minor: current.version.minor + 1,
                patch: 0,
            },
            VersionBump::Patch => SemanticVersion {
                major: current.version.major,
                minor: current.version.minor,
                patch: current.version.patch + 1,
            },
        };

        let version = StrategyVersion {
            id: Uuid::new_v4(),
            strategy_id,
            version: new_version,
            config: new_config,
            created_at: Utc::now(),
            created_by: "user".to_string(),
            parent_version: Some(current.id),
            change_description: change_description.to_string(),
            is_active: false,  // Não ativa automaticamente
            performance_metrics: None,
        };

        self.persist_version(&version).await?;

        Ok(version)
    }

    /// Ativa uma versão específica (desativa outras)
    pub async fn activate_version(
        &self,
        strategy_id: Uuid,
        version_id: Uuid,
    ) -> Result<(), Error> {
        let mut tx = self.db.begin().await?;

        // Desativar todas as versões da estratégia
        sqlx::query!(
            "UPDATE strategy_versions SET is_active = 0 WHERE strategy_id = ?",
            strategy_id
        )
        .execute(&mut *tx)
        .await?;

        // Ativar a versão específica
        sqlx::query!(
            "UPDATE strategy_versions SET is_active = 1 WHERE id = ?",
            version_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        tracing::info!(
            strategy_id = %strategy_id,
            version_id = %version_id,
            "Strategy version activated"
        );

        Ok(())
    }

    /// Compara performance entre versões
    pub async fn compare_versions(
        &self,
        version_a: Uuid,
        version_b: Uuid,
    ) -> Result<VersionComparison, Error> {
        let a = self.get_version(version_a).await?;
        let b = self.get_version(version_b).await?;

        let metrics_a = a.performance_metrics
            .ok_or(Error::NoMetrics(version_a))?;
        let metrics_b = b.performance_metrics
            .ok_or(Error::NoMetrics(version_b))?;

        Ok(VersionComparison {
            version_a: a.version,
            version_b: b.version,
            return_diff: metrics_b.total_return - metrics_a.total_return,
            sharpe_diff: metrics_b.sharpe_ratio - metrics_a.sharpe_ratio,
            drawdown_diff: metrics_b.max_drawdown - metrics_a.max_drawdown,
            win_rate_diff: metrics_b.win_rate - metrics_a.win_rate,
        })
    }

    /// Lista histórico de versões
    pub async fn get_version_history(
        &self,
        strategy_id: Uuid,
    ) -> Result<Vec<StrategyVersion>, Error> {
        sqlx::query_as!(
            StrategyVersion,
            r#"
            SELECT * FROM strategy_versions
            WHERE strategy_id = ?
            ORDER BY version_major DESC, version_minor DESC, version_patch DESC
            "#,
            strategy_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(Into::into)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VersionBump {
    Major,  // Mudança fundamental na lógica
    Minor,  // Novo indicador ou regra
    Patch,  // Ajuste de parâmetro
}
```

### Integração com Execução

```rust
impl TradingWorker {
    async fn execute_strategy(&self, strategy_id: Uuid) -> Result<(), Error> {
        // Sempre usa a versão ativa
        let version = self.version_manager
            .get_active_version(strategy_id)
            .await?;

        let signal = self.evaluate_strategy(&version.config).await?;

        if let Some(order) = signal.to_order() {
            // Ordem referencia a versão específica
            let order = Order {
                strategy_version_id: version.id,
                ..order
            };

            self.place_order(order).await?;
        }

        Ok(())
    }
}
```

## Consequências

### Positivas

- **Rastreabilidade completa**: Toda ordem associada à versão exata
- **Reprodutibilidade**: Backtests podem usar qualquer versão histórica
- **Rollback seguro**: Voltar para versão anterior é trivial
- **A/B testing**: Múltiplas versões podem rodar em paralelo (paper)
- **Auditoria**: Histórico completo de mudanças com descrições
- **Comparação**: Métricas de performance por versão facilitam decisões

### Negativas

- **Storage adicional**: Cada versão armazena config completa
- **Complexidade**: Sistema de versionamento adiciona código
- **Migração**: Mudanças no schema de StrategyConfig requerem migração

### Neutras

- Versões antigas nunca são deletadas (podem ser arquivadas)
- Performance snapshot é opcional (calculado sob demanda)

## Alternativas Consideradas

### Alternativa 1: Git para Estratégias

- **Descrição**: Armazenar configs como arquivos versionados no Git
- **Prós**: Sistema maduro, diff/merge built-in
- **Contras**: Complexo integrar com SQLite, overhead de processo externo
- **Motivo da rejeição**: Overkill, preferimos solução integrada

### Alternativa 2: Append-Only Log

- **Descrição**: Event sourcing de mudanças de configuração
- **Prós**: Histórico completo de eventos, auditoria perfeita
- **Contras**: Reconstruir estado atual é lento, complexidade
- **Motivo da rejeição**: Não precisamos de granularidade de eventos

### Alternativa 3: Sem Versionamento

- **Descrição**: Apenas manter configuração atual
- **Prós**: Simplicidade máxima
- **Contras**: Sem histórico, sem rollback, sem comparação
- **Motivo da rejeição**: Inadequado para sistema de trading sério

## Referências

- [Semantic Versioning 2.0.0](https://semver.org/)
- [Feature Flags and A/B Testing](https://martinfowler.com/articles/feature-toggles.html)
- [Event Sourcing Pattern](https://martinfowler.com/eaaDev/EventSourcing.html)
