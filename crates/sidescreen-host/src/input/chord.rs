//! A shortcut a person typed: keys pressed together, such as Ctrl+Space.
//!
//! A chord is written as the names a web page gives keys (`KeyboardEvent.code`) joined by `+`,
//! for example `ControlLeft+Space`. The window records those names as the person presses the
//! keys, and this module is the one place that knows which of them are understood.

use std::fmt;

use super::vk;

/// Most keys a chord may hold. A longer one is a mistake, not a shortcut.
const MAX_KEYS: usize = 5;

const SHIFT: u16 = 0x10;
const RIGHT_WIN: u16 = 0x5C;
const CAPS_LOCK: u16 = 0x14;
const PRINT_SCREEN: u16 = 0x2C;
const INSERT: u16 = 0x2D;

/// Why a chord could not be understood.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChordError {
    /// No key at all.
    Empty,
    /// More keys than [`MAX_KEYS`].
    TooManyKeys,
    /// A name that is not a key this program knows.
    UnknownKey(String),
}

impl fmt::Display for ChordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("the shortcut has no keys"),
            Self::TooManyKeys => write!(f, "a shortcut can have at most {MAX_KEYS} keys"),
            Self::UnknownKey(name) => write!(f, "{name:?} is not a key that can be used"),
        }
    }
}

impl std::error::Error for ChordError {}

/// Keys pressed together, in this order, and released in the opposite order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chord {
    keys: Vec<u16>,
}

impl Chord {
    /// Reads a chord such as `ControlLeft+Space`.
    ///
    /// # Errors
    ///
    /// [`ChordError`] if the text has no keys, too many, or a name that is not understood.
    pub fn parse(text: &str) -> Result<Self, ChordError> {
        let names: Vec<&str> = text
            .split('+')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .collect();
        if names.is_empty() {
            return Err(ChordError::Empty);
        }
        if names.len() > MAX_KEYS {
            return Err(ChordError::TooManyKeys);
        }
        let keys = names
            .into_iter()
            .map(|name| key_for_code(name).ok_or_else(|| ChordError::UnknownKey(name.to_owned())))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { keys })
    }

    /// The Windows virtual-key codes, in the order they are pressed.
    #[must_use]
    pub fn keys(&self) -> &[u16] {
        &self.keys
    }
}

/// The virtual-key code for a `KeyboardEvent.code` name.
///
/// Left and right modifiers are the same key to Windows' shortcuts, so both give the generic
/// code (the Windows key is the exception, which has two).
fn key_for_code(code: &str) -> Option<u16> {
    if let Some(letter) = code.strip_prefix("Key") {
        return single(letter, |byte| {
            byte.is_ascii_uppercase()
                .then(|| 0x41 + u16::from(byte - b'A'))
        });
    }
    if let Some(digit) = code.strip_prefix("Digit") {
        return single(digit, |byte| {
            byte.is_ascii_digit().then(|| 0x30 + u16::from(byte - b'0'))
        });
    }
    if let Some(number) = code
        .strip_prefix('F')
        .and_then(|rest| rest.parse::<u16>().ok())
    {
        return (1..=24).contains(&number).then(|| 0x70 + number - 1);
    }
    Some(match code {
        "ControlLeft" | "ControlRight" => vk::CONTROL,
        "ShiftLeft" | "ShiftRight" => SHIFT,
        "AltLeft" | "AltRight" => vk::MENU,
        "MetaLeft" => vk::LWIN,
        "MetaRight" => RIGHT_WIN,
        "Space" => vk::SPACE,
        "Enter" | "NumpadEnter" => vk::RETURN,
        "Tab" => vk::TAB,
        "Escape" => vk::ESCAPE,
        "Backspace" => vk::BACK,
        "Delete" => vk::DELETE,
        "Insert" => INSERT,
        "Home" => vk::HOME,
        "End" => vk::END,
        "PageUp" => vk::PRIOR,
        "PageDown" => vk::NEXT,
        "ArrowLeft" => vk::LEFT,
        "ArrowUp" => vk::UP,
        "ArrowRight" => vk::RIGHT,
        "ArrowDown" => vk::DOWN,
        "CapsLock" => CAPS_LOCK,
        "PrintScreen" => PRINT_SCREEN,
        "Backquote" => 0xC0,
        "Minus" => 0xBD,
        "Equal" => 0xBB,
        "BracketLeft" => 0xDB,
        "BracketRight" => 0xDD,
        "Backslash" => 0xDC,
        "Semicolon" => 0xBA,
        "Quote" => 0xDE,
        "Comma" => 0xBC,
        "Period" => 0xBE,
        "Slash" => 0xBF,
        "AudioVolumeUp" => vk::VOLUME_UP,
        "AudioVolumeDown" => vk::VOLUME_DOWN,
        "AudioVolumeMute" => vk::VOLUME_MUTE,
        "MediaPlayPause" => vk::MEDIA_PLAY_PAUSE,
        "MediaTrackNext" => vk::MEDIA_NEXT,
        "MediaTrackPrevious" => vk::MEDIA_PREVIOUS,
        _ => return None,
    })
}

