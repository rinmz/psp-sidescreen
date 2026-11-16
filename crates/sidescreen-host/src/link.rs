//! The conversation with the PSP, independent of how bytes reach it.
//!
//! The USB specifics sit behind [`Pipe`], so the handshake and the frame exchange are tested
//! here without a console.

use std::fmt;
use std::thread;
use std::time::Duration;

use sidescreen_proto::{
    FrameHeader, HELLO_LEN, HOSTFS_MAGIC, RAW_FRAME_LEN, Response, USB_RESPONSE_LEN,
};

/// Why a transfer on the pipe failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipeError {
    /// The PSP did not take or give data in time.
    Timeout,
    /// The PSP is gone, for example because the cable was pulled.
    Disconnected,
    /// Any other failure, described by the underlying layer.
    Other(String),
}

impl fmt::Display for PipeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout => f.write_str("the PSP did not answer in time"),
            Self::Disconnected => f.write_str("the PSP is no longer connected"),
            Self::Other(message) => f.write_str(message),
        }
    }
}

/// Two bulk endpoints of the PSP: the one the host writes to and the one it reads from.
pub trait Pipe {
    /// Writes as much of `data` as the PSP takes and returns how many bytes that was.
    ///
    /// # Errors
    ///
    /// [`PipeError`] if the transfer failed.
    fn write(&mut self, data: &[u8]) -> Result<usize, PipeError>;

    /// Reads up to `buf.len()` bytes and returns how many arrived.
    ///
    /// # Errors
    ///
    /// [`PipeError`] if the transfer failed.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, PipeError>;
}

/// Why the exchange with the PSP failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkError {
    /// A transfer failed.
    Pipe(PipeError),
    /// The PSP sent something that is not a valid response.
    Protocol(sidescreen_proto::Error),
    /// The image has a size the frame cannot carry; for a raw frame, anything but one screen.
    RawSize(usize),
}

impl fmt::Display for LinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pipe(error) => write!(f, "USB transfer failed: {error}"),
            Self::Protocol(error) => write!(f, "invalid answer from the PSP: {error}"),
            Self::RawSize(size) => write!(f, "an image of {size} bytes does not fit a frame"),
        }
    }
}

impl std::error::Error for LinkError {}

impl From<PipeError> for LinkError {
    fn from(error: PipeError) -> Self {
        Self::Pipe(error)
    }
}

impl From<sidescreen_proto::Error> for LinkError {
    fn from(error: sidescreen_proto::Error) -> Self {
        Self::Protocol(error)
    }
}

/// An opened link to the PSP application.
#[derive(Debug)]
pub struct Link<P> {
    pipe: P,
}

impl<P: Pipe> Link<P> {
    /// Opens the link: the `usbhostfs` hello that PSPdisp sends first.
    ///
    /// `settle` is the pause after the first four bytes. PSPdisp waits 500 ms, because without
    /// it the next write reports success without reaching the PSP.
    ///
    /// # Errors
    ///
    /// [`LinkError`] if the PSP does not take the hello or does not answer it.
    pub fn open(mut pipe: P, settle: Duration) -> Result<Self, LinkError> {
        write_all(&mut pipe, &HOSTFS_MAGIC)?;
        thread::sleep(settle);

        // If the PSP is already initialised this write goes through, and it discards the bytes
        // because they carry no valid magic. Then nothing more is needed.
        let filler = [0; HELLO_LEN];
        match pipe.write(&filler) {
            Ok(HELLO_LEN) => {}
            Ok(_) | Err(PipeError::Timeout) => {
                let mut hello = [0; HELLO_LEN];
                read_exact(&mut pipe, &mut hello)?;
                write_all(&mut pipe, &hello)?;
            }
            Err(error) => return Err(error.into()),
        }
        Ok(Self { pipe })
    }

    /// Sends one JPEG frame and returns what the PSP reports back.
    ///
    /// The header and the image go out as two writes, as PSPdisp does, because the PSP reads
    /// the header on its own before it reads the image.
    ///
    /// # Errors
    ///
    /// [`LinkError`] if a transfer fails or the response is not valid.
    pub fn send_jpeg(&mut self, jpeg: &[u8]) -> Result<Response, LinkError> {
        self.send(FrameHeader::jpeg, jpeg)
    }

    /// Sends one uncompressed frame, `RAW_FRAME_LEN` bytes of blue, green, red pixels with the
    /// rows from the bottom up, and returns what the PSP reports back.
    ///
    /// # Errors
    ///
    /// [`LinkError::RawSize`] if `bgr` is not exactly one frame, otherwise as
    /// [`send_jpeg`](Self::send_jpeg).
    pub fn send_raw(&mut self, bgr: &[u8]) -> Result<Response, LinkError> {
        if bgr.len() != RAW_FRAME_LEN {
            return Err(LinkError::RawSize(bgr.len()));
        }
        self.send(FrameHeader::uncompressed, bgr)
    }

