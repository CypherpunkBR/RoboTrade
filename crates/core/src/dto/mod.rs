//! Data Transfer Objects (DTOs) do RoboTrade
//!
//! Estruturas para comunicação entre camadas e com o frontend via Tauri.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::entities::{
    ExchangeId, FearGreedClassification, OrderSide, OrderStatus, OrderType, PositionSide,
    PositionStatus, SignalStatus, SignalStrength, TimeFrame, TradeCloseReason, TradeDirection,
};

// ============================================================================
// DTOs de Dashboard
// ============================================================================

/// Resumo do estado atual para o dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    /// Modo atual (paper/live)
    pub trading_mode: TradingMode,
    /// Saldo total em USDT
    pub total_balance_usdt: Decimal,
    /// P&L do dia
    pub daily_pnl: Decimal,
    /// P&L do dia em percentual
    pub daily_pnl_pct: Decimal,
    /// Número de posições abertas
    pub open_positions_count: u32,
    /// Número de ordens ativas
    pub active_orders_count: u32,
    /// Número de sinais ativos
    pub active_signals_count: u32,
    /// Fear & Greed Index atual
    pub fear_greed_value: Option<u8>,
    /// Classificação do Fear & Greed
    pub fear_greed_classification: Option<FearGreedClassification>,
    /// Status das conexões
    pub connection_status: ConnectionStatus,
    /// Última atualização
    pub updated_at: DateTime<Utc>,
}

/// Modo de trading
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TradingMode {
    /// Paper trading (simulado)
    Paper,
    /// Trading real
    Live,
}

/// Status das conexões
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    /// Conexão com Binance
    pub binance: ServiceStatus,
    /// Conexão com Fear & Greed API
    pub fear_greed_api: ServiceStatus,
    /// Status do banco de dados
    pub database: ServiceStatus,
}

/// Status de um serviço
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    /// Conectado e funcionando
    Connected,
    /// Desconectado
    Disconnected,
    /// Conectando
    Connecting,
    /// Erro
    Error,
}

// ============================================================================
// DTOs de Posição
// ============================================================================

/// DTO de posição para o frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionDto {
    /// ID da posição
    pub id: String,
    /// Exchange
    pub exchange: ExchangeId,
    /// Símbolo
    pub symbol: String,
    /// Lado
    pub side: PositionSide,
    /// Quantidade
    pub quantity: Decimal,
    /// Preço de entrada
    pub entry_price: Decimal,
    /// Preço atual
    pub current_price: Decimal,
    /// Alavancagem
    pub leverage: u32,
    /// P&L não realizado
    pub unrealized_pnl: Decimal,
    /// P&L em percentual
    pub unrealized_pnl_pct: Decimal,
    /// Stop loss
    pub stop_loss_price: Option<Decimal>,
    /// Take profit
    pub take_profit_price: Option<Decimal>,
    /// Preço de liquidação
    pub liquidation_price: Option<Decimal>,
    /// Status
    pub status: PositionStatus,
    /// Quando abriu
    pub opened_at: DateTime<Utc>,
    /// Duração formatada
    pub duration: String,
}

// ============================================================================
// DTOs de Ordem
// ============================================================================

/// DTO de ordem para o frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderDto {
    /// ID da ordem
    pub id: String,
    /// ID na exchange
    pub exchange_order_id: Option<String>,
    /// Exchange
    pub exchange: ExchangeId,
    /// Símbolo
    pub symbol: String,
    /// Lado
    pub side: OrderSide,
    /// Tipo
    pub order_type: OrderType,
    /// Quantidade
    pub quantity: Decimal,
    /// Preço
    pub price: Option<Decimal>,
    /// Status
    pub status: OrderStatus,
    /// Quantidade preenchida
    pub filled_quantity: Decimal,
    /// Preço médio de preenchimento
    pub average_fill_price: Option<Decimal>,
    /// Data de criação
    pub created_at: DateTime<Utc>,
}

/// Request para criar uma ordem via frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    /// Símbolo
    pub symbol: String,
    /// Lado
    pub side: OrderSide,
    /// Tipo
    pub order_type: OrderType,
    /// Quantidade
    pub quantity: Decimal,
    /// Preço (para ordens limit)
    pub price: Option<Decimal>,
    /// Stop loss
    pub stop_loss: Option<Decimal>,
    /// Take profit
    pub take_profit: Option<Decimal>,
    /// Alavancagem
    pub leverage: Option<u32>,
}

