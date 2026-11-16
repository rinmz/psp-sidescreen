//! Starting and stopping the session on a thread of its own.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};

use serde::Serialize;
use sidescreen_host::session::{self, Event, Options};

/// Where the session has got to, in the shape the window listens for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub(crate) enum State {
    Idle,
    Starting,
    Waiting,
    Streaming { fps: u32, width: u32, height: u32 },
    Reconnecting,
    Error { message: String },
}

impl From<Event> for State {
    fn from(event: Event) -> Self {
        match event {
            Event::Starting => Self::Starting,
            Event::Waiting => Self::Waiting,
            Event::Streaming { fps, width, height } => Self::Streaming { fps, width, height },
            Event::Reconnecting => Self::Reconnecting,
        }
    }
}

struct Running {
    stop: Arc<AtomicBool>,
    thread: JoinHandle<()>,
}

/// The session that may be running.
#[derive(Default)]
pub(crate) struct Control {
    running: Mutex<Option<Running>>,
}

impl Control {
    /// Starts a session. `emit` is called, from the session's thread, with every state.
    ///
    /// # Errors
    ///
    /// A message for the person if a session is already running.
    pub(crate) fn start(
        &self,
        options: Options,
        emit: impl Fn(&State) + Send + 'static,
    ) -> Result<(), String> {
        let mut slot = self.running.lock().unwrap_or_else(PoisonError::into_inner);
        if slot
            .as_ref()
            .is_some_and(|running| !running.thread.is_finished())
        {
            return Err("Sidescreen is already running".to_owned());
        }
        // A session that has ended is replaced; its thread is done, so joining returns at once.
        if let Some(finished) = slot.take() {
            let _ = finished.thread.join();
        }

        let stop = Arc::new(AtomicBool::new(false));
        let session_stop = Arc::clone(&stop);
        let thread = thread::spawn(move || {
            let result = session::run(&options, &session_stop, |event| emit(&event.into()));
            emit(&match result {
                Ok(()) => State::Idle,
                Err(error) => State::Error {
                    message: error.to_string(),
                },
            });
        });
        *slot = Some(Running { stop, thread });
        Ok(())
    }

    /// Asks the session to stop. It reports `Idle` itself when it has.
    pub(crate) fn stop(&self) {
        let slot = self.running.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(running) = slot.as_ref() {
            running.stop.store(true, Ordering::SeqCst);
        }
    }

    /// Stops the session and waits until it has ended, so that the virtual monitor has been
    /// taken off the desktop before the program exits.
    pub(crate) fn stop_and_wait(&self) {
        let running = self
            .running
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(running) = running {
            running.stop.store(true, Ordering::SeqCst);
            let _ = running.thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn states_are_sent_to_the_window_in_the_shape_it_expects() {
        let json = |state: &State| serde_json::to_string(state).unwrap();
        assert_eq!(json(&State::Idle), r#"{"kind":"idle"}"#);
        assert_eq!(
            json(&State::Streaming {
                fps: 30,
                width: 1366,
                height: 768
            }),
            r#"{"kind":"streaming","fps":30,"width":1366,"height":768}"#
        );
        assert_eq!(
            json(&State::Error {
                message: "no".to_owned()
            }),
            r#"{"kind":"error","message":"no"}"#
        );
    }

    #[test]
    fn events_of_a_session_become_states() {
        assert_eq!(State::from(Event::Waiting), State::Waiting);
        assert_eq!(State::from(Event::Reconnecting), State::Reconnecting);
        assert_eq!(
            State::from(Event::Streaming {
                fps: 1,
                width: 2,
                height: 3
            }),
            State::Streaming {
                fps: 1,
                width: 2,
                height: 3
            }
        );
    }

    #[test]
    fn stopping_when_nothing_runs_does_nothing() {
        let control = Control::default();
        control.stop();
        control.stop_and_wait();
    }
}
