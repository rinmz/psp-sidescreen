/// A button of the PSP, as a bit of the `buttons` field of a [`Response`](crate::Response).
///
/// The bit values are the PSP's own (`SceCtrlButtons`). Only the twelve buttons every PSP has
/// are listed. The top byte of the field is not buttons: PSPdisp's on-screen keyboard uses it to
/// send a key code, so it is ignored here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    /// The Select button.
    Select,
    /// The Start button.
    Start,
    /// The directional pad, up.
    Up,
    /// The directional pad, right.
    Right,
    /// The directional pad, down.
    Down,
    /// The directional pad, left.
    Left,
    /// The left shoulder button.
    L,
    /// The right shoulder button.
    R,
    /// The triangle button.
    Triangle,
    /// The circle button.
    Circle,
    /// The cross button.
    Cross,
    /// The square button.
    Square,
}

impl Button {
    /// Every button, in the order a window would list them.
    pub const ALL: [Self; 12] = [
        Self::Cross,
        Self::Circle,
        Self::Square,
        Self::Triangle,
        Self::Up,
        Self::Down,
        Self::Left,
        Self::Right,
        Self::L,
        Self::R,
        Self::Start,
        Self::Select,
    ];

    /// The bit of the button in the `buttons` field.
    #[must_use]
    pub const fn mask(self) -> u32 {
        match self {
            Self::Select => 0x0000_0001,
            Self::Start => 0x0000_0008,
            Self::Up => 0x0000_0010,
            Self::Right => 0x0000_0020,
            Self::Down => 0x0000_0040,
            Self::Left => 0x0000_0080,
            Self::L => 0x0000_0100,
            Self::R => 0x0000_0200,
            Self::Triangle => 0x0000_1000,
            Self::Circle => 0x0000_2000,
            Self::Cross => 0x0000_4000,
            Self::Square => 0x0000_8000,
        }
    }

    /// Whether the button is down in `buttons`.
    #[must_use]
    pub const fn is_down_in(self, buttons: u32) -> bool {
        buttons & self.mask() != 0
    }

    /// The text a window shows for the button.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Start => "Start",
            Self::Up => "D-pad up",
            Self::Right => "D-pad right",
            Self::Down => "D-pad down",
            Self::Left => "D-pad left",
            Self::L => "L trigger",
            Self::R => "R trigger",
            Self::Triangle => "Triangle",
            Self::Circle => "Circle",
            Self::Cross => "Cross",
            Self::Square => "Square",
        }
    }

    /// A short name for the button, lower case, stable enough to be saved in a file.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Select => "select",
            Self::Start => "start",
            Self::Up => "up",
            Self::Right => "right",
            Self::Down => "down",
            Self::Left => "left",
            Self::L => "l",
            Self::R => "r",
            Self::Triangle => "triangle",
            Self::Circle => "circle",
            Self::Cross => "cross",
            Self::Square => "square",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_are_the_psp_control_bits() {
        assert_eq!(Button::Select.mask(), 0x1);
        assert_eq!(Button::Start.mask(), 0x8);
        assert_eq!(Button::Cross.mask(), 0x4000);
        assert_eq!(Button::Square.mask(), 0x8000);
    }

    #[test]
    fn every_button_has_its_own_bit_below_the_top_byte() {
        let mut seen = 0_u32;
        for button in Button::ALL {
            assert_eq!(seen & button.mask(), 0, "{button:?} shares a bit");
            assert_eq!(button.mask() & 0xFF00_0000, 0);
            seen |= button.mask();
        }
        assert_eq!(Button::ALL.len(), 12);
    }

    #[test]
    fn a_button_is_down_when_its_bit_is_set() {
        let pressed = Button::Circle.mask() | Button::Up.mask();
        assert!(Button::Circle.is_down_in(pressed));
        assert!(Button::Up.is_down_in(pressed));
        assert!(!Button::Cross.is_down_in(pressed));
    }

    #[test]
    fn the_top_byte_is_not_a_button() {
        // PSPdisp's keyboard sends a key code in the top byte.
        for button in Button::ALL {
            assert!(!button.is_down_in(0xFF00_0000));
        }
    }

    #[test]
    fn names_are_unique() {
        let mut names: Vec<_> = Button::ALL.iter().map(|button| button.name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 12);
    }

    #[test]
    fn labels_are_unique() {
        let mut labels: Vec<_> = Button::ALL.iter().map(|button| button.label()).collect();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), 12);
    }
}
