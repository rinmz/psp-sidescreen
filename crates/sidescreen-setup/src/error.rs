use std::fmt;
use std::io;
use std::path::PathBuf;

/// Why a step of the setup failed. The text is written for the person running the installer.
#[derive(Debug)]
pub enum Error {
    /// The step changes the system and needs administrator rights.
    NotElevated,
    /// A driver package folder is missing a file.
    IncompletePackage {
        /// The folder that was checked.
        dir: PathBuf,
        /// The file that was not found.
        file: &'static str,
    },
    /// A call to Windows failed.
    Windows {
        /// What was being done.
        step: &'static str,
        /// What Windows said.
        source: windows::core::Error,
    },
    /// Windows refused the driver because it is not signed.
    UnsignedDriver,
    /// A file could not be read or written.
    Io(io::Error),
    /// There is no PSP Memory Stick to copy to: none was found, or the one named is not one.
    NoMemoryStick,
    /// More than one Memory Stick is connected, so the one to use has to be named.
    SeveralMemoryCards(Vec<PathBuf>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotElevated => f.write_str("this step needs administrator rights"),
            Self::IncompletePackage { dir, file } => {
                write!(f, "the package in {} is missing {file}", dir.display())
            }
            Self::Windows { step, source } => write!(f, "could not {step}: {source}"),
            Self::UnsignedDriver => f.write_str(
                "Windows refused the driver because it is not signed by a publisher it trusts",
            ),
            Self::Io(error) => write!(f, "file error: {error}"),
            Self::NoMemoryStick => f.write_str(
                "no PSP Memory Stick was found; connect the PSP and choose USB Connection in its Settings",
            ),
            Self::SeveralMemoryCards(drives) => {
                let names: Vec<_> = drives.iter().map(|drive| drive.display().to_string()).collect();
                write!(f, "more than one Memory Stick is connected ({}); name one", names.join(", "))
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl Error {
    /// The error for a driver install that Windows rejected: an unsigned package is told apart,
    /// because the person can be given a better answer for it than for any other failure.
    pub(crate) fn from_install(step: &'static str, source: windows::core::Error) -> Self {
        if is_unsigned(source.code().0) {
            Self::UnsignedDriver
        } else {
            Self::Windows { step, source }
        }
    }
}

/// `ERROR_NO_CATALOG_FOR_OEM_INF`: "the third-party INF does not contain digital signature
/// information". Windows reports it as it is, or folded into an `HRESULT` with the Win32 prefix,
/// depending on the call.
const NO_CATALOG_FOR_OEM_INF: u32 = 0xE000_022F;

fn is_unsigned(code: i32) -> bool {
    let code = code.cast_unsigned();
    code == NO_CATALOG_FOR_OEM_INF || code == (0x8007_0000 | (NO_CATALOG_FOR_OEM_INF & 0xFFFF))
}

/// Attaches the name of a step to a Windows error.
pub(crate) trait Step<T> {
    fn step(self, name: &'static str) -> Result<T, Error>;
}

impl<T> Step<T> for windows::core::Result<T> {
    fn step(self, name: &'static str) -> Result<T, Error> {
        self.map_err(|source| Error::Windows { step: name, source })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_signature_is_recognised_in_both_forms_windows_reports_it() {
        assert!(is_unsigned(0xE000_022F_u32.cast_signed()));
        assert!(is_unsigned(0x8007_022F_u32.cast_signed()));
    }

    #[test]
    fn other_codes_are_not_taken_for_a_missing_signature() {
        for code in [
            0,
            5,
            0x8007_0005_u32.cast_signed(),
            0xE000_0203_u32.cast_signed(),
        ] {
            assert!(!is_unsigned(code), "{code:#x}");
        }
    }

    #[test]
    fn messages_are_written_for_a_person() {
        assert!(Error::NotElevated.to_string().contains("administrator"));
        assert!(Error::NoMemoryStick.to_string().contains("USB Connection"));
        assert!(Error::UnsignedDriver.to_string().contains("not signed"));
    }
}
