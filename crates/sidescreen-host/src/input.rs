//! What the PSP's buttons and stick do on the PC.
//!
//! This module only decides: it turns what the PSP reports into a list of [`Event`]s. The
//! [`inject`](crate::inject) module carries them out. Keeping the two apart lets the decisions,
//! which are where the mistakes would be (a key left held down, a stick that drifts), be tested
//! without moving a real mouse.

use std::time::Duration;

use sidescreen_proto::{Button, Response};

/// Windows virtual-key codes used by the actions.
mod vk {
    pub(super) const BACK: u16 = 0x08;
    pub(super) const TAB: u16 = 0x09;
    pub(super) const RETURN: u16 = 0x0D;
    pub(super) const CONTROL: u16 = 0x11;
    pub(super) const MENU: u16 = 0x12;
    pub(super) const ESCAPE: u16 = 0x1B;
    pub(super) const SPACE: u16 = 0x20;
    pub(super) const PRIOR: u16 = 0x21;
    pub(super) const NEXT: u16 = 0x22;
    pub(super) const END: u16 = 0x23;
    pub(super) const HOME: u16 = 0x24;
    pub(super) const LEFT: u16 = 0x25;
    pub(super) const UP: u16 = 0x26;
    pub(super) const RIGHT: u16 = 0x27;
    pub(super) const DOWN: u16 = 0x28;
    pub(super) const DELETE: u16 = 0x2E;
    pub(super) const C: u16 = 0x43;
    pub(super) const D: u16 = 0x44;
    pub(super) const V: u16 = 0x56;
    pub(super) const Z: u16 = 0x5A;
    pub(super) const LWIN: u16 = 0x5B;
    pub(super) const F5: u16 = 0x74;
    pub(super) const VOLUME_MUTE: u16 = 0xAD;
    pub(super) const VOLUME_DOWN: u16 = 0xAE;
    pub(super) const VOLUME_UP: u16 = 0xAF;
    pub(super) const MEDIA_NEXT: u16 = 0xB0;
    pub(super) const MEDIA_PREVIOUS: u16 = 0xB1;
    pub(super) const MEDIA_PLAY_PAUSE: u16 = 0xB3;
}

/// A mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pointer {
    /// The left button.
    Left,
    /// The right button.
    Right,
    /// The middle button, the wheel.
    Middle,
}

/// What an action does on the PC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Output {
    /// Keys pressed together, in this order, and released in the opposite order.
    Keys(&'static [u16]),
    /// A mouse button, held while the PSP button is held.
    Click(Pointer),
    /// One notch of the mouse wheel; positive scrolls up.
    Wheel(i32),
}

/// Something a PSP button can be made to do. The list is fixed, so the window offers it as a
/// menu and a file stores only the [`id`](Self::id).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// The button does nothing.
    Nothing,
    /// Enter.
    Enter,
    /// Escape.
    Escape,
    /// Space.
    Space,
    /// Tab.
    Tab,
    /// Backspace.
    Backspace,
    /// Delete.
    Delete,
    /// Page up.
    PageUp,
    /// Page down.
    PageDown,
    /// Home.
    Home,
    /// End.
    End,
    /// The up arrow.
    ArrowUp,
    /// The down arrow.
    ArrowDown,
    /// The left arrow.
    ArrowLeft,
    /// The right arrow.
    ArrowRight,
    /// Alt+Tab, to switch window.
    SwitchWindow,
    /// Win+Tab, the task view.
    TaskView,
    /// Win+D, show the desktop.
    ShowDesktop,
    /// Ctrl+C.
    Copy,
    /// Ctrl+V.
    Paste,
    /// Ctrl+Z.
    Undo,
    /// F5, refresh.
    Refresh,
    /// Play or pause the music.
    PlayPause,
    /// The next track.
    NextTrack,
    /// The previous track.
    PreviousTrack,
    /// Louder.
    VolumeUp,
    /// Quieter.
    VolumeDown,
    /// Mute.
    Mute,
    /// The left mouse button.
    LeftClick,
    /// The right mouse button.
    RightClick,
    /// The middle mouse button.
    MiddleClick,
    /// One notch of the wheel up.
    ScrollUp,
    /// One notch of the wheel down.
    ScrollDown,
}

