//! RoboTrade - Aplicação Desktop Tauri
//!
//! Este módulo contém a lógica principal da aplicação Tauri,
//! incluindo comandos, estado e integração com o system tray.

mod commands;
mod exchange_service;
mod state;
mod trading_worker;
mod tray;

pub use commands::*;
pub use exchange_service::*;
pub use state::*;
pub use trading_worker::*;
pub use tray::*;

use robotrade_core::traits::FearGreedProvider;
use robotrade_infra::database::{self, DatabaseConfig};
use robotrade_market_data::AlternativeMeFearGreedProvider;
use tauri::Manager;
use tracing::{error, info, warn};

/// Configura e executa a aplicação Tauri
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
            // Trading Worker
            commands::start_trading_worker,
            commands::stop_trading_worker,
            commands::is_worker_running,
            commands::get_risk_stats,
            commands::set_trading_enabled,
            commands::reset_daily_losses,
        ])
        .run(tauri::generate_context!())
        .expect("Erro ao executar aplicação Tauri");
}
