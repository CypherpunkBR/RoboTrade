//! Comandos Tauri para Ledger, P&L, Sync e Reconciliation
//!
//! Novos comandos para o sistema de ledger financeiro:
//! - Consulta e gestão de entradas do ledger
//! - Cálculo de P&L (FIFO/Average Cost)
//! - Sincronização de dados de exchanges
//! - Reconciliação de saldos

use chrono::{DateTime, Utc};
use robotrade_trading_worker::{
    ledger_service::LedgerFilters,
    pnl_calculator::{CostBasisMethod, PositionSide},
    sync_engine::SyncDataType,
    ws_manager::Exchange,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, info};

use crate::commands::CommandError;
use crate::state::AppState;

type CommandResult<T> = Result<T, CommandError>;

/// Converte string de exchange para enum Exchange
fn parse_exchange(exchange: &str) -> Exchange {
    match exchange.to_lowercase().as_str() {
        "binance" | "binance_futures" => Exchange::Binance,
        "kraken" | "kraken_futures" => Exchange::Kraken,
        // Default to Binance for unknown exchanges
        _ => Exchange::Binance,
    }
}

/// Converte Exchange para string
fn exchange_to_string(exchange: Exchange) -> String {
    match exchange {
        Exchange::Binance => "binance_futures".to_string(),
        Exchange::Kraken => "kraken_futures".to_string(),
    }
}

/// Converte string de side para PositionSide
fn parse_position_side(side: &str) -> PositionSide {
    match side.to_lowercase().as_str() {
        "long" | "buy" => PositionSide::Long,
        "short" | "sell" => PositionSide::Short,
        _ => PositionSide::Long,
    }
}

/// Converte string de data type para SyncDataType
fn parse_sync_data_type(data_type: &str) -> SyncDataType {
    match data_type.to_lowercase().as_str() {
        "trades" => SyncDataType::Trades,
        "orders" => SyncDataType::Orders,
        "positions" => SyncDataType::Positions,
        "balances" => SyncDataType::Balances,
        "deposits" => SyncDataType::Deposits,
        "withdrawals" => SyncDataType::Withdrawals,
        "funding" => SyncDataType::Funding,
        _ => SyncDataType::Trades,
    }
}

/// Verifica se um lote é de longo prazo (> 1 ano)
fn is_long_term(acquired_at: DateTime<Utc>) -> bool {
    let one_year_ago = Utc::now() - chrono::Duration::days(365);
    acquired_at < one_year_ago
}

// =============================================================================
// DTOs para Ledger
// =============================================================================

/// DTO para entrada do ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntryDto {
    pub id: String,
    pub exchange: String,
    pub entry_type: String,
    pub asset: String,
    pub amount: Decimal,
    pub balance_after: Decimal,
    pub reference_id: Option<String>,
    pub reference_type: Option<String>,
    pub description: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// DTO para saldo de ativo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetBalanceDto {
    pub exchange: String,
    pub asset: String,
    pub balance: Decimal,
    pub last_updated: DateTime<Utc>,
}

/// DTO para resumo do ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerSummaryDto {
    pub exchange: String,
    pub asset: String,
    pub total_deposits: Decimal,
    pub total_withdrawals: Decimal,
    pub total_realized_pnl: Decimal,
    pub total_funding: Decimal,
    pub total_commissions: Decimal,
    pub net_change: Decimal,
    pub current_balance: Decimal,
    pub period_start: Option<DateTime<Utc>>,
    pub period_end: Option<DateTime<Utc>>,
}

/// Request para filtrar ledger
#[derive(Debug, Clone, Deserialize)]
pub struct LedgerFilterRequest {
    pub exchange: Option<String>,
    pub entry_type: Option<String>,
    pub asset: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub min_amount: Option<Decimal>,
    pub max_amount: Option<Decimal>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

// =============================================================================
// DTOs para P&L
// =============================================================================

/// DTO para lote de custo (tax lot)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxLotDto {
    pub id: String,
    pub exchange: String,
    pub symbol: String,
    pub side: String,
    pub acquired_quantity: Decimal,
    pub remaining_quantity: Decimal,
    pub cost_per_unit: Decimal,
    pub total_cost: Decimal,
    pub acquired_at: DateTime<Utc>,
    pub acquisition_type: String,
    pub trade_id: String,
    pub is_closed: bool,
    pub is_long_term: bool,
}

/// DTO para P&L realizado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealizedPnLDto {
    pub id: String,
    pub exchange: String,
    pub symbol: String,
    pub quantity: Decimal,
    pub proceeds: Decimal,
    pub cost_basis: Decimal,
    pub gain_loss: Decimal,
    pub is_short_term: bool,
    pub is_long_term: bool,
    pub acquired_at: DateTime<Utc>,
    pub disposed_at: DateTime<Utc>,
    pub method: String,
}

