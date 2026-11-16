//! The virtual display driver: the extra screen Windows shows for the PSP.
//!
//! The open-source Virtual Display Driver (MIT licence) is an Indirect Display Driver. Windows
//! only creates extra monitors through such a driver, so the setup installs it and the program
//! finds its monitor afterwards. The package is the signed one the driver's project releases,
//! fetched by `scripts/fetch-vdd-driver.ps1` against a fixed hash.

mod device;
mod package;

pub use device::{Outcome, Status, install, status, uninstall};
pub use package::Package;

/// The hardware id the driver's INF binds to. A device with this id is the virtual display
/// adapter.
pub const HARDWARE_ID: &str = "Root\\MttVDD";

/// Where the driver reads its settings from.
pub const SETTINGS_DIR: &str = r"C:\VirtualDisplayDriver";

/// The name of the driver's settings file inside [`SETTINGS_DIR`].
pub const SETTINGS_FILE: &str = "vdd_settings.xml";
