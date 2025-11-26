# Segurança

Este documento descreve as medidas de segurança implementadas no RoboTrade.

## Princípios de Segurança

1. **Defense in Depth** - Múltiplas camadas de proteção
2. **Least Privilege** - Mínimo de permissões necessárias
3. **Fail Secure** - Em caso de dúvida, bloquear operação
4. **No Trust** - Nunca confiar em dados externos ou do frontend
5. **Audit Trail** - Registrar todas as operações sensíveis

## Modo Paper vs Live

### Visão Geral

```
┌─────────────────────────────────────────────────────────────────┐
│                     MODO DE OPERAÇÃO                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌───────────────────┐          ┌───────────────────┐          │
│  │    PAPER MODE     │          │    LIVE MODE      │          │
│  │    (Padrão)       │          │   (Produção)      │          │
│  ├───────────────────┤          ├───────────────────┤          │
│  │ ✓ Ativado por     │          │ ✗ Requer config   │          │
│  │   padrão          │          │   explícita       │          │
│  │                   │          │                   │          │
│  │ ✓ Simula todas    │          │ ✗ Requer chaves   │          │
│  │   as ordens       │          │   reais           │          │
│  │                   │          │                   │          │
│  │ ✓ Sem risco       │          │ ✗ Confirmação     │          │
│  │   financeiro      │          │   dupla           │          │
│  │                   │          │                   │          │
│  │ ✓ Dados reais     │          │ ✗ Validação de    │          │
│  │   de mercado      │          │   limites         │          │
│  │                   │          │                   │          │
│  │ ✓ Ideal para      │          │ ✓ Operações       │          │
│  │   testes          │          │   reais           │          │
│  └───────────────────┘          └───────────────────┘          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Paper Mode (Padrão)

O modo paper é ativado por padrão e simula todas as operações:

```toml
# config.toml
[trading]
mode = "paper"  # paper | live
```

```rust
pub enum TradingMode {
    Paper,
    Live,
}

impl Default for TradingMode {
    fn default() -> Self {
        TradingMode::Paper
    }
}
```

### Ativando Live Mode

Para ativar o modo live, são necessárias múltiplas verificações:

```rust
pub fn validate_live_mode_activation(config: &AppConfig) -> Result<(), SecurityError> {
    // 1. Verificar flag de configuração explícita
    if config.trading.mode != TradingMode::Live {
        return Err(SecurityError::LiveModeNotEnabled);
    }

    // 2. Verificar se existem API keys configuradas
    if !config.has_valid_api_keys() {
        return Err(SecurityError::NoApiKeysConfigured);
    }

    // 3. Verificar limites de risco configurados
    if config.risk.max_position_size.is_none() {
        return Err(SecurityError::RiskLimitsNotSet);
    }

    // 4. Verificar se não é primeira execução
    if config.is_first_run() {
        return Err(SecurityError::FirstRunNotAllowed);
    }

    // 5. Verificar ambiente de produção
    if config.general.environment != Environment::Production {
        tracing::warn!("Live mode enabled in non-production environment");
    }

    Ok(())
}
```

### Confirmação Dupla

Antes de executar ordens em live mode:

```rust
pub struct OrderConfirmation {
    pub order_request: OrderRequest,
    pub confirmation_code: String,
    pub confirmed_at: DateTime<Utc>,
    pub ip_address: Option<String>,
}

impl TradingWorker {
    pub async fn place_order_live(
        &self,
        request: OrderRequest,
        confirmation: OrderConfirmation,
    ) -> WorkerResult<Order> {
        // Verificar código de confirmação
        if !self.verify_confirmation(&confirmation) {
            return Err(WorkerError::ConfirmationFailed);
        }

        // Verificar se a ordem é a mesma que foi confirmada
        if confirmation.order_request != request {
            return Err(WorkerError::OrderMismatch);
        }

        // Verificar tempo de expiração da confirmação (5 minutos)
        if Utc::now() - confirmation.confirmed_at > Duration::minutes(5) {
            return Err(WorkerError::ConfirmationExpired);
        }

        // Executar ordem
        self.gateway.submit_order(request).await
    }
}
```

## Armazenamento de API Keys

### Keyring do Sistema

API keys são armazenadas no keyring nativo do sistema operacional:

```rust
use keyring::Entry;

