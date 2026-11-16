//! Turns what the PSP reports into things to do on the PC.

use std::time::Duration;

use sidescreen_proto::{Button, Response};

use super::binding::Binding;
use super::controls::Controls;

/// One thing to do on the PC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A button went down: press what it is set to.
    Press(Binding),
    /// A button came up: release what it is set to.
    Release(Binding),
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
            let binding = self.controls.binding(button);
            if *binding == Binding::Nothing {
                continue;
            }
            match (button.is_down_in(self.down), button.is_down_in(now)) {
                (false, true) => events.push(Event::Press(binding.clone())),
                (true, false) => events.push(Event::Release(binding.clone())),
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
            .map(|button| self.controls.binding(button))
            .filter(|binding| **binding != Binding::Nothing)
            .map(|binding| Event::Release(binding.clone()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Action;

    fn report(buttons: u32, x: u8, y: u8) -> Response {
        Response {
            flags: 0,
            buttons,
            analog_x: x,
            analog_y: y,
        }
    }

    fn controls() -> Controls {
        let mut controls = Controls::new(true, false);
        controls.set(Button::Cross, Action::Enter);
        controls.set(Button::Circle, Action::Escape);
        controls
    }

    fn disabled() -> Controls {
        let mut controls = controls();
        controls.enabled = false;
        controls
    }

    fn with_stick(analog_mouse: bool) -> Controls {
        let mut controls = controls();
        controls.analog_mouse = analog_mouse;
        controls
    }

    fn press(action: Action) -> Event {
        Event::Press(Binding::from(action))
    }

    fn release(action: Action) -> Event {
        Event::Release(Binding::from(action))
    }

    const FRAME: Duration = Duration::from_millis(33);

    #[test]
    fn nothing_happens_until_controls_are_enabled() {
        let mut mapper = Mapper::new(disabled());
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
            [press(Action::Enter)]
        );
        assert_eq!(
            mapper.update(&report(0, 128, 128), FRAME),
            [release(Action::Enter)]
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
        assert!(events.contains(&press(Action::Enter)));
        assert!(events.contains(&press(Action::Escape)));
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
    fn a_typed_shortcut_is_held_for_as_long_as_the_button_is() {
        let handy = Binding::from_setting("custom:ControlLeft+Space");
        let mut controls = Controls::new(true, false);
        controls.set(Button::R, handy.clone());
        let mut mapper = Mapper::new(controls);
        let held = Button::R.mask();
        assert_eq!(
            mapper.update(&report(held, 128, 128), FRAME),
            [Event::Press(handy.clone())]
        );
        assert_eq!(mapper.update(&report(held, 128, 128), FRAME), []);
        assert_eq!(
            mapper.update(&report(0, 128, 128), FRAME),
            [Event::Release(handy)]
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
        assert!(events.contains(&release(Action::Enter)));
        assert!(events.contains(&release(Action::Escape)));
        assert_eq!(mapper.release_all(), [], "and only once");
    }

    #[test]
    fn a_stick_at_rest_does_not_move_the_pointer() {
        let mut mapper = Mapper::new(with_stick(true));
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
        let mut mapper = Mapper::new(with_stick(true));
        let events = mapper.update(&report(0, 255, 0), Duration::from_secs(1));
        // Fully right and fully up for a second: the full speed, right and up.
        assert_eq!(events, [Event::Move { dx: 900, dy: -900 }]);
    }

    #[test]
    fn a_slow_stick_still_gets_there_through_the_remainder() {
        let mut mapper = Mapper::new(with_stick(true));
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
        let mut mapper = Mapper::new(with_stick(false));
        assert_eq!(
            mapper.update(&report(0, 255, 255), Duration::from_secs(1)),
            []
        );
    }
}
