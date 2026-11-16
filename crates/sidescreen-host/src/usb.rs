//! The PSP as a USB device, through libusb (WinUSB on Windows).

use std::time::Duration;

use rusb::{Context, DeviceHandle, UsbContext};

use crate::link::{Pipe, PipeError};

/// Sony's USB vendor id.
const VENDOR_ID: u16 = 0x054C;
/// Product ids the PSP application presents: type B (libusb driver) and type C (WinUSB).
const PRODUCT_IDS: [u16; 2] = [0x01C9, 0x01CA];
/// Bulk endpoint the host writes to.
const ENDPOINT_OUT: u8 = 0x02;
/// Bulk endpoint the host reads from.
const ENDPOINT_IN: u8 = 0x81;

/// Why the PSP could not be opened.
#[derive(Debug)]
pub enum OpenError {
    /// No PSP running the application is attached.
    NotFound,
    /// The USB layer reported an error.
    Usb(rusb::Error),
}

impl std::fmt::Display for OpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => f.write_str(
                "no PSP found; start the PSP application and choose \"Start using USB\"",
            ),
            Self::Usb(error) => write!(f, "cannot open the PSP: {error}"),
        }
    }
}

impl std::error::Error for OpenError {}

impl From<rusb::Error> for OpenError {
    fn from(error: rusb::Error) -> Self {
        Self::Usb(error)
    }
}

/// The bulk endpoints of an opened PSP.
pub struct UsbPipe {
    handle: DeviceHandle<Context>,
    timeout: Duration,
}

impl UsbPipe {
    /// Finds the PSP and claims its interface.
    ///
    /// `timeout` bounds every transfer, so a PSP that stops answering cannot hang the host.
    ///
    /// # Errors
    ///
    /// [`OpenError::NotFound`] if no PSP is attached, [`OpenError::Usb`] if it cannot be
    /// opened, for example because another program (PSPdisp) holds it.
    pub fn open(timeout: Duration) -> Result<Self, OpenError> {
        let context = Context::new()?;
        for device in context.devices()?.iter() {
            let descriptor = device.device_descriptor()?;
            if descriptor.vendor_id() == VENDOR_ID && PRODUCT_IDS.contains(&descriptor.product_id())
            {
                let handle = device.open()?;
                handle.claim_interface(0)?;
                return Ok(Self { handle, timeout });
            }
        }
        Err(OpenError::NotFound)
    }
}

impl Pipe for UsbPipe {
    fn write(&mut self, data: &[u8]) -> Result<usize, PipeError> {
        self.handle
            .write_bulk(ENDPOINT_OUT, data, self.timeout)
            .map_err(pipe_error)
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, PipeError> {
        self.handle
            .read_bulk(ENDPOINT_IN, buf, self.timeout)
            .map_err(pipe_error)
    }
}

fn pipe_error(error: rusb::Error) -> PipeError {
    match error {
        rusb::Error::Timeout => PipeError::Timeout,
        rusb::Error::NoDevice | rusb::Error::Io => PipeError::Disconnected,
        other => PipeError::Other(other.to_string()),
    }
}