impl Action {
    /// Every action with its stable id, the label the window shows, and what it does.
    pub const ALL: [Self; 33] = [
        Self::Nothing,
        Self::Enter,
        Self::Escape,
        Self::Space,
        Self::Tab,
        Self::Backspace,
        Self::Delete,
        Self::PageUp,
        Self::PageDown,
        Self::Home,
        Self::End,
        Self::ArrowUp,
        Self::ArrowDown,
        Self::ArrowLeft,
        Self::ArrowRight,
        Self::SwitchWindow,
        Self::TaskView,
        Self::ShowDesktop,
        Self::Copy,
        Self::Paste,
        Self::Undo,
        Self::Refresh,
        Self::PlayPause,
        Self::NextTrack,
        Self::PreviousTrack,
        Self::VolumeUp,
        Self::VolumeDown,
        Self::Mute,
        Self::LeftClick,
        Self::RightClick,
        Self::MiddleClick,
        Self::ScrollUp,
        Self::ScrollDown,
    ];

    /// The name saved in a settings file. It never changes once released.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Nothing => "nothing",
            Self::Enter => "enter",
            Self::Escape => "escape",
            Self::Space => "space",
            Self::Tab => "tab",
            Self::Backspace => "backspace",
            Self::Delete => "delete",
            Self::PageUp => "page_up",
            Self::PageDown => "page_down",
            Self::Home => "home",
            Self::End => "end",
            Self::ArrowUp => "arrow_up",
            Self::ArrowDown => "arrow_down",
            Self::ArrowLeft => "arrow_left",
            Self::ArrowRight => "arrow_right",
            Self::SwitchWindow => "switch_window",
            Self::TaskView => "task_view",
            Self::ShowDesktop => "show_desktop",
            Self::Copy => "copy",
            Self::Paste => "paste",
            Self::Undo => "undo",
            Self::Refresh => "refresh",
            Self::PlayPause => "play_pause",
            Self::NextTrack => "next_track",
            Self::PreviousTrack => "previous_track",
            Self::VolumeUp => "volume_up",
            Self::VolumeDown => "volume_down",
            Self::Mute => "mute",
            Self::LeftClick => "left_click",
            Self::RightClick => "right_click",
            Self::MiddleClick => "middle_click",
            Self::ScrollUp => "scroll_up",
            Self::ScrollDown => "scroll_down",
        }
    }

    /// The text the window shows for the action.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Nothing => "Nothing",
            Self::Enter => "Enter",
            Self::Escape => "Escape",
            Self::Space => "Space",
            Self::Tab => "Tab",
            Self::Backspace => "Backspace",
            Self::Delete => "Delete",
            Self::PageUp => "Page up",
            Self::PageDown => "Page down",
            Self::Home => "Home",
            Self::End => "End",
            Self::ArrowUp => "Arrow up",
            Self::ArrowDown => "Arrow down",
            Self::ArrowLeft => "Arrow left",
            Self::ArrowRight => "Arrow right",
            Self::SwitchWindow => "Switch window (Alt+Tab)",
            Self::TaskView => "Task view (Win+Tab)",
            Self::ShowDesktop => "Show desktop (Win+D)",
            Self::Copy => "Copy (Ctrl+C)",
            Self::Paste => "Paste (Ctrl+V)",
            Self::Undo => "Undo (Ctrl+Z)",
            Self::Refresh => "Refresh (F5)",
            Self::PlayPause => "Play or pause music",
            Self::NextTrack => "Next track",
            Self::PreviousTrack => "Previous track",
            Self::VolumeUp => "Volume up",
            Self::VolumeDown => "Volume down",
            Self::Mute => "Mute",
            Self::LeftClick => "Left click",
            Self::RightClick => "Right click",
            Self::MiddleClick => "Middle click",
            Self::ScrollUp => "Scroll up",
            Self::ScrollDown => "Scroll down",
        }
    }

    /// The action with this id, or `None` for a name that is not an action.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|action| action.id() == id)
    }

    /// What the action does on the PC; `None` for [`Nothing`](Self::Nothing).
    #[must_use]
    pub const fn output(self) -> Option<Output> {
        Some(match self {
            Self::Nothing => return None,
            Self::Enter => Output::Keys(&[vk::RETURN]),
            Self::Escape => Output::Keys(&[vk::ESCAPE]),
            Self::Space => Output::Keys(&[vk::SPACE]),
            Self::Tab => Output::Keys(&[vk::TAB]),
            Self::Backspace => Output::Keys(&[vk::BACK]),
            Self::Delete => Output::Keys(&[vk::DELETE]),
            Self::PageUp => Output::Keys(&[vk::PRIOR]),
            Self::PageDown => Output::Keys(&[vk::NEXT]),
            Self::Home => Output::Keys(&[vk::HOME]),
            Self::End => Output::Keys(&[vk::END]),
            Self::ArrowUp => Output::Keys(&[vk::UP]),
            Self::ArrowDown => Output::Keys(&[vk::DOWN]),
            Self::ArrowLeft => Output::Keys(&[vk::LEFT]),
            Self::ArrowRight => Output::Keys(&[vk::RIGHT]),
            Self::SwitchWindow => Output::Keys(&[vk::MENU, vk::TAB]),
            Self::TaskView => Output::Keys(&[vk::LWIN, vk::TAB]),
            Self::ShowDesktop => Output::Keys(&[vk::LWIN, vk::D]),
            Self::Copy => Output::Keys(&[vk::CONTROL, vk::C]),
            Self::Paste => Output::Keys(&[vk::CONTROL, vk::V]),
            Self::Undo => Output::Keys(&[vk::CONTROL, vk::Z]),
            Self::Refresh => Output::Keys(&[vk::F5]),
            Self::PlayPause => Output::Keys(&[vk::MEDIA_PLAY_PAUSE]),
            Self::NextTrack => Output::Keys(&[vk::MEDIA_NEXT]),
            Self::PreviousTrack => Output::Keys(&[vk::MEDIA_PREVIOUS]),
            Self::VolumeUp => Output::Keys(&[vk::VOLUME_UP]),
            Self::VolumeDown => Output::Keys(&[vk::VOLUME_DOWN]),
            Self::Mute => Output::Keys(&[vk::VOLUME_MUTE]),
            Self::LeftClick => Output::Click(Pointer::Left),
            Self::RightClick => Output::Click(Pointer::Right),
            Self::MiddleClick => Output::Click(Pointer::Middle),
            Self::ScrollUp => Output::Wheel(1),
            Self::ScrollDown => Output::Wheel(-1),
        })
    }
}

