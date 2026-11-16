//! The fixed list of things a PSP button can be made to do.

use super::vk;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
