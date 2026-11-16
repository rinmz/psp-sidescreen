//! Listing the drives that could be a PSP Memory Stick.
//!
//! This module calls the Windows API and needs `unsafe`. Each block says what makes it safe.
#![allow(unsafe_code)]

use std::path::PathBuf;

use windows::Win32::Storage::FileSystem::{GetDriveTypeW, GetLogicalDrives};
use windows::core::PCWSTR;

/// `DRIVE_REMOVABLE`: what Windows calls a drive whose medium can be taken out, which is how a
/// PSP connected with USB Connection shows up.
const DRIVE_REMOVABLE: u32 = 2;

/// The root of every removable drive, such as `E:\`.
pub(super) fn removable_roots() -> Vec<PathBuf> {
    // SAFETY: no arguments; it returns a bit mask of the drive letters in use.
    let mask = unsafe { GetLogicalDrives() };
    (0..26_u8)
        .filter(|bit| mask & (1 << bit) != 0)
        .map(|bit| format!("{}:\\", char::from(b'A' + bit)))
        .filter(|root| {
            let wide: Vec<u16> = root.encode_utf16().chain(std::iter::once(0)).collect();
            // SAFETY: the root is a NUL-terminated string that outlives the call.
            unsafe { GetDriveTypeW(PCWSTR(wide.as_ptr())) == DRIVE_REMOVABLE }
        })
        .map(PathBuf::from)
        .collect()
}
