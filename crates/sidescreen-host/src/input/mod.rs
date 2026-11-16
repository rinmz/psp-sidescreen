//! What the PSP's buttons and stick do on the PC.
//!
//! This module only decides: it turns what the PSP reports into a list of [`Event`]s, and those
//! into [`Step`]s. The [`inject`](crate::inject) module carries the steps out. Keeping the two
//! apart lets the decisions, which are where the mistakes would be (a key left held down, a stick
//! that drifts), be tested without moving a real mouse.
//!
//! - [`Action`]: the fixed list of things a button can do.
//! - [`Chord`]: a shortcut the person typed, such as Ctrl+Space.
//! - [`Binding`]: what one button is set to do, one or the other.
//! - [`Controls`]: the bindings of all buttons, and whether the stick moves the mouse.
//! - [`Mapper`]: turns the PSP's reports into [`Event`]s, and remembers what is held.

mod action;
mod binding;
mod chord;
mod controls;
mod mapper;
mod steps;
mod vk;

pub use action::{Action, Output, Pointer};
pub use binding::Binding;
pub use chord::{Chord, ChordError};
pub use controls::Controls;
pub use mapper::{Event, Mapper};
pub use steps::{Step, steps};
