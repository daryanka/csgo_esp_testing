use tauri::{AppHandle, Manager};

use crate::AppStateType;

pub async fn start(app_state: AppStateType, app_handle: AppHandle) {
    println!("START CALLED");
    let _ = app_handle.emit_all("app_state_change", true);
    loop {
        {
            let state = app_state.0.read().await;
            if !state.is_reading_memory {
                let _ = app_handle.emit_all("app_state_change", false);
                return;
            }
        }

        // sleep 1 second
        std::thread::sleep(std::time::Duration::from_millis(300));
        println!("Hello, world!");
    }
}
