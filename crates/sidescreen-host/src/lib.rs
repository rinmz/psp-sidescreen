//! Windows host of PSP Sidescreen.
//!
//! [`dpi`] makes the process aware of display scaling, [`monitor`] gives the PSP a virtual
//! monitor of its own, [`capture`] grabs a monitor, [`convert`] reduces it to the PSP's screen as
//! a JPEG or raw pixels, [`link`] holds the conversation with the PSP independent of the USB
//! layer, [`usb`] connects it to a real device, and [`session`] runs all of it from Start to Stop.

pub mod capture;
pub mod convert;
pub mod dpi;
pub mod inject;
pub mod input;
pub mod link;
pub mod monitor;
pub mod session;
pub mod usb;