/// DTO para P&L não realizado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnrealizedPnLDto {
    pub exchange: String,
    pub symbol: String,
    pub side: String,
    pub quantity: Decimal,
    pub cost_basis: Decimal,
    pub avg_cost_per_unit: Decimal,
    pub current_price: Decimal,
    pub current_value: Decimal,
    pub unrealized_gain_loss: Decimal,
    pub unrealized_pct: Decimal,
    pub oldest_lot_date: Option<DateTime<Utc>>,
    pub short_term_quantity: Decimal,
    pub long_term_quantity: Decimal,
}

/// DTO para resumo de P&L
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnLSummaryDto {
    pub exchange: Option<String>,
    pub symbol: Option<String>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_realized: Decimal,
    pub short_term_gains: Decimal,
    pub short_term_losses: Decimal,
    pub long_term_gains: Decimal,
    pub long_term_losses: Decimal,
    pub net_short_term: Decimal,
    pub net_long_term: Decimal,
    pub total_proceeds: Decimal,
    pub total_cost_basis: Decimal,
    pub num_trades: u64,
}

// =============================================================================
// DTOs para Sync
// =============================================================================

/// DTO para estado de sincronização
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStateDto {
    pub exchange: String,
    pub data_type: String,
    pub status: String,
    pub last_timestamp: Option<DateTime<Utc>>,
    pub items_synced: u64,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub error: Option<String>,
}