// ============================================================================
// DTOs de Sinal
// ============================================================================

/// DTO de sinal para o frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalDto {
    /// ID do sinal
    pub id: String,
    /// ID da estratégia
    pub strategy_id: String,
    /// Nome da estratégia
    pub strategy_name: String,
    /// Símbolo
    pub symbol: String,
    /// Direção
    pub direction: TradeDirection,
    /// Força
    pub strength: SignalStrength,
    /// Preço de trigger
    pub trigger_price: Decimal,
    /// Entrada sugerida
    pub suggested_entry: Option<Decimal>,
    /// Stop loss sugerido
    pub suggested_stop_loss: Option<Decimal>,
    /// Take profit sugerido
    pub suggested_take_profit: Option<Decimal>,
    /// Risk/Reward ratio
    pub risk_reward_ratio: Option<Decimal>,
    /// Motivo
    pub reason: String,
    /// Confiança (0-100)
    pub confidence: u8,
    /// Status
    pub status: SignalStatus,
    /// Requer confirmação
    pub requires_confirmation: bool,
    /// Quando foi gerado
    pub generated_at: DateTime<Utc>,
    /// Quando expira
    pub expires_at: DateTime<Utc>,
}

/// Request para confirmar um sinal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmSignalRequest {
    /// ID do sinal
    pub signal_id: String,
    /// Ajustar entrada (opcional)
    pub adjusted_entry: Option<Decimal>,
    /// Ajustar stop loss (opcional)
    pub adjusted_stop_loss: Option<Decimal>,
    /// Ajustar take profit (opcional)
    pub adjusted_take_profit: Option<Decimal>,
}

// ============================================================================
// DTOs de Trade
// ============================================================================

/// DTO de trade para o frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeDto {
    /// ID do trade
    pub id: String,
    /// Exchange
    pub exchange: ExchangeId,
    /// Símbolo
    pub symbol: String,
    /// Lado
    pub side: PositionSide,
    /// Quantidade
    pub quantity: Decimal,
    /// Preço de entrada
    pub entry_price: Decimal,
    /// Preço de saída
    pub exit_price: Decimal,
    /// P&L líquido
    pub net_pnl: Decimal,
    /// P&L em percentual
    pub pnl_pct: Decimal,
    /// ROI
    pub roi_pct: Decimal,
    /// Alavancagem
    pub leverage: u32,
    /// Motivo do fechamento
    pub close_reason: TradeCloseReason,
    /// Duração formatada
    pub duration: String,
    /// Quando entrou
    pub entered_at: DateTime<Utc>,
    /// Quando saiu
    pub exited_at: DateTime<Utc>,
    /// Se foi lucrativo
    pub is_profitable: bool,
}

// ============================================================================
// DTOs de Fear & Greed
// ============================================================================

/// DTO do Fear & Greed Index para o frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FearGreedDto {
    /// Valor atual
    pub value: u8,
    /// Classificação
    pub classification: FearGreedClassification,
    /// Descrição em português
    pub classification_text: String,
    /// Média 7 dias
    pub avg_7d: Option<f64>,
    /// Média 30 dias
    pub avg_30d: Option<f64>,
    /// Tendência
    pub trend: Option<String>,
    /// Última atualização
    pub updated_at: DateTime<Utc>,
}

/// Histórico do Fear & Greed para gráficos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FearGreedHistoryDto {
    /// Data (ISO 8601)
    pub dates: Vec<String>,
    /// Valores
    pub values: Vec<u8>,
}

// ============================================================================
// DTOs de Estratégia
// ============================================================================

/// DTO de estratégia para o frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyDto {
    /// ID da estratégia
    pub id: String,
    /// Nome
    pub name: String,
    /// Descrição
    pub description: String,
    /// Símbolos que opera
    pub symbols: Vec<String>,
    /// Timeframes
    pub timeframes: Vec<TimeFrame>,
    /// Se está ativa
    pub is_active: bool,
    /// Estatísticas
    pub stats: Option<StrategyStatsDto>,
}

/// Estatísticas de uma estratégia
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyStatsDto {
    /// Total de sinais gerados
    pub total_signals: u32,
    /// Sinais executados
    pub executed_signals: u32,
    /// Trades vencedores
    pub winning_trades: u32,
    /// Trades perdedores
    pub losing_trades: u32,
    /// Win rate
    pub win_rate_pct: Decimal,
    /// P&L total
    pub total_pnl: Decimal,
    /// Profit factor
    pub profit_factor: Option<Decimal>,
}

