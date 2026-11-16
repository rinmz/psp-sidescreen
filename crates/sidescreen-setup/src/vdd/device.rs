//! Installing the virtual display driver through the Windows device installer (`SetupAPI`).
//!
//! The standard tools cannot do this in one step: `pnputil` can add a driver package but, on
//! current Windows builds, cannot create the root-enumerated device that an indirect-display
//! driver attaches to. This does what Microsoft's `devcon install` does: create the device node,
//! register it, and update its driver from the INF.
//!
//! This module calls the Windows API and needs `unsafe`. Each block says what makes it safe.
#![allow(unsafe_code)]

use std::path::Path;

use windows::Win32::Devices::DeviceAndDriverInstallation::{
    DICD_GENERATE_ID, DIF_REGISTERDEVICE, DIF_REMOVE, INSTALLFLAG_FORCE, SP_DEVINFO_DATA,
    SPDRP_HARDWAREID, SetupDiCallClassInstaller, SetupDiCreateDeviceInfoList,
    SetupDiCreateDeviceInfoW, SetupDiSetDeviceRegistryPropertyW, SetupUninstallOEMInfW,
    UpdateDriverForPlugAndPlayDevicesW,
};
use windows::core::{BOOL, GUID, PCWSTR};

use super::{HARDWARE_ID, Package, SETTINGS_DIR, SETTINGS_FILE};
use crate::error::{Error, Step};
use crate::system::{
    DeviceSet, Scope, hardware_ids, is_elevated, present_devices, size_u32, wide, wide_str,
};

/// `SUOI_FORCEDELETE`: delete the package even if a device still uses it.
const SUOI_FORCEDELETE: u32 = 1;

/// The device setup class of display adapters.
const DISPLAY_CLASS: GUID = GUID::from_u128(0x4d36_e968_e325_11ce_bfc1_0800_2be1_0318);

/// What [`install`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The driver was already installed; nothing was changed.
    AlreadyInstalled,
    /// The driver was installed.
    Installed {
        /// Whether Windows asks for a restart to finish.
        reboot_required: bool,
    },
}

/// Whether the driver is installed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Status {
    /// How many virtual display adapters exist. Normally 0 or 1.
    pub adapters: usize,
}

impl Status {
    /// Whether at least one virtual display adapter exists.
    #[must_use]
    pub const fn is_installed(self) -> bool {
        self.adapters > 0
    }
}

/// Reports whether the virtual display adapter exists. Needs no special rights.
///
/// # Errors
///
/// [`Error::Windows`] if the device list cannot be read.
pub fn status() -> Result<Status, Error> {
    Ok(Status {
        adapters: matching_devices()?.1.len(),
    })
}

/// Installs the driver from `package`. Does nothing if it is already installed, which includes a
/// driver put there by another program that uses the same one.
///
/// If the package ships a settings file and none exists yet in [`SETTINGS_DIR`], it is copied
/// there first, because the driver reads its monitor list at start-up. An existing file is never
/// touched: it may belong to another program.
///
/// # Errors
///
/// [`Error::NotElevated`] without administrator rights, [`Error::UnsignedDriver`] if Windows does
/// not trust the package's signature, otherwise [`Error::Windows`] naming the step that was
/// rejected. A device created before a failure is removed again, so a failed install leaves
/// nothing behind.
pub fn install(package: &Package) -> Result<Outcome, Error> {
    if !is_elevated() {
        return Err(Error::NotElevated);
    }
    if status()?.is_installed() {
        return Ok(Outcome::AlreadyInstalled);
    }

    place_default_settings(package)?;

    let inf = wide(&package.inf());
    let hardware_id = wide_str(HARDWARE_ID);

    // SAFETY: the class GUID outlives the call.
    let set = unsafe { SetupDiCreateDeviceInfoList(Some(&DISPLAY_CLASS), None) }
        .step("create the device list")?;
    let set = DeviceSet(set);

    let mut data = SP_DEVINFO_DATA {
        cbSize: size_u32::<SP_DEVINFO_DATA>(),
        ..Default::default()
    };
    // SAFETY: the set and data are valid, and the name and class outlive the call.
    unsafe {
        SetupDiCreateDeviceInfoW(
            set.0,
            windows::core::w!("Display"),
            &DISPLAY_CLASS,
            PCWSTR::null(),
            None,
            DICD_GENERATE_ID,
            Some(&raw mut data),
        )
    }
    .step("create the device node")?;

    // The hardware id is a multi-string: the id, its terminator, and a final terminator.
    let mut multi_sz = hardware_id.clone();
    multi_sz.push(0);
    // SAFETY: `multi_sz` is a live buffer of u16, viewed as the bytes the API takes.
    let property =
        unsafe { std::slice::from_raw_parts(multi_sz.as_ptr().cast::<u8>(), multi_sz.len() * 2) };

    let outcome = (|| {
        // SAFETY: valid set and data; `property` is a live buffer.
        unsafe {
            SetupDiSetDeviceRegistryPropertyW(
                set.0,
                &raw mut data,
                SPDRP_HARDWAREID,
                Some(property),
            )
        }
        .step("set the hardware id")?;
        // SAFETY: valid set and data.
        unsafe { SetupDiCallClassInstaller(DIF_REGISTERDEVICE, set.0, Some(&raw const data)) }
            .step("register the device")?;

        let mut reboot = BOOL(0);
        // SAFETY: both strings are NUL-terminated and live; `reboot` is a valid out-pointer.
        unsafe {
            UpdateDriverForPlugAndPlayDevicesW(
                None,
                PCWSTR(hardware_id.as_ptr()),
                PCWSTR(inf.as_ptr()),
                INSTALLFLAG_FORCE,
                Some(&raw mut reboot),
            )
        }
        .map_err(|source| Error::from_install("install the driver", source))?;
        Ok(Outcome::Installed {
            reboot_required: reboot.as_bool(),
        })
    })();

    if outcome.is_err() {
        // Do not leave a device with no driver behind. The original error matters more than a
        // failure to clean up, so this one is ignored.
        // SAFETY: valid set and data.
        let _ = unsafe { SetupDiCallClassInstaller(DIF_REMOVE, set.0, Some(&raw const data)) };
    }
    outcome
}

