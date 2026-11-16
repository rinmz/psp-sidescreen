//! The PSP Sidescreen window and tray icon.
//!
//! The window is a web page (`app/`) in a system web view; this program gives it the settings
//! file, the monitor list and Start and Stop, and runs the session from `sidescreen-host`.

// No console window behind the window in a release build.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod control;
mod settings;
mod tray;

use tauri::{Manager, RunEvent, WindowEvent};

use crate::control::Control;

fn main() {
    // Before anything asks Windows about monitors; see `sidescreen_host::dpi`.
    sidescreen_host::dpi::make_process_aware();

    let app = tauri::Builder::default()
        .manage(Control::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::list_monitors,
            commands::start_session,
            commands::stop_session,
        ])
        .setup(|app| {
            tray::create(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the window leaves the program running in the notification area, so a
            // session goes on; Quit in the tray menu ends it.
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .build(tauri::generate_context!())
        // The configuration is compiled in; if it is wrong the program cannot start at all.
        .expect("the Tauri configuration is invalid");

    app.run(|handle, event| {
        if let RunEvent::Exit = event {
            // The session takes the virtual monitor off the desktop as it ends.
            handle.state::<Control>().stop_and_wait();
        }
    });
}
