//! Entidades de Backtesting
//!
//! Modelos para resultados de backtest.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

use super::StrategyVersionId;

/// ID único de um backtest
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BacktestId(pub Uuid);

impl BacktestId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl Default for BacktestId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for BacktestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Status do backtest
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BacktestStatus {
    /// Pendente
    Pending,
    /// Em execução
    Running,
    /// Completado
    Completed,
    /// Falhou
    Failed,
    /// Cancelado
    Cancelled,
}

impl fmt::Display for BacktestStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BacktestStatus::Pending => write!(f, "pending"),
            BacktestStatus::Running => write!(f, "running"),
            BacktestStatus::Completed => write!(f, "completed"),
            BacktestStatus::Failed => write!(f, "failed"),
            BacktestStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for BacktestStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(BacktestStatus::Pending),
            "running" => Ok(BacktestStatus::Running),
            "completed" => Ok(BacktestStatus::Completed),
            "failed" => Ok(BacktestStatus::Failed),
            "cancelled" => Ok(BacktestStatus::Cancelled),
            _ => Err(format!("Status de backtest inválido: {}", s)),
        }
    }
}

/// Modelo de preenchimento de ordens
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum FillModel {
    /// Preenche no preço de fechamento
    #[default]
    Close,
    /// Preenche no preço de abertura do próximo candle
    NextOpen,
    /// Preenche no OHLC4 (média)
    Ohlc4,
    /// Preenche no pior preço (pessimista)
    Worst,
}


impl fmt::Display for FillModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FillModel::Close => write!(f, "close"),
            FillModel::NextOpen => write!(f, "next_open"),
            FillModel::Ohlc4 => write!(f, "ohlc4"),
            FillModel::Worst => write!(f, "worst"),
        }
    }
}

/// Modelo de dimensionamento de posição
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum PositionSizing {
    /// Tamanho fixo
    #[default]
    Fixed,
    /// Percentual do capital
    PercentOfEquity,
    /// Baseado em risco fixo
    FixedRisk,
    /// Kelly Criterion
    Kelly,
}


impl fmt::Display for PositionSizing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PositionSizing::Fixed => write!(f, "fixed"),
            PositionSizing::PercentOfEquity => write!(f, "percent_of_equity"),
            PositionSizing::FixedRisk => write!(f, "fixed_risk"),
            PositionSizing::Kelly => write!(f, "kelly"),
        }
    }
}

/// Configuração de backtest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    /// Capital inicial
    pub initial_capital: Decimal,
    /// Taxa de comissão (ex: 0.001 = 0.1%)
    pub commission_rate: Decimal,
    /// Slippage em basis points
    pub slippage_bps: u32,
    /// Modelo de preenchimento
    pub fill_model: FillModel,
    /// Modelo de dimensionamento
    pub position_sizing: PositionSizing,
    /// Percentual do capital por trade (para PercentOfEquity)
    pub position_size_pct: Option<Decimal>,
    /// Risco por trade (para FixedRisk)
    pub risk_per_trade: Option<Decimal>,
    /// Alavancagem
    pub leverage: u32,
    /// Permitir shorts?
    pub allow_shorts: bool,
    /// Reinvestir lucros?
    pub reinvest_profits: bool,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: Decimal::from(10000),
            commission_rate: Decimal::new(1, 3), // 0.1%
            slippage_bps: 0,
            fill_model: FillModel::Close,
            position_sizing: PositionSizing::Fixed,
            position_size_pct: Some(Decimal::from(10)),
            risk_per_trade: None,
            leverage: 1,
            allow_shorts: true,
            reinvest_profits: true,
        }
    }
}

/// Execução de backtest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backtest {
    /// ID único
    pub id: BacktestId,
    /// ID da versão da estratégia
    pub strategy_version_id: StrategyVersionId,
    /// ID do símbolo
    pub symbol_id: i64,
    /// Timeframe
    pub timeframe: String,
    /// Data inicial
    pub start_date: NaiveDate,
    /// Data final
    pub end_date: NaiveDate,
    /// Configuração
    pub config: BacktestConfig,
    /// Status
    pub status: BacktestStatus,
    /// Progresso (0-100)
    pub progress_pct: u8,
    /// Mensagem de erro
    pub error_message: Option<String>,
    /// Início da execução
    pub started_at: Option<DateTime<Utc>>,
    /// Fim da execução
    pub completed_at: Option<DateTime<Utc>>,
    /// Tempo de execução em ms
    pub execution_time_ms: Option<i64>,
    /// Resultados
    pub results: Option<BacktestResults>,
    /// Metadados adicionais
    pub metadata: Option<serde_json::Value>,
    /// Data de criação
    pub created_at: DateTime<Utc>,
}