/// What the buttons and the stick do. Nothing is done until `enabled` is set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Controls {
    /// Whether the PSP controls the PC at all.
    pub enabled: bool,
    /// Whether the analog stick moves the mouse pointer.
    pub analog_mouse: bool,
    /// The action of each button, in the order of [`Button::ALL`].
    buttons: [Action; 12],
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            enabled: false,
            analog_mouse: false,
            buttons: [Action::Nothing; 12],
        }
    }
}

impl Controls {
    /// Controls with every button doing nothing.
    #[must_use]
    pub fn new(enabled: bool, analog_mouse: bool) -> Self {
        Self {
            enabled,
            analog_mouse,
            buttons: [Action::Nothing; 12],
        }
    }

    /// The action of `button`.
    #[must_use]
    pub fn action(&self, button: Button) -> Action {
        self.buttons[index(button)]
    }

    /// Makes `button` do `action`.
    pub fn set(&mut self, button: Button, action: Action) {
        self.buttons[index(button)] = action;
    }
}

fn index(button: Button) -> usize {
    Button::ALL
        .iter()
        .position(|candidate| *candidate == button)
        .unwrap_or(0)
}

/// One thing to do on the PC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// A button went down: press the action.
    Press(Action),
    /// A button came up: release the action.
    Release(Action),
    /// Move the mouse pointer by this many pixels.
    Move {
        /// To the right, in pixels; negative is left.
        dx: i32,
        /// Down, in pixels; negative is up.
        dy: i32,
    },
}

