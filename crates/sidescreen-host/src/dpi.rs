//! Makes the process aware of display scaling.
//!
//! A program that does not say it handles scaling is shown a virtualised desktop: sizes are
//! scaled, and a running program keeps seeing the monitor sizes it saw at start. Then a change of
//! resolution goes unnoticed, and pointer coordinates do not match the captured picture. Both
//! were seen with PSPdisp, which is from before display scaling. Asking for per-monitor
//! awareness makes every size and position a physical pixel, the unit Desktop Duplication uses.
//!
//! This module calls the Windows API and needs `unsafe`.
#![allow(unsafe_code)]

use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext,
};

/// Declares the process per-monitor DPI aware. Call it first thing in `main`, before anything
/// asks Windows about monitors.
///
/// Returns `false` if Windows refused, for example because the awareness was already set; the
/// program then goes on with whatever awareness it has.
pub fn make_process_aware() -> bool {
    // SAFETY: the call takes a constant context handle and has no other input.
    unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2).is_ok() }
}
