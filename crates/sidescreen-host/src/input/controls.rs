//! What the buttons and the stick do, all together.

use sidescreen_proto::Button;

use super::binding::Binding;

/// What the buttons and the stick do. Nothing is done until `enabled` is set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Controls {
    /// Whether the PSP controls the PC at all.
    pub enabled: bool,
    /// Whether the analog stick moves the mouse pointer.
    pub analog_mouse: bool,
    /// The binding of each button, in the order of [`Button::ALL`].
    buttons: Vec<Binding>,
}

impl Default for Controls {
    fn default() -> Self {
        Self::new(false, false)
    }
}

impl Controls {
    /// Controls with every button doing nothing.
    #[must_use]
    pub fn new(enabled: bool, analog_mouse: bool) -> Self {
        Self {
            enabled,
            analog_mouse,
            buttons: vec![Binding::Nothing; Button::ALL.len()],
        }
    }

    /// What `button` does.
    #[must_use]
    pub fn binding(&self, button: Button) -> &Binding {
        &self.buttons[index(button)]
    }

    /// Makes `button` do `binding`; an [`Action`](super::Action) can be given as it is.
    pub fn set(&mut self, button: Button, binding: impl Into<Binding>) {
        self.buttons[index(button)] = binding.into();
    }
}

fn index(button: Button) -> usize {
    Button::ALL
        .iter()
        .position(|candidate| *candidate == button)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Action;

    #[test]
    fn controls_start_off_with_every_button_doing_nothing() {
        let controls = Controls::default();
        assert!(!controls.enabled && !controls.analog_mouse);
        for button in Button::ALL {
            assert_eq!(*controls.binding(button), Binding::Nothing);
        }
    }

    #[test]
    fn a_binding_is_kept_for_its_own_button_only() {
        let mut controls = Controls::default();
        controls.set(Button::Cross, Action::Enter);
        assert_eq!(
            *controls.binding(Button::Cross),
            Binding::Preset(Action::Enter)
        );
        assert_eq!(*controls.binding(Button::Circle), Binding::Nothing);
    }

    #[test]
    fn setting_a_button_again_replaces_what_it_did() {
        let mut controls = Controls::default();
        controls.set(Button::L, Action::Enter);
        controls.set(Button::L, Binding::from_setting("custom:ControlLeft+Space"));
        assert!(matches!(controls.binding(Button::L), Binding::Custom(_)));
    }
}
