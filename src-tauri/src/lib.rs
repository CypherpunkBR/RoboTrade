//! RoboTrade - Aplicação Desktop Tauri
//!
//! Este módulo contém a lógica principal da aplicação Tauri,
//! incluindo comandos, estado e integração com o system tray.

mod commands;
mod exchange_service;
mod ledger_commands;
mod ledger_state;
mod market_data_service;
mod pnl_state;
mod reconciliation_state;
mod state;
mod sync_state;
mod trading_worker;
mod tray;

pub use commands::*;
pub use exchange_service::*;
pub use ledger_commands::*;
pub use ledger_state::*;
pub use market_data_service::*;
pub use pnl_state::*;
pub use reconciliation_state::*;
pub use state::*;
pub use sync_state::*;
pub use trading_worker::*;
pub use tray::*;

use robotrade_core::traits::FearGreedProvider;
use robotrade_infra::database::{self, DatabaseConfig};
use robotrade_market_data::AlternativeMeFearGreedProvider;
use tauri::Manager;
use tracing::{error, info, warn};

/// Carrega variáveis de ambiente do arquivo .env
fn load_dotenv() {
  // Tenta carregar o .env do diretório atual ou do diretório pai
  match dotenvy::dotenv() {
    Ok(path) => {
      info!("Arquivo .env carregado de: {:?}", path);
    }
    Err(e) => {
      // Não é erro crítico se não encontrar o arquivo
      warn!(
        "Não foi possível carregar .env: {} (isso é normal em produção)",
        e
      );
    }
  }
}

/// Inicializa exchanges a partir das variáveis de ambiente (async)
async fn initialize_exchanges(app_state: &AppState) {
  // Binance Futures
  if let (Ok(api_key), Ok(api_secret)) = (
    std::env::var("BINANCE_API_KEY"),
    std::env::var("BINANCE_API_SECRET"),
  ) {
    if !api_key.is_empty() && !api_secret.is_empty() {
      let is_testnet = std::env::var("BINANCE_TESTNET")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(true);

      info!(testnet = is_testnet, "Inicializando Binance Futures...");
      app_state
        .exchange
        .initialize_binance_async(ExchangeCredentials {
          api_key,
          api_secret,
          is_testnet,
        })
        .await;
      app_state.set_binance_connected(true);
    }
  }

  // Kraken Futures
  if let (Ok(api_key), Ok(api_secret)) = (
    std::env::var("KRAKEN_FUTURES_API_KEY"),
    std::env::var("KRAKEN_FUTURES_API_SECRET"),
  ) {
    if !api_key.is_empty() && !api_secret.is_empty() {
      let is_demo = std::env::var("KRAKEN_FUTURES_DEMO")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(true);

      info!(demo = is_demo, "Inicializando Kraken Futures...");
      app_state
        .exchange
        .initialize_kraken_async(ExchangeCredentials {
          api_key,
          api_secret,
          is_testnet: is_demo,
        })
        .await;
      app_state.set_kraken_connected(true);
    }
  }
}

