//! What one PSP button is set to do.

use super::action::{Action, Output, Pointer};
use super::chord::Chord;

/// What a button does: nothing, one of the fixed actions, or a shortcut the person typed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Binding {
    /// The button does nothing.
    #[default]
    Nothing,
    /// One of the fixed actions.
    Preset(Action),
    /// A shortcut the person typed.
    Custom(Chord),
}

/// What carrying out a binding means, once it is known to do something.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Plan {
    /// Keys pressed together, in this order, and released in the opposite order.
    Keys(Vec<u16>),
    /// A mouse button, held while the PSP button is held.
    Click(Pointer),
    /// One notch of the wheel; positive is up.
    Wheel(i32),
}

/// The prefix of a setting that holds a typed shortcut: `custom:ControlLeft+Space`.
const CUSTOM_PREFIX: &str = "custom:";

impl Binding {
    /// The binding a settings file holds: an action id, or `custom:` and a chord.
    ///
    /// A setting that is not understood gives [`Nothing`](Self::Nothing): an old or damaged file
    /// must not make a button do something the person did not choose.
    #[must_use]
    pub fn from_setting(setting: &str) -> Self {
        if let Some(chord) = setting.strip_prefix(CUSTOM_PREFIX) {
            return Chord::parse(chord).map_or(Self::Nothing, Self::Custom);
        }
        Action::from_id(setting).map_or(Self::Nothing, Self::from)
    }

    /// What to do for this binding, or `None` if it does nothing.
    pub(super) fn plan(&self) -> Option<Plan> {
        match self {
            Self::Nothing => None,
            Self::Preset(action) => action.output().map(|output| match output {
                Output::Keys(keys) => Plan::Keys(keys.to_vec()),
                Output::Click(pointer) => Plan::Click(pointer),
                Output::Wheel(notches) => Plan::Wheel(notches),
            }),
            Self::Custom(chord) => Some(Plan::Keys(chord.keys().to_vec())),
        }
    }
}

impl From<Action> for Binding {
    fn from(action: Action) -> Self {
        if action == Action::Nothing {
            Self::Nothing
        } else {
            Self::Preset(action)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_preset_id_gives_that_action() {
        assert_eq!(
            Binding::from_setting("enter"),
            Binding::Preset(Action::Enter)
        );
    }

    #[test]
    fn a_custom_setting_gives_the_typed_shortcut() {
        let binding = Binding::from_setting("custom:ControlLeft+Space");
        assert_eq!(
            binding,
            Binding::Custom(Chord::parse("ControlLeft+Space").unwrap())
        );
    }

    #[test]
    fn settings_that_are_not_understood_do_nothing() {
        for setting in [
            "",
            "nothing",
            "launch_missiles",
            "custom:",
            "custom:Launch",
            "custom:+",
        ] {
            assert_eq!(
                Binding::from_setting(setting),
                Binding::Nothing,
                "{setting:?}"
            );
        }
    }

    #[test]
    fn nothing_has_no_plan_and_the_others_do() {
        assert_eq!(Binding::Nothing.plan(), None);
        assert!(Binding::from(Action::Enter).plan().is_some());
        assert_eq!(Binding::from(Action::Nothing), Binding::Nothing);
    }

    #[test]
    fn a_typed_shortcut_presses_its_keys_in_order() {
        let binding = Binding::from_setting("custom:ControlLeft+Space");
        assert_eq!(binding.plan(), Some(Plan::Keys(vec![0x11, 0x20])));
    }

    #[test]
    fn presets_keep_their_meaning() {
        assert_eq!(
            Binding::from(Action::LeftClick).plan(),
            Some(Plan::Click(Pointer::Left))
        );
        assert_eq!(Binding::from(Action::ScrollUp).plan(), Some(Plan::Wheel(1)));
    }
}
