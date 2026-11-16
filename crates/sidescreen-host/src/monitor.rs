//! The virtual monitor that gives the PSP a screen of its own.
//!
//! Windows only shows an extra monitor that a display driver provides. PSP Sidescreen uses the
//! open-source Virtual Display Driver (MIT licence), whose monitor is always present and is
//! brought onto the desktop with the same call as the Win+P menu, so no administrator rights are
//! needed. The monitor is found by its hardware id, because its name (`\\.\DISPLAY5`,
//! `\\.\DISPLAY6`, ...) changes whenever the driver recreates it.
//!
//! The driver offers a fixed list of sizes; the size in use must be set explicitly, through the
//! call the Windows settings page uses. That choice is remembered by Windows.
//!
//! This module calls the Windows API and needs `unsafe`.
#![allow(unsafe_code)]

use std::fmt;
use std::thread;
use std::time::Duration;

use windows::Win32::Devices::Display::{
    SDC_APPLY, SDC_TOPOLOGY_EXTEND, SDC_TOPOLOGY_INTERNAL, SET_DISPLAY_CONFIG_FLAGS,
    SetDisplayConfig,
};
use windows::Win32::Graphics::Gdi::{
    CDS_TYPE, CDS_UPDATEREGISTRY, ChangeDisplaySettingsExW, DEVMODEW, DISP_CHANGE_SUCCESSFUL,
    DISPLAY_DEVICE_ATTACHED_TO_DESKTOP, DISPLAY_DEVICEW, DM_DISPLAYFREQUENCY, DM_PELSHEIGHT,
    DM_PELSWIDTH, ENUM_DISPLAY_SETTINGS_MODE, EnumDisplayDevicesW, EnumDisplaySettingsW,
};
use windows::core::PCWSTR;

/// Hardware id of the monitor the driver creates.
const MONITOR_ID: &str = "MTT1337";
/// How long to wait for Windows to bring the monitor onto the desktop.
const APPEAR_WITHIN: Duration = Duration::from_secs(5);
/// How often to look while waiting.
const APPEAR_POLL: Duration = Duration::from_millis(100);

/// Why the virtual monitor could not be set up.
#[derive(Debug, PartialEq, Eq)]
pub enum MonitorError {
    /// Windows refused to change the display configuration; carries its result code.
    Refused(i32),
    /// The driver's monitor did not appear on the desktop. The driver may not be installed.
    NotFound,
    /// The driver does not offer the requested size.
    NotOffered {
        /// Requested width.
        width: u32,
        /// Requested height.
        height: u32,
    },
}

impl fmt::Display for MonitorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused(code) => write!(f, "Windows refused the display change (code {code})"),
            Self::NotFound => f.write_str(
                "the virtual monitor did not appear; is the Virtual Display Driver installed?",
            ),
            Self::NotOffered { width, height } => {
                write!(f, "the virtual monitor does not offer {width}x{height}")
            }
        }
    }
}

impl std::error::Error for MonitorError {}

/// A mode a monitor can run in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mode {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Refresh rate in hertz.
    pub refresh_hz: u32,
}

/// The virtual monitor on the desktop.
///
/// Dropping it takes the monitor off the desktop again ("PC screen only"), so windows return to
/// the main screen.
#[derive(Debug)]
pub struct VirtualMonitor {
    name: String,
}

impl VirtualMonitor {
    /// Puts the virtual monitor on the desktop, to the right of the main screen, at the given
    /// size and, if the driver offers it, refresh rate.
    ///
    /// # Errors
    ///
    /// [`MonitorError`] if Windows refuses, the monitor does not appear, or the size is not
    /// offered. The desktop is put back as it was before an error is returned.
    pub fn attach(width: u32, height: u32, refresh_hz: u32) -> Result<Self, MonitorError> {
        apply(SDC_TOPOLOGY_EXTEND)?;
        // From here on every failure must take the monitor off the desktop again, which the
        // drop of this value does.
        let mut monitor = Self {
            name: String::new(),
        };
        let name = wait_for_monitor().ok_or(MonitorError::NotFound)?;
        set_mode(&name, width, height, refresh_hz)?;
        monitor.name = name;
        Ok(monitor)
    }

    /// The name Windows gives the monitor, for example `\\.\DISPLAY5`.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for VirtualMonitor {
    fn drop(&mut self) {
        // Nothing can be reported from here. A failure leaves the monitor where it was; `detach`
        // or the Windows display settings can still remove it.
        let _ = detach();
    }
}

/// Shows the PC's own screen only, taking the virtual monitor off the desktop. Windows that
/// were on it move to the main screen.
///
/// Needed when the host was ended without the chance to clean up (Task Manager, a crash).
///
/// # Errors
///
/// [`MonitorError::Refused`] with Windows' result code if it refuses.
pub fn detach() -> Result<(), MonitorError> {
    apply(SDC_TOPOLOGY_INTERNAL)
}

fn apply(topology: SET_DISPLAY_CONFIG_FLAGS) -> Result<(), MonitorError> {
    // SAFETY: with `SDC_APPLY` and a topology flag, `SetDisplayConfig` takes no path or mode
    // arrays, so there are no pointers for it to read.
    let code = unsafe { SetDisplayConfig(None, None, SDC_APPLY | topology) };
    if code == 0 {
        Ok(())
    } else {
        Err(MonitorError::Refused(code))
    }
}

/// Waits until the virtual monitor is part of the desktop and returns its name.
fn wait_for_monitor() -> Option<String> {
    let mut waited = Duration::ZERO;
    loop {
        if let Some(name) = find_virtual_monitor() {
            return Some(name);
        }
        if waited >= APPEAR_WITHIN {
            return None;
        }
        thread::sleep(APPEAR_POLL);
        waited += APPEAR_POLL;
    }
}