// ============================================================================
// DTOs de Backtest
// ============================================================================

/// Request para executar um backtest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestRequest {
    /// ID da estratégia
    pub strategy_id: String,
    /// Símbolo para testar
    pub symbol: String,
    /// Timeframe
    pub timeframe: TimeFrame,
    /// Data inicial (ISO 8601)
    pub start_date: String,
    /// Data final (ISO 8601)
    pub end_date: String,
    /// Capital inicial
    pub initial_capital: Decimal,
    /// Alavancagem para simulação
    pub leverage: u32,
}

/// Resultado do backtest para o frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResultDto {
    /// ID do backtest
    pub id: String,
    /// Estratégia
    pub strategy_id: String,
    /// Símbolo
    pub symbol: String,
    /// Período
    pub period: String,
    /// Capital inicial
    pub initial_capital: Decimal,
    /// Capital final
    pub final_capital: Decimal,
    /// Retorno total em percentual
    pub total_return_pct: Decimal,
    /// Total de trades
    pub total_trades: u32,
    /// Trades vencedores
    pub winning_trades: u32,
    /// Win rate
    pub win_rate_pct: Decimal,
    /// Profit factor
    pub profit_factor: Decimal,
    /// Sharpe ratio
    pub sharpe_ratio: Decimal,
    /// Max drawdown
    pub max_drawdown_pct: Decimal,
    /// Trades executados (resumo)
    pub trades: Vec<BacktestTradeDto>,
    /// Curva de equity (para gráfico)
    pub equity_curve: Vec<EquityPoint>,
}

/// Trade do backtest (simplificado)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestTradeDto {
    /// Data de entrada
    pub entry_date: String,
    /// Data de saída
    pub exit_date: String,
    /// Lado
    pub side: PositionSide,
    /// Preço de entrada
    pub entry_price: Decimal,
    /// Preço de saída
    pub exit_price: Decimal,
    /// P&L em percentual
    pub pnl_pct: Decimal,
}

/// Ponto da curva de equity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    /// Data (ISO 8601)
    pub date: String,
    /// Valor do portfolio
    pub equity: Decimal,
    /// Drawdown atual
    pub drawdown: Decimal,
}

// ============================================================================
// DTOs de System Tray
// ============================================================================

/// Dados para o System Tray
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayData {
    /// Modo de trading
    pub mode: TradingMode,
    /// P&L do dia
    pub daily_pnl: Decimal,
    /// P&L em percentual
    pub daily_pnl_pct: Decimal,
    /// Posições abertas
    pub open_positions: u32,
    /// Fear & Greed atual
    pub fear_greed: Option<u8>,
    /// Status da conexão
    pub is_connected: bool,
}

// ============================================================================
// DTOs de Configuração
// ============================================================================

/// Configurações do usuário
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettingsDto {
    /// Modo de trading padrão
    pub default_trading_mode: TradingMode,
    /// Alavancagem padrão
    pub default_leverage: u32,
    /// Tamanho padrão da posição (% do capital)
    pub default_position_size_pct: Decimal,
    /// Stop loss padrão (%)
    pub default_stop_loss_pct: Decimal,
    /// Take profit padrão (%)
    pub default_take_profit_pct: Decimal,
    /// Notificações habilitadas
    pub notifications_enabled: bool,
    /// Sons habilitados
    pub sounds_enabled: bool,
    /// Tema (dark/light)
    pub theme: String,
}

impl Default for UserSettingsDto {
    fn default() -> Self {
        Self {
            default_trading_mode: TradingMode::Paper,
            default_leverage: 5,
            default_position_size_pct: Decimal::from(5),
            default_stop_loss_pct: Decimal::from(2),
            default_take_profit_pct: Decimal::from(4),
            notifications_enabled: true,
            sounds_enabled: true,
            theme: "dark".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trading_mode_serde() {
        let mode = TradingMode::Paper;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"paper\"");

        let parsed: TradingMode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, TradingMode::Paper);
    }

    #[test]
    fn test_user_settings_default() {
        let settings = UserSettingsDto::default();
        assert_eq!(settings.default_trading_mode, TradingMode::Paper);
        assert_eq!(settings.default_leverage, 5);
    }
}
