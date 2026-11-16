//! What the person chose, kept in a file between runs.

use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sidescreen_host::convert::Quality;
use sidescreen_host::session::Options;

/// What the PSP shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Mode {
    /// The PSP gets a screen of its own.
    Extended,
    /// The PSP shows a monitor that already exists.
    Mirror,
}

/// How the picture is compressed; the names the window shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum QualityName {
    Lossless,
    High,
    Medium,
    Low,
}

impl From<QualityName> for Quality {
    fn from(name: QualityName) -> Self {
        match name {
            QualityName::Lossless => Self::Lossless,
            QualityName::High => Self::High,
            QualityName::Medium => Self::Medium,
            QualityName::Low => Self::Low,
        }
    }
}

/// The settings of the window, as it sends and receives them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Settings {
    pub(crate) mode: Mode,
    /// Size of the PSP's own screen, such as `1366x768`.
    pub(crate) size: String,
    /// Index of the monitor to show when `mode` is `mirror`.
    pub(crate) monitor: usize,
    pub(crate) quality: QualityName,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Extended,
            size: "1366x768".to_owned(),
            monitor: 0,
            quality: QualityName::Medium,
        }
    }
}

impl Settings {
    /// The options a session is run with.
    ///
    /// # Errors
    ///
    /// A message for the person if the size is not written as `WIDTHxHEIGHT`.
    pub(crate) fn to_options(&self) -> Result<Options, String> {
        let size = self
            .size
            .split_once('x')
            .and_then(|(width, height)| Some((width.parse().ok()?, height.parse().ok()?)))
            .ok_or_else(|| format!("the size {:?} is not written like 1366x768", self.size))?;
        Ok(Options {
            virtual_monitor: self.mode == Mode::Extended,
            size,
            monitor: self.monitor,
            quality: self.quality.into(),
        })
    }
}

/// Reads the settings from `path`. A file that is missing or cannot be understood gives the
/// defaults: a damaged settings file should not stop the program from starting.
#[must_use]
pub(crate) fn load(path: &Path) -> Settings {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Writes the settings to `path`, creating its folder.
///
/// # Errors
///
/// The I/O error if the file cannot be written.
pub(crate) fn save(path: &Path, settings: &Settings) -> io::Result<()> {
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }
    let text = serde_json::to_string_pretty(settings).map_err(io::Error::other)?;
    fs::write(path, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file(name: &str) -> std::path::PathBuf {
        let folder =
            std::env::temp_dir().join(format!("sidescreen-test-{}-{name}", std::process::id()));
        fs::create_dir_all(&folder).unwrap();
        folder.join("settings.json")
    }

    #[test]
    fn settings_survive_a_round_trip() {
        let path = temp_file("round-trip");
        let settings = Settings {
            mode: Mode::Mirror,
            size: "1920x1080".to_owned(),
            monitor: 2,
            quality: QualityName::Lossless,
        };
        save(&path, &settings).unwrap();
        assert_eq!(load(&path), settings);
    }

    #[test]
    fn missing_file_gives_the_defaults() {
        assert_eq!(
            load(Path::new("no/such/settings.json")),
            Settings::default()
        );
    }

    #[test]
    fn damaged_file_gives_the_defaults() {
        let path = temp_file("damaged");
        fs::write(&path, "{ not json").unwrap();
        assert_eq!(load(&path), Settings::default());
    }

    #[test]
    fn file_written_by_the_window_is_understood() {
        // The shape the window sends: lower-case names.
        let text = r#"{"mode":"mirror","size":"800x600","monitor":1,"quality":"low"}"#;
        let settings: Settings = serde_json::from_str(text).unwrap();
        assert_eq!(settings.mode, Mode::Mirror);
        assert_eq!(settings.quality, QualityName::Low);
    }

    #[test]
    fn extended_mode_asks_for_a_virtual_monitor_of_the_chosen_size() {
        let options = Settings::default().to_options().unwrap();
        assert!(options.virtual_monitor);
        assert_eq!(options.size, (1366, 768));
        assert_eq!(options.quality, Quality::Medium);
    }

    #[test]
    fn mirror_mode_shows_the_chosen_monitor() {
        let settings = Settings {
            mode: Mode::Mirror,
            monitor: 3,
            ..Settings::default()
        };
        let options = settings.to_options().unwrap();
        assert!(!options.virtual_monitor);
        assert_eq!(options.monitor, 3);
    }

    #[test]
    fn size_that_is_not_width_by_height_is_refused() {
        for bad in ["", "1366", "1366x", "x768", "axb", "1366*768"] {
            let settings = Settings {
                size: bad.to_owned(),
                ..Settings::default()
            };
            assert!(settings.to_options().is_err(), "{bad:?} should be refused");
        }
    }
}
