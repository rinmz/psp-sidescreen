//! The inputs that carry out an event, the unit the injector sends.

use super::action::Pointer;
use super::binding::Plan;
use super::mapper::Event;

/// One input the PC is given, the unit [`inject`](crate::inject) sends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// A key goes down, or comes up when `up` is set.
    Key {
        /// The Windows virtual-key code.
        vk: u16,
        /// Whether the key comes up.
        up: bool,
    },
    /// A mouse button goes down, or comes up when `up` is set.
    Button {
        /// Which button.
        pointer: Pointer,
        /// Whether the button comes up.
        up: bool,
    },
    /// The wheel turns by this many notches; positive is up.
    Wheel(i32),
    /// The pointer moves by this many pixels.
    Move {
        /// To the right; negative is left.
        dx: i32,
        /// Down; negative is up.
        dy: i32,
    },
}

/// The inputs that carry out `event`.
///
/// Keys go down in the order of the binding and come up in the opposite order, as a person's
/// fingers would, so Alt+Tab is Alt down, Tab down, Tab up, Alt up.
#[must_use]
pub fn steps(event: &Event) -> Vec<Step> {
    match event {
        Event::Move { dx, dy } => vec![Step::Move { dx: *dx, dy: *dy }],
        Event::Press(binding) => match binding.plan() {
            None => Vec::new(),
            Some(Plan::Keys(keys)) => keys.iter().map(|&vk| Step::Key { vk, up: false }).collect(),
            Some(Plan::Click(pointer)) => vec![Step::Button { pointer, up: false }],
            Some(Plan::Wheel(notches)) => vec![Step::Wheel(notches)],
        },
        Event::Release(binding) => match binding.plan() {
            None | Some(Plan::Wheel(_)) => Vec::new(),
            Some(Plan::Keys(keys)) => keys
                .iter()
                .rev()
                .map(|&vk| Step::Key { vk, up: true })
                .collect(),
            Some(Plan::Click(pointer)) => vec![Step::Button { pointer, up: true }],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{Action, Binding};

    fn press(action: Action) -> Event {
        Event::Press(Binding::from(action))
    }

    fn release(action: Action) -> Event {
        Event::Release(Binding::from(action))
    }

    #[test]
    fn a_chord_goes_down_in_order_and_up_in_reverse() {
        let down = steps(&press(Action::SwitchWindow));
        assert_eq!(
            down,
            [
                Step::Key {
                    vk: 0x12,
                    up: false
                },
                Step::Key {
                    vk: 0x09,
                    up: false
                }
            ]
        );
        let up = steps(&release(Action::SwitchWindow));
        assert_eq!(
            up,
            [
                Step::Key { vk: 0x09, up: true },
                Step::Key { vk: 0x12, up: true }
            ]
        );
    }

    #[test]
    fn a_typed_shortcut_is_pressed_and_released_the_same_way() {
        let handy = Binding::from_setting("custom:ControlLeft+Space");
        assert_eq!(
            steps(&Event::Press(handy.clone())),
            [
                Step::Key {
                    vk: 0x11,
                    up: false
                },
                Step::Key {
                    vk: 0x20,
                    up: false
                }
            ]
        );
        assert_eq!(
            steps(&Event::Release(handy)),
            [
                Step::Key { vk: 0x20, up: true },
                Step::Key { vk: 0x11, up: true }
            ]
        );
    }

    #[test]
    fn a_click_is_held_for_as_long_as_the_button_is() {
        assert_eq!(
            steps(&press(Action::LeftClick)),
            [Step::Button {
                pointer: Pointer::Left,
                up: false
            }]
        );
        assert_eq!(
            steps(&release(Action::LeftClick)),
            [Step::Button {
                pointer: Pointer::Left,
                up: true
            }]
        );
    }

    #[test]
    fn a_wheel_notch_is_sent_on_the_press_only() {
        assert_eq!(steps(&press(Action::ScrollDown)), [Step::Wheel(-1)]);
        assert_eq!(steps(&release(Action::ScrollDown)), []);
    }

    #[test]
    fn every_key_that_goes_down_comes_up() {
        let keys = |list: &[Step], up_wanted: bool| {
            list.iter()
                .filter(|step| matches!(step, Step::Key { up, .. } if *up == up_wanted))
                .count()
        };
        for action in Action::ALL {
            let downs = steps(&press(action));
            let ups = steps(&release(action));
            assert_eq!(keys(&downs, false), keys(&ups, true), "{action:?}");
        }
    }

    #[test]
    fn nothing_and_a_move_give_the_expected_steps() {
        assert_eq!(steps(&Event::Press(Binding::Nothing)), []);
        assert_eq!(
            steps(&Event::Move { dx: 3, dy: -4 }),
            [Step::Move { dx: 3, dy: -4 }]
        );
    }
}