impl Backtest {
    /// Cria um novo backtest
    pub fn new(
        strategy_version_id: StrategyVersionId,
        symbol_id: i64,
        timeframe: impl Into<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Self {
        Self {
            id: BacktestId::new(),
            strategy_version_id,
            symbol_id,
            timeframe: timeframe.into(),
            start_date,
            end_date,
            config: BacktestConfig::default(),
            status: BacktestStatus::Pending,
            progress_pct: 0,
            error_message: None,
            started_at: None,
            completed_at: None,
            execution_time_ms: None,
            results: None,
            metadata: None,
            created_at: Utc::now(),
        }
    }

    /// Define configuração
    pub fn with_config(mut self, config: BacktestConfig) -> Self {
        self.config = config;
        self
    }

    /// Inicia execução
    pub fn start(&mut self) {
        self.status = BacktestStatus::Running;
        self.started_at = Some(Utc::now());
    }

    /// Atualiza progresso
    pub fn update_progress(&mut self, progress: u8) {
        self.progress_pct = progress.min(100);
    }

    /// Completa com sucesso
    pub fn complete(&mut self, results: BacktestResults) {
        let now = Utc::now();
        self.status = BacktestStatus::Completed;
        self.completed_at = Some(now);
        self.progress_pct = 100;
        self.results = Some(results);

        if let Some(started) = self.started_at {
            self.execution_time_ms = Some((now - started).num_milliseconds());
        }
    }

    /// Falha com erro
    pub fn fail(&mut self, error: impl Into<String>) {
        self.status = BacktestStatus::Failed;
        self.error_message = Some(error.into());
        self.completed_at = Some(Utc::now());
    }

    /// Cancela
    pub fn cancel(&mut self) {
        self.status = BacktestStatus::Cancelled;
        self.completed_at = Some(Utc::now());
    }
}

/// Resultados de backtest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResults {
    // Retornos
    /// Capital final
    pub final_capital: Decimal,
    /// Retorno total
    pub total_return: Decimal,
    /// Retorno total percentual
    pub total_return_pct: Decimal,
    /// Retorno anualizado
    pub annualized_return: Option<Decimal>,

    // Métricas de risco
    /// Sharpe Ratio
    pub sharpe_ratio: Option<Decimal>,
    /// Sortino Ratio
    pub sortino_ratio: Option<Decimal>,
    /// Max Drawdown
    pub max_drawdown: Decimal,
    /// Max Drawdown percentual
    pub max_drawdown_pct: Decimal,
    /// Calmar Ratio
    pub calmar_ratio: Option<Decimal>,
    /// Volatilidade
    pub volatility: Option<Decimal>,

    // Estatísticas de trades
    /// Total de trades
    pub total_trades: u32,
    /// Trades vencedores
    pub winning_trades: u32,
    /// Trades perdedores
    pub losing_trades: u32,
    /// Win rate
    pub win_rate: Decimal,
    /// Profit factor
    pub profit_factor: Option<Decimal>,
    /// Expectancy
    pub expectancy: Option<Decimal>,

    // Médias
    /// P&L médio por trade
    pub avg_trade_pnl: Decimal,
    /// Lucro médio em trades vencedores
    pub avg_winning_trade: Decimal,
    /// Perda média em trades perdedores
    pub avg_losing_trade: Decimal,
    /// Maior lucro
    pub largest_win: Decimal,
    /// Maior perda
    pub largest_loss: Decimal,

    // Sequências
    /// Maior sequência de vitórias
    pub max_win_streak: u32,
    /// Maior sequência de derrotas
    pub max_loss_streak: u32,

    // Dados detalhados
    /// Curva de equity (JSON array de pontos)
    pub equity_curve: Option<serde_json::Value>,
    /// Retornos mensais (JSON)
    pub monthly_returns: Option<serde_json::Value>,
    /// Lista de trades (JSON)
    pub trades_json: Option<serde_json::Value>,
}

impl Default for BacktestResults {
    fn default() -> Self {
        Self {
            final_capital: Decimal::ZERO,
            total_return: Decimal::ZERO,
            total_return_pct: Decimal::ZERO,
            annualized_return: None,
            sharpe_ratio: None,
            sortino_ratio: None,
            max_drawdown: Decimal::ZERO,
            max_drawdown_pct: Decimal::ZERO,
            calmar_ratio: None,
            volatility: None,
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            win_rate: Decimal::ZERO,
            profit_factor: None,
            expectancy: None,
            avg_trade_pnl: Decimal::ZERO,
            avg_winning_trade: Decimal::ZERO,
            avg_losing_trade: Decimal::ZERO,
            largest_win: Decimal::ZERO,
            largest_loss: Decimal::ZERO,
            max_win_streak: 0,
            max_loss_streak: 0,
            equity_curve: None,
            monthly_returns: None,
            trades_json: None,
        }
    }
}

