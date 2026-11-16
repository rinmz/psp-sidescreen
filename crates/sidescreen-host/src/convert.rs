//! Turns a captured frame into the picture the PSP shows.
//!
//! The frame is reduced to the PSP's screen with a filter that keeps thin strokes (a plain
//! nearest-neighbour or bilinear reduction makes small text break up), then either compressed
//! as a JPEG or, for the best quality, sent as raw pixels.

use std::fmt;
use std::str::FromStr;

use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};
use jpeg_encoder::{ColorType, Encoder};
use sidescreen_proto::{RAW_FRAME_LEN, SCREEN_HEIGHT, SCREEN_WIDTH};

use crate::capture::Frame;

/// How much the picture is compressed, from none to a lot. Less compression looks better and
/// needs more of the USB cable's capacity, so the frame rate falls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    /// No compression: every pixel arrives exactly as reduced. Sharpest text.
    Lossless,
    /// JPEG at quality 92.
    High,
    /// JPEG at quality 80.
    Medium,
    /// JPEG at quality 50. Smallest frames and the highest frame rate.
    Low,
}

impl Quality {
    /// The JPEG quality, or `None` when the picture is not compressed.
    #[must_use]
    pub const fn jpeg_quality(self) -> Option<u8> {
        match self {
            Self::Lossless => None,
            Self::High => Some(92),
            Self::Medium => Some(80),
            Self::Low => Some(50),
        }
    }
}

/// The text is not a quality name.
#[derive(Debug, PartialEq, Eq)]
pub struct ParseQualityError(String);

impl fmt::Display for ParseQualityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown quality {:?}; use lossless, high, medium or low",
            self.0
        )
    }
}

impl std::error::Error for ParseQualityError {}

impl FromStr for Quality {
    type Err = ParseQualityError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text.to_ascii_lowercase().as_str() {
            "lossless" => Ok(Self::Lossless),
            "high" => Ok(Self::High),
            "medium" => Ok(Self::Medium),
            "low" => Ok(Self::Low),
            _ => Err(ParseQualityError(text.to_owned())),
        }
    }
}

/// A converted picture, borrowed from the [`Converter`] that made it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Picture<'a> {
    /// A JPEG file.
    Jpeg(&'a [u8]),
    /// Raw pixels in the PSP's uncompressed layout.
    Raw(&'a [u8]),
}

/// Why a frame could not be converted.
#[derive(Debug)]
pub enum ConvertError {
    /// The frame's pixel buffer does not match its width and height.
    BadFrame,
    /// The resizer failed.
    Resize(fast_image_resize::ResizeError),
    /// The JPEG encoder failed.
    Jpeg(jpeg_encoder::EncodingError),
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadFrame => f.write_str("the captured frame is malformed"),
            Self::Resize(error) => write!(f, "cannot reduce the frame: {error}"),
            Self::Jpeg(error) => write!(f, "cannot compress the frame: {error}"),
        }
    }
}

impl std::error::Error for ConvertError {}

/// Converts frames for the PSP, reusing its buffers from one frame to the next.
pub struct Converter {
    resizer: Resizer,
    options: ResizeOptions,
    small: Image<'static>,
    output: Vec<u8>,
    quality: Quality,
}

impl Converter {
    /// Creates a converter for the given quality.
    #[must_use]
    pub fn new(quality: Quality) -> Self {
        Self {
            resizer: Resizer::new(),
            // The alpha channel of a captured screen carries no meaning, so it is not used to
            // weight the colours.
            options: ResizeOptions::new()
                .resize_alg(ResizeAlg::Convolution(FilterType::CatmullRom))
                .use_alpha(false),
            small: Image::new(SCREEN_WIDTH, SCREEN_HEIGHT, PixelType::U8x4),
            output: Vec::new(),
            quality,
        }
    }

    /// Reduces `frame` to the PSP's screen and returns it, valid until the next call.
    ///
    /// # Errors
    ///
    /// [`ConvertError`] if the frame is malformed or the resizer or encoder fails.
    pub fn convert(&mut self, frame: &Frame) -> Result<Picture<'_>, ConvertError> {
        let source = ImageRef::new(frame.width, frame.height, &frame.bgra, PixelType::U8x4)
            .map_err(|_| ConvertError::BadFrame)?;
        self.resizer
            .resize(&source, &mut self.small, &self.options)
            .map_err(ConvertError::Resize)?;

        self.output.clear();
        let Some(jpeg_quality) = self.quality.jpeg_quality() else {
            to_bgr_bottom_up(self.small.buffer(), SCREEN_WIDTH as usize, &mut self.output);
            return Ok(Picture::Raw(&self.output));
        };

        // SCREEN_WIDTH and SCREEN_HEIGHT are 480 and 272, well inside u16.
        #[allow(clippy::cast_possible_truncation)]
        Encoder::new(&mut self.output, jpeg_quality)
            .encode(
                self.small.buffer(),
                SCREEN_WIDTH as u16,
                SCREEN_HEIGHT as u16,
                ColorType::Bgra,
            )
            .map_err(ConvertError::Jpeg)?;
        Ok(Picture::Jpeg(&self.output))
    }
}

impl fmt::Debug for Converter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Converter")
            .field("quality", &self.quality)
            .finish_non_exhaustive()
    }
}

