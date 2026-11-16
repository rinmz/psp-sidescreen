//! Windows host of PSP Sidescreen.
//!
//! [`link`] holds the conversation with the PSP and is independent of the USB layer; [`usb`]
//! connects it to a real device.

pub mod link;
pub mod usb;
