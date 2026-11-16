use std::fs;
use std::path::Path;

use crate::error::Error;

/// Where the program goes on the Memory Stick, below its root.
const TARGET: [&str; 3] = ["PSP", "GAME", "Sidescreen"];

/// The settings file of the PSP program. The person may have changed it on the PSP, so an
/// existing one is kept.
const SETTINGS_FILE: &str = "pspdisp.cfg";

/// What [`install_package`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Installed {
    /// How many files were written.
    pub files: usize,
    /// Whether an existing settings file was left as it was.
    pub kept_settings: bool,
}

/// Whether `root` looks like a PSP Memory Stick: it has the `PSP` folder the PSP makes.
#[must_use]
pub fn is_memory_stick(root: &Path) -> bool {
    root.join("PSP").is_dir()
}

/// Copies the PSP program from `package` (the folder that holds `PSP/GAME/Sidescreen`) to the
/// Memory Stick at `stick`.
///
/// Only the folder `PSP/GAME/Sidescreen` on the stick is written to. Files of an earlier version
/// are replaced, except the settings file, and nothing else on the stick is touched, deleted or
/// formatted.
///
/// # Errors
///
/// [`Error::NoMemoryStick`] if `stick` has no `PSP` folder, otherwise the I/O error.
pub fn install_package(package: &Path, stick: &Path) -> Result<Installed, Error> {
    if !is_memory_stick(stick) {
        return Err(Error::NoMemoryStick);
    }
    let source = TARGET
        .iter()
        .fold(package.to_path_buf(), |path, part| path.join(part));
    let target = TARGET
        .iter()
        .fold(stick.to_path_buf(), |path, part| path.join(part));

    let mut installed = Installed {
        files: 0,
        kept_settings: false,
    };
    copy_folder(&source, &target, true, &mut installed)?;
    Ok(installed)
}

fn copy_folder(
    source: &Path,
    target: &Path,
    top_level: bool,
    installed: &mut Installed,
) -> Result<(), Error> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let destination = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_folder(&entry.path(), &destination, false, installed)?;
        } else if top_level && entry.file_name() == SETTINGS_FILE && destination.exists() {
            installed.kept_settings = true;
        } else {
            fs::copy(entry.path(), &destination)?;
            installed.files += 1;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("sidescreen-stick-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A package with a program, its settings and a folder of pictures.
    fn package(root: &Path) {
        let folder = root.join("PSP/GAME/Sidescreen");
        fs::create_dir_all(folder.join("graphics")).unwrap();
        fs::write(folder.join("EBOOT.PBP"), b"program").unwrap();
        fs::write(folder.join("pspdisp.cfg"), b"autoMode 2\n").unwrap();
        fs::write(folder.join("graphics/keys.png"), b"picture").unwrap();
    }

    fn stick(root: &Path) {
        fs::create_dir_all(root.join("PSP/GAME")).unwrap();
    }

    #[test]
    fn a_folder_with_a_psp_folder_is_a_memory_stick() {
        let dir = temp_dir("is-stick");
        assert!(!is_memory_stick(&dir));
        stick(&dir);
        assert!(is_memory_stick(&dir));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn the_program_is_copied_with_its_folders() {
        let (source, target) = (temp_dir("copy-source"), temp_dir("copy-target"));
        package(&source);
        stick(&target);

        let installed = install_package(&source, &target).unwrap();

        assert_eq!(
            installed,
            Installed {
                files: 3,
                kept_settings: false
            }
        );
        let folder = target.join("PSP/GAME/Sidescreen");
        assert_eq!(fs::read(folder.join("EBOOT.PBP")).unwrap(), b"program");
        assert_eq!(
            fs::read(folder.join("graphics/keys.png")).unwrap(),
            b"picture"
        );
        fs::remove_dir_all(source).unwrap();
        fs::remove_dir_all(target).unwrap();
    }

    #[test]
    fn an_earlier_version_is_replaced_but_the_settings_the_person_made_are_kept() {
        let (source, target) = (temp_dir("update-source"), temp_dir("update-target"));
        package(&source);
        stick(&target);
        let folder = target.join("PSP/GAME/Sidescreen");
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join("EBOOT.PBP"), b"old program").unwrap();
        fs::write(folder.join("pspdisp.cfg"), b"autoMode 0\n").unwrap();

        let installed = install_package(&source, &target).unwrap();

        assert!(installed.kept_settings);
        assert_eq!(fs::read(folder.join("EBOOT.PBP")).unwrap(), b"program");
        assert_eq!(
            fs::read(folder.join("pspdisp.cfg")).unwrap(),
            b"autoMode 0\n"
        );
        fs::remove_dir_all(source).unwrap();
        fs::remove_dir_all(target).unwrap();
    }

    #[test]
    fn nothing_else_on_the_stick_is_touched() {
        let (source, target) = (temp_dir("other-source"), temp_dir("other-target"));
        package(&source);
        stick(&target);
        let game = target.join("PSP/GAME/SomeoneElsesGame");
        fs::create_dir_all(&game).unwrap();
        fs::write(game.join("EBOOT.PBP"), b"their game").unwrap();
        fs::write(target.join("MUSIC.mp3"), b"a song").unwrap();

        install_package(&source, &target).unwrap();

        assert_eq!(fs::read(game.join("EBOOT.PBP")).unwrap(), b"their game");
        assert_eq!(fs::read(target.join("MUSIC.mp3")).unwrap(), b"a song");
        fs::remove_dir_all(source).unwrap();
        fs::remove_dir_all(target).unwrap();
    }

    #[test]
    fn a_drive_that_is_not_a_memory_stick_is_refused_and_left_alone() {
        let (source, target) = (temp_dir("refuse-source"), temp_dir("refuse-target"));
        package(&source);

        assert!(matches!(
            install_package(&source, &target),
            Err(Error::NoMemoryStick)
        ));
        assert_eq!(
            fs::read_dir(&target).unwrap().count(),
            0,
            "nothing was written"
        );
        fs::remove_dir_all(source).unwrap();
        fs::remove_dir_all(target).unwrap();
    }

    #[test]
    fn a_package_that_is_not_there_is_an_io_error() {
        let target = temp_dir("no-package-target");
        stick(&target);
        let missing = std::env::temp_dir().join("sidescreen-stick-no-such-package");
        assert!(matches!(
            install_package(&missing, &target),
            Err(Error::Io(_))
        ));
        fs::remove_dir_all(target).unwrap();
    }
}