    /// Asks the PSP what its buttons are doing, without sending a picture.
    ///
    /// # Errors
    ///
    /// [`LinkError`] if a transfer fails or the response is not valid.
    pub fn poll(&mut self) -> Result<Response, LinkError> {
        self.send(|_| FrameHeader::poll(), &[])
    }

    fn send(
        &mut self,
        header: impl FnOnce(u32) -> FrameHeader,
        image: &[u8],
    ) -> Result<Response, LinkError> {
        // A frame for a 480x272 screen is far below 4 GiB; a larger one is refused rather than
        // cut short.
        let size = u32::try_from(image.len()).map_err(|_| LinkError::RawSize(image.len()))?;
        write_all(&mut self.pipe, &header(size).encode())?;
        write_all(&mut self.pipe, image)?;

        let mut response = [0; USB_RESPONSE_LEN];
        read_exact(&mut self.pipe, &mut response)?;
        Ok(Response::decode(&response)?)
    }
}

/// Writes all of `data`, in as many transfers as the PSP needs.
fn write_all<P: Pipe>(pipe: &mut P, mut data: &[u8]) -> Result<(), PipeError> {
    while !data.is_empty() {
        let written = pipe.write(data)?;
        if written == 0 {
            // A transfer that moves nothing would loop for ever; treat it as a stall.
            return Err(PipeError::Timeout);
        }
        data = &data[written..];
    }
    Ok(())
}

