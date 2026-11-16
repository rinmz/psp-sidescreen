//! The USB driver for the PSP while its program runs.
//!
//! The PSP program presents itself to Windows as a USB device with a vendor-specific interface
//! (Sony's vendor id, product `01CA`, or `01C9`). Windows has the driver that such a device needs,
//! WinUSB, but it must be told that this device is one that uses it. That is what the INF of the
//! package does. An INF that is not signed is refused by Windows, so until the package is signed
//! the install ends in [`Error::UnsignedDriver`] and the status says what is missing.
//!
//! This module calls the Windows API and needs `unsafe`. Each block says what makes it safe.
#![allow(unsafe_code)]

use std::path::Path;

use windows::Win32::Devices::DeviceAndDriverInstallation::{DIIRFLAG_FORCE_INF, DiInstallDriverW};
use windows::core::{BOOL, PCWSTR};

use crate::error::Error;
use crate::system::{Scope, hardware_ids, is_elevated, present_devices, service, wide};

/// Sony's USB vendor id and the product ids the PSP program presents.
const DEVICE_IDS: [&str; 2] = ["USB\\VID_054C&PID_01CA", "USB\\VID_054C&PID_01C9"];

/// The driver service that talks to the PSP program.
const SERVICE: &str = "WinUSB";

/// Where the PSP stands as far as the driver is concerned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// No PSP running its program is connected, so nothing can be said about the driver.
    NotConnected,
    /// The PSP is connected and Windows has no driver for it.
    NeedsDriver,
    /// The PSP is connected and its driver is in use.
    Ready,
}

/// Looks at the PSP, if one running its program is connected. Needs no special rights.
///
/// # Errors
///
/// [`Error::Windows`] if the device list cannot be read.
pub fn status() -> Result<Status, Error> {
    let (set, devices) = present_devices(&Scope::Enumerator("USB"))?;
    let mut found = Status::NotConnected;
    for data in &devices {
        let is_psp = hardware_ids(&set, data).iter().any(|id| {
            DEVICE_IDS
                .iter()
                .any(|known| id.to_ascii_uppercase().starts_with(known))
        });
        if !is_psp {
            continue;
        }
        if service(&set, data).is_some_and(|name| name.eq_ignore_ascii_case(SERVICE)) {
            return Ok(Status::Ready);
        }
        found = Status::NeedsDriver;
    }
    Ok(found)
}

/// Whether installing the driver restarts Windows to finish.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Installed {
    /// Whether Windows asks for a restart.
    pub reboot_required: bool,
}

/// Installs the driver from the INF at `inf`, for a PSP that is connected now or later.
///
/// # Errors
///
/// [`Error::NotElevated`] without administrator rights, [`Error::UnsignedDriver`] if Windows does
/// not trust the package's signature, otherwise [`Error::Windows`].
pub fn install(inf: &Path) -> Result<Installed, Error> {
    if !is_elevated() {
        return Err(Error::NotElevated);
    }
    let inf = wide(inf);
    let mut reboot = BOOL(0);
    // SAFETY: the path is a NUL-terminated string that outlives the call, and `reboot` is a valid
    // out-pointer.
    unsafe {
        DiInstallDriverW(
            None,
            PCWSTR(inf.as_ptr()),
            DIIRFLAG_FORCE_INF,
            Some(&raw mut reboot),
        )
    }
    .map_err(|source| Error::from_install("install the PSP driver", source))?;
    Ok(Installed {
        reboot_required: reboot.as_bool(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asking_for_the_status_needs_no_special_rights() {
        // Whether a PSP is connected depends on the machine; that it can be asked is checked.
        assert!(status().is_ok());
    }

    #[test]
    fn installing_without_administrator_rights_is_refused_before_anything_is_touched() {
        // The tests do not run elevated; a machine where they do would install nothing, because
        // the path does not exist.
        if !is_elevated() {
            assert!(matches!(
                install(Path::new("missing.inf")),
                Err(Error::NotElevated)
            ));
        }
    }
}