/// DTO para progresso de sincronização
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProgressDto {
    pub exchange: String,
    pub data_type: String,
    pub status: String,
    pub items_synced: u64,
    pub total_items: Option<u64>,
    pub current_timestamp: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

/// Request para iniciar sincronização
#[derive(Debug, Clone, Deserialize)]
pub struct StartSyncRequest {
    pub exchange: String,
    pub data_types: Option<Vec<String>>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

// =============================================================================
// DTOs para Reconciliation
// =============================================================================

/// DTO para discrepância
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscrepancyDto {
    pub asset: String,
    pub discrepancy_type: String,
    pub ledger_balance: Decimal,
    pub exchange_balance: Decimal,
    pub difference: Decimal,
    pub difference_pct: Decimal,
}

/// DTO para snapshot de reconciliação
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationSnapshotDto {
    pub id: String,
    pub exchange: String,
    pub status: String,
    pub discrepancies: Vec<DiscrepancyDto>,
    pub total_discrepancy_value: Decimal,
    pub created_at: DateTime<Utc>,
    pub notes: Option<String>,
}

/// DTO para saúde da reconciliação
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationHealthDto {
    pub exchange: String,
    pub status: String,
    pub last_reconciliation: Option<DateTime<Utc>>,
    pub consecutive_discrepancies: u32,
    pub total_discrepancy_value: Decimal,
}

// =============================================================================
// Comandos Ledger
// =============================================================================

/// Lista entradas do ledger
#[tauri::command]
pub async fn get_ledger_entries(
    state: State<'_, AppState>,
    filters: LedgerFilterRequest,
) -> CommandResult<Vec<LedgerEntryDto>> {
    debug!(exchange = ?filters.exchange, asset = ?filters.asset, "Buscando entradas do ledger...");

    if !state.ledger.is_initialized() {
        return Err(CommandError {
            code: "NOT_INITIALIZED".into(),
            message: "LedgerService não inicializado".into(),
        });
    }

    let ledger_filters = LedgerFilters {
        exchange: filters.exchange.as_ref().map(|e| parse_exchange(e)),
        entry_type: None, // TODO: Parse entry_type filter
        asset: filters.asset.clone(),
        start_time: filters.start_time,
        end_time: filters.end_time,
        min_amount: filters.min_amount,
        max_amount: filters.max_amount,
        limit: filters.limit,
        offset: filters.offset,
    };

    let entries = state.ledger.get_entries(&ledger_filters).await;

    Ok(entries
        .into_iter()
        .map(|e| LedgerEntryDto {
            id: e.id,
            exchange: exchange_to_string(e.exchange),
            entry_type: format!("{:?}", e.entry_type),
            asset: e.asset,
            amount: e.amount,
            balance_after: e.balance_after,
            reference_id: e.reference_id,
            reference_type: e.reference_type,
            description: e.description,
            timestamp: e.timestamp,
            created_at: e.created_at,
        })
        .collect())
}

/// Retorna saldo de um ativo específico
#[tauri::command]
pub async fn get_ledger_balance(
    state: State<'_, AppState>,
    exchange: String,
    asset: String,
) -> CommandResult<Decimal> {
    debug!(exchange = %exchange, asset = %asset, "Buscando saldo do ledger...");

    if !state.ledger.is_initialized() {
        return Err(CommandError {
            code: "NOT_INITIALIZED".into(),
            message: "LedgerService não inicializado".into(),
        });
    }

    let ex = parse_exchange(&exchange);
    let balance = state.ledger.get_balance(ex, &asset).await;
    Ok(balance)
}

/// Retorna todos os saldos do ledger
#[tauri::command]
pub async fn get_all_ledger_balances(
    state: State<'_, AppState>,
    exchange: Option<String>,
) -> CommandResult<Vec<AssetBalanceDto>> {
    debug!(exchange = ?exchange, "Buscando todos os saldos do ledger...");

    if !state.ledger.is_initialized() {
        return Err(CommandError {
            code: "NOT_INITIALIZED".into(),
            message: "LedgerService não inicializado".into(),
        });
    }

    let ex = exchange
        .as_ref()
        .map(|e| parse_exchange(e))
        .unwrap_or(Exchange::Binance);

    let balances = state.ledger.get_all_balances(ex).await;

    Ok(balances
        .into_iter()
        .map(|b| AssetBalanceDto {
            exchange: exchange_to_string(b.exchange),
            asset: b.asset,
            balance: b.balance,
            last_updated: b.last_updated,
        })
        .collect())
}

/// Retorna resumo do ledger para um ativo
#[tauri::command]
pub async fn get_ledger_summary(
    state: State<'_, AppState>,
    exchange: String,
    asset: String,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
) -> CommandResult<LedgerSummaryDto> {
    debug!(exchange = %exchange, asset = %asset, "Buscando resumo do ledger...");

    if !state.ledger.is_initialized() {
        return Err(CommandError {
            code: "NOT_INITIALIZED".into(),
            message: "LedgerService não inicializado".into(),
        });
    }

    let ex = parse_exchange(&exchange);

    if let Some(summary) = state.ledger.get_summary(ex, &asset, start_time, end_time).await {
        Ok(LedgerSummaryDto {
            exchange: exchange_to_string(ex),
            asset: summary.asset.clone(),
            total_deposits: summary.total_deposits,
            total_withdrawals: summary.total_withdrawals,
            total_realized_pnl: summary.total_realized_pnl,
            total_funding: summary.total_funding,
            total_commissions: summary.total_commissions,
            net_change: summary.net_change,
            current_balance: summary.current_balance,
            period_start: summary.period_start,
            period_end: summary.period_end,
        })
    } else {
        Ok(LedgerSummaryDto {
            exchange,
            asset,
            total_deposits: Decimal::ZERO,
            total_withdrawals: Decimal::ZERO,
            total_realized_pnl: Decimal::ZERO,
            total_funding: Decimal::ZERO,
            total_commissions: Decimal::ZERO,
            net_change: Decimal::ZERO,
            current_balance: Decimal::ZERO,
            period_start: start_time,
            period_end: end_time,
        })
    }
}

/// Conta entradas do ledger
#[tauri::command]
pub async fn count_ledger_entries(
    state: State<'_, AppState>,
    filters: LedgerFilterRequest,
) -> CommandResult<u64> {
    debug!("Contando entradas do ledger...");

    if !state.ledger.is_initialized() {
        return Err(CommandError {
            code: "NOT_INITIALIZED".into(),
            message: "LedgerService não inicializado".into(),
        });
    }

    let ledger_filters = LedgerFilters {
        exchange: filters.exchange.as_ref().map(|e| parse_exchange(e)),
        entry_type: None,
        asset: filters.asset.clone(),
        start_time: filters.start_time,
        end_time: filters.end_time,
        min_amount: filters.min_amount,
        max_amount: filters.max_amount,
        limit: filters.limit,
        offset: filters.offset,
    };

    let count = state.ledger.count_entries(&ledger_filters).await;
    Ok(count)
}

// =============================================================================
// Comandos P&L
// =============================================================================

/// Retorna lotes de custo abertos
#[tauri::command]
pub async fn get_open_tax_lots(
    state: State<'_, AppState>,
    exchange: String,
    symbol: String,
    side: String,
) -> CommandResult<Vec<TaxLotDto>> {
    debug!(exchange = %exchange, symbol = %symbol, side = %side, "Buscando lotes abertos...");

    if !state.pnl.is_initialized() {
        return Err(CommandError {
            code: "NOT_INITIALIZED".into(),
            message: "PnLCalculator não inicializado".into(),
        });
    }

    let ex = parse_exchange(&exchange);
    let pos_side = parse_position_side(&side);

    let lots = state.pnl.get_open_lots(ex, &symbol, pos_side).await;

    Ok(lots
        .into_iter()
        .map(|lot| TaxLotDto {
            id: lot.id,
            exchange: exchange_to_string(lot.exchange),
            symbol: lot.symbol,
            side: format!("{:?}", lot.side),
            acquired_quantity: lot.acquired_quantity,
            remaining_quantity: lot.remaining_quantity,
            cost_per_unit: lot.cost_per_unit,
            total_cost: lot.total_cost,
            acquired_at: lot.acquired_at,
            acquisition_type: format!("{:?}", lot.acquisition_type),
            trade_id: lot.trade_id,
            is_closed: lot.is_closed,
            is_long_term: is_long_term(lot.acquired_at),
        })
        .collect())
}

/// Retorna P&L realizado
#[tauri::command]
pub async fn get_realized_pnl(
    state: State<'_, AppState>,
    exchange: Option<String>,
    symbol: Option<String>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> CommandResult<Vec<RealizedPnLDto>> {
    debug!(
        exchange = ?exchange,
        symbol = ?symbol,
        start = %start_time,
        end = %end_time,
        "Buscando P&L realizado..."
    );

    if !state.pnl.is_initialized() {
        return Err(CommandError {
            code: "NOT_INITIALIZED".into(),
            message: "PnLCalculator não inicializado".into(),
        });
    }

    let ex = exchange.as_ref().map(|e| parse_exchange(e));
    let sym = symbol.as_deref();

    let pnl_entries = state.pnl.get_realized_pnl(ex, sym, start_time, end_time).await;

    Ok(pnl_entries
        .into_iter()
        .map(|pnl| RealizedPnLDto {
            id: pnl.id,
            exchange: exchange_to_string(pnl.exchange),
            symbol: pnl.symbol,
            quantity: pnl.quantity,
            proceeds: pnl.proceeds,
            cost_basis: pnl.cost_basis,
            gain_loss: pnl.gain_loss,
            is_short_term: pnl.is_short_term,
            is_long_term: pnl.is_long_term,
            acquired_at: pnl.acquired_at,
            disposed_at: pnl.disposed_at,
            method: format!("{:?}", pnl.method),
        })
        .collect())
}

/// Calcula P&L não realizado para uma posição
#[tauri::command]
pub async fn calculate_unrealized_pnl(
    state: State<'_, AppState>,
    exchange: String,
    symbol: String,
    side: String,
    current_price: Decimal,
) -> CommandResult<Option<UnrealizedPnLDto>> {
    debug!(
        exchange = %exchange,
        symbol = %symbol,
        price = %current_price,
        "Calculando P&L não realizado..."
    );

    if !state.pnl.is_initialized() {
        return Err(CommandError {
            code: "NOT_INITIALIZED".into(),
            message: "PnLCalculator não inicializado".into(),
        });
    }

    let ex = parse_exchange(&exchange);
    let pos_side = parse_position_side(&side);

    let unrealized = state.pnl.calculate_unrealized(ex, &symbol, pos_side, current_price).await;

    Ok(unrealized.map(|u| UnrealizedPnLDto {
        exchange: exchange_to_string(u.exchange),
        symbol: u.symbol,
        side: format!("{:?}", u.side),
        quantity: u.quantity,
        cost_basis: u.cost_basis,
        avg_cost_per_unit: u.avg_cost_per_unit,
        current_price: u.current_price,
        current_value: u.current_value,
        unrealized_gain_loss: u.unrealized_gain_loss,
        unrealized_pct: u.unrealized_pct,
        oldest_lot_date: u.oldest_lot_date,
        short_term_quantity: u.short_term_quantity,
        long_term_quantity: u.long_term_quantity,
    }))
}

/// Retorna resumo de P&L
#[tauri::command]
pub async fn get_pnl_summary(
    state: State<'_, AppState>,
    exchange: Option<String>,
    symbol: Option<String>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> CommandResult<PnLSummaryDto> {
    debug!(
        exchange = ?exchange,
        symbol = ?symbol,
        "Buscando resumo de P&L..."
    );

    if !state.pnl.is_initialized() {
        return Err(CommandError {
            code: "NOT_INITIALIZED".into(),
            message: "PnLCalculator não inicializado".into(),
        });
    }

    let ex = exchange.as_ref().map(|e| parse_exchange(e));
    let sym = symbol.as_deref();

    let summary = state.pnl.get_summary(ex, sym, start_time, end_time).await;

    Ok(PnLSummaryDto {
        exchange: exchange.clone(),
        symbol: symbol.clone(),
        period_start: summary.period_start,
        period_end: summary.period_end,
        total_realized: summary.total_realized,
        short_term_gains: summary.short_term_gains,
        short_term_losses: summary.short_term_losses,
        long_term_gains: summary.long_term_gains,
        long_term_losses: summary.long_term_losses,
        net_short_term: summary.net_short_term,
        net_long_term: summary.net_long_term,
        total_proceeds: summary.total_proceeds,
        total_cost_basis: summary.total_cost_basis,
        num_trades: summary.num_trades,
    })
}

/// Define método de custo (FIFO, LIFO, Average)
#[tauri::command]
pub async fn set_cost_basis_method(
    state: State<'_, AppState>,
    method: String,
) -> CommandResult<()> {
    info!(method = %method, "Definindo método de custo...");

    if !state.pnl.is_initialized() {
        return Err(CommandError {
            code: "NOT_INITIALIZED".into(),
            message: "PnLCalculator não inicializado".into(),
        });
    }

    let cost_method = match method.to_lowercase().as_str() {
        "fifo" => CostBasisMethod::FIFO,
        "lifo" => CostBasisMethod::LIFO,
        "average" | "avg" => CostBasisMethod::AverageCost,
        _ => {
            return Err(CommandError {
                code: "INVALID_METHOD".into(),
                message: format!("Método inválido: {}. Use fifo, lifo ou average.", method),
            });
        }
    };

    state.pnl.set_method(cost_method).await;
    info!(method = %method, "Método de custo definido com sucesso");

    Ok(())
}

// =============================================================================
// Comandos Sync
// =============================================================================

/// Inicia sincronização de dados
#[tauri::command]
pub async fn start_sync(
    state: State<'_, AppState>,
    request: StartSyncRequest,
) -> CommandResult<()> {
    info!(
        exchange = %request.exchange,
        data_types = ?request.data_types,
        "Iniciando sincronização..."
    );

    if !state.sync.is_initialized() {
        return Err(CommandError {
            code: "NOT_INITIALIZED".into(),
            message: "SyncEngine não inicializado".into(),
        });
    }

    let exchange = parse_exchange(&request.exchange);
    let data_types = request.data_types.map(|types| {
        types.iter().map(|t| parse_sync_data_type(t)).collect()
    });

    state.sync.start_sync(exchange, data_types, request.start_time, request.end_time)
        .await
        .map_err(|e| CommandError {
            code: "SYNC_ERROR".into(),
            message: e,
        })
}

/// Para a sincronização
#[tauri::command]
pub async fn stop_sync(state: State<'_, AppState>) -> CommandResult<()> {
    info!("Parando sincronização...");

    if !state.sync.is_initialized() {
        return Err(CommandError {
            code: "NOT_INITIALIZED".into(),
            message: "SyncEngine não inicializado".into(),
        });
    }

    state.sync.stop_sync()
        .await
        .map_err(|e| CommandError {
            code: "SYNC_ERROR".into(),
            message: e,
        })
}

/// Pausa a sincronização
#[tauri::command]
pub async fn pause_sync(state: State<'_, AppState>) -> CommandResult<()> {
    info!("Pausando sincronização...");

    if !state.sync.is_initialized() {
        return Err(CommandError {
            code: "NOT_INITIALIZED".into(),
            message: "SyncEngine não inicializado".into(),
        });
    }

    state.sync.pause_sync()
        .await
        .map_err(|e| CommandError {
            code: "SYNC_ERROR".into(),
            message: e,
        })
}

/// Retoma a sincronização
#[tauri::command]
pub async fn resume_sync(state: State<'_, AppState>) -> CommandResult<()> {
    info!("Retomando sincronização...");

    if !state.sync.is_initialized() {
        return Err(CommandError {
            code: "NOT_INITIALIZED".into(),
            message: "SyncEngine não inicializado".into(),
        });
    }

    state.sync.resume_sync()
        .await
        .map_err(|e| CommandError {
            code: "SYNC_ERROR".into(),
            message: e,
        })
}

/// Retorna estado atual de sincronização
#[tauri::command]
pub async fn get_sync_state(
    state: State<'_, AppState>,
    exchange: String,
    data_type: String,
) -> CommandResult<Option<SyncStateDto>> {
    debug!(exchange = %exchange, data_type = %data_type, "Buscando estado de sincronização...");

    if !state.sync.is_initialized() {
        return Err(CommandError {
            code: "NOT_INITIALIZED".into(),
            message: "SyncEngine não inicializado".into(),
        });
    }

    let ex = parse_exchange(&exchange);
    let dt = parse_sync_data_type(&data_type);

    let sync_state = state.sync.get_sync_state(ex, dt).await;

    Ok(sync_state.map(|s| SyncStateDto {
        exchange: exchange_to_string(s.exchange),
        data_type: format!("{:?}", s.data_type),
        status: format!("{:?}", s.status),
        last_timestamp: s.last_timestamp,
        items_synced: s.items_synced,
        started_at: s.started_at,
        updated_at: s.updated_at,
        error: s.error,
    }))
}

/// Retorna estados de sincronização de todos os tipos
#[tauri::command]
pub async fn get_all_sync_states(
    state: State<'_, AppState>,
    exchange: String,
) -> CommandResult<Vec<SyncStateDto>> {
    debug!(exchange = %exchange, "Buscando todos os estados de sincronização...");

    if !state.sync.is_initialized() {
        return Err(CommandError {
            code: "NOT_INITIALIZED".into(),
            message: "SyncEngine não inicializado".into(),
        });
    }

    let ex = parse_exchange(&exchange);
    let sync_states = state.sync.get_all_sync_states(ex).await;

    Ok(sync_states
        .into_iter()
        .map(|s| SyncStateDto {
            exchange: exchange_to_string(s.exchange),
            data_type: format!("{:?}", s.data_type),
            status: format!("{:?}", s.status),
            last_timestamp: s.last_timestamp,
            items_synced: s.items_synced,
            started_at: s.started_at,
            updated_at: s.updated_at,
            error: s.error,
        })
        .collect())
}

/// Reseta sincronização para um tipo de dado
#[tauri::command]
pub async fn reset_sync(
    state: State<'_, AppState>,
    exchange: String,
    data_type: String,
) -> CommandResult<()> {
    info!(exchange = %exchange, data_type = %data_type, "Resetando sincronização...");

    if !state.sync.is_initialized() {
        return Err(CommandError {
            code: "NOT_INITIALIZED".into(),
            message: "SyncEngine não inicializado".into(),
        });
    }

    let ex = parse_exchange(&exchange);
    let dt = parse_sync_data_type(&data_type);

    state.sync.reset_sync(ex, dt)
        .await
        .map_err(|e| CommandError {
            code: "SYNC_ERROR".into(),
            message: e,
        })
}

// =============================================================================
// Comandos Reconciliation
// =============================================================================

/// Executa reconciliação
#[tauri::command]
pub async fn run_reconciliation(
    state: State<'_, AppState>,
    exchange: String,
) -> CommandResult<ReconciliationSnapshotDto> {
    info!(exchange = %exchange, "Executando reconciliação...");

    if !state.reconciliation.is_initialized() {
        return Err(CommandError {
            code: "NOT_INITIALIZED".into(),
            message: "ReconciliationService não inicializado".into(),
        });
    }

    let ex = parse_exchange(&exchange);

    let snapshot = state.reconciliation.run_reconciliation(ex)
        .await
        .map_err(|e| CommandError {
            code: "RECONCILIATION_ERROR".into(),
            message: e,
        })?;

    Ok(ReconciliationSnapshotDto {
        id: snapshot.id,
        exchange: exchange_to_string(snapshot.exchange),
        status: format!("{:?}", snapshot.status),
        discrepancies: snapshot.discrepancies
            .into_iter()
            .map(|d| DiscrepancyDto {
                asset: d.asset,
                discrepancy_type: format!("{:?}", d.discrepancy_type),
                ledger_balance: d.ledger_balance,
                exchange_balance: d.exchange_balance,
                difference: d.difference,
                difference_pct: d.difference_pct,
            })
            .collect(),
        total_discrepancy_value: snapshot.total_discrepancy_value,
        created_at: snapshot.created_at,
        notes: snapshot.notes,
    })
}

/// Retorna snapshots de reconciliação recentes
#[tauri::command]
pub async fn get_recent_reconciliations(
    state: State<'_, AppState>,
    exchange: Option<String>,
    limit: Option<usize>,
) -> CommandResult<Vec<ReconciliationSnapshotDto>> {
    debug!(exchange = ?exchange, limit = ?limit, "Buscando reconciliações recentes...");

    if !state.reconciliation.is_initialized() {
        return Err(CommandError {
            code: "NOT_INITIALIZED".into(),
            message: "ReconciliationService não inicializado".into(),
        });
    }

    let ex = exchange.as_ref().map(|e| parse_exchange(e));
    let lim = limit.unwrap_or(10);

    let snapshots = state.reconciliation.get_recent_snapshots(ex, lim).await;

    Ok(snapshots
        .into_iter()
        .map(|snapshot| ReconciliationSnapshotDto {
            id: snapshot.id,
            exchange: exchange_to_string(snapshot.exchange),
            status: format!("{:?}", snapshot.status),
            discrepancies: snapshot.discrepancies
                .into_iter()
                .map(|d| DiscrepancyDto {
                    asset: d.asset,
                    discrepancy_type: format!("{:?}", d.discrepancy_type),
                    ledger_balance: d.ledger_balance,
                    exchange_balance: d.exchange_balance,
                    difference: d.difference,
                    difference_pct: d.difference_pct,
                })
                .collect(),
            total_discrepancy_value: snapshot.total_discrepancy_value,
            created_at: snapshot.created_at,
            notes: snapshot.notes,
        })
        .collect())
}

/// Retorna snapshot específico
#[tauri::command]
pub async fn get_reconciliation_snapshot(
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<Option<ReconciliationSnapshotDto>> {
    debug!(id = %id, "Buscando snapshot de reconciliação...");

    if !state.reconciliation.is_initialized() {
        return Err(CommandError {
            code: "NOT_INITIALIZED".into(),
            message: "ReconciliationService não inicializado".into(),
        });
    }

    let snapshot = state.reconciliation.get_snapshot(&id).await;

    Ok(snapshot.map(|s| ReconciliationSnapshotDto {
        id: s.id,
        exchange: exchange_to_string(s.exchange),
        status: format!("{:?}", s.status),
        discrepancies: s.discrepancies
            .into_iter()
            .map(|d| DiscrepancyDto {
                asset: d.asset,
                discrepancy_type: format!("{:?}", d.discrepancy_type),
                ledger_balance: d.ledger_balance,
                exchange_balance: d.exchange_balance,
                difference: d.difference,
                difference_pct: d.difference_pct,
            })
            .collect(),
        total_discrepancy_value: s.total_discrepancy_value,
        created_at: s.created_at,
        notes: s.notes,
    }))
}

/// Retorna histórico de discrepâncias
#[tauri::command]
pub async fn get_discrepancy_history(
    state: State<'_, AppState>,
    exchange: Option<String>,
    limit: Option<usize>,
) -> CommandResult<Vec<ReconciliationSnapshotDto>> {
    debug!(exchange = ?exchange, "Buscando histórico de discrepâncias...");

    if !state.reconciliation.is_initialized() {
        return Err(CommandError {
            code: "NOT_INITIALIZED".into(),
            message: "ReconciliationService não inicializado".into(),
        });
    }

    let ex = exchange.as_ref().map(|e| parse_exchange(e));
    let lim = limit.unwrap_or(50);

    // Get recent snapshots and filter only those with discrepancies
    let snapshots = state.reconciliation.get_recent_snapshots(ex, lim).await;

    Ok(snapshots
        .into_iter()
        .filter(|s| !s.discrepancies.is_empty())
        .map(|snapshot| ReconciliationSnapshotDto {
            id: snapshot.id,
            exchange: exchange_to_string(snapshot.exchange),
            status: format!("{:?}", snapshot.status),
            discrepancies: snapshot.discrepancies
                .into_iter()
                .map(|d| DiscrepancyDto {
                    asset: d.asset,
                    discrepancy_type: format!("{:?}", d.discrepancy_type),
                    ledger_balance: d.ledger_balance,
                    exchange_balance: d.exchange_balance,
                    difference: d.difference,
                    difference_pct: d.difference_pct,
                })
                .collect(),
            total_discrepancy_value: snapshot.total_discrepancy_value,
            created_at: snapshot.created_at,
            notes: snapshot.notes,
        })
        .collect())
}

/// Retorna saúde da reconciliação
#[tauri::command]
pub async fn get_reconciliation_health(
    state: State<'_, AppState>,
    exchange: String,
) -> CommandResult<ReconciliationHealthDto> {
    debug!(exchange = %exchange, "Buscando saúde da reconciliação...");

    if !state.reconciliation.is_initialized() {
        return Err(CommandError {
            code: "NOT_INITIALIZED".into(),
            message: "ReconciliationService não inicializado".into(),
        });
    }

    let ex = parse_exchange(&exchange);
    let health = state.reconciliation.get_health(ex).await;

    Ok(ReconciliationHealthDto {
        exchange: exchange_to_string(health.exchange),
        status: format!("{:?}", health.status),
        last_reconciliation: health.last_reconciliation,
        consecutive_discrepancies: health.consecutive_discrepancies,
        total_discrepancy_value: health.total_discrepancy_value,
    })
}

// =============================================================================
// Comandos de Relatórios
// =============================================================================

/// DTO para relatório de taxa (IRS Form 8949 style)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxReportDto {
    pub year: i32,
    pub short_term_sales: Vec<TaxSaleDto>,
    pub long_term_sales: Vec<TaxSaleDto>,
    pub total_short_term_gain: Decimal,
    pub total_long_term_gain: Decimal,
    pub total_proceeds: Decimal,
    pub total_cost_basis: Decimal,
    pub net_gain: Decimal,
}

/// DTO para venda individual no relatório de taxa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxSaleDto {
    pub description: String,
    pub acquired_date: DateTime<Utc>,
    pub sold_date: DateTime<Utc>,
    pub proceeds: Decimal,
    pub cost_basis: Decimal,
    pub gain_loss: Decimal,
    pub holding_period: String,
}

/// Gera relatório de taxas para um ano
#[tauri::command]
pub async fn generate_tax_report(
    state: State<'_, AppState>,
    year: i32,
    exchange: Option<String>,
) -> CommandResult<TaxReportDto> {
    info!(year = year, exchange = ?exchange, "Gerando relatório de taxas...");

    // TODO: Implementar quando PnLCalculator estiver integrado
    Ok(TaxReportDto {
        year,
        short_term_sales: Vec::new(),
        long_term_sales: Vec::new(),
        total_short_term_gain: Decimal::ZERO,
        total_long_term_gain: Decimal::ZERO,
        total_proceeds: Decimal::ZERO,
        total_cost_basis: Decimal::ZERO,
        net_gain: Decimal::ZERO,
    })
}

/// DTO para relatório mensal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyReportDto {
    pub year: i32,
    pub month: u32,
    pub exchange: Option<String>,
    pub starting_balance: Decimal,
    pub ending_balance: Decimal,
    pub deposits: Decimal,
    pub withdrawals: Decimal,
    pub realized_pnl: Decimal,
    pub funding_payments: Decimal,
    pub commissions: Decimal,
    pub net_change: Decimal,
    pub num_trades: u64,
    pub win_rate: Decimal,
}

/// Gera relatório mensal
#[tauri::command]
pub async fn generate_monthly_report(
    state: State<'_, AppState>,
    year: i32,
    month: u32,
    exchange: Option<String>,
) -> CommandResult<MonthlyReportDto> {
    info!(year = year, month = month, exchange = ?exchange, "Gerando relatório mensal...");

    // TODO: Implementar quando LedgerService e PnLCalculator estiverem integrados
    Ok(MonthlyReportDto {
        year,
        month,
        exchange,
        starting_balance: Decimal::ZERO,
        ending_balance: Decimal::ZERO,
        deposits: Decimal::ZERO,
        withdrawals: Decimal::ZERO,
        realized_pnl: Decimal::ZERO,
        funding_payments: Decimal::ZERO,
        commissions: Decimal::ZERO,
        net_change: Decimal::ZERO,
        num_trades: 0,
        win_rate: Decimal::ZERO,
    })
}

/// DTO para relatório por símbolo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolReportDto {
    pub symbol: String,
    pub exchange: Option<String>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub num_trades: u64,
    pub total_volume: Decimal,
    pub realized_pnl: Decimal,
    pub funding_payments: Decimal,
    pub commissions: Decimal,
    pub win_rate: Decimal,
    pub avg_trade_duration_secs: i64,
    pub largest_win: Decimal,
    pub largest_loss: Decimal,
}

/// Gera relatório por símbolo
#[tauri::command]
pub async fn generate_symbol_report(
    state: State<'_, AppState>,
    symbol: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    exchange: Option<String>,
) -> CommandResult<SymbolReportDto> {
    info!(symbol = %symbol, exchange = ?exchange, "Gerando relatório por símbolo...");

    // TODO: Implementar
    Ok(SymbolReportDto {
        symbol,
        exchange,
        period_start: start_time,
        period_end: end_time,
        num_trades: 0,
        total_volume: Decimal::ZERO,
        realized_pnl: Decimal::ZERO,
        funding_payments: Decimal::ZERO,
        commissions: Decimal::ZERO,
        win_rate: Decimal::ZERO,
        avg_trade_duration_secs: 0,
        largest_win: Decimal::ZERO,
        largest_loss: Decimal::ZERO,
    })
}

/// Exporta relatório para CSV
#[tauri::command]
pub async fn export_report_csv(
    state: State<'_, AppState>,
    report_type: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    exchange: Option<String>,
) -> CommandResult<String> {
    info!(
        report_type = %report_type,
        exchange = ?exchange,
        "Exportando relatório para CSV..."
    );

    // TODO: Implementar exportação CSV
    // Retorna caminho do arquivo gerado
    Err(CommandError {
        code: "NOT_IMPLEMENTED".into(),
        message: "Exportação CSV ainda não implementada".into(),
    })
}