/// Writes `bgra` (rows from the top, four bytes per pixel) as the PSP's raw layout: three bytes
/// per pixel, blue first, rows from the bottom up.
fn to_bgr_bottom_up(bgra: &[u8], width: usize, out: &mut Vec<u8>) {
    out.reserve(RAW_FRAME_LEN);
    for row in bgra.chunks_exact(width * 4).rev() {
        for pixel in row.chunks_exact(4) {
            out.extend_from_slice(&pixel[..3]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(width: u32, height: u32, bgra: [u8; 4]) -> Frame {
        Frame {
            width,
            height,
            bgra: bgra.repeat((width * height) as usize),
        }
    }

    fn jpeg_of(converter: &mut Converter, frame: &Frame) -> Vec<u8> {
        match converter.convert(frame).unwrap() {
            Picture::Jpeg(bytes) => bytes.to_vec(),
            Picture::Raw(_) => panic!("expected a JPEG"),
        }
    }

    #[test]
    fn full_hd_frame_becomes_a_jpeg() {
        let mut converter = Converter::new(Quality::Medium);
        let jpeg = jpeg_of(&mut converter, &solid(1920, 1080, [10, 120, 200, 255]));
        assert_eq!(&jpeg[..2], [0xFF, 0xD8], "starts with the JPEG marker");
        assert_eq!(
            &jpeg[jpeg.len() - 2..],
            [0xFF, 0xD9],
            "ends with the JPEG marker"
        );
    }

    #[test]
    fn jpeg_has_the_size_of_the_psp_screen() {
        let mut converter = Converter::new(Quality::Medium);
        let jpeg = jpeg_of(&mut converter, &solid(1366, 768, [0, 0, 0, 255]));
        // The start-of-frame segment holds height then width, big-endian.
        let marker = jpeg
            .windows(2)
            .position(|pair| pair == [0xFF, 0xC0])
            .unwrap();
        let height = u16::from_be_bytes([jpeg[marker + 5], jpeg[marker + 6]]);
        let width = u16::from_be_bytes([jpeg[marker + 7], jpeg[marker + 8]]);
        assert_eq!(
            (u32::from(width), u32::from(height)),
            (SCREEN_WIDTH, SCREEN_HEIGHT)
        );
    }

    #[test]
    fn frame_already_the_size_of_the_screen_is_accepted() {
        let mut converter = Converter::new(Quality::Medium);
        assert!(converter.convert(&solid(480, 272, [1, 2, 3, 255])).is_ok());
    }

    #[test]
    fn buffer_that_does_not_match_the_size_is_rejected() {
        let mut converter = Converter::new(Quality::Medium);
        let frame = Frame {
            width: 100,
            height: 100,
            bgra: vec![0; 10],
        };
        assert!(matches!(
            converter.convert(&frame),
            Err(ConvertError::BadFrame)
        ));
    }

    #[test]
    fn buffers_are_reused_across_frames() {
        let mut converter = Converter::new(Quality::Medium);
        let frame = solid(800, 600, [255, 0, 0, 255]);
        assert_eq!(
            jpeg_of(&mut converter, &frame),
            jpeg_of(&mut converter, &frame)
        );
    }

    #[test]
    fn higher_quality_makes_a_larger_file_for_detailed_pictures() {
        // A picture with detail at every scale, so compression cannot hide the difference.
        let mut frame = solid(480, 272, [0, 0, 0, 255]);
        for (index, byte) in frame.bgra.iter_mut().enumerate() {
            *byte = u8::try_from(index.wrapping_mul(2_654_435_761) >> 24 & 0xFF).unwrap();
        }
        let size = |quality| jpeg_of(&mut Converter::new(quality), &frame).len();
        assert!(size(Quality::High) > size(Quality::Medium));
        assert!(size(Quality::Medium) > size(Quality::Low));
    }

    #[test]
    fn lossless_is_one_raw_screen() {
        let mut converter = Converter::new(Quality::Lossless);
        let picture = converter.convert(&solid(960, 544, [1, 2, 3, 255])).unwrap();
        let Picture::Raw(bytes) = picture else {
            panic!("expected raw pixels")
        };
        assert_eq!(bytes.len(), RAW_FRAME_LEN);
        assert_eq!(&bytes[..3], [1, 2, 3], "blue, green, red");
    }

    #[test]
    fn raw_layout_is_bgr_with_the_bottom_row_first() {
        // Two rows of two pixels: the top row is blue, the bottom row is red.
        let bgra = [
            255, 0, 0, 255, 255, 0, 0, 255, // top: blue
            0, 0, 255, 255, 0, 0, 255, 255, // bottom: red
        ];
        let mut out = Vec::new();
        to_bgr_bottom_up(&bgra, 2, &mut out);
        assert_eq!(out, [0, 0, 255, 0, 0, 255, 255, 0, 0, 255, 0, 0]);
    }

    #[test]
    fn quality_names_parse_in_any_case() {
        assert_eq!("Lossless".parse(), Ok(Quality::Lossless));
        assert_eq!("HIGH".parse(), Ok(Quality::High));
        assert_eq!("medium".parse(), Ok(Quality::Medium));
        assert_eq!("low".parse(), Ok(Quality::Low));
    }

    #[test]
    fn unknown_quality_name_is_an_error() {
        assert_eq!(
            "best".parse::<Quality>(),
            Err(ParseQualityError("best".to_owned()))
        );
    }
}