/// Where the stick rests. The PSP's stick reads about 127 or 128 when it is not touched.
const STICK_CENTRE: f32 = 128.0;
/// Part of the stick's travel, from the centre, that does nothing: the stick never rests exactly
/// in the middle.
const DEAD_ZONE: f32 = 0.2;
/// Pointer speed with the stick fully over, in pixels a second.
const MAX_SPEED: f32 = 900.0;

/// Turns the PSP's reports into events, remembering what is held.
#[derive(Debug)]
pub struct Mapper {
    controls: Controls,
    down: u32,
    remainder: (f32, f32),
}

impl Mapper {
    /// A mapper for these controls, with nothing held.
    #[must_use]
    pub fn new(controls: Controls) -> Self {
        Self {
            controls,
            down: 0,
            remainder: (0.0, 0.0),
        }
    }

    /// What to do now that the PSP reported `response`, `elapsed` after the previous report.
    pub fn update(&mut self, response: &Response, elapsed: Duration) -> Vec<Event> {
        if !self.controls.enabled {
            return Vec::new();
        }
        let mut events = Vec::new();

        let now = response.buttons;
        for button in Button::ALL {
            let action = self.controls.action(button);
            if action == Action::Nothing {
                continue;
            }
            match (button.is_down_in(self.down), button.is_down_in(now)) {
                (false, true) => events.push(Event::Press(action)),
                (true, false) => events.push(Event::Release(action)),
                _ => {}
            }
        }
        self.down = now;

        if self.controls.analog_mouse {
            if let Some(event) = self.stick(response.analog_x, response.analog_y, elapsed) {
                events.push(event);
            }
        }
        events
    }

    /// Releases everything that is held, for when the session ends or the PSP goes away, so no
    /// key is left pressed.
    pub fn release_all(&mut self) -> Vec<Event> {
        let events = Button::ALL
            .into_iter()
            .filter(|button| button.is_down_in(self.down))
            .map(|button| self.controls.action(button))
            .filter(|action| *action != Action::Nothing)
            .map(Event::Release)
            .collect();
        self.down = 0;
        self.remainder = (0.0, 0.0);
        events
    }