pub struct SecureKeyStorage {
    service_name: String,
}

impl SecureKeyStorage {
    pub fn new() -> Self {
        Self {
            service_name: "robotrade".to_string(),
        }
    }

    pub fn store_api_key(
        &self,
        exchange: &str,
        key_type: &str,  // "api_key" ou "secret_key"
        value: &str,
    ) -> InfraResult<()> {
        let entry = Entry::new(&self.service_name, &format!("{}_{}", exchange, key_type))
            .map_err(|e| InfraError::KeyringError(e.to_string()))?;

        entry
            .set_password(value)
            .map_err(|e| InfraError::KeyringError(e.to_string()))?;

        tracing::info!(
            exchange = exchange,
            key_type = key_type,
            "API key stored securely"
        );

        Ok(())
    }

    pub fn get_api_key(
        &self,
        exchange: &str,
        key_type: &str,
    ) -> InfraResult<Option<String>> {
        let entry = Entry::new(&self.service_name, &format!("{}_{}", exchange, key_type))
            .map_err(|e| InfraError::KeyringError(e.to_string()))?;

        match entry.get_password() {
            Ok(password) => Ok(Some(password)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(InfraError::KeyringError(e.to_string())),
        }
    }

    pub fn delete_api_key(
        &self,
        exchange: &str,
        key_type: &str,
    ) -> InfraResult<()> {
        let entry = Entry::new(&self.service_name, &format!("{}_{}", exchange, key_type))
            .map_err(|e| InfraError::KeyringError(e.to_string()))?;

        entry
            .delete_password()
            .map_err(|e| InfraError::KeyringError(e.to_string()))?;

        Ok(())
    }
}
```

### Nunca Armazenar em Arquivo

```rust
// ❌ NUNCA fazer isso
pub struct BadConfig {
    api_key: String,      // Nunca em config.toml
    secret_key: String,   // Nunca em .env
}

// ✅ Correto: usar referência para keyring
pub struct ExchangeConfig {
    pub enabled: bool,
    pub testnet: bool,
    pub api_key_id: String,  // ID para buscar no keyring
}
```

### Proteção de Memória

```rust
use secrecy::{ExposeSecret, Secret};

pub struct ApiCredentials {
    pub api_key: Secret<String>,
    pub secret_key: Secret<String>,
}

impl ApiCredentials {
    pub fn new(api_key: String, secret_key: String) -> Self {
        Self {
            api_key: Secret::new(api_key),
            secret_key: Secret::new(secret_key),
        }
    }

