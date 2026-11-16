//! The installer's helper, run by the installer and by the program's Setup page.
//!
//! - `sidescreen-setup status` says what is installed.
//! - `sidescreen-setup install-vdd <package folder>` installs the virtual display driver.
//! - `sidescreen-setup uninstall-vdd` removes it.
//! - `sidescreen-setup install-psp-driver <inf file>` installs the USB driver for the PSP.
//! - `sidescreen-setup install-psp <package folder> [drive]` puts the program on the PSP.
//!
//! The exit code says what happened, so that the installer can tell the person what to do:
//! see the `EXIT_` constants. Success is 0; a driver that was already there is 6, so that an
//! uninstaller removes only what its installer put on the machine.

use std::path::Path;
use std::process::ExitCode;

use sidescreen_setup::vdd::{self, Outcome};
use sidescreen_setup::{Error, psp_driver, stick};

/// A step failed in a way that has no code of its own.
const EXIT_FAILED: u8 = 1;
/// The command was not understood.
const EXIT_USAGE: u8 = 2;
/// The step needs administrator rights.
const EXIT_NOT_ELEVATED: u8 = 3;
/// Windows refused a driver because it is not signed.
const EXIT_UNSIGNED: u8 = 4;
/// There is no Memory Stick to copy to, or the choice is not clear.
const EXIT_NO_STICK: u8 = 5;
/// Nothing was installed because the driver was already there and working.
const EXIT_ALREADY: u8 = 6;

const USAGE: &str = "usage: sidescreen-setup status | install-vdd <folder> | uninstall-vdd | \
install-psp-driver <inf> | install-psp <folder> [drive]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = match args.as_slice() {
        ["status"] => status().map(|()| 0),
        ["install-vdd", folder] => install_vdd(Path::new(folder)),
        ["uninstall-vdd"] => uninstall_vdd().map(|()| 0),
        ["install-psp-driver", inf] => install_psp_driver(Path::new(inf)),
        ["install-psp", folder] => install_psp(Path::new(folder), None).map(|()| 0),
        ["install-psp", folder, drive] => {
            install_psp(Path::new(folder), Some(Path::new(drive))).map(|()| 0)
        }
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(EXIT_USAGE);
        }
    };
    match result {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(exit_code(&error))
        }
    }
}

/// The exit code that tells the installer what kind of failure this was.
fn exit_code(error: &Error) -> u8 {
    match error {
        Error::NotElevated => EXIT_NOT_ELEVATED,
        Error::UnsignedDriver => EXIT_UNSIGNED,
        Error::NoMemoryStick | Error::SeveralMemoryCards(_) => EXIT_NO_STICK,
        _ => EXIT_FAILED,
    }
}

fn status() -> Result<(), Error> {
    let display = if vdd::status()?.is_installed() {
        "installed"
    } else {
        "missing"
    };
    let driver = match psp_driver::status()? {
        psp_driver::Status::Ready => "ready",
        psp_driver::Status::NeedsDriver => "needs-driver",
        psp_driver::Status::NotConnected => "not-connected",
    };
    println!("virtual-display: {display}");
    println!("psp-driver: {driver}");
    let sticks = stick::memory_sticks();
    if sticks.is_empty() {
        println!("memory-stick: none");
    }
    for root in sticks {
        println!("memory-stick: {}", root.display());
    }
    Ok(())
}

fn install_vdd(folder: &Path) -> Result<u8, Error> {
    let package = vdd::Package::open(folder)?;
    Ok(match vdd::install(&package)? {
        Outcome::AlreadyInstalled => {
            println!("the virtual display driver was already installed");
            EXIT_ALREADY
        }
        Outcome::Installed {
            reboot_required: false,
        } => {
            println!("the virtual display driver is installed");
            0
        }
        Outcome::Installed {
            reboot_required: true,
        } => {
            println!("the virtual display driver is installed; Windows asks for a restart");
            0
        }
    })
}

fn uninstall_vdd() -> Result<(), Error> {
    vdd::uninstall()?;
    println!("the virtual display driver is removed");
    Ok(())
}

fn install_psp_driver(inf: &Path) -> Result<u8, Error> {
    if psp_driver::status()? == psp_driver::Status::Ready {
        println!("the PSP driver is already working");
        return Ok(EXIT_ALREADY);
    }
    let installed = psp_driver::install(inf)?;
    println!(
        "the PSP driver is installed{}",
        if installed.reboot_required {
            "; Windows asks for a restart"
        } else {
            ""
        }
    );
    Ok(0)
}

fn install_psp(package: &Path, drive: Option<&Path>) -> Result<(), Error> {
    let root = stick::choose(drive, stick::memory_sticks())?;
    let installed = stick::install_package(package, &root)?;
    println!(
        "the program is on the PSP ({} files on {}){}",
        installed.files,
        root.display(),
        if installed.kept_settings {
            "; its settings were kept"
        } else {
            ""
        }
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_kind_of_failure_has_its_own_exit_code() {
        assert_eq!(exit_code(&Error::NotElevated), EXIT_NOT_ELEVATED);
        assert_eq!(exit_code(&Error::UnsignedDriver), EXIT_UNSIGNED);
        assert_eq!(exit_code(&Error::NoMemoryStick), EXIT_NO_STICK);
        assert_eq!(
            exit_code(&Error::SeveralMemoryCards(Vec::new())),
            EXIT_NO_STICK
        );
        assert_eq!(
            exit_code(&Error::Io(std::io::Error::other("x"))),
            EXIT_FAILED
        );
    }

    #[test]
    fn the_codes_are_distinct_and_none_is_success() {
        let codes = [
            EXIT_FAILED,
            EXIT_USAGE,
            EXIT_NOT_ELEVATED,
            EXIT_UNSIGNED,
            EXIT_NO_STICK,
            EXIT_ALREADY,
        ];
        let mut sorted = codes.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), codes.len());
        assert!(!codes.contains(&0));
    }
}
