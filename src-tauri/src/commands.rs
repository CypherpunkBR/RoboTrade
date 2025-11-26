//! Comandos Tauri para comunicação com o frontend

use robotrade_core::dto::{
    ConnectionStatus, DashboardSummary, FearGreedDto, OrderDto, PositionDto, ServiceStatus,
    TradingMode, TradeDto, CreateOrderRequest,
};
use robotrade_infra::AppConfig;
use rust_decimal::Decimal;
use serde::Serialize;
use tauri::State;
use tracing::{debug, info};

use crate::state::AppState;

/// Erro serializado para o frontend
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl From<String> for CommandError {
    fn from(s: String) -> Self {
        Self {
            code: "ERROR".into(),
            message: s,
        }
    }
}

impl From<&str> for CommandError {
    fn from(s: &str) -> Self {
        Self {
            code: "ERROR".into(),
            message: s.to_string(),
        }
    }
}

type CommandResult<T> = Result<T, CommandError>;

// =============================================================================
// Dashboard
// =============================================================================

/// Retorna resumo do dashboard
#[tauri::command]
pub async fn get_dashboard_summary(state: State<'_, AppState>) -> CommandResult<DashboardSummary> {
    debug!("Buscando resumo do dashboard...");

    let (binance, fear_greed_api, database) = state.connection_status();
    let fear_greed = state.fear_greed();

    Ok(DashboardSummary {
        trading_mode: state.trading_mode(),
        total_balance_usdt: Decimal::from(10000), // TODO: Buscar saldo real
        daily_pnl: state.daily_pnl(),
        daily_pnl_pct: Decimal::ZERO,
        open_positions_count: state.open_positions_count() as u32,
        active_orders_count: 0,
        active_signals_count: 0,
        fear_greed_value: fear_greed.as_ref().map(|f| f.value),
        fear_greed_classification: fear_greed.as_ref().map(|f| f.classification),
        connection_status: ConnectionStatus {
            binance: if binance {
                ServiceStatus::Connected
            } else {
                ServiceStatus::Disconnected
            },
            fear_greed_api: if fear_greed_api {
                ServiceStatus::Connected
            } else {
                ServiceStatus::Disconnected
            },
            database: if database {
                ServiceStatus::Connected
            } else {
                ServiceStatus::Disconnected
            },
        },
        updated_at: chrono::Utc::now(),
    })
}

/// Retorna status das conexões
#[tauri::command]
pub async fn get_connection_status(state: State<'_, AppState>) -> CommandResult<ConnectionStatus> {
    let (binance, fear_greed_api, database) = state.connection_status();

    Ok(ConnectionStatus {
        binance: if binance {
            ServiceStatus::Connected
        } else {
            ServiceStatus::Disconnected
        },
        fear_greed_api: if fear_greed_api {
            ServiceStatus::Connected
        } else {
            ServiceStatus::Disconnected
        },
        database: if database {
            ServiceStatus::Connected
        } else {
            ServiceStatus::Disconnected
        },
    })
}

// =============================================================================
// Fear & Greed
// =============================================================================

/// Retorna Fear & Greed Index atual
#[tauri::command]
pub async fn get_fear_greed_current(state: State<'_, AppState>) -> CommandResult<Option<FearGreedDto>> {
    debug!("Buscando Fear & Greed atual...");

    let fear_greed = state.fear_greed();

    Ok(fear_greed.map(|fg| FearGreedDto {
        value: fg.value,
        classification: fg.classification,
        classification_text: fg.classification.description_pt().to_string(),
        avg_7d: None,
        avg_30d: None,
        trend: None,
        updated_at: fg.collected_at,
    }))
}

/// Retorna histórico do Fear & Greed Index
#[tauri::command]
pub async fn get_fear_greed_history(days: u32) -> CommandResult<Vec<FearGreedDto>> {
    debug!(days = %days, "Buscando histórico Fear & Greed...");

    // TODO: Buscar do banco de dados
    Ok(vec![])
}

// =============================================================================
// Posições e Ordens
// =============================================================================

/// Retorna posições abertas
#[tauri::command]
pub async fn get_positions(state: State<'_, AppState>) -> CommandResult<Vec<PositionDto>> {
    debug!("Buscando posições...");

    let positions = state.positions();

    let dtos: Vec<PositionDto> = positions
        .into_iter()
        .map(|p| {
            let now = chrono::Utc::now();
            let duration = now - p.opened_at;
            let duration_str = format_duration(duration.num_seconds());

            PositionDto {
                id: p.id.to_string(),
                exchange: p.exchange,
                symbol: p.symbol,
                side: p.side,
                quantity: p.quantity,
                entry_price: p.entry_price,
                current_price: p.current_price,
                leverage: p.leverage,
                unrealized_pnl: p.unrealized_pnl,
                unrealized_pnl_pct: p.unrealized_pnl_pct,
                stop_loss_price: p.stop_loss_price,
                take_profit_price: p.take_profit_price,
                liquidation_price: p.liquidation_price,
                status: p.status,
                opened_at: p.opened_at,
                duration: duration_str,
            }
        })
        .collect();

    Ok(dtos)
}