    // Usar apenas quando necessário
    pub fn sign_request(&self, message: &str) -> String {
        let secret = self.secret_key.expose_secret();
        // ... assinar request
    }
}

// Secret é automaticamente zerado da memória quando dropado
impl Drop for ApiCredentials {
    fn drop(&mut self) {
        tracing::debug!("Credentials securely dropped");
    }
}
```

## Validação de Entrada

### Validação Server-Side

Toda entrada é validada no backend, independente do frontend:

```rust
pub fn validate_order_request(request: &OrderRequest) -> SecurityResult<()> {
    // 1. Validar símbolo
    if !is_valid_symbol(&request.symbol) {
        return Err(SecurityError::InvalidSymbol(request.symbol.clone()));
    }

    // 2. Validar quantidade
    if request.quantity <= Decimal::ZERO {
        return Err(SecurityError::InvalidQuantity("must be positive"));
    }

    if request.quantity > MAX_ORDER_QUANTITY {
        return Err(SecurityError::InvalidQuantity("exceeds maximum"));
    }

    // 3. Validar preço (se presente)
    if let Some(price) = &request.price {
        if *price <= Decimal::ZERO {
            return Err(SecurityError::InvalidPrice("must be positive"));
        }
    }

    // 4. Validar stop loss (obrigatório para live)
    if !request.stop_loss.is_some() && is_live_mode() {
        return Err(SecurityError::StopLossRequired);
    }

    Ok(())
}
```

### Sanitização de Strings

```rust
pub fn sanitize_symbol(input: &str) -> Option<String> {
    let sanitized: String = input
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(20)
        .collect();

    if sanitized.is_empty() {
        None
    } else {
        Some(sanitized.to_uppercase())
    }
}

pub fn sanitize_user_input(input: &str) -> String {
    input
        .chars()
        .filter(|c| !c.is_control())
        .take(1000)
        .collect()
}
```

## Limites de Risco

### Configuração de Limites

```toml
# config.toml
[risk]
# Tamanho máximo de posição (em quote currency)
max_position_size = "10000.00"

# Percentual máximo do capital por trade
max_risk_per_trade = "0.02"  # 2%

# Perda máxima diária permitida
max_daily_loss = "500.00"

# Número máximo de ordens por minuto
max_orders_per_minute = 10

# Número máximo de posições abertas simultaneamente
max_open_positions = 5

# Alavancagem máxima permitida
max_leverage = 10

# Stop loss obrigatório
require_stop_loss = true

# Distância mínima do stop loss (%)
min_stop_loss_distance = "0.5"
```

### Enforcement de Limites

```rust
pub struct RiskManager {
    config: RiskConfig,
    daily_pnl: AtomicDecimal,
    orders_this_minute: AtomicU32,
}

impl RiskManager {
    pub fn validate_order(&self, request: &OrderRequest, portfolio: &Portfolio) -> SecurityResult<()> {
        // 1. Verificar tamanho máximo de posição
        let position_value = request.quantity * request.price.unwrap_or_default();
        if position_value > self.config.max_position_size {
            return Err(SecurityError::PositionSizeExceeded {
                requested: position_value,
                max: self.config.max_position_size,
            });
        }

        // 2. Verificar risco por trade
        let risk_amount = self.calculate_risk_amount(request);
        let max_risk = portfolio.total_equity * self.config.max_risk_per_trade;
        if risk_amount > max_risk {
            return Err(SecurityError::RiskPerTradeExceeded {
                risk: risk_amount,
                max: max_risk,
            });
        }

        // 3. Verificar perda diária
        if self.daily_pnl.load() < -self.config.max_daily_loss {
            return Err(SecurityError::DailyLossLimitReached);
        }

        // 4. Verificar rate limit
        if self.orders_this_minute.load(Ordering::Relaxed) >= self.config.max_orders_per_minute {
            return Err(SecurityError::OrderRateLimitExceeded);
        }

        // 5. Verificar número de posições
        if portfolio.open_positions.len() >= self.config.max_open_positions as usize {
            return Err(SecurityError::MaxPositionsReached);
        }

        // 6. Verificar stop loss obrigatório
        if self.config.require_stop_loss && request.stop_loss.is_none() {
            return Err(SecurityError::StopLossRequired);
        }

        Ok(())
    }

    fn calculate_risk_amount(&self, request: &OrderRequest) -> Decimal {
        match request.stop_loss {
            Some(sl) => {
                let entry = request.price.unwrap_or_default();
                let distance = (entry - sl).abs();
                distance * request.quantity
            }
            None => request.quantity * request.price.unwrap_or_default(),
        }
    }
}
```

### Circuit Breakers

```rust
pub struct CircuitBreaker {
    consecutive_losses: AtomicU32,
    max_consecutive_losses: u32,
    cooldown_until: RwLock<Option<DateTime<Utc>>>,
}

impl CircuitBreaker {
    pub fn record_trade_result(&self, pnl: Decimal) {
        if pnl < Decimal::ZERO {
            let losses = self.consecutive_losses.fetch_add(1, Ordering::SeqCst) + 1;

            if losses >= self.max_consecutive_losses {
                self.trigger_cooldown(Duration::hours(1));
            }
        } else {
            self.consecutive_losses.store(0, Ordering::SeqCst);
        }
    }

    pub fn is_trading_allowed(&self) -> bool {
        let cooldown = self.cooldown_until.read();
        match *cooldown {
            Some(until) => Utc::now() > until,
            None => true,
        }
    }

