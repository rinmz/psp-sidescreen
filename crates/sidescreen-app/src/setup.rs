//! The Setup page: what is installed, and putting the program on the PSP.
//!
//! The drivers are the installer's work, because they need administrator rights; here they are
//! only looked at. Putting the program on the PSP needs no such rights, and needs the PSP in a
//! mode the person has to choose, so the page waits for it and asks before it writes anything.

// Tauri hands a command its `AppHandle` by value; it cannot be borrowed.
#![allow(clippy::needless_pass_by_value)]

use std::path::PathBuf;

use serde::Serialize;
use sidescreen_setup::{Error, psp_driver, stick, vdd};
use tauri::{AppHandle, Manager};

/// What the Setup page shows.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetupStatus {
    /// Whether the virtual display driver is installed.
    virtual_display: bool,
    /// `ready`, `needs-driver` or `not-connected`; see `psp_driver::Status`.
    psp_driver: &'static str,
    /// The drive of a connected PSP Memory Stick, such as `E:\`.
    memory_stick: Option<String>,
    /// Whether the program is on that Memory Stick already.
    program_on_stick: bool,
}

/// What putting the program on the PSP did.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PspInstall {
    drive: String,
    files: usize,
    kept_settings: bool,
}

fn message(error: &Error) -> String {
    error.to_string()
}

#[tauri::command]
pub(crate) fn get_setup() -> Result<SetupStatus, String> {
    let virtual_display = vdd::status()
        .map_err(|error| message(&error))?
        .is_installed();
    let psp_driver = match psp_driver::status().map_err(|error| message(&error))? {
        psp_driver::Status::Ready => "ready",
        psp_driver::Status::NeedsDriver => "needs-driver",
        psp_driver::Status::NotConnected => "not-connected",
    };
    // With more than one Memory Stick connected the first is shown; putting the program on one
    // asks the person to disconnect the others, which is rare enough not to need its own page.
    let memory_stick = stick::memory_sticks().into_iter().next();
    let program_on_stick = memory_stick
        .as_deref()
        .is_some_and(stick::program_is_installed);
    Ok(SetupStatus {
        virtual_display,
        psp_driver,
        memory_stick: memory_stick.map(|root| root.display().to_string()),
        program_on_stick,
    })
}

#[tauri::command]
pub(crate) fn install_psp_program(app: AppHandle) -> Result<PspInstall, String> {
    let package = package_dir(&app)?;
    let root = stick::choose(None, stick::memory_sticks()).map_err(|error| message(&error))?;
    let installed = stick::install_package(&package, &root).map_err(|error| message(&error))?;
    Ok(PspInstall {
        drive: root.display().to_string(),
        files: installed.files,
        kept_settings: installed.kept_settings,
    })
}

/// The folder that holds the PSP program the installer carried (`PSP/GAME/Sidescreen` below it).
fn package_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let installed = app
        .path()
        .resource_dir()
        .map_err(|error| error.to_string())?
        .join("psp");
    if installed.is_dir() {
        return Ok(installed);
    }
    // Running from a build folder, as during development, the package is in the repository.
    if cfg!(debug_assertions) {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../psp/package");
        if source.is_dir() {
            return Ok(source);
        }
    }
    Err(
        "the PSP program is missing from this installation; install PSP Sidescreen again"
            .to_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_status_is_sent_to_the_window_in_the_shape_it_expects() {
        let status = SetupStatus {
            virtual_display: true,
            psp_driver: "ready",
            memory_stick: Some("E:\\".to_owned()),
            program_on_stick: false,
        };
        assert_eq!(
            serde_json::to_string(&status).unwrap(),
            r#"{"virtualDisplay":true,"pspDriver":"ready","memoryStick":"E:\\","programOnStick":false}"#
        );
    }

    #[test]
    fn the_result_of_a_copy_is_sent_in_the_shape_the_window_expects() {
        let done = PspInstall {
            drive: "E:\\".to_owned(),
            files: 3,
            kept_settings: true,
        };
        assert_eq!(
            serde_json::to_string(&done).unwrap(),
            r#"{"drive":"E:\\","files":3,"keptSettings":true}"#
        );
    }

    #[test]
    fn the_package_in_the_repository_is_found_while_developing() {
        // The tests run from the build folder, where the committed package is the fallback.
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../psp/package");
        assert!(source.join("PSP/GAME/Sidescreen/EBOOT.PBP").is_file());
    }
}
