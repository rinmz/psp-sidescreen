use std::path::{Path, PathBuf};

use super::SETTINGS_FILE;
use crate::error::Error;

/// Files a complete driver package must contain, compared without regard to case.
const REQUIRED_FILES: [&str; 3] = ["MttVDD.inf", "MttVDD.dll", "mttvdd.cat"];

/// A folder that holds a complete driver package.
#[derive(Debug, Clone)]
pub struct Package {
    dir: PathBuf,
}

impl Package {
    /// Opens the package in `dir`, checking that every required file is there.
    ///
    /// # Errors
    ///
    /// [`Error::IncompletePackage`] naming the first missing file, or the I/O error if the
    /// folder cannot be read.
    pub fn open(dir: impl Into<PathBuf>) -> Result<Self, Error> {
        let dir = dir.into();
        let present: Vec<String> = std::fs::read_dir(&dir)?
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().to_lowercase())
            .collect();
        for file in REQUIRED_FILES {
            if !present.contains(&file.to_lowercase()) {
                return Err(Error::IncompletePackage { dir, file });
            }
        }
        Ok(Self { dir })
    }

    /// The driver's INF file, which tells Windows what the package installs.
    #[must_use]
    pub fn inf(&self) -> PathBuf {
        self.dir.join("MttVDD.inf")
    }

    /// The settings file that ships with the package, if any.
    #[must_use]
    pub fn default_settings(&self) -> Option<PathBuf> {
        let path = self.dir.join(SETTINGS_FILE);
        path.is_file().then_some(path)
    }

    /// The folder the package lives in.
    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("sidescreen-vdd-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_complete_package_opens_whatever_the_case_of_the_names() {
        let dir = temp_dir("complete");
        for file in ["mttvdd.INF", "MTTVDD.dll", "MttVDD.cat"] {
            std::fs::write(dir.join(file), b"x").unwrap();
        }
        let package = Package::open(&dir).unwrap();
        assert_eq!(package.dir(), dir);
        assert_eq!(package.default_settings(), None);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn the_first_missing_file_is_named() {
        let dir = temp_dir("missing");
        std::fs::write(dir.join("MttVDD.inf"), b"x").unwrap();
        std::fs::write(dir.join("MttVDD.dll"), b"x").unwrap();
        match Package::open(&dir) {
            Err(Error::IncompletePackage { file, .. }) => assert_eq!(file, "mttvdd.cat"),
            other => panic!("expected an incomplete package, got {other:?}"),
        }
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_folder_that_is_not_there_is_an_io_error() {
        let missing = std::env::temp_dir().join("sidescreen-vdd-does-not-exist");
        assert!(matches!(Package::open(missing), Err(Error::Io(_))));
    }

    #[test]
    fn the_settings_that_ship_with_the_package_are_found() {
        let dir = temp_dir("settings");
        for file in REQUIRED_FILES.iter().copied().chain([SETTINGS_FILE]) {
            std::fs::write(dir.join(file), b"x").unwrap();
        }
        let package = Package::open(&dir).unwrap();
        assert!(package.default_settings().unwrap().ends_with(SETTINGS_FILE));
        std::fs::remove_dir_all(dir).unwrap();
    }
}