    fn trigger_cooldown(&self, duration: Duration) {
        let mut cooldown = self.cooldown_until.write();
        *cooldown = Some(Utc::now() + duration);

        tracing::warn!(
            consecutive_losses = self.consecutive_losses.load(Ordering::SeqCst),
            cooldown_duration = ?duration,
            "Circuit breaker triggered"
        );
    }
}
```

## Audit Log

### Registro de Operações Sensíveis

```rust
pub struct AuditLogger {
    db: DbPool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditEvent {
    pub event_type: AuditEventType,
    pub user_action: bool,
    pub details: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum AuditEventType {
    // Autenticação
    ApiKeyAdded { exchange: String },
    ApiKeyRemoved { exchange: String },

    // Configuração
    ConfigChanged { key: String, old_value: String, new_value: String },
    TradingModeChanged { from: TradingMode, to: TradingMode },

    // Trading
    OrderSubmitted { order_id: String, symbol: String, side: String, quantity: String },
    OrderCancelled { order_id: String },
    PositionOpened { position_id: String, symbol: String },
    PositionClosed { position_id: String, pnl: String },

    // Risk
    RiskLimitTriggered { limit_type: String, value: String },
    CircuitBreakerTriggered { reason: String },

    // Sistema
    ApplicationStarted,
    ApplicationShutdown,
    WorkerStarted,
    WorkerStopped,
}

impl AuditLogger {
    pub async fn log(&self, event: AuditEvent) -> InfraResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO audit_log (event_type, user_action, details, timestamp, ip_address)
            VALUES (?, ?, ?, ?, ?)
            "#,
            event.event_type.to_string(),
            event.user_action,
            serde_json::to_string(&event.details)?,
            event.timestamp.to_rfc3339(),
            event.ip_address,
        )
        .execute(&self.db)
        .await?;

        // Log também para arquivo
        tracing::info!(
            event_type = ?event.event_type,
            user_action = event.user_action,
            "Audit event recorded"
        );

        Ok(())
    }
}
```

## Proteção de Comunicação

### HTTPS/TLS para APIs Externas

```rust
impl BinanceFuturesClient {
    pub fn new(credentials: ApiCredentials, testnet: bool) -> Self {
        let client = reqwest::Client::builder()
            .use_rustls_tls()           // Usar TLS
            .min_tls_version(reqwest::tls::Version::TLS_1_2)
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        let base_url = if testnet {
            "https://testnet.binancefuture.com"
        } else {
            "https://fapi.binance.com"
        };

        Self {
            client,
            base_url: base_url.to_string(),
            credentials,
            testnet,
        }
    }
}
```

### Validação de Assinaturas

```rust
impl BinanceSigner {
    pub fn sign_request(&self, params: &mut Vec<(String, String)>) {
        // Adicionar timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();
        params.push(("timestamp".to_string(), timestamp));

        // Criar query string
        let query: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        // Assinar com HMAC-SHA256
        let mut mac = Hmac::<Sha256>::new_from_slice(
            self.credentials.secret_key.expose_secret().as_bytes()
        ).expect("HMAC accepts any key size");
        mac.update(query.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        params.push(("signature".to_string(), signature));
    }

    pub fn verify_webhook_signature(
        &self,
        payload: &[u8],
        signature: &str,
    ) -> bool {
        let mut mac = Hmac::<Sha256>::new_from_slice(
            self.credentials.secret_key.expose_secret().as_bytes()
        ).expect("HMAC accepts any key size");
        mac.update(payload);

        let expected = hex::encode(mac.finalize().into_bytes());
        signature == expected
    }
}
```

## Checklist de Segurança

### Antes de Ativar Live Mode

- [ ] API keys armazenadas no keyring (não em arquivos)
- [ ] Limites de risco configurados
- [ ] Stop loss obrigatório ativado
- [ ] Rate limits configurados
- [ ] Circuit breakers ativos
- [ ] Modo testnet testado exaustivamente
- [ ] Backtests realizados
- [ ] Audit log funcionando
- [ ] Alertas configurados

### Review Periódico

- [ ] Verificar logs de audit
- [ ] Revisar limites de risco
- [ ] Atualizar API keys se necessário
- [ ] Verificar métricas de erro
- [ ] Testar circuit breakers
- [ ] Backup de configurações

---

**Próximo**: [Sistema de Configuração](./config-system.md)