/// Removes the virtual display adapter and its driver package.
///
/// # Errors
///
/// [`Error::NotElevated`] without administrator rights, [`Error::Windows`] if a device cannot be
/// removed.
pub fn uninstall() -> Result<(), Error> {
    if !is_elevated() {
        return Err(Error::NotElevated);
    }
    let (set, devices) = matching_devices()?;
    for data in &devices {
        // SAFETY: valid set and data obtained from the enumeration.
        unsafe { SetupDiCallClassInstaller(DIF_REMOVE, set.0, Some(data)) }
            .step("remove the device")?;
    }
    remove_driver_packages()
}

/// Finds the present display devices whose hardware id is the virtual display driver's.
fn matching_devices() -> Result<(DeviceSet, Vec<SP_DEVINFO_DATA>), Error> {
    let (set, all) = present_devices(&Scope::Class(&DISPLAY_CLASS))?;
    let matching = all
        .into_iter()
        .filter(|data| {
            hardware_ids(&set, data)
                .iter()
                .any(|id| id.eq_ignore_ascii_case(HARDWARE_ID))
        })
        .collect();
    Ok((set, matching))
}

/// Puts the package's settings where the driver reads them, unless a file is already there.
fn place_default_settings(package: &Package) -> Result<(), Error> {
    let Some(shipped) = package.default_settings() else {
        return Ok(());
    };
    std::fs::create_dir_all(SETTINGS_DIR)?;
    let target = Path::new(SETTINGS_DIR).join(SETTINGS_FILE);
    if !target.exists() {
        std::fs::copy(shipped, target)?;
    }
    Ok(())
}

/// Deletes the driver packages in the driver store that belong to the virtual display driver,
/// found by looking for its name in the `oem*.inf` copies Windows keeps.
fn remove_driver_packages() -> Result<(), Error> {
    let inf_dir = Path::new(r"C:\Windows\INF");
    for entry in std::fs::read_dir(inf_dir)?.filter_map(Result::ok) {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        let is_inf = Path::new(&name)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("inf"));
        if !(name.starts_with("oem") && is_inf) {
            continue;
        }
        // INF files are often UTF-16, so drop the zero bytes before searching the text.
        let text: Vec<u8> = std::fs::read(entry.path())?
            .into_iter()
            .filter(|&byte| byte != 0)
            .collect();
        if !String::from_utf8_lossy(&text).contains("MttVDD") {
            continue;
        }
        let wide_name = wide_str(&name);
        // SAFETY: the name is a NUL-terminated string that outlives the call.
        unsafe { SetupUninstallOEMInfW(PCWSTR(wide_name.as_ptr()), SUOI_FORCEDELETE, None) }
            .ok()
            .step("delete the driver package")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_driver_is_installed_only_when_an_adapter_exists() {
        assert!(!Status { adapters: 0 }.is_installed());
        assert!(Status { adapters: 1 }.is_installed());
    }

    #[test]
    fn asking_for_the_status_needs_no_special_rights() {
        // The answer depends on the machine; that it can be given is what is checked.
        assert!(status().is_ok());
    }
}
