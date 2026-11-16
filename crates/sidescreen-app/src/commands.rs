//! What the window can ask for. The names are the ones `app/src/lib/tauriBackend.ts` calls.

// Tauri hands a command its `AppHandle`, `State` and arguments by value; they cannot be borrowed.
#![allow(clippy::needless_pass_by_value)]

use std::path::PathBuf;

use serde::Serialize;
use sidescreen_host::capture;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::control::Control;
use crate::settings::{self, Settings};

/// The event the window listens to for the state of the session.
const STATE_EVENT: &str = "session-state";

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|folder| folder.join("settings.json"))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn get_settings(app: AppHandle) -> Result<Settings, String> {
    Ok(settings::load(&settings_path(&app)?))
}

#[tauri::command]
pub(crate) fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    settings::save(&settings_path(&app)?, &settings).map_err(|error| error.to_string())
}

/// A monitor that can be shown, in the shape the window expects.
#[derive(Serialize)]
pub(crate) struct MonitorInfo {
    index: usize,
    name: String,
    width: u32,
    height: u32,
}

#[tauri::command]
pub(crate) fn list_monitors() -> Result<Vec<MonitorInfo>, String> {
    let monitors = capture::list_monitors().map_err(|error| error.to_string())?;
    Ok(monitors
        .into_iter()
        .map(|monitor| MonitorInfo {
            index: monitor.index,
            name: monitor.name,
            width: monitor.width,
            height: monitor.height,
        })
        .collect())
}

#[tauri::command]
pub(crate) fn start_session(app: AppHandle, control: State<'_, Control>) -> Result<(), String> {
    // The settings come from the file, not from the window, so what runs is what was saved.
    let options = settings::load(&settings_path(&app)?).to_options()?;
    control.start(options, move |state| {
        // The window may be closed already; there is nobody to tell then.
        let _ = app.emit(STATE_EVENT, state);
    })
}

#[tauri::command]
pub(crate) fn stop_session(control: State<'_, Control>) {
    control.stop();
}