/// Fills `buf` completely, in as many transfers as the PSP needs.
fn read_exact<P: Pipe>(pipe: &mut P, mut buf: &mut [u8]) -> Result<(), PipeError> {
    while !buf.is_empty() {
        let read = pipe.read(buf)?;
        if read == 0 {
            return Err(PipeError::Timeout);
        }
        buf = &mut buf[read..];
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    /// A pipe that plays back scripted answers and records what was written.
    #[derive(Debug, Default)]
    struct Scripted {
        writes: Vec<Vec<u8>>,
        write_answers: VecDeque<Result<usize, PipeError>>,
        read_answers: VecDeque<Result<Vec<u8>, PipeError>>,
        /// Most bytes one write takes; `None` takes everything.
        write_limit: Option<usize>,
    }

    impl Pipe for Scripted {
        fn write(&mut self, data: &[u8]) -> Result<usize, PipeError> {
            let taken = match self.write_answers.pop_front() {
                Some(Ok(taken)) => taken,
                Some(Err(error)) => {
                    // A failed attempt still reached the pipe, so it is recorded.
                    self.writes.push(data.to_vec());
                    return Err(error);
                }
                None => self
                    .write_limit
                    .map_or(data.len(), |limit| data.len().min(limit)),
            };
            self.writes.push(data[..taken.min(data.len())].to_vec());
            Ok(taken)
        }

        fn read(&mut self, buf: &mut [u8]) -> Result<usize, PipeError> {
            let chunk = self.read_answers.pop_front().ok_or(PipeError::Timeout)??;
            buf[..chunk.len()].copy_from_slice(&chunk);
            Ok(chunk.len())
        }
    }

    fn valid_response(buttons: u32) -> Vec<u8> {
        let mut bytes = vec![0x00, 0x06, 0xFE, 0xAF, 0, 0, 0, 0];
        bytes.extend_from_slice(&buttons.to_le_bytes());
        bytes.extend_from_slice(&[127, 127]);
        bytes.resize(USB_RESPONSE_LEN, 0);
        bytes
    }

    fn open(pipe: Scripted) -> Result<Link<Scripted>, LinkError> {
        Link::open(pipe, Duration::ZERO)
    }

    #[test]
    fn fresh_psp_gets_the_hello_and_echoes_its_answer() {
        let hello: Vec<u8> = (1..=12).collect();
        let pipe = Scripted {
            write_answers: VecDeque::from([Ok(4), Err(PipeError::Timeout)]),
            read_answers: VecDeque::from([Ok(hello.clone())]),
            ..Scripted::default()
        };
        let link = open(pipe).unwrap();
        let writes = &link.pipe.writes;
        assert_eq!(writes[0], HOSTFS_MAGIC);
        assert_eq!(writes[1], [0; HELLO_LEN]);
        assert_eq!(writes[2], hello);
    }

    #[test]
    fn initialised_psp_needs_no_hello_answer() {
        let pipe = Scripted {
            write_answers: VecDeque::from([Ok(4), Ok(HELLO_LEN)]),
            ..Scripted::default()
        };
        let link = open(pipe).unwrap();
        assert_eq!(link.pipe.writes.len(), 2);
    }

    #[test]
    fn disconnect_during_the_hello_is_reported() {
        let pipe = Scripted {
            write_answers: VecDeque::from([Err(PipeError::Disconnected)]),
            ..Scripted::default()
        };
        assert_eq!(
            open(pipe).unwrap_err(),
            LinkError::Pipe(PipeError::Disconnected)
        );
    }

    #[test]
    fn frame_goes_out_as_header_then_image_and_the_response_is_read() {
        let pipe = Scripted {
            write_answers: VecDeque::from([Ok(4), Ok(HELLO_LEN)]),
            // The response arrives in two pieces, as USB packets may split it.
            read_answers: VecDeque::from([
                Ok(valid_response(0x4001)[..50].to_vec()),
                Ok(valid_response(0x4001)[50..].to_vec()),
            ]),
            ..Scripted::default()
        };
        let mut link = open(pipe).unwrap();
        let response = link.send_jpeg(&[0xFF, 0xD8, 0xFF, 0xD9]).unwrap();

        assert_eq!(response.buttons, 0x4001);
        let writes = &link.pipe.writes;
        assert_eq!(writes[2], FrameHeader::jpeg(4).encode());
        assert_eq!(writes[3], [0xFF, 0xD8, 0xFF, 0xD9]);
    }

    #[test]
    fn raw_frame_goes_out_with_the_uncompressed_header() {
        let pipe = Scripted {
            write_answers: VecDeque::from([Ok(4), Ok(HELLO_LEN)]),
            read_answers: VecDeque::from([Ok(valid_response(0))]),
            ..Scripted::default()
        };
        let mut link = open(pipe).unwrap();
        let frame = vec![7; RAW_FRAME_LEN];
        link.send_raw(&frame).unwrap();

        let writes = &link.pipe.writes;
        assert_eq!(writes[2], FrameHeader::uncompressed(391_680).encode());
        assert_eq!(writes[3].len(), RAW_FRAME_LEN);
    }

    #[test]
    fn a_poll_sends_the_header_alone_and_reads_the_buttons() {
        let pipe = Scripted {
            write_answers: VecDeque::from([Ok(4), Ok(HELLO_LEN)]),
            read_answers: VecDeque::from([Ok(valid_response(0x4000))]),
            ..Scripted::default()
        };
        let mut link = open(pipe).unwrap();
        let response = link.poll().unwrap();

        assert_eq!(response.buttons, 0x4000);
        let writes = &link.pipe.writes;
        assert_eq!(
            writes.len(),
            3,
            "the hello and the header, nothing after it"
        );
        assert_eq!(writes[2], FrameHeader::poll().encode());
    }

    #[test]
    fn raw_frame_of_the_wrong_size_is_refused_before_anything_is_sent() {
        let pipe = Scripted {
            write_answers: VecDeque::from([Ok(4), Ok(HELLO_LEN)]),
            ..Scripted::default()
        };
        let mut link = open(pipe).unwrap();
        assert_eq!(link.send_raw(&[0; 10]).unwrap_err(), LinkError::RawSize(10));
        assert_eq!(link.pipe.writes.len(), 2, "only the hello was written");
    }

    #[test]
    fn partial_writes_are_continued_until_the_image_is_out() {
        let pipe = Scripted {
            write_answers: VecDeque::from([Ok(4), Ok(HELLO_LEN)]),
            read_answers: VecDeque::from([Ok(valid_response(0))]),
            write_limit: Some(3),
            ..Scripted::default()
        };
        let mut link = open(pipe).unwrap();
        link.send_jpeg(&[1, 2, 3, 4, 5, 6, 7]).unwrap();

        // Everything after the hello is the header followed by the image, however it was cut.
        let sent: Vec<u8> = link.pipe.writes[2..].concat();
        let mut expected = FrameHeader::jpeg(7).encode().to_vec();
        expected.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(sent, expected);
    }

    #[test]
    fn response_with_a_bad_magic_is_a_protocol_error() {
        let mut bad = valid_response(0);
        bad[3] = 0;
        let pipe = Scripted {
            write_answers: VecDeque::from([Ok(4), Ok(HELLO_LEN)]),
            read_answers: VecDeque::from([Ok(bad)]),
            ..Scripted::default()
        };
        let mut link = open(pipe).unwrap();
        assert!(matches!(link.send_jpeg(&[1]), Err(LinkError::Protocol(_))));
    }

    #[test]
    fn silent_psp_is_a_timeout_not_a_hang() {
        let pipe = Scripted {
            write_answers: VecDeque::from([Ok(4), Ok(HELLO_LEN)]),
            ..Scripted::default()
        };
        let mut link = open(pipe).unwrap();
        assert_eq!(
            link.send_jpeg(&[1]).unwrap_err(),
            LinkError::Pipe(PipeError::Timeout)
        );
    }
}
