//! The program on the PSP: finding the Memory Stick and putting the program on it.
//!
//! The PSP shows its Memory Stick to Windows as a removable drive when its USB Connection is
//! chosen in Settings. Only the folder `PSP/GAME/Sidescreen` on it is ever written to.

mod drives;
mod install;

use std::path::{Path, PathBuf};

pub use install::{Installed, install_package, is_memory_stick, program_is_installed};

use crate::error::Error;

/// The Memory Sticks that are connected: removable drives that have a `PSP` folder.
#[must_use]
pub fn memory_sticks() -> Vec<PathBuf> {
    drives::removable_roots()
        .into_iter()
        .filter(|root| is_memory_stick(root))
        .collect()
}

/// Which Memory Stick to use: the one named, if it is one; otherwise the only one found.
///
/// # Errors
///
/// [`Error::NoMemoryStick`] if there is none, or the one named is not one;
/// [`Error::SeveralMemoryCards`] if more than one was found and none was named.
pub fn choose(named: Option<&Path>, found: Vec<PathBuf>) -> Result<PathBuf, Error> {
    if let Some(named) = named {
        return if is_memory_stick(named) {
            Ok(named.to_path_buf())
        } else {
            Err(Error::NoMemoryStick)
        };
    }
    let mut found = found;
    match found.len() {
        0 => Err(Error::NoMemoryStick),
        1 => Ok(found.remove(0)),
        _ => Err(Error::SeveralMemoryCards(found)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stick(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("sidescreen-choose-{name}-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("PSP")).unwrap();
        dir
    }

    #[test]
    fn the_only_stick_is_used() {
        let only = stick("only");
        assert_eq!(choose(None, vec![only.clone()]).unwrap(), only);
        std::fs::remove_dir_all(only).unwrap();
    }

    #[test]
    fn no_stick_is_an_error_the_person_can_act_on() {
        assert!(matches!(
            choose(None, Vec::new()),
            Err(Error::NoMemoryStick)
        ));
    }

    #[test]
    fn several_sticks_must_be_told_apart() {
        let (a, b) = (stick("a"), stick("b"));
        match choose(None, vec![a.clone(), b.clone()]) {
            Err(Error::SeveralMemoryCards(list)) => assert_eq!(list, [a.clone(), b.clone()]),
            other => panic!("expected several cards, got {other:?}"),
        }
        std::fs::remove_dir_all(a).unwrap();
        std::fs::remove_dir_all(b).unwrap();
    }

    #[test]
    fn a_stick_that_is_named_wins_over_the_list() {
        let (named, other) = (stick("named"), stick("other"));
        assert_eq!(choose(Some(&named), vec![other.clone()]).unwrap(), named);
        std::fs::remove_dir_all(named).unwrap();
        std::fs::remove_dir_all(other).unwrap();
    }

    #[test]
    fn a_named_drive_that_is_not_a_stick_is_refused() {
        let not_a_stick = std::env::temp_dir();
        assert!(matches!(
            choose(Some(&not_a_stick.join("nothing-here")), Vec::new()),
            Err(Error::NoMemoryStick)
        ));
    }
}
