//! The Windows calls the setup shares: administrator rights, and listing devices.
//!
//! This module calls the Windows API and needs `unsafe`. Each block says what makes it safe.
#![allow(unsafe_code)]

use std::ffi::c_void;
use std::path::Path;

use windows::Win32::Devices::DeviceAndDriverInstallation::{
    DIGCF_ALLCLASSES, DIGCF_PRESENT, HDEVINFO, SETUP_DI_REGISTRY_PROPERTY, SP_DEVINFO_DATA,
    SPDRP_HARDWAREID, SPDRP_SERVICE, SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo,
    SetupDiGetClassDevsW, SetupDiGetDeviceRegistryPropertyW,
};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::core::{GUID, PCWSTR};

use crate::error::{Error, Step};

/// Whether this process runs with administrator rights.
#[must_use]
pub fn is_elevated() -> bool {
    let mut token = HANDLE::default();
    // SAFETY: `token` is a valid out-pointer and the current-process pseudo handle needs no
    // closing.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &raw mut token) }.is_err() {
        return false;
    }
    let mut elevation = TOKEN_ELEVATION::default();
    let mut returned = 0_u32;
    // SAFETY: the buffer is a `TOKEN_ELEVATION` and its exact size is passed.
    let queried = unsafe {
        GetTokenInformation(
            token,
            TokenElevation,
            Some((&raw mut elevation).cast::<c_void>()),
            size_u32::<TOKEN_ELEVATION>(),
            &raw mut returned,
        )
    };
    // SAFETY: `token` was opened above and is not used afterwards.
    let _ = unsafe { CloseHandle(token) };
    queried.is_ok() && elevation.TokenIsElevated != 0
}

/// A device information set, destroyed when dropped.
pub(crate) struct DeviceSet(pub(crate) HDEVINFO);

impl Drop for DeviceSet {
    fn drop(&mut self) {
        // SAFETY: the handle came from a successful creation call and is destroyed once.
        let _ = unsafe { SetupDiDestroyDeviceInfoList(self.0) };
    }
}

/// Which devices to list.
pub(crate) enum Scope<'a> {
    /// The devices of one setup class.
    Class(&'a GUID),
    /// The devices of every class that sit under one enumerator, such as `USB`.
    Enumerator(&'a str),
}

/// The devices that are present, in the scope.
pub(crate) fn present_devices(
    scope: &Scope<'_>,
) -> Result<(DeviceSet, Vec<SP_DEVINFO_DATA>), Error> {
    let enumerator = match scope {
        Scope::Enumerator(name) => Some(wide_str(name)),
        Scope::Class(_) => None,
    };
    let (class, flags) = match scope {
        Scope::Class(guid) => (Some(std::ptr::from_ref(*guid)), DIGCF_PRESENT),
        Scope::Enumerator(_) => (None, DIGCF_PRESENT | DIGCF_ALLCLASSES),
    };
    let enumerator_pointer = enumerator
        .as_ref()
        .map_or_else(PCWSTR::null, |text| PCWSTR(text.as_ptr()));
    // SAFETY: the class GUID and the enumerator text outlive the call; no parent window is used.
    let set = unsafe { SetupDiGetClassDevsW(class, enumerator_pointer, None, flags) }
        .step("list the devices")?;
    let set = DeviceSet(set);

    let mut found = Vec::new();
    for index in 0.. {
        let mut data = SP_DEVINFO_DATA {
            cbSize: size_u32::<SP_DEVINFO_DATA>(),
            ..Default::default()
        };
        // SAFETY: valid set and out-pointer. The call fails when the list is exhausted.
        if unsafe { SetupDiEnumDeviceInfo(set.0, index, &raw mut data) }.is_err() {
            break;
        }
        found.push(data);
    }
    Ok((set, found))
}

/// A device's hardware ids, such as `USB\VID_054C&PID_01CA`.
pub(crate) fn hardware_ids(set: &DeviceSet, data: &SP_DEVINFO_DATA) -> Vec<String> {
    property_bytes(set, data, SPDRP_HARDWAREID)
        .map(|bytes| {
            let units = utf16(&bytes);
            units
                .split(|&unit| unit == 0)
                .filter(|id| !id.is_empty())
                .map(String::from_utf16_lossy)
                .collect()
        })
        .unwrap_or_default()
}

/// The name of the driver service that runs a device, or `None` if it has none.
pub(crate) fn service(set: &DeviceSet, data: &SP_DEVINFO_DATA) -> Option<String> {
    let bytes = property_bytes(set, data, SPDRP_SERVICE)?;
    let name = String::from_utf16_lossy(&utf16(&bytes))
        .trim_end_matches('\0')
        .to_owned();
    (!name.is_empty()).then_some(name)
}

fn property_bytes(
    set: &DeviceSet,
    data: &SP_DEVINFO_DATA,
    property: SETUP_DI_REGISTRY_PROPERTY,
) -> Option<Vec<u8>> {
    let mut buffer = vec![0_u8; 1024];
    let mut required = 0_u32;
    // SAFETY: valid set and data; the buffer and the size out-pointer are live. A device that
    // lacks the property makes the call fail, which is read as "none".
    let read = unsafe {
        SetupDiGetDeviceRegistryPropertyW(
            set.0,
            data,
            property,
            None,
            Some(&mut buffer),
            Some(&raw mut required),
        )
    };
    read.is_ok().then(|| {
        buffer.truncate(required as usize);
        buffer
    })
}

fn utf16(bytes: &[u8]) -> Vec<u16> {
    bytes
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect()
}

/// A path as the NUL-terminated wide string Windows takes.
pub(crate) fn wide(path: &Path) -> Vec<u16> {
    wide_str(&path.to_string_lossy())
}

pub(crate) fn wide_str(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// The size of a Windows structure as the `u32` its API expects. These structures are a few
/// hundred bytes at most.
pub(crate) fn size_u32<T>() -> u32 {
    u32::try_from(size_of::<T>()).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_strings_end_in_a_nul() {
        assert_eq!(wide_str("ab"), [0x61, 0x62, 0]);
        assert_eq!(wide_str(""), [0]);
    }

    #[test]
    fn utf16_reads_little_endian_pairs_and_ignores_a_stray_byte() {
        assert_eq!(utf16(&[0x61, 0x00, 0x62, 0x00, 0xFF]), [0x61, 0x62]);
    }
}
