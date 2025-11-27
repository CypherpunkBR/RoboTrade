//! System Tray do RoboTrade

use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Manager, Runtime,
};
use tracing::{debug, error, info};

/// Configura o system tray
pub fn setup_tray<R: Runtime>(app: &App<R>) -> Result<(), Box<dyn std::error::Error>> {
    info!("Configurando System Tray...");

    // Cria itens do menu
    let show_item = MenuItem::with_id(app, "show", "Mostrar RoboTrade", true, None::<&str>)?;
    let status_item =
        MenuItem::with_id(app, "status", "Status: Paper Trading", false, None::<&str>)?;
    let separator1 = MenuItem::with_id(app, "sep1", "──────────────", false, None::<&str>)?;
    let pnl_item = MenuItem::with_id(app, "pnl", "P&L Hoje: $0.00", false, None::<&str>)?;
    let positions_item = MenuItem::with_id(app, "positions", "Posições: 0", false, None::<&str>)?;
    let fear_greed_item =
        MenuItem::with_id(app, "fear_greed", "Fear & Greed: --", false, None::<&str>)?;
    let separator2 = MenuItem::with_id(app, "sep2", "──────────────", false, None::<&str>)?;
    let pause_item = MenuItem::with_id(app, "pause", "Pausar Trading", true, None::<&str>)?;
    let close_all_item =
        MenuItem::with_id(app, "close_all", "Fechar Posições", true, None::<&str>)?;
    let separator3 = MenuItem::with_id(app, "sep3", "──────────────", false, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;

    // Cria menu
    let menu = Menu::with_items(
        app,
        &[
            &show_item,
            &status_item,
            &separator1,
            &pnl_item,
            &positions_item,
            &fear_greed_item,
            &separator2,
            &pause_item,
            &close_all_item,
            &separator3,
            &quit_item,
        ],
    )?;

    // Carrega o ícone do tray (embutido em compile time)
    // O ícone é carregado de src-tauri/icons/icon.png
    let icon = include_bytes!("../icons/icon.png");
    let icon_image = Image::from_bytes(icon).map_err(|e| {
        error!("Erro ao carregar ícone do tray: {}", e);
        e
    })?;

    // Cria tray icon
    let _tray = TrayIconBuilder::new()
        .icon(icon_image)
        .menu(&menu)
        .tooltip("RoboTrade - Paper Trading")
        .on_menu_event(|app, event| {
            debug!("Menu event: {:?}", event.id.as_ref());

            match event.id.as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "pause" => {
                    info!("Pausando trading...");
                    // TODO: Implementar pausa
                }
                "close_all" => {
                    info!("Fechando todas as posições...");
                    // TODO: Implementar fechamento de posições
                }
                "quit" => {
                    info!("Encerrando RoboTrade...");
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    info!("System Tray configurado com sucesso");
    Ok(())
}

/// Atualiza o tooltip do tray com informações atuais
pub fn update_tray_tooltip<R: Runtime>(
    _app: &tauri::AppHandle<R>,
    mode: &str,
    pnl: &str,
    positions: u32,
) {
    let tooltip = format!(
        "RoboTrade - {}\nP&L: {}\nPosições: {}",
        mode, pnl, positions
    );

    // TODO: Atualizar tooltip do tray
    debug!("Tooltip atualizado: {}", tooltip);
}
