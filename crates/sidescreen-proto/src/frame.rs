use crate::Error;

/// Size of a frame header on the wire.
pub const FRAME_HEADER_LEN: usize = 16;

/// Magic number that starts every header, in both directions.
pub(crate) const MAGIC: u32 = 0xAFFE_0600;

/// Flags of a frame header.
///
/// Only the flags the host uses are named. Other bits are kept when a header is decoded, so a
/// header survives a round trip unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FrameFlags(u32);

impl FrameFlags {
    /// The frame carries an image.
    pub const CONTAINS_IMAGE: Self = Self(0x0000_0001);
    /// The image is a JPEG.
    pub const IMAGE_IS_JPEG: Self = Self(0x0000_0010);
    /// The image is raw pixels: 480x272, three bytes per pixel in blue, green, red order, rows
    /// from the bottom up (like a Windows bitmap).
    pub const IMAGE_IS_UNCOMPRESSED: Self = Self(0x0000_0040);
    /// Clear the screen before drawing.
    pub const CLEAR_SCREEN: Self = Self(0x0010_0000);

    /// Flags with no bit set.
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Flags from their raw value.
    #[must_use]
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// The raw value.
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Whether every bit of `other` is set.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// The union of both sets.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

/// Header that precedes every frame the host sends to the PSP.
///
/// The image bytes follow the header directly, then `settings_size` bytes of settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameHeader {
    /// What the frame carries.
    pub flags: FrameFlags,
    /// Length of the image that follows the header, in bytes.
    pub image_size: u32,
    /// Length of the settings that follow the image, in bytes.
    pub settings_size: u32,
}

impl FrameHeader {
    /// Header for a JPEG frame without settings.
    #[must_use]
    pub const fn jpeg(image_size: u32) -> Self {
        Self {
            flags: FrameFlags::CONTAINS_IMAGE.union(FrameFlags::IMAGE_IS_JPEG),
            image_size,
            settings_size: 0,
        }
    }

    /// Header for an uncompressed frame without settings; see
    /// [`FrameFlags::IMAGE_IS_UNCOMPRESSED`] for the pixel layout.
    #[must_use]
    pub const fn uncompressed(image_size: u32) -> Self {
        Self {
            flags: FrameFlags::CONTAINS_IMAGE.union(FrameFlags::IMAGE_IS_UNCOMPRESSED),
            image_size,
            settings_size: 0,
        }
    }

    /// The header as it goes on the wire, little-endian.
    #[must_use]
    pub fn encode(&self) -> [u8; FRAME_HEADER_LEN] {
        let mut out = [0; FRAME_HEADER_LEN];
        out[0..4].copy_from_slice(&MAGIC.to_le_bytes());
        out[4..8].copy_from_slice(&self.flags.bits().to_le_bytes());
        out[8..12].copy_from_slice(&self.image_size.to_le_bytes());
        out[12..16].copy_from_slice(&self.settings_size.to_le_bytes());
        out
    }

    /// Reads a header from the start of `bytes`; later bytes are ignored.
    ///
    /// # Errors
    ///
    /// [`Error::Truncated`] if fewer than [`FRAME_HEADER_LEN`] bytes are given, and
    /// [`Error::BadMagic`] if the magic number is wrong.
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        let bytes = bytes.get(..FRAME_HEADER_LEN).ok_or(Error::Truncated {
            needed: FRAME_HEADER_LEN,
            got: bytes.len(),
        })?;
        let magic = word(bytes, 0);
        if magic != MAGIC {
            return Err(Error::BadMagic(magic));
        }
        Ok(Self {
            flags: FrameFlags::from_bits(word(bytes, 4)),
            image_size: word(bytes, 8),
            settings_size: word(bytes, 12),
        })
    }
}

/// Little-endian `u32` at `offset`; the caller has checked the length.
pub(crate) fn word(bytes: &[u8], offset: usize) -> u32 {
    let mut raw = [0; 4];
    raw.copy_from_slice(&bytes[offset..offset + 4]);
    u32::from_le_bytes(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jpeg_header_has_the_layout_pspdisp_expects() {
        let wire = FrameHeader::jpeg(0x0102_0304).encode();
        assert_eq!(
            wire,
            [
                0x00, 0x06, 0xFE, 0xAF, // magic 0xAFFE0600
                0x11, 0x00, 0x00, 0x00, // image + JPEG
                0x04, 0x03, 0x02, 0x01, // image size
                0x00, 0x00, 0x00, 0x00, // settings size
            ]
        );
    }

    #[test]
    fn uncompressed_header_sets_the_raw_flag_and_not_the_jpeg_flag() {
        let wire = FrameHeader::uncompressed(391_680).encode();
        assert_eq!(&wire[4..8], [0x41, 0x00, 0x00, 0x00]);
        assert_eq!(&wire[8..12], 391_680_u32.to_le_bytes());
    }

    #[test]
    fn header_round_trips_with_unknown_flag_bits() {
        let header = FrameHeader {
            flags: FrameFlags::from_bits(0x0100_0711),
            image_size: 7,
            settings_size: 9,
        };
        assert_eq!(FrameHeader::decode(&header.encode()), Ok(header));
    }

    #[test]
    fn decode_ignores_bytes_after_the_header() {
        let mut wire = FrameHeader::jpeg(5).encode().to_vec();
        wire.extend_from_slice(&[0xFF; 5]);
        assert_eq!(FrameHeader::decode(&wire), Ok(FrameHeader::jpeg(5)));
    }

    #[test]
    fn truncated_header_is_an_error() {
        let wire = FrameHeader::jpeg(5).encode();
        for len in 0..FRAME_HEADER_LEN {
            assert_eq!(
                FrameHeader::decode(&wire[..len]),
                Err(Error::Truncated {
                    needed: FRAME_HEADER_LEN,
                    got: len
                })
            );
        }
    }

    #[test]
    fn wrong_magic_is_an_error() {
        let mut wire = FrameHeader::jpeg(5).encode();
        wire[3] = 0xAE;
        assert_eq!(
            FrameHeader::decode(&wire),
            Err(Error::BadMagic(0xAEFE_0600))
        );
    }

    #[test]
    fn flags_contains_checks_every_bit() {
        let both = FrameFlags::CONTAINS_IMAGE.union(FrameFlags::IMAGE_IS_JPEG);
        assert!(both.contains(FrameFlags::IMAGE_IS_JPEG));
        assert!(!FrameFlags::CONTAINS_IMAGE.contains(both));
        assert!(FrameFlags::empty().contains(FrameFlags::empty()));
    }
}
