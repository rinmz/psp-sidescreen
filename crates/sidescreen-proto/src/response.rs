use crate::Error;
use crate::frame::{MAGIC, word};

/// Size of the response header the PSP sends after every frame.
pub const RESPONSE_HEADER_LEN: usize = 14;

/// Size of the response over USB: the header plus the PSP's copy of the PC settings
/// (19 little-endian `u32` fields, which the host does not need).
pub const USB_RESPONSE_LEN: usize = RESPONSE_HEADER_LEN + 19 * 4;

/// What the PSP reports after it has taken a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Response {
    /// Raw flags of the response.
    pub flags: u32,
    /// Pressed buttons, in the PSP's `SceCtrlButtons` bit layout.
    pub buttons: u32,
    /// Analog stick, horizontal; 0 is left, 127 or 128 is centred, 255 is right.
    pub analog_x: u8,
    /// Analog stick, vertical; 0 is up, 127 or 128 is centred, 255 is down.
    pub analog_y: u8,
}

impl Response {
    /// Reads a response from the start of `bytes`; the settings that follow the header over
    /// USB are ignored.
    ///
    /// # Errors
    ///
    /// [`Error::Truncated`] if fewer than [`RESPONSE_HEADER_LEN`] bytes are given, and
    /// [`Error::BadMagic`] if the magic number is wrong.
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        let bytes = bytes.get(..RESPONSE_HEADER_LEN).ok_or(Error::Truncated {
            needed: RESPONSE_HEADER_LEN,
            got: bytes.len(),
        })?;
        let magic = word(bytes, 0);
        if magic != MAGIC {
            return Err(Error::BadMagic(magic));
        }
        Ok(Self {
            flags: word(bytes, 4),
            buttons: word(bytes, 8),
            analog_x: bytes[12],
            analog_y: bytes[13],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wire(buttons: u32, x: u8, y: u8) -> Vec<u8> {
        let mut out = vec![0x00, 0x06, 0xFE, 0xAF, 0, 0, 0, 0];
        out.extend_from_slice(&buttons.to_le_bytes());
        out.extend_from_slice(&[x, y]);
        out
    }

    #[test]
    fn usb_response_is_ninety_bytes() {
        assert_eq!(USB_RESPONSE_LEN, 90);
    }

    #[test]
    fn decodes_buttons_and_stick() {
        let response = Response::decode(&wire(0x0000_4001, 10, 250)).unwrap();
        assert_eq!(
            response,
            Response {
                flags: 0,
                buttons: 0x4001,
                analog_x: 10,
                analog_y: 250
            }
        );
    }

    #[test]
    fn settings_after_the_header_are_ignored() {
        let mut bytes = wire(1, 127, 127);
        bytes.resize(USB_RESPONSE_LEN, 0xEE);
        assert_eq!(Response::decode(&bytes).unwrap().buttons, 1);
    }

    #[test]
    fn truncated_response_is_an_error() {
        let bytes = wire(1, 127, 127);
        assert_eq!(
            Response::decode(&bytes[..13]),
            Err(Error::Truncated {
                needed: RESPONSE_HEADER_LEN,
                got: 13
            })
        );
    }

    #[test]
    fn wrong_magic_is_an_error() {
        let mut bytes = wire(1, 127, 127);
        bytes[0] = 0x01;
        assert_eq!(Response::decode(&bytes), Err(Error::BadMagic(0xAFFE_0601)));
    }
}
