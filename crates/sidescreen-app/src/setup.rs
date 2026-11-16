//! The Setup page: what is installed, and putting the program on the PSP.
//!
//! The drivers are the installer's work, because they need administrator rights; here they are
//! only looked at. Putting the program on the PSP needs no such rights, and needs the PSP in a
//! mode the person has to choose, so the page waits for it and asks before it writes anything.

// Tauri hands a command its `AppHandle` by value; it cannot be borrowed.
#![allow(clippy::needless_pass_by_value)]

use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

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

/// `CREATE_NO_WINDOW`: no console window flashes up for the helper.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Exit codes of `sidescreen-setup`, and the one the wrapper uses when the person says no to the
/// administrator prompt.
const EXIT_UNSIGNED: i32 = 4;
const EXIT_ALREADY: i32 = 6;
const EXIT_DECLINED: i32 = 100;

/// What the window is told about installing the PSP driver: `installed`, `already`, `unsigned`,
/// `declined` or `failed`.
fn driver_outcome(exit_code: i32) -> &'static str {
    match exit_code {
        0 => "installed",
        EXIT_ALREADY => "already",
        EXIT_UNSIGNED => "unsigned",
        EXIT_DECLINED => "declined",
        _ => "failed",
    }
}

/// Installs the USB driver for the PSP. Installing needs administrator rights, so the helper is
/// started with the prompt for them; this program stays as it is.
///
/// The paths go to PowerShell in environment variables, not in the command text, so that no path
/// can be taken for part of a command.
#[tauri::command]
pub(crate) fn install_psp_driver(app: AppHandle) -> Result<&'static str, String> {
    let resources = resource_root(&app)?;
    let helper = resources.join("sidescreen-setup.exe");
    let inf = resources
        .join("drivers")
        .join("psp")
        .join("sidescreen-psp.inf");
    if !helper.is_file() || !inf.is_file() {
        return Err(
            "the driver is missing from this installation; install PSP Sidescreen again".to_owned(),
        );
    }
    let script = "try { $p = Start-Process -FilePath $env:SIDESCREEN_HELPER         -ArgumentList 'install-psp-driver', ('\"' + $env:SIDESCREEN_INF + '\"')         -Verb RunAs -Wait -PassThru -WindowStyle Hidden; exit $p.ExitCode } catch { exit 100 }";
    let status = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .env("SIDESCREEN_HELPER", &helper)
        .env("SIDESCREEN_INF", &inf)
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|error| error.to_string())?;
    Ok(driver_outcome(status.code().unwrap_or(-1)))
}

/// The folder the installer put its extra files in; while developing, the build folder and the
/// repository stand in for it.
fn resource_root(app: &AppHandle) -> Result<PathBuf, String> {
    let installed = app
        .path()
        .resource_dir()
        .map_err(|error| error.to_string())?;
    if installed.join("sidescreen-setup.exe").is_file() {
        return Ok(installed);
    }
    if cfg!(debug_assertions) {
        let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let staged = repository.join("crates/sidescreen-app/resources");
        if staged.join("sidescreen-setup.exe").is_file() {
            return Ok(staged);
        }
    }
    Err("the installer's files were not found".to_owned())
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
    fn every_exit_code_of_the_helper_has_an_outcome_for_the_window() {
        assert_eq!(driver_outcome(0), "installed");
        assert_eq!(driver_outcome(6), "already");
        assert_eq!(driver_outcome(4), "unsigned");
        assert_eq!(driver_outcome(100), "declined");
        for other in [1, 2, 3, 5, -1, 1223] {
            assert_eq!(driver_outcome(other), "failed", "{other}");
        }
    }

    #[test]
    fn the_package_in_the_repository_is_found_while_developing() {
        // The tests run from the build folder, where the committed package is the fallback.
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../psp/package");
        assert!(source.join("PSP/GAME/Sidescreen/EBOOT.PBP").is_file());
    }
}
