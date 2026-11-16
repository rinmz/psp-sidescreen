//! Windows host of PSP Sidescreen.
//!
//! [`monitor`] gives the PSP a virtual monitor of its own, [`capture`] grabs a monitor,
//! [`convert`] reduces it to the PSP's screen as a JPEG, [`link`] holds the conversation with
//! the PSP independent of the USB layer, and [`usb`] connects it to a real device.

pub mod capture;
pub mod convert;
pub mod link;
pub mod monitor;
pub mod usb;
