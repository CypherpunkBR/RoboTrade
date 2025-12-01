//! Comandos Tauri para comunicação com o frontend

use robotrade_core::dto::{
    ConnectionStatus, CreateOrderRequest, DashboardSummary, FearGreedDto, OrderDto, PositionDto,
    ServiceStatus, TradeDto, TradingMode,
};
use robotrade_core::entities::{OrderRequest, TimeInForce};
use robotrade_infra::AppConfig;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
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

    // Busca posições e ordens de todas as exchanges
    let positions = state.exchange.get_all_positions().await;
    let orders = state.exchange.get_all_open_orders().await;

    // Calcula saldo total de todas as exchanges (excluindo paper se tiver exchanges reais)
    let all_balances = state.exchange.get_all_balances().await;

    // Verifica se temos exchanges reais conectadas
    let has_real_exchange = all_balances.contains_key("binance") || all_balances.contains_key("kraken");

    // Calcula saldo - se tiver exchange real, ignora paper trading
    let total_balance_usdt: Decimal = all_balances
        .iter()
        .filter(|(exchange, _)| {
            // Se temos exchange real, ignora paper
            if has_real_exchange {
                *exchange != "paper"
            } else {
                true
            }
        })
        .flat_map(|(exchange, balances)| {
            debug!(exchange = %exchange, count = balances.len(), "Saldos da exchange");
            let exchange = exchange.clone();
            balances.iter().inspect(move |b| {
                debug!(
                    exchange = %exchange,
                    asset = %b.asset,
                    free = %b.free,
                    locked = %b.locked,
                    "Saldo encontrado"
                );
            })
        })
        .filter(|b| b.asset == "USDT" || b.asset == "USD")
        .map(|b| b.free + b.locked)
        .sum();

    info!(
        total_balance = %total_balance_usdt,
        has_real_exchange = has_real_exchange,
        exchanges = ?all_balances.keys().collect::<Vec<_>>(),
        "Saldo total calculado"
    );

    // Calcula PnL não realizado total
    let unrealized_pnl: Decimal = positions.iter().map(|p| p.unrealized_pnl).sum();
    let daily_pnl = state.daily_pnl() + unrealized_pnl;

    // Calcula percentual do PnL baseado no saldo total
    let daily_pnl_pct = if total_balance_usdt > Decimal::ZERO {
        // PnL% = (daily_pnl / (total_balance - daily_pnl)) * 100
        // Onde (total_balance - daily_pnl) é o saldo inicial do dia
        let initial_balance = total_balance_usdt - daily_pnl;
        if initial_balance > Decimal::ZERO {
            (daily_pnl / initial_balance) * dec!(100)
        } else {
            Decimal::ZERO
        }
    } else {
        Decimal::ZERO
    };

    Ok(DashboardSummary {
        trading_mode: state.trading_mode(),
        total_balance_usdt,
        daily_pnl,
        daily_pnl_pct,
        open_positions_count: positions.len() as u32,
        active_orders_count: orders.len() as u32,
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

/// Retorna posições abertas de todas as exchanges
#[tauri::command]
pub async fn get_positions(state: State<'_, AppState>) -> CommandResult<Vec<PositionDto>> {
    debug!("Buscando posições de todas as exchanges...");

    // Busca posições de todas as exchanges conectadas (Binance, Kraken, Paper)
    let positions = state.exchange.get_all_positions().await;

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

/// Retorna ordens abertas de todas as exchanges
#[tauri::command]
pub async fn get_open_orders(
    state: State<'_, AppState>,
    symbol: Option<String>,
) -> CommandResult<Vec<OrderDto>> {
    debug!(symbol = ?symbol, "Buscando ordens abertas de todas as exchanges...");

    // Busca ordens de todas as exchanges conectadas
    let mut orders = state.exchange.get_all_open_orders().await;

    // Filtra por símbolo se especificado
    if let Some(ref s) = symbol {
        orders.retain(|o| o.symbol == *s);
    }

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

/// Retorna saldos de todas as exchanges
#[tauri::command]
pub async fn get_balances(state: State<'_, AppState>) -> CommandResult<Vec<BalanceDto>> {
    debug!("Buscando saldos de todas as exchanges...");

    // Busca saldos de todas as exchanges conectadas
    let all_balances = state.exchange.get_all_balances().await;

    let mut dtos: Vec<BalanceDto> = Vec::new();
    for (exchange, balances) in all_balances {
        for b in balances {
            dtos.push(BalanceDto {
                exchange: exchange.clone(),
                asset: b.asset,
                free: b.free,
                locked: b.locked,
                total: b.free + b.locked,
            });
        }
    }

    debug!(count = dtos.len(), "Saldos encontrados de todas as exchanges");
    Ok(dtos)
}

#[derive(Debug, Serialize)]
pub struct BalanceDto {
    pub exchange: String,
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

// =============================================================================
// Market Data
// =============================================================================

use crate::market_data_service::MarketDataService;
use robotrade_core::entities::{PriceAlert, PriceAlertCondition};

/// DTO para candle enviado ao frontend
#[derive(Debug, Serialize)]
pub struct CandleDto {
    pub time: i64,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
}

/// DTO para alerta de preco
#[derive(Debug, Serialize, serde::Deserialize)]
pub struct PriceAlertDto {
    pub id: String,
    pub symbol: String,
    pub condition: String,
    pub target_price: Decimal,
    pub percent: Option<Decimal>,
    pub status: String,
    pub message: Option<String>,
    pub recurring: bool,
    pub trigger_count: u32,
    pub created_at: String,
    pub triggered_at: Option<String>,
}

/// Request para criar alerta
#[derive(Debug, serde::Deserialize)]
pub struct CreateAlertRequest {
    pub symbol: String,
    pub condition: String,
    pub target_price: Decimal,
    pub percent: Option<Decimal>,
    pub message: Option<String>,
    pub recurring: bool,
}

/// Busca candles/klines
#[tauri::command]
pub async fn get_klines(
    state: State<'_, AppState>,
    symbol: String,
    interval: String,
    limit: Option<u32>,
) -> CommandResult<Vec<CandleDto>> {
    debug!(symbol = %symbol, interval = %interval, "Buscando klines...");

    let candles = state
        .market_data
        .get_klines(&symbol, &interval, limit)
        .await
        .map_err(|e| CommandError {
            code: "KLINES_ERROR".into(),
            message: e,
        })?;

    let dtos: Vec<CandleDto> = candles
        .into_iter()
        .map(|c| CandleDto {
            time: c.open_time.timestamp(),
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume,
        })
        .collect();

    debug!(count = dtos.len(), "Klines carregados");
    Ok(dtos)
}

/// Busca candles com range de tempo
#[tauri::command]
pub async fn get_klines_range(
    state: State<'_, AppState>,
    symbol: String,
    interval: String,
    start_time: Option<i64>,
    end_time: Option<i64>,
    limit: Option<u32>,
) -> CommandResult<Vec<CandleDto>> {
    debug!(
        symbol = %symbol,
        interval = %interval,
        start = ?start_time,
        end = ?end_time,
        "Buscando klines com range..."
    );

    let candles = state
        .market_data
        .get_klines_range(&symbol, &interval, start_time, end_time, limit)
        .await
        .map_err(|e| CommandError {
            code: "KLINES_ERROR".into(),
            message: e,
        })?;

    let dtos: Vec<CandleDto> = candles
        .into_iter()
        .map(|c| CandleDto {
            time: c.open_time.timestamp(),
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume,
        })
        .collect();

    debug!(count = dtos.len(), "Klines com range carregados");
    Ok(dtos)
}

/// Busca historico completo de klines
#[tauri::command]
pub async fn get_klines_history(
    state: State<'_, AppState>,
    symbol: String,
    interval: String,
    start_time: i64,
    end_time: Option<i64>,
) -> CommandResult<Vec<CandleDto>> {
    info!(
        symbol = %symbol,
        interval = %interval,
        start = %start_time,
        "Buscando historico completo de klines..."
    );

    let candles = state
        .market_data
        .get_all_klines(&symbol, &interval, start_time, end_time)
        .await
        .map_err(|e| CommandError {
            code: "KLINES_HISTORY_ERROR".into(),
            message: e,
        })?;

    let dtos: Vec<CandleDto> = candles
        .into_iter()
        .map(|c| CandleDto {
            time: c.open_time.timestamp(),
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume,
        })
        .collect();

    info!(count = dtos.len(), "Historico de klines carregado");
    Ok(dtos)
}

/// Calcula indicadores tecnicos
#[tauri::command]
pub async fn calculate_indicators(
    state: State<'_, AppState>,
    symbol: String,
    interval: String,
    limit: Option<u32>,
    sma_periods: Option<Vec<u32>>,
    ema_periods: Option<Vec<u32>>,
) -> CommandResult<IndicatorsDto> {
    let candles = state
        .market_data
        .get_klines(&symbol, &interval, limit)
        .await
        .map_err(|e| CommandError {
            code: "INDICATORS_ERROR".into(),
            message: e,
        })?;

    // Convert candles to DTOs
    let candle_dtos: Vec<CandleDto> = candles
        .iter()
        .map(|c| CandleDto {
            time: c.open_time.timestamp(),
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume,
        })
        .collect();

    let mut sma_map = std::collections::HashMap::new();
    let mut ema_map = std::collections::HashMap::new();

    // Calcula SMAs
    if let Some(periods) = sma_periods {
        for period in periods {
            let values = MarketDataService::calculate_sma(&candles, period as usize);
            sma_map.insert(
                period.to_string(),
                values.into_iter().map(|v| v.map(|d| d.to_string())).collect(),
            );
        }
    }

    // Calcula EMAs
    if let Some(periods) = ema_periods {
        for period in periods {
            let values = MarketDataService::calculate_ema(&candles, period as usize);
            ema_map.insert(
                period.to_string(),
                values.into_iter().map(|v| v.map(|d| d.to_string())).collect(),
            );
        }
    }

    Ok(IndicatorsDto {
        symbol,
        interval,
        candles: candle_dtos,
        sma: sma_map,
        ema: ema_map,
    })
}

#[derive(Debug, Serialize)]
pub struct IndicatorsDto {
    pub symbol: String,
    pub interval: String,
    pub candles: Vec<CandleDto>,
    pub sma: std::collections::HashMap<String, Vec<Option<String>>>,
    pub ema: std::collections::HashMap<String, Vec<Option<String>>>,
}

// =============================================================================
// Price Alerts
// =============================================================================

/// Lista todos os alertas
#[tauri::command]
pub async fn list_price_alerts(state: State<'_, AppState>) -> CommandResult<Vec<PriceAlertDto>> {
    let alerts = state.market_data.list_alerts();

    let dtos: Vec<PriceAlertDto> = alerts
        .into_iter()
        .map(|a| PriceAlertDto {
            id: a.id.to_string(),
            symbol: a.symbol,
            condition: format!("{:?}", a.condition).to_lowercase(),
            target_price: a.target_price,
            percent: a.percent,
            status: format!("{:?}", a.status).to_lowercase(),
            message: a.message,
            recurring: a.recurring,
            trigger_count: a.trigger_count,
            created_at: a.created_at.to_rfc3339(),
            triggered_at: a.triggered_at.map(|t| t.to_rfc3339()),
        })
        .collect();

    Ok(dtos)
}

/// Cria um novo alerta de preco
#[tauri::command]
pub async fn create_price_alert(
    state: State<'_, AppState>,
    request: CreateAlertRequest,
) -> CommandResult<PriceAlertDto> {
    info!(
        symbol = %request.symbol,
        condition = %request.condition,
        target = %request.target_price,
        "Criando alerta de preco..."
    );

    let condition = match request.condition.as_str() {
        "above" => PriceAlertCondition::Above,
        "below" => PriceAlertCondition::Below,
        "cross_above" => PriceAlertCondition::CrossAbove,
        "cross_below" => PriceAlertCondition::CrossBelow,
        "percent_up" => PriceAlertCondition::PercentUp,
        "percent_down" => PriceAlertCondition::PercentDown,
        _ => {
            return Err(CommandError {
                code: "INVALID_CONDITION".into(),
                message: format!("Condicao invalida: {}", request.condition),
            })
        }
    };

    let mut alert = PriceAlert::new(&request.symbol, condition, request.target_price);
    alert.message = request.message;
    alert.recurring = request.recurring;
    alert.percent = request.percent;

    let id = state.market_data.create_alert(alert.clone());

    Ok(PriceAlertDto {
        id: id.to_string(),
        symbol: alert.symbol,
        condition: request.condition,
        target_price: alert.target_price,
        percent: alert.percent,
        status: "active".into(),
        message: alert.message,
        recurring: alert.recurring,
        trigger_count: 0,
        created_at: alert.created_at.to_rfc3339(),
        triggered_at: None,
    })
}

/// Remove um alerta
#[tauri::command]
pub async fn delete_price_alert(state: State<'_, AppState>, id: String) -> CommandResult<bool> {
    info!(alert_id = %id, "Removendo alerta de preco...");
    Ok(state.market_data.remove_alert(&id))
}

/// Desabilita um alerta
#[tauri::command]
pub async fn disable_price_alert(state: State<'_, AppState>, id: String) -> CommandResult<bool> {
    info!(alert_id = %id, "Desabilitando alerta de preco...");
    Ok(state.market_data.disable_alert(&id))
}

/// Reativa um alerta
#[tauri::command]
pub async fn enable_price_alert(state: State<'_, AppState>, id: String) -> CommandResult<bool> {
    info!(alert_id = %id, "Reativando alerta de preco...");
    Ok(state.market_data.reactivate_alert(&id))
}

// =============================================================================
// Fill History (Execucoes)
// =============================================================================

/// DTO para fill/execucao do historico
#[derive(Debug, Serialize)]
pub struct FillDto {
    pub id: String,
    pub exchange: String,
    pub symbol: String,
    pub side: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub fee: Decimal,
    pub fee_asset: String,
    pub realized_pnl: Option<Decimal>,
    pub timestamp: String,
}

/// Busca historico de fills/execucoes das exchanges
#[tauri::command]
pub async fn get_fill_history(
    state: State<'_, AppState>,
    symbol: Option<String>,
    limit: Option<u32>,
) -> CommandResult<Vec<FillDto>> {
    info!(symbol = ?symbol, limit = ?limit, "Buscando historico de fills...");

    let mut all_fills: Vec<FillDto> = Vec::new();
    let limit = limit.unwrap_or(100) as usize;
    let mut exchanges_checked = 0;

    // Busca fills da Kraken
    {
        let guard = state.exchange.kraken_client().await;
        if let Some(ref client) = *guard {
            exchanges_checked += 1;
            info!("Buscando fills da Kraken Futures...");
            match client.get_fills(None).await {
                Ok(fills) => {
                    let kraken_count = fills.len();
                    for fill in fills {
                        // Filtrar por símbolo se especificado
                        if let Some(ref sym) = symbol {
                            if !fill.symbol.contains(sym) {
                                continue;
                            }
                        }

                        let fee = fill
                            .fee_paid
                            .as_ref()
                            .and_then(|f| f.parse::<Decimal>().ok())
                            .unwrap_or(Decimal::ZERO);

                        all_fills.push(FillDto {
                            id: fill.fill_id.clone(),
                            exchange: "kraken_futures".to_string(),
                            symbol: fill.symbol.clone(),
                            side: fill.side.clone(),
                            price: fill.price.parse().unwrap_or(Decimal::ZERO),
                            quantity: fill.size.parse().unwrap_or(Decimal::ZERO),
                            fee,
                            fee_asset: fill.fee_currency.clone().unwrap_or_default(),
                            realized_pnl: None,
                            timestamp: fill.fill_time.clone(),
                        });
                    }
                    info!(total = kraken_count, added = all_fills.len(), "Fills Kraken carregados");
                }
                Err(e) => {
                    warn!("Erro ao buscar fills Kraken: {}", e);
                }
            }
        } else {
            debug!("Cliente Kraken não inicializado - credenciais não configuradas");
        }
    }

    // Busca fills da Binance
    {
        let guard = state.exchange.binance_client().await;
        if let Some(ref client) = *guard {
            exchanges_checked += 1;
            // Binance requer símbolo específico, então busca para símbolos comuns
            let symbols = if let Some(ref sym) = symbol {
                vec![sym.clone()]
            } else {
                // Lista expandida de pares populares
                vec![
                    "BTCUSDT".to_string(),
                    "ETHUSDT".to_string(),
                    "BNBUSDT".to_string(),
                    "SOLUSDT".to_string(),
                    "XRPUSDT".to_string(),
                    "DOGEUSDT".to_string(),
                    "ADAUSDT".to_string(),
                    "AVAXUSDT".to_string(),
                    "LINKUSDT".to_string(),
                    "DOTUSDT".to_string(),
                ]
            };

            info!(symbols = ?symbols, "Buscando fills da Binance Futures...");

            for sym in symbols {
                match client.get_user_trades(&sym, None, None, Some(50)).await {
                    Ok(trades) => {
                        let binance_count = trades.len();
                        for trade in trades {
                            let pnl = trade.realized_pnl.parse::<Decimal>().ok();
                            let fee = trade.commission.parse::<Decimal>().unwrap_or(Decimal::ZERO);

                            all_fills.push(FillDto {
                                id: trade.id.to_string(),
                                exchange: "binance_futures".to_string(),
                                symbol: trade.symbol.clone(),
                                side: trade.side.clone(),
                                price: trade.price.parse().unwrap_or(Decimal::ZERO),
                                quantity: trade.qty.parse().unwrap_or(Decimal::ZERO),
                                fee,
                                fee_asset: trade.commission_asset.clone(),
                                realized_pnl: pnl,
                                timestamp: chrono::DateTime::from_timestamp_millis(trade.time)
                                    .map(|dt| dt.to_rfc3339())
                                    .unwrap_or_default(),
                            });
                        }
                        if binance_count > 0 {
                            debug!(symbol = %sym, count = binance_count, "Trades Binance encontrados");
                        }
                    }
                    Err(e) => {
                        // Não loga erro para símbolos sem trades (comum)
                        debug!(symbol = %sym, error = %e, "Erro ao buscar trades Binance");
                    }
                }
            }
            info!(total = all_fills.len(), "Fills Binance carregados");
        } else {
            debug!("Cliente Binance não inicializado - credenciais não configuradas");
        }
    }

    // Ordena por timestamp (mais recentes primeiro)
    all_fills.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // Limita resultados
    all_fills.truncate(limit);

    info!(
        exchanges_checked = exchanges_checked,
        total_fills = all_fills.len(),
        "Historico de fills carregado"
    );

    if exchanges_checked == 0 {
        debug!("Nenhuma exchange configurada - configure credenciais de API no arquivo .env");
    }

    Ok(all_fills)
}

// =============================================================================
// User Preferences
// =============================================================================

/// DTO para preferências do usuário
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferencesDto {
    pub default_exchange: String,
    pub default_currency: String,
}

/// Retorna preferências do usuário
#[tauri::command]
pub async fn get_user_preferences() -> CommandResult<UserPreferencesDto> {
    debug!("Buscando preferências do usuário...");

    let config = AppConfig::load()
        .await
        .map_err(|e| format!("Erro ao carregar configuração: {}", e))?;

    Ok(UserPreferencesDto {
        default_exchange: config.user_preferences.default_exchange,
        default_currency: config.user_preferences.default_currency,
    })
}

/// Salva preferências do usuário
#[tauri::command]
pub async fn save_user_preferences(prefs: UserPreferencesDto) -> CommandResult<()> {
    info!(
        exchange = %prefs.default_exchange,
        currency = %prefs.default_currency,
        "Salvando preferências do usuário..."
    );

    let mut config = AppConfig::load()
        .await
        .map_err(|e| format!("Erro ao carregar configuração: {}", e))?;

    config.user_preferences.default_exchange = prefs.default_exchange;
    config.user_preferences.default_currency = prefs.default_currency;

    config
        .save()
        .await
        .map_err(|e| format!("Erro ao salvar configuração: {}", e).into())
}

/// Retorna lista de moedas disponíveis
#[tauri::command]
pub async fn get_available_currencies(state: State<'_, AppState>) -> CommandResult<Vec<String>> {
    debug!("Buscando moedas disponíveis...");

    // Busca saldos de todas as exchanges para extrair as moedas
    let all_balances = state.exchange.get_all_balances().await;

    let mut currencies: Vec<String> = all_balances
        .values()
        .flat_map(|balances| balances.iter().map(|b| b.asset.clone()))
        .collect();

    // Remove duplicatas e ordena
    currencies.sort();
    currencies.dedup();

    // Adiciona as principais moedas de trading se não estiverem presentes
    for currency in ["USDT", "BTC", "ETH", "USD"] {
        if !currencies.contains(&currency.to_string()) {
            currencies.push(currency.to_string());
        }
    }

    currencies.sort();
    Ok(currencies)
}
