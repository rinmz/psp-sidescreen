//! Wire format between the PSP Sidescreen host (Windows) and the PSP application.
//!
//! The format is the one used by PSPdisp 0.6 (BSD licence), so the host can drive the
//! unmodified PSPdisp application on the console. This crate holds the encoding only: no I/O,
//! no dependencies. The normative description is `docs/PROTOCOL.md`.

mod buttons;
mod error;
mod frame;
mod handshake;
mod response;

pub use buttons::Button;
pub use error::Error;
pub use frame::{FRAME_HEADER_LEN, FrameFlags, FrameHeader};
pub use handshake::{HELLO_LEN, HOSTFS_MAGIC};
pub use response::{RESPONSE_HEADER_LEN, Response, USB_RESPONSE_LEN};

/// Width of the PSP screen in pixels.
pub const SCREEN_WIDTH: u32 = 480;

/// Height of the PSP screen in pixels.
pub const SCREEN_HEIGHT: u32 = 272;

/// Size of an uncompressed frame: three bytes for each pixel of the screen.
pub const RAW_FRAME_LEN: usize = SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize * 3;
