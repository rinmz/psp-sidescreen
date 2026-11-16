//! Carries out the inputs of [`input`](crate::input) on the PC, as if the keys and the mouse
//! were used.
//!
//! This module calls the Windows API and is small on purpose: what is to be sent is decided in
//! `input`, where it is tested.
#![allow(unsafe_code)]

use std::fmt;
use std::mem::size_of;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBD_EVENT_FLAGS, KEYBDINPUT,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, MOUSE_EVENT_FLAGS, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE,
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEINPUT, SendInput,
    VIRTUAL_KEY,
};

use crate::input::{Event, Pointer, Step, steps};

/// One notch of the mouse wheel, in the units Windows uses.
const WHEEL_DELTA: i32 = 120;

/// Windows did not take the input, usually because the window in front runs with more rights
/// than this program (an administrator prompt, for one) or the screen is locked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Blocked;

impl fmt::Display for Blocked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            "Windows did not accept the input; the window in front may need administrator rights",
        )
    }
}

impl std::error::Error for Blocked {}

/// Keys that Windows tells apart from the number pad by the extended-key flag.
const fn is_extended(vk: u16) -> bool {
    matches!(vk, 0x21..=0x28 | 0x2D | 0x2E | 0x5B)
}

fn key_input(vk: u16, up: bool) -> INPUT {
    let mut flags = KEYBD_EVENT_FLAGS(0);
    if up {
        flags |= KEYEVENTF_KEYUP;
    }
    if is_extended(vk) {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn mouse_input(flags: MOUSE_EVENT_FLAGS, dx: i32, dy: i32, data: i32) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                // The wheel's amount is signed but travels in an unsigned field.
                mouseData: data.cast_unsigned(),
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn to_input(step: Step) -> INPUT {
    match step {
        Step::Key { vk, up } => key_input(vk, up),
        Step::Button { pointer, up } => {
            let flags = match (pointer, up) {
                (Pointer::Left, false) => MOUSEEVENTF_LEFTDOWN,
                (Pointer::Left, true) => MOUSEEVENTF_LEFTUP,
                (Pointer::Right, false) => MOUSEEVENTF_RIGHTDOWN,
                (Pointer::Right, true) => MOUSEEVENTF_RIGHTUP,
                (Pointer::Middle, false) => MOUSEEVENTF_MIDDLEDOWN,
                (Pointer::Middle, true) => MOUSEEVENTF_MIDDLEUP,
            };
            mouse_input(flags, 0, 0, 0)
        }
        Step::Wheel(notches) => mouse_input(MOUSEEVENTF_WHEEL, 0, 0, notches * WHEEL_DELTA),
        Step::Move { dx, dy } => mouse_input(MOUSEEVENTF_MOVE, dx, dy, 0),
    }
}

/// Sends the inputs that carry out `event`.
///
/// # Errors
///
/// [`Blocked`] if Windows takes none or only some of them.
pub fn apply(event: &Event) -> Result<(), Blocked> {
    let inputs: Vec<INPUT> = steps(event).into_iter().map(to_input).collect();
    if inputs.is_empty() {
        return Ok(());
    }
    let size = i32::try_from(size_of::<INPUT>()).map_err(|_| Blocked)?;
    // SAFETY: `inputs` is a slice of fully initialised INPUT structures that lives through the
    // call, and `size` is the size of one of them, as `SendInput` requires.
    let sent = unsafe { SendInput(&inputs, size) };
    if sent as usize == inputs.len() {
        Ok(())
    } else {
        Err(Blocked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_keys_are_extended_and_letters_are_not() {
        for vk in [0x25, 0x26, 0x27, 0x28, 0x21, 0x22, 0x23, 0x24, 0x2E, 0x5B] {
            assert!(is_extended(vk), "{vk:#x}");
        }
        for vk in [0x0D, 0x1B, 0x20, 0x09, 0x43, 0x56, 0xAD, 0xB3] {
            assert!(!is_extended(vk), "{vk:#x}");
        }
    }
}