/// Retorna ordens abertas
#[tauri::command]
pub async fn get_open_orders() -> CommandResult<Vec<OrderDto>> {
    debug!("Buscando ordens abertas...");

    // TODO: Buscar ordens da exchange
    Ok(vec![])
}

/// Retorna saldos da conta
#[tauri::command]
pub async fn get_balances() -> CommandResult<Vec<BalanceDto>> {
    debug!("Buscando saldos...");

    // TODO: Buscar saldos da exchange
    Ok(vec![BalanceDto {
        asset: "USDT".into(),
        free: Decimal::from(10000),
        locked: Decimal::ZERO,
        total: Decimal::from(10000),
    }])
}

#[derive(Debug, Serialize)]
pub struct BalanceDto {
    pub asset: String,
    pub free: Decimal,
    pub locked: Decimal,
    pub total: Decimal,
}

// =============================================================================
// Trading
// =============================================================================

/// Cria uma nova ordem
#[tauri::command]
pub async fn create_order(
    state: State<'_, AppState>,
    request: CreateOrderRequest,
) -> CommandResult<OrderDto> {
    info!(
        symbol = %request.symbol,
        side = ?request.side,
        "Criando ordem..."
    );

    // Verifica modo de trading
    if state.trading_mode() == TradingMode::Paper {
        debug!("Modo paper trading - ordem simulada");
    }

    // TODO: Implementar criação de ordem real
    Err("Criação de ordens ainda não implementada".into())
}

/// Cancela uma ordem
#[tauri::command]
pub async fn cancel_order(order_id: String) -> CommandResult<bool> {
    info!(order_id = %order_id, "Cancelando ordem...");

    // TODO: Implementar cancelamento
    Err("Cancelamento de ordens ainda não implementado".into())
}

// =============================================================================
// Configuração
// =============================================================================

/// Retorna configuração atual
#[tauri::command]
pub async fn get_config() -> CommandResult<AppConfig> {
    debug!("Buscando configuração...");

    AppConfig::load()
        .await
        .map_err(|e| format!("Erro ao carregar configuração: {}", e).into())
}

/// Salva configuração
#[tauri::command]
pub async fn save_config(config: AppConfig) -> CommandResult<()> {
    info!("Salvando configuração...");

    config
        .save()
        .await
        .map_err(|e| format!("Erro ao salvar configuração: {}", e).into())
}

/// Retorna modo de trading atual
#[tauri::command]
pub async fn get_trading_mode(state: State<'_, AppState>) -> CommandResult<TradingMode> {
    Ok(state.trading_mode())
}

/// Define modo de trading
#[tauri::command]
pub async fn set_trading_mode(
    state: State<'_, AppState>,
    mode: TradingMode,
) -> CommandResult<()> {
    info!(mode = ?mode, "Alterando modo de trading...");

    // Se mudando para Live, requer confirmação adicional
    if mode == TradingMode::Live {
        // TODO: Implementar triple-lock confirmation
        return Err("Modo Live requer confirmação adicional".into());
    }

    state.set_trading_mode(mode);
    Ok(())
}

// =============================================================================
// Histórico
// =============================================================================

/// Retorna histórico de trades
#[tauri::command]
pub async fn get_trade_history(
    symbol: Option<String>,
    limit: Option<u32>,
) -> CommandResult<Vec<TradeDto>> {
    debug!(symbol = ?symbol, limit = ?limit, "Buscando histórico de trades...");

    // TODO: Buscar do banco de dados
    Ok(vec![])
}

/// Retorna estatísticas de trades
#[tauri::command]
pub async fn get_trade_stats() -> CommandResult<TradeStatsDto> {
    debug!("Buscando estatísticas de trades...");

    // TODO: Calcular estatísticas reais
    Ok(TradeStatsDto {
        total_trades: 0,
        winning_trades: 0,
        losing_trades: 0,
        win_rate_pct: Decimal::ZERO,
        total_pnl: Decimal::ZERO,
        gross_profit: Decimal::ZERO,
        gross_loss: Decimal::ZERO,
        profit_factor: Decimal::ZERO,
        avg_win: Decimal::ZERO,
        avg_loss: Decimal::ZERO,
        largest_win: Decimal::ZERO,
        largest_loss: Decimal::ZERO,
        total_fees: Decimal::ZERO,
    })
}

#[derive(Debug, Serialize)]
pub struct TradeStatsDto {
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub win_rate_pct: Decimal,
    pub total_pnl: Decimal,
    pub gross_profit: Decimal,
    pub gross_loss: Decimal,
    pub profit_factor: Decimal,
    pub avg_win: Decimal,
    pub avg_loss: Decimal,
    pub largest_win: Decimal,
    pub largest_loss: Decimal,
    pub total_fees: Decimal,
}

// =============================================================================
// Helpers
// =============================================================================

/// Formata duração em formato legível
fn format_duration(seconds: i64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}