impl BacktestResults {
    /// Verifica se o backtest foi lucrativo
    pub fn is_profitable(&self) -> bool {
        self.total_return > Decimal::ZERO
    }

    /// Calcula score geral do backtest (0-100)
    pub fn calculate_score(&self) -> u8 {
        let mut score = 0u32;

        // Retorno positivo (+20)
        if self.is_profitable() {
            score += 20;
        }

        // Win rate > 50% (+15)
        if self.win_rate > Decimal::from(50) {
            score += 15;
        }

        // Profit factor > 1.5 (+15)
        if let Some(pf) = self.profit_factor {
            if pf > Decimal::new(15, 1) {
                score += 15;
            }
        }

        // Sharpe > 1 (+15)
        if let Some(sharpe) = self.sharpe_ratio {
            if sharpe > Decimal::ONE {
                score += 15;
            }
        }

        // Max drawdown < 20% (+15)
        if self.max_drawdown_pct.abs() < Decimal::from(20) {
            score += 15;
        }

        // Mais de 30 trades (+10)
        if self.total_trades >= 30 {
            score += 10;
        }

        // Sortino > 1.5 (+10)
        if let Some(sortino) = self.sortino_ratio {
            if sortino > Decimal::new(15, 1) {
                score += 10;
            }
        }

        score.min(100) as u8
    }

    /// Retorna classificação baseada no score
    pub fn grade(&self) -> &'static str {
        match self.calculate_score() {
            90..=100 => "A+",
            80..=89 => "A",
            70..=79 => "B",
            60..=69 => "C",
            50..=59 => "D",
            _ => "F",
        }
    }
}

/// Ponto na curva de equity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Valor do equity
    pub equity: Decimal,
    /// Drawdown atual
    pub drawdown: Decimal,
    /// Drawdown percentual
    pub drawdown_pct: Decimal,
}

/// Retorno mensal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyReturn {
    /// Ano
    pub year: i32,
    /// Mês (1-12)
    pub month: u32,
    /// Retorno
    pub return_value: Decimal,
    /// Retorno percentual
    pub return_pct: Decimal,
    /// Número de trades
    pub trades: u32,
}

/// Comparação entre backtests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestComparison {
    /// IDs dos backtests comparados
    pub backtest_ids: Vec<BacktestId>,
    /// Melhor retorno
    pub best_return_id: BacktestId,
    /// Melhor Sharpe
    pub best_sharpe_id: Option<BacktestId>,
    /// Menor drawdown
    pub lowest_drawdown_id: BacktestId,
    /// Melhor win rate
    pub best_winrate_id: BacktestId,
    /// Resumo comparativo
    pub summary: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_backtest_lifecycle() {
        let mut backtest = Backtest::new(
            StrategyVersionId::new(),
            1,
            "4h",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
        );

        assert_eq!(backtest.status, BacktestStatus::Pending);

        backtest.start();
        assert_eq!(backtest.status, BacktestStatus::Running);
        assert!(backtest.started_at.is_some());

        backtest.update_progress(50);
        assert_eq!(backtest.progress_pct, 50);

        let results = BacktestResults {
            total_return: dec!(1000),
            total_return_pct: dec!(10),
            final_capital: dec!(11000),
            total_trades: 50,
            winning_trades: 30,
            losing_trades: 20,
            win_rate: dec!(60),
            ..Default::default()
        };

        backtest.complete(results);
        assert_eq!(backtest.status, BacktestStatus::Completed);
        assert!(backtest.results.is_some());
    }

    #[test]
    fn test_backtest_results_score() {
        let results = BacktestResults {
            total_return: dec!(5000),
            total_return_pct: dec!(50),
            final_capital: dec!(15000),
            total_trades: 100,
            winning_trades: 60,
            losing_trades: 40,
            win_rate: dec!(60),
            profit_factor: Some(dec!(2.0)),
            sharpe_ratio: Some(dec!(1.5)),
            sortino_ratio: Some(dec!(2.0)),
            max_drawdown: dec!(-1000),
            max_drawdown_pct: dec!(-10),
            ..Default::default()
        };

        let score = results.calculate_score();
        assert!(score >= 80); // Deve ser pelo menos B

        let grade = results.grade();
        assert!(grade == "A+" || grade == "A");
    }
}
