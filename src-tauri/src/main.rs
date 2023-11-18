// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod csgo;

use std::sync::Arc;

use csgo::ScreenAndDynamicOffsets;
use tauri::{
    async_runtime::{self, RwLock},
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
async fn start(
    state: tauri::State<'_, AppStateType>,
    app_handle: AppHandle,
    data: ScreenAndDynamicOffsets,
) -> Result<(), ()> {
    {
        let mut state = state.0.write().await;

        if state.is_reading_memory {
            return Ok(());
        }
        state.is_reading_memory = true;
    }

    let state = AppStateType(state.0.clone());
    async_runtime::spawn(async move {
        csgo::start(state, app_handle, data).await;
    });

    Ok(())
}

#[tauri::command]
async fn stop(state: tauri::State<'_, AppStateType>) -> Result<(), ()> {
    {
        let mut state = state.0.write().await;

        if state.is_reading_memory {
            state.is_reading_memory = false;
        }
    }

    std::thread::sleep(std::time::Duration::from_millis(500));

    Ok(())
}

pub struct AppStateType(Arc<RwLock<MyAppState>>);

pub struct MyAppState {
    pub is_reading_memory: bool,
}

fn main() {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let show_all = CustomMenuItem::new("show_all".to_string(), "Open Settings");
    let tray_menu = SystemTrayMenu::new()
        .add_item(quit)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(show_all);

    let tray = SystemTray::new().with_menu(tray_menu);

    let state: AppStateType = AppStateType(Arc::new(RwLock::new(MyAppState {
        is_reading_memory: false,
    })));

    tauri::Builder::default()
        .manage(state)
        .system_tray(tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                "show_all" => {
                    match app.get_window("overlay") {
                        Some(window) => {
                            window.show().unwrap();
                        }
                        None => {}
                    }
                    match app.get_window("settings") {
                        Some(window) => {
                            window.show().unwrap();
                        }
                        None => {}
                    }
                }
                _ => {}
            },
            _ => {}
        })
        .setup(|app| {
            let overlay_window = app.get_window("overlay").expect("Failed to get window");
            overlay_window
                .set_ignore_cursor_events(true)
                .expect("Failed to setup overlay");
            Ok(())
        })
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![start, stop])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// async fn app_testing(app: tauri::AppHandle) {
//     std::thread::sleep(std::time::Duration::from_secs(5));
//     let utc_unix_time = chrono::Utc::now().timestamp_millis();
//     let _ = app.emit_all(
//         "testing",
//         Data {
//             unix_time_stamp: utc_unix_time,
//         },
//     );
// }
