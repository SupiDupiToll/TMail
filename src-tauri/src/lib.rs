//! Tauri shell for TMail.
//!
//! Responsibilities kept here, and only here: window lifecycle, tray, native
//! dialogs, notifications, `mailto:` arguments and forwarding core events to
//! the renderer. Mail logic lives in `tmail-core`.

mod commands;
mod mailto;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager, WindowEvent};
use tauri_plugin_notification::{Action, ActionType, NotificationExt};
use tmail_core::CoreService;

use commands::AppState;

pub const NOTIFICATION_ACTION_TYPE: &str = "tmail-mail";

pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            commands::focus_main(app);
            commands::emit_mailto(app, &argv);
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let data_dir = std::env::var("TMAIL_DATA_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| tmail_core::default_data_dir());
            let api_url = std::env::var("TMAIL_API_URL").ok();
            let service = Arc::new(
                CoreService::new(data_dir, api_url)
                    .map_err(|error| format!("could not start the mail core: {error}"))?,
            );
            app.manage(AppState {
                service: Arc::clone(&service),
                quitting: AtomicBool::new(false),
                tray_available: AtomicBool::new(false),
                pending_mailto: std::sync::Mutex::new(None),
            });

            match build_tray(app.handle()) {
                Ok(()) => {
                    app.state::<AppState>()
                        .tray_available
                        .store(true, Ordering::SeqCst);
                }
                Err(error) => {
                    // Without a tray, close-to-tray would strand the window.
                    log::warn!("tray icon unavailable: {error}");
                }
            }
            register_notification_actions(app.handle());
            forward_events(app.handle().clone(), service);

            let args: Vec<String> = std::env::args().collect();
            commands::store_mailto(app.handle(), &args);
            commands::focus_main(app.handle());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<AppState>();
                if state.tray_available.load(Ordering::SeqCst)
                    && !state.quitting.load(Ordering::SeqCst)
                {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::login,
            commands::submit_totp,
            commands::cancel_login,
            commands::last_account,
            commands::resume,
            commands::snapshot,
            commands::logout,
            commands::list_folders,
            commands::classify_recipients,
            commands::load_contacts,
            commands::send_notification,
            commands::unread_counts,
            commands::open_folder,
            commands::refresh_folder,
            commands::open_mail,
            commands::load_thread,
            commands::search,
            commands::set_unread,
            commands::move_mails,
            commands::trash,
            commands::archive,
            commands::delete_forever,
            commands::apply_labels,
            commands::set_current_folder,
            commands::clear_cache,
            commands::send_mail,
            commands::save_draft,
            commands::save_attachment,
            commands::read_attachment,
            commands::open_external,
            commands::quit_app,
            commands::take_pending_mailto,
        ]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running TMail");
}

fn build_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open TMail", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit TMail", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;

    TrayIconBuilder::with_id("tmail-tray")
        .icon(
            app.default_window_icon()
                .cloned()
                .expect("the app icon is bundled"),
        )
        .tooltip("TMail — unofficial Tuta Mail client")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => commands::focus_main(app),
            "quit" => {
                let state = app.state::<AppState>();
                state.quitting.store(true, Ordering::SeqCst);
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                commands::focus_main(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

fn register_notification_actions(app: &tauri::AppHandle) {
    let open = Action::builder("open", "Open email").build();
    let action_type = ActionType::builder(NOTIFICATION_ACTION_TYPE)
        .actions(vec![open])
        .build();
    if let Err(error) = app.notification().register_action_types(vec![action_type]) {
        log::warn!("could not register notification actions: {error}");
    }
    let handle = app.clone();
    if let Err(error) = app.notification().on_action(move |performed| {
        if performed.action_id() == "dismiss" {
            return;
        }
        commands::focus_main(&handle);
        if let Some(notification) = performed.notification() {
            if let Some(mail_id) = notification
                .extra()
                .get("mail_id")
                .and_then(|value| value.as_str())
            {
                let _ = handle.emit("open-mail", mail_id.to_string());
            }
        }
    }) {
        log::warn!("could not register notification action handler: {error}");
    }
}

fn forward_events(app: tauri::AppHandle, service: Arc<CoreService>) {
    let mut receiver = service.subscribe();
    tauri::async_runtime::spawn(async move {
        loop {
            match receiver.recv().await {
                Ok(event) => {
                    let _ = app.emit("sync-event", event);
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}