/// Finds the virtual monitor among the monitors on the desktop and returns its current name.
#[must_use]
pub fn find_virtual_monitor() -> Option<String> {
    for adapter_index in 0.. {
        let mut adapter = display_device();
        // SAFETY: `adapter` is a valid structure whose size field is set.
        let more =
            unsafe { EnumDisplayDevicesW(PCWSTR::null(), adapter_index, &raw mut adapter, 0) };
        if !more.as_bool() {
            return None;
        }
        if (adapter.StateFlags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP).0 == 0 {
            continue;
        }
        let mut monitor = display_device();
        // SAFETY: the adapter name is a NUL-terminated buffer and `monitor` is valid.
        let found = unsafe {
            EnumDisplayDevicesW(PCWSTR(adapter.DeviceName.as_ptr()), 0, &raw mut monitor, 0)
        };
        if found.as_bool() && to_string(&monitor.DeviceID).contains(MONITOR_ID) {
            return Some(to_string(&adapter.DeviceName));
        }
    }
    None
}

/// Lists the modes `device_name` can run in.
fn modes(device_name: &str) -> Vec<Mode> {
    let name = wide(device_name);
    let mut found = Vec::new();
    for index in 0.. {
        let mut devmode = devmode();
        // SAFETY: the name is NUL-terminated and `devmode` is valid with its size field set.
        let more = unsafe {
            EnumDisplaySettingsW(
                PCWSTR(name.as_ptr()),
                ENUM_DISPLAY_SETTINGS_MODE(index),
                &raw mut devmode,
            )
        };
        if !more.as_bool() {
            break;
        }
        let listed = Mode {
            width: devmode.dmPelsWidth,
            height: devmode.dmPelsHeight,
            refresh_hz: devmode.dmDisplayFrequency,
        };
        if !found.contains(&listed) {
            found.push(listed);
        }
    }
    found
}

/// Picks from `offered` the mode of the given size: the one at `refresh_hz` if there is one,
/// otherwise the fastest.
fn choose_mode(offered: &[Mode], width: u32, height: u32, refresh_hz: u32) -> Option<Mode> {
    let same_size = || {
        offered
            .iter()
            .filter(|mode| mode.width == width && mode.height == height)
    };
    same_size()
        .find(|mode| mode.refresh_hz == refresh_hz)
        .or_else(|| same_size().max_by_key(|mode| mode.refresh_hz))
        .copied()
}

/// Makes `device_name` run at the given size.
fn set_mode(
    device_name: &str,
    width: u32,
    height: u32,
    refresh_hz: u32,
) -> Result<Mode, MonitorError> {
    let chosen = choose_mode(&modes(device_name), width, height, refresh_hz)
        .ok_or(MonitorError::NotOffered { width, height })?;

    let name = wide(device_name);
    let mut devmode = devmode();
    devmode.dmPelsWidth = chosen.width;
    devmode.dmPelsHeight = chosen.height;
    devmode.dmDisplayFrequency = chosen.refresh_hz;
    devmode.dmFields = DM_PELSWIDTH | DM_PELSHEIGHT | DM_DISPLAYFREQUENCY;
    // SAFETY: the name is NUL-terminated and `devmode` is a valid, filled-in structure.
    let result = unsafe {
        ChangeDisplaySettingsExW(
            PCWSTR(name.as_ptr()),
            Some(&raw const devmode),
            None,
            CDS_TYPE(CDS_UPDATEREGISTRY.0),
            None,
        )
    };
    if result == DISP_CHANGE_SUCCESSFUL {
        Ok(chosen)
    } else {
        Err(MonitorError::Refused(result.0))
    }
}

fn display_device() -> DISPLAY_DEVICEW {
    DISPLAY_DEVICEW {
        // The structure is a few hundred bytes.
        cb: u32::try_from(size_of::<DISPLAY_DEVICEW>()).unwrap_or(0),
        ..Default::default()
    }
}

fn devmode() -> DEVMODEW {
    DEVMODEW {
        // The structure is a few hundred bytes.
        dmSize: u16::try_from(size_of::<DEVMODEW>()).unwrap_or(0),
        ..Default::default()
    }
}

fn to_string(wide: &[u16]) -> String {
    let end = wide
        .iter()
        .position(|&unit| unit == 0)
        .unwrap_or(wide.len());
    String::from_utf16_lossy(&wide[..end])
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn mode(width: u32, height: u32, refresh_hz: u32) -> Mode {
        Mode {
            width,
            height,
            refresh_hz,
        }
    }

    #[test]
    fn exact_refresh_rate_is_preferred() {
        let offered = [
            mode(1366, 768, 30),
            mode(1366, 768, 60),
            mode(1366, 768, 144),
        ];
        assert_eq!(
            choose_mode(&offered, 1366, 768, 60),
            Some(mode(1366, 768, 60))
        );
    }

    #[test]
    fn fastest_rate_is_used_when_the_wanted_one_is_not_offered() {
        let offered = [mode(1366, 768, 30), mode(1366, 768, 90)];
        assert_eq!(
            choose_mode(&offered, 1366, 768, 60),
            Some(mode(1366, 768, 90))
        );
    }

    #[test]
    fn size_that_is_not_offered_gives_nothing() {
        let offered = [mode(1920, 1080, 60)];
        assert_eq!(choose_mode(&offered, 1366, 768, 60), None);
    }

    #[test]
    fn other_sizes_are_ignored() {
        let offered = [mode(1920, 1080, 240), mode(1366, 768, 30)];
        assert_eq!(
            choose_mode(&offered, 1366, 768, 60),
            Some(mode(1366, 768, 30))
        );
    }
}