/// Applies `convert` to the one byte of `text`, or gives `None` if `text` is not one byte.
fn single(text: &str, convert: impl Fn(u8) -> Option<u16>) -> Option<u16> {
    match text.as_bytes() {
        [byte] => convert(*byte),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handys_shortcut_is_understood() {
        let chord = Chord::parse("ControlLeft+Space").unwrap();
        assert_eq!(chord.keys(), [vk::CONTROL, vk::SPACE]);
    }

    #[test]
    fn letters_digits_and_function_keys_map_to_their_codes() {
        assert_eq!(Chord::parse("KeyA").unwrap().keys(), [0x41]);
        assert_eq!(Chord::parse("KeyZ").unwrap().keys(), [0x5A]);
        assert_eq!(Chord::parse("Digit0").unwrap().keys(), [0x30]);
        assert_eq!(Chord::parse("Digit9").unwrap().keys(), [0x39]);
        assert_eq!(Chord::parse("F1").unwrap().keys(), [0x70]);
        assert_eq!(Chord::parse("F12").unwrap().keys(), [0x7B]);
        assert_eq!(Chord::parse("F24").unwrap().keys(), [0x87]);
    }

    #[test]
    fn left_and_right_modifiers_are_the_same_key() {
        assert_eq!(
            Chord::parse("ControlLeft").unwrap(),
            Chord::parse("ControlRight").unwrap()
        );
        assert_eq!(
            Chord::parse("ShiftLeft").unwrap(),
            Chord::parse("ShiftRight").unwrap()
        );
        assert_eq!(
            Chord::parse("AltLeft").unwrap(),
            Chord::parse("AltRight").unwrap()
        );
    }

    #[test]
    fn keys_keep_the_order_they_were_typed_in() {
        let chord = Chord::parse("ControlLeft+ShiftLeft+AltLeft+KeyH").unwrap();
        assert_eq!(chord.keys(), [vk::CONTROL, SHIFT, vk::MENU, 0x48]);
    }

    #[test]
    fn spaces_around_the_names_are_ignored() {
        assert_eq!(
            Chord::parse(" ControlLeft + Space ").unwrap(),
            Chord::parse("ControlLeft+Space").unwrap()
        );
    }

    #[test]
    fn nothing_is_not_a_shortcut() {
        for text in ["", "+", " + + "] {
            assert_eq!(Chord::parse(text), Err(ChordError::Empty), "{text:?}");
        }
    }

    #[test]
    fn a_name_that_is_not_a_key_is_refused_by_name() {
        assert_eq!(
            Chord::parse("ControlLeft+Launch"),
            Err(ChordError::UnknownKey("Launch".to_owned()))
        );
        for bad in ["Keya", "KeyAB", "Key", "Digit10", "F0", "F25", "Fx"] {
            assert!(Chord::parse(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn too_many_keys_is_refused() {
        let six = "ControlLeft+ShiftLeft+AltLeft+MetaLeft+KeyA+KeyB";
        assert_eq!(Chord::parse(six), Err(ChordError::TooManyKeys));
        assert!(Chord::parse("ControlLeft+ShiftLeft+AltLeft+MetaLeft+KeyA").is_ok());
    }

    #[test]
    fn every_name_a_window_may_send_for_a_common_key_is_known() {
        for code in [
            "Space",
            "Enter",
            "Tab",
            "Escape",
            "Backspace",
            "Delete",
            "Insert",
            "Home",
            "End",
            "PageUp",
            "PageDown",
            "ArrowLeft",
            "ArrowUp",
            "ArrowRight",
            "ArrowDown",
            "Backquote",
            "Minus",
            "Equal",
            "BracketLeft",
            "BracketRight",
            "Backslash",
            "Semicolon",
            "Quote",
            "Comma",
            "Period",
            "Slash",
            "MetaLeft",
            "MetaRight",
            "AudioVolumeUp",
            "MediaPlayPause",
            "PrintScreen",
            "CapsLock",
        ] {
            assert!(key_for_code(code).is_some(), "{code}");
        }
    }
}
