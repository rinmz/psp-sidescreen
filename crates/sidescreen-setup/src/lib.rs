//! What the PSP Sidescreen installer does, as a library so that it is tested and so that the
//! program's own Setup page can use the same steps.
//!
//! - [`vdd`]: the virtual display driver, the extra screen Windows shows for the PSP.
//! - [`psp_driver`]: the USB driver for the PSP while its program runs.
//! - [`stick`]: finding the PSP's Memory Stick and putting the program on it.
//!
//! Steps that change the system need administrator rights and say so with
//! [`Error::NotElevated`]; looking at the state of things never does.

mod error;
pub mod psp_driver;
pub mod stick;
mod system;
pub mod vdd;

pub use error::Error;
pub use system::is_elevated;