/// Configura e executa a aplicação Tauri
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  // Carrega variáveis de ambiente do .env ANTES de inicializar logging
  load_dotenv();

  // Inicializa logging
  tracing_subscriber::fmt()
    .with_env_filter("robotrade=debug,info")
    .init();

  info!("Iniciando RoboTrade...");

  tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_notification::init())
    .setup(|app| {
      info!("Configurando aplicação...");

      // Inicializa estado da aplicação
      let app_state = AppState::new();

      // Inicializa exchanges a partir de variáveis de ambiente
      let state_for_exchanges = app_state.clone();
      tauri::async_runtime::spawn(async move {
        initialize_exchanges(&state_for_exchanges).await;
      });

      // Inicializa banco de dados
      let state_clone = app_state.clone();
      tauri::async_runtime::spawn(async move {
        let config = DatabaseConfig::default();
        match database::init_database(&config).await {
          Ok(pool) => {
            info!("Banco de dados inicializado com sucesso");
            state_clone.set_db_pool(pool);
            state_clone.set_database_connected(true);
          }
          Err(e) => {
            error!("Erro ao inicializar banco de dados: {}", e);
          }
        }
      });

      // Inicia coleta inicial do Fear & Greed Index
      let state_for_fg = app_state.clone();
      tauri::async_runtime::spawn(async move {
        info!("Iniciando coleta do Fear & Greed Index...");
        let provider = AlternativeMeFearGreedProvider::new();

        match provider.fetch_current().await {
          Ok(data) => {
            info!(
                value = data.value,
                classification = ?data.classification,
                "Fear & Greed Index atualizado"
            );
            state_for_fg.set_fear_greed(data);
            state_for_fg.set_fear_greed_api_connected(true);
          }
          Err(e) => {
            warn!("Erro ao buscar Fear & Greed: {}", e);
          }
        }

        // Loop de atualização periódica (a cada 30 minutos)
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30 * 60));
        interval.tick().await; // Skip first tick

        loop {
          interval.tick().await;
          match provider.fetch_current().await {
            Ok(data) => {
              info!(
                  value = data.value,
                  classification = ?data.classification,
                  "Fear & Greed Index atualizado"
              );
              state_for_fg.set_fear_greed(data);
              state_for_fg.set_fear_greed_api_connected(true);
            }
            Err(e) => {
              warn!("Erro ao atualizar Fear & Greed: {}", e);
              state_for_fg.set_fear_greed_api_connected(false);
            }
          }
        }
      });

      app.manage(app_state);

      // Configura system tray
      setup_tray(app)?;

      info!("RoboTrade iniciado com sucesso!");
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      // Dashboard
      commands::get_dashboard_summary,
      commands::get_connection_status,
      // Fear & Greed
      commands::get_fear_greed_current,
      commands::get_fear_greed_history,
      // Posições e ordens
      commands::get_positions,
      commands::get_open_orders,
      commands::get_balances,
      // Trading
      commands::create_order,
      commands::cancel_order,
      // Configuração
      commands::get_config,
      commands::save_config,
      commands::get_trading_mode,
      commands::set_trading_mode,
      // Histórico
      commands::get_trade_history,
      commands::get_trade_stats,
      commands::get_fill_history,
      // Trading Worker
      commands::start_trading_worker,
      commands::stop_trading_worker,
      commands::is_worker_running,
      commands::get_risk_stats,
      commands::set_trading_enabled,
      commands::reset_daily_losses,
      // Market Data - Klines
      commands::get_klines,
      commands::get_klines_range,
      commands::get_klines_history,
      commands::calculate_indicators,
      // Price Alerts
      commands::list_price_alerts,
      commands::create_price_alert,
      commands::delete_price_alert,
      commands::disable_price_alert,
      commands::enable_price_alert,
      // User Preferences
      commands::get_user_preferences,
      commands::save_user_preferences,
      commands::get_available_currencies,
      // Ledger
      ledger_commands::get_ledger_entries,
      ledger_commands::get_ledger_balance,
      ledger_commands::get_all_ledger_balances,
      ledger_commands::get_ledger_summary,
      ledger_commands::count_ledger_entries,
      // P&L
      ledger_commands::get_open_tax_lots,
      ledger_commands::get_realized_pnl,
      ledger_commands::calculate_unrealized_pnl,
      ledger_commands::get_pnl_summary,
      ledger_commands::set_cost_basis_method,
      // Sync
      ledger_commands::start_sync,
      ledger_commands::stop_sync,
      ledger_commands::pause_sync,
      ledger_commands::resume_sync,
      ledger_commands::get_sync_state,
      ledger_commands::get_all_sync_states,
      ledger_commands::reset_sync,
      // Reconciliation
      ledger_commands::run_reconciliation,
      ledger_commands::get_recent_reconciliations,
      ledger_commands::get_reconciliation_snapshot,
      ledger_commands::get_discrepancy_history,
      ledger_commands::get_reconciliation_health,
      // Reports
      ledger_commands::generate_tax_report,
      ledger_commands::generate_monthly_report,
      ledger_commands::generate_symbol_report,
      ledger_commands::export_report_csv,
    ])
    .run(tauri::generate_context!())
    .expect("Erro ao executar aplicação Tauri");
}