    /// The pointer movement for the stick's position over `elapsed`.
    #[allow(clippy::cast_possible_truncation)]
    fn stick(&mut self, x: u8, y: u8, elapsed: Duration) -> Option<Event> {
        let seconds = elapsed.as_secs_f32();
        let step = |value: u8, remainder: f32| {
            let travel = (f32::from(value) - STICK_CENTRE) / 127.0;
            let past = (travel.abs() - DEAD_ZONE).max(0.0) / (1.0 - DEAD_ZONE);
            // Squared, so small movements of the stick give fine control of the pointer.
            let speed = travel.signum() * past.min(1.0).powi(2) * MAX_SPEED;
            let wanted = speed * seconds + remainder;
            let whole = wanted.trunc();
            (whole as i32, wanted - whole)
        };
        let (dx, remainder_x) = step(x, self.remainder.0);
        let (dy, remainder_y) = step(y, self.remainder.1);
        self.remainder = (remainder_x, remainder_y);
        (dx != 0 || dy != 0).then_some(Event::Move { dx, dy })
    }
}

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
/// Keys go down in the order of the action and come up in the opposite order, as a person's
/// fingers would, so Alt+Tab is Alt down, Tab down, Tab up, Alt up.
#[must_use]
pub fn steps(event: &Event) -> Vec<Step> {
    match *event {
        Event::Move { dx, dy } => vec![Step::Move { dx, dy }],
        Event::Press(action) => match action.output() {
            None => Vec::new(),
            Some(Output::Keys(keys)) => {
                keys.iter().map(|&vk| Step::Key { vk, up: false }).collect()
            }
            Some(Output::Click(pointer)) => vec![Step::Button { pointer, up: false }],
            Some(Output::Wheel(notches)) => vec![Step::Wheel(notches)],
        },
        Event::Release(action) => match action.output() {
            None | Some(Output::Wheel(_)) => Vec::new(),
            Some(Output::Keys(keys)) => keys
                .iter()
                .rev()
                .map(|&vk| Step::Key { vk, up: true })
                .collect(),
            Some(Output::Click(pointer)) => vec![Step::Button { pointer, up: true }],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(buttons: u32, x: u8, y: u8) -> Response {
        Response {
            flags: 0,
            buttons,
            analog_x: x,
            analog_y: y,
        }
    }

    fn controls() -> Controls {
        let mut controls = Controls {
            enabled: true,
            ..Controls::default()
        };
        controls.set(Button::Cross, Action::Enter);
        controls.set(Button::Circle, Action::Escape);
        controls
    }

    const FRAME: Duration = Duration::from_millis(33);

    #[test]
    fn action_ids_are_unique_and_found_again() {
        let mut ids: Vec<_> = Action::ALL.iter().map(|action| action.id()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), Action::ALL.len());
        for action in Action::ALL {
            assert_eq!(Action::from_id(action.id()), Some(action));
        }
        assert_eq!(Action::from_id("launch_missiles"), None);
    }

    #[test]
    fn only_nothing_has_no_output() {
        for action in Action::ALL {
            assert_eq!(
                action.output().is_none(),
                action == Action::Nothing,
                "{action:?}"
            );
        }
    }

    #[test]
    fn nothing_happens_until_controls_are_enabled() {
        let mut mapper = Mapper::new(Controls {
            enabled: false,
            ..controls()
        });
        assert_eq!(
            mapper.update(&report(Button::Cross.mask(), 255, 255), FRAME),
            []
        );
    }

    #[test]
    fn a_button_pressed_and_released_gives_a_press_then_a_release() {
        let mut mapper = Mapper::new(controls());
        let cross = Button::Cross.mask();
        assert_eq!(
            mapper.update(&report(cross, 128, 128), FRAME),
            [Event::Press(Action::Enter)]
        );
        assert_eq!(
            mapper.update(&report(0, 128, 128), FRAME),
            [Event::Release(Action::Enter)]
        );
    }

    #[test]
    fn a_button_that_stays_down_is_pressed_only_once() {
        let mut mapper = Mapper::new(controls());
        let cross = Button::Cross.mask();
        assert_eq!(mapper.update(&report(cross, 128, 128), FRAME).len(), 1);
        for _ in 0..5 {
            assert_eq!(mapper.update(&report(cross, 128, 128), FRAME), []);
        }
    }

    #[test]
    fn two_buttons_at_once_are_two_events() {
        let mut mapper = Mapper::new(controls());
        let both = Button::Cross.mask() | Button::Circle.mask();
        let events = mapper.update(&report(both, 128, 128), FRAME);
        assert_eq!(events.len(), 2);
        assert!(events.contains(&Event::Press(Action::Enter)));
        assert!(events.contains(&Event::Press(Action::Escape)));
    }

    #[test]
    fn a_button_with_no_action_is_ignored() {
        let mut mapper = Mapper::new(controls());
        assert_eq!(
            mapper.update(&report(Button::Square.mask(), 128, 128), FRAME),
            []
        );
    }

    #[test]
    fn nothing_is_left_held_when_everything_is_released() {
        let mut mapper = Mapper::new(controls());
        mapper.update(
            &report(Button::Cross.mask() | Button::Circle.mask(), 128, 128),
            FRAME,
        );
        let events = mapper.release_all();
        assert_eq!(events.len(), 2);
        assert!(events.contains(&Event::Release(Action::Enter)));
        assert!(events.contains(&Event::Release(Action::Escape)));
        assert_eq!(mapper.release_all(), [], "and only once");
    }

    #[test]
    fn a_stick_at_rest_does_not_move_the_pointer() {
        let mut mapper = Mapper::new(Controls {
            analog_mouse: true,
            ..controls()
        });
        for centre in [(128, 128), (127, 127), (130, 125), (140, 116)] {
            assert_eq!(
                mapper.update(&report(0, centre.0, centre.1), FRAME),
                [],
                "{centre:?}"
            );
        }
    }

    #[test]
    fn the_stick_moves_the_pointer_in_its_direction() {
        let mut mapper = Mapper::new(Controls {
            analog_mouse: true,
            ..controls()
        });
        let events = mapper.update(&report(0, 255, 0), Duration::from_secs(1));
        // Fully right and fully up for a second: the full speed, right and up.
        assert_eq!(events, [Event::Move { dx: 900, dy: -900 }]);
    }

    #[test]
    fn a_slow_stick_still_gets_there_through_the_remainder() {
        let mut mapper = Mapper::new(Controls {
            analog_mouse: true,
            ..controls()
        });
        // Just past the dead zone the speed is a few pixels a second, under one pixel a frame.
        let mut total = 0;
        for _ in 0..60 {
            if let Some(Event::Move { dx, .. }) = mapper.update(&report(0, 200, 128), FRAME).pop() {
                total += dx;
            }
        }
        assert!(total > 0, "the pointer moved");
    }

    #[test]
    fn the_stick_is_ignored_when_it_is_not_asked_for() {
        let mut mapper = Mapper::new(Controls {
            analog_mouse: false,
            ..controls()
        });
        assert_eq!(
            mapper.update(&report(0, 255, 255), Duration::from_secs(1)),
            []
        );
    }

    #[test]
    fn a_chord_goes_down_in_order_and_up_in_reverse() {
        let down = steps(&Event::Press(Action::SwitchWindow));
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
        let up = steps(&Event::Release(Action::SwitchWindow));
        assert_eq!(
            up,
            [
                Step::Key { vk: 0x09, up: true },
                Step::Key { vk: 0x12, up: true }
            ]
        );
    }

    #[test]
    fn a_click_is_held_for_as_long_as_the_button_is() {
        assert_eq!(
            steps(&Event::Press(Action::LeftClick)),
            [Step::Button {
                pointer: Pointer::Left,
                up: false
            }]
        );
        assert_eq!(
            steps(&Event::Release(Action::LeftClick)),
            [Step::Button {
                pointer: Pointer::Left,
                up: true
            }]
        );
    }

    #[test]
    fn a_wheel_notch_is_sent_on_the_press_only() {
        assert_eq!(steps(&Event::Press(Action::ScrollDown)), [Step::Wheel(-1)]);
        assert_eq!(steps(&Event::Release(Action::ScrollDown)), []);
    }

    #[test]
    fn every_key_that_goes_down_comes_up() {
        let keys = |list: &[Step], up_wanted: bool| {
            list.iter()
                .filter(|step| matches!(step, Step::Key { up, .. } if *up == up_wanted))
                .count()
        };
        for action in Action::ALL {
            let downs = steps(&Event::Press(action));
            let ups = steps(&Event::Release(action));
            assert_eq!(keys(&downs, false), keys(&ups, true), "{action:?}");
        }
    }

    #[test]
    fn nothing_and_a_move_give_the_expected_steps() {
        assert_eq!(steps(&Event::Press(Action::Nothing)), []);
        assert_eq!(
            steps(&Event::Move { dx: 3, dy: -4 }),
            [Step::Move { dx: 3, dy: -4 }]
        );
    }
}
