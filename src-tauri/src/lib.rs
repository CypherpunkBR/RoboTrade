//! RoboTrade - Aplicação Desktop Tauri
//!
//! Este módulo contém a lógica principal da aplicação Tauri,
//! incluindo comandos, estado e integração com o system tray.

mod commands;
mod state;
mod tray;

pub use commands::*;
pub use state::*;
pub use tray::*;

use tauri::Manager;
use tracing::info;

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
        ])
        .run(tauri::generate_context!())
        .expect("Erro ao executar aplicação Tauri");
}
