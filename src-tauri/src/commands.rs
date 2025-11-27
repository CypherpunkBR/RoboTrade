//! Comandos Tauri para comunicação com o frontend

use robotrade_core::dto::{
    ConnectionStatus, CreateOrderRequest, DashboardSummary, FearGreedDto, OrderDto, PositionDto,
    ServiceStatus, TradeDto, TradingMode,
};
use robotrade_core::entities::{OrderRequest, TimeInForce};
use robotrade_infra::AppConfig;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Serialize;
use tauri::State;
use tracing::{debug, info, warn};

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
pub async fn get_fear_greed_current(
    state: State<'_, AppState>,
) -> CommandResult<Option<FearGreedDto>> {
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
pub async fn get_fear_greed_history(
    state: State<'_, AppState>,
    days: u32,
) -> CommandResult<Vec<FearGreedDto>> {
    debug!(days = %days, "Buscando histórico Fear & Greed...");

    let history = state
        .get_fear_greed_history(days)
        .await
        .map_err(|e| CommandError {
            code: "FEAR_GREED_ERROR".into(),
            message: e,
        })?;

    let dtos: Vec<FearGreedDto> = history
        .into_iter()
        .map(|fg| FearGreedDto {
            value: fg.value,
            classification: fg.classification,
            classification_text: fg.classification.description_pt().to_string(),
            avg_7d: None,
            avg_30d: None,
            trend: None,
            updated_at: fg.collected_at,
        })
        .collect();

    debug!(count = dtos.len(), "Histórico Fear & Greed carregado");
    Ok(dtos)
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
pub async fn get_open_orders(
    state: State<'_, AppState>,
    symbol: Option<String>,
) -> CommandResult<Vec<OrderDto>> {
    debug!(symbol = ?symbol, "Buscando ordens abertas...");

    let orders = state
        .exchange
        .get_open_orders(symbol.as_deref())
        .await
        .map_err(|e| CommandError {
            code: "ORDERS_ERROR".into(),
            message: format!("Erro ao buscar ordens: {}", e),
        })?;

    let dtos: Vec<OrderDto> = orders
        .into_iter()
        .map(|o| OrderDto {
            id: o.id.to_string(),
            exchange_order_id: o.exchange_order_id,
            exchange: o.exchange,
            symbol: o.symbol,
            side: o.side,
            order_type: o.order_type,
            quantity: o.quantity,
            price: o.price,
            status: o.status,
            filled_quantity: o.filled_quantity,
            average_fill_price: o.average_fill_price,
            created_at: o.created_at,
        })
        .collect();

    debug!(count = dtos.len(), "Ordens abertas encontradas");
    Ok(dtos)
}

/// Retorna saldos da conta
#[tauri::command]
pub async fn get_balances(state: State<'_, AppState>) -> CommandResult<Vec<BalanceDto>> {
    debug!("Buscando saldos...");

    let balances = state
        .exchange
        .get_balances()
        .await
        .map_err(|e| CommandError {
            code: "BALANCES_ERROR".into(),
            message: format!("Erro ao buscar saldos: {}", e),
        })?;

    let dtos: Vec<BalanceDto> = balances
        .into_iter()
        .map(|b| BalanceDto {
            asset: b.asset,
            free: b.free,
            locked: b.locked,
            total: b.free + b.locked,
        })
        .collect();

    debug!(count = dtos.len(), "Saldos encontrados");
    Ok(dtos)
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
    let is_paper = state.trading_mode() == TradingMode::Paper;
    if is_paper {
        debug!("Modo paper trading - ordem simulada");
    }

    // Converte request do DTO para OrderRequest do core
    let order_request = OrderRequest {
        symbol: request.symbol.clone(),
        side: request.side,
        order_type: request.order_type,
        quantity: request.quantity,
        price: request.price,
        stop_price: request.stop_loss, // Stop price para ordens stop
        stop_loss: request.stop_loss,
        take_profit: request.take_profit,
        time_in_force: TimeInForce::GTC,
        leverage: request.leverage,
        reduce_only: false,
    };

    // Envia ordem para exchange
    let order = state
        .exchange
        .submit_order(order_request)
        .await
        .map_err(|e| CommandError {
            code: "ORDER_ERROR".into(),
            message: format!("Erro ao criar ordem: {}", e),
        })?;

    info!(
        order_id = %order.id,
        status = ?order.status,
        "Ordem criada com sucesso"
    );

    // Converte para DTO
    Ok(OrderDto {
        id: order.id.to_string(),
        exchange_order_id: order.exchange_order_id,
        exchange: order.exchange,
        symbol: order.symbol,
        side: order.side,
        order_type: order.order_type,
        quantity: order.quantity,
        price: order.price,
        status: order.status,
        filled_quantity: order.filled_quantity,
        average_fill_price: order.average_fill_price,
        created_at: order.created_at,
    })
}

/// Cancela uma ordem
#[tauri::command]
pub async fn cancel_order(
    state: State<'_, AppState>,
    symbol: String,
    order_id: String,
) -> CommandResult<bool> {
    info!(order_id = %order_id, symbol = %symbol, "Cancelando ordem...");

    let result = state
        .exchange
        .cancel_order(&symbol, &order_id)
        .await
        .map_err(|e| CommandError {
            code: "CANCEL_ERROR".into(),
            message: format!("Erro ao cancelar ordem: {}", e),
        })?;

    if result {
        info!(order_id = %order_id, "Ordem cancelada com sucesso");
    } else {
        warn!(order_id = %order_id, "Ordem não encontrada ou já cancelada");
    }

    Ok(result)
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
pub async fn set_trading_mode(state: State<'_, AppState>, mode: TradingMode) -> CommandResult<()> {
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
    state: State<'_, AppState>,
    symbol: Option<String>,
    limit: Option<u32>,
) -> CommandResult<Vec<TradeDto>> {
    debug!(symbol = ?symbol, limit = ?limit, "Buscando histórico de trades...");

    // Busca trades do cache no state
    let trades = state.trades();
    let limit = limit.unwrap_or(100) as usize;

    let filtered: Vec<TradeDto> = trades
        .into_iter()
        .filter(|t| symbol.as_ref().map(|s| &t.symbol == s).unwrap_or(true))
        .take(limit)
        .map(|t| {
            let duration_secs = t.duration_seconds;
            let duration = format_duration(duration_secs);
            TradeDto {
                id: t.id.to_string(),
                exchange: t.exchange,
                symbol: t.symbol,
                side: t.side,
                quantity: t.quantity,
                entry_price: t.entry_price,
                exit_price: t.exit_price,
                net_pnl: t.net_pnl,
                pnl_pct: t.pnl_pct,
                roi_pct: t.roi_pct,
                leverage: t.leverage,
                close_reason: t.close_reason,
                duration,
                entered_at: t.entered_at,
                exited_at: t.exited_at,
                is_profitable: t.net_pnl > Decimal::ZERO,
            }
        })
        .collect();

    debug!(count = filtered.len(), "Trades encontrados no histórico");
    Ok(filtered)
}

/// Retorna estatísticas de trades
#[tauri::command]
pub async fn get_trade_stats(state: State<'_, AppState>) -> CommandResult<TradeStatsDto> {
    debug!("Buscando estatísticas de trades...");

    let trades = state.trades();

    if trades.is_empty() {
        return Ok(TradeStatsDto {
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
        });
    }

    let total_trades = trades.len() as u32;
    let mut winning_trades = 0u32;
    let mut losing_trades = 0u32;
    let mut gross_profit = Decimal::ZERO;
    let mut gross_loss = Decimal::ZERO;
    let mut total_fees = Decimal::ZERO;
    let mut largest_win = Decimal::ZERO;
    let mut largest_loss = Decimal::ZERO;

    for trade in &trades {
        total_fees += trade.total_fees;

        if trade.net_pnl > Decimal::ZERO {
            winning_trades += 1;
            gross_profit += trade.net_pnl;
            if trade.net_pnl > largest_win {
                largest_win = trade.net_pnl;
            }
        } else if trade.net_pnl < Decimal::ZERO {
            losing_trades += 1;
            gross_loss += trade.net_pnl.abs();
            if trade.net_pnl.abs() > largest_loss {
                largest_loss = trade.net_pnl.abs();
            }
        }
    }

    let total_pnl = gross_profit - gross_loss;
    let win_rate_pct = if total_trades > 0 {
        Decimal::from(winning_trades) / Decimal::from(total_trades) * dec!(100)
    } else {
        Decimal::ZERO
    };

    let profit_factor = if gross_loss > Decimal::ZERO {
        gross_profit / gross_loss
    } else if gross_profit > Decimal::ZERO {
        dec!(999.99) // Infinity substitute
    } else {
        Decimal::ZERO
    };

    let avg_win = if winning_trades > 0 {
        gross_profit / Decimal::from(winning_trades)
    } else {
        Decimal::ZERO
    };

    let avg_loss = if losing_trades > 0 {
        gross_loss / Decimal::from(losing_trades)
    } else {
        Decimal::ZERO
    };

    Ok(TradeStatsDto {
        total_trades,
        winning_trades,
        losing_trades,
        win_rate_pct,
        total_pnl,
        gross_profit,
        gross_loss,
        profit_factor,
        avg_win,
        avg_loss,
        largest_win,
        largest_loss,
        total_fees,
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

// =============================================================================
// Trading Worker
// =============================================================================

use crate::trading_worker::RiskStats;

/// Inicia o worker de trading automático
#[tauri::command]
pub async fn start_trading_worker(state: State<'_, AppState>) -> CommandResult<()> {
    info!("Iniciando worker de trading...");

    state
        .trading_worker
        .start()
        .await
        .map_err(|e| CommandError {
            code: "WORKER_ERROR".into(),
            message: e,
        })
}

/// Para o worker de trading automático
#[tauri::command]
pub async fn stop_trading_worker(state: State<'_, AppState>) -> CommandResult<()> {
    info!("Parando worker de trading...");

    state
        .trading_worker
        .stop()
        .await
        .map_err(|e| CommandError {
            code: "WORKER_ERROR".into(),
            message: e,
        })
}

/// Retorna se o worker está rodando
#[tauri::command]
pub async fn is_worker_running(state: State<'_, AppState>) -> CommandResult<bool> {
    Ok(state.trading_worker.is_running())
}

/// Retorna estatísticas de risco
#[tauri::command]
pub async fn get_risk_stats(state: State<'_, AppState>) -> CommandResult<RiskStats> {
    Ok(state.trading_worker.risk_stats().await)
}

/// Habilita/desabilita trading
#[tauri::command]
pub async fn set_trading_enabled(state: State<'_, AppState>, enabled: bool) -> CommandResult<()> {
    info!(enabled = %enabled, "Alterando estado do trading...");

    if enabled {
        state.trading_worker.enable_trading().await;
    } else {
        state.trading_worker.disable_trading("Desabilitado pelo usuário").await;
    }
    Ok(())
}

/// Reseta perdas diárias
#[tauri::command]
pub async fn reset_daily_losses(state: State<'_, AppState>) -> CommandResult<()> {
    info!("Resetando perdas diárias...");

    state.trading_worker.reset_daily_losses().await;
    Ok(())
}
