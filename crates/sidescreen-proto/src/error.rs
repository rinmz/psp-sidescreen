use std::fmt;

/// Why a message could not be decoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Fewer bytes were given than the message needs.
    Truncated {
        /// Bytes the message needs.
        needed: usize,
        /// Bytes that were given.
        got: usize,
    },
    /// The first four bytes are not the PSPdisp magic number.
    BadMagic(u32),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated { needed, got } => {
                write!(f, "message needs {needed} bytes, got {got}")
            }
            Self::BadMagic(magic) => write!(f, "bad magic number {magic:#010x}"),
        }
    }
}

impl std::error::Error for Error {}
