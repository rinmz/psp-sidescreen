//! One session: from Start to Stop.
//!
//! A session gets the PSP a screen, waits for the PSP to answer, and sends it pictures until it
//! is told to stop. It keeps going through the things that happen to a screen and a cable: a
//! change of resolution, Win+P, the PSP application being closed and opened again. The command
//! line and the window both run it and listen to what it reports.

use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use crate::capture::{self, CaptureError, Capturer, Check, Frame};
use crate::convert::{ConvertError, Converter, Picture, Quality};
use crate::inject;
use crate::input::{Controls, Event as InputEvent, Mapper};
use crate::link::{Link, LinkError, PipeError};
use crate::monitor::{self, MonitorError, VirtualMonitor};
use crate::usb::{OpenError, UsbPipe};
use sidescreen_proto::Response;

/// How long one transfer may take before the PSP is considered stuck.
const TRANSFER_TIMEOUT: Duration = Duration::from_millis(1000);
/// Pause PSPdisp makes after the first bytes of the hello.
const HELLO_SETTLE: Duration = Duration::from_millis(500);
/// How long to wait for the screen to change before looking again.
const CAPTURE_WAIT: Duration = Duration::from_millis(33);
/// A picture is sent again after this long without change, so the PSP knows the host is alive
/// and does not go into power saving.
const KEEP_ALIVE: Duration = Duration::from_millis(500);
/// How often the captured monitor is looked at again, to notice a change of resolution.
const RECHECK_EVERY: Duration = Duration::from_millis(500);
/// How often to look for a monitor that went away.
const RETRY_EVERY: Duration = Duration::from_millis(500);
/// How often to try the PSP again while it does not answer.
const CONNECT_RETRY: Duration = Duration::from_secs(1);
/// Refresh rate asked of the virtual monitor.
const VIRTUAL_REFRESH_HZ: u32 = 60;

/// What to show and how.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    /// Give the PSP a virtual monitor of its own instead of showing an existing one.
    pub virtual_monitor: bool,
    /// Size of the virtual monitor. Small text stays readable on the PSP's 480x272 screen at the
    /// default, and the driver offers it.
    pub size: (u32, u32),
    /// Existing monitor to show when `virtual_monitor` is off.
    pub monitor: usize,
    /// How the picture is compressed.
    pub quality: Quality,
    /// What the PSP's buttons and stick do on the PC.
    pub controls: Controls,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            virtual_monitor: false,
            size: (1366, 768),
            monitor: 0,
            quality: Quality::Medium,
            controls: Controls::default(),
        }
    }
}

/// What a session reports as it goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// The session is setting up the screen.
    Starting,
    /// The PSP does not answer; it is tried again every second.
    Waiting,
    /// The PSP is being sent pictures; `fps` counts the last second and is 0 until the first.
    Streaming {
        /// Pictures sent in the last second.
        fps: u32,
        /// Width of the screen being shown.
        width: u32,
        /// Height of the screen being shown.
        height: u32,
    },
    /// The monitor changed or went away and is being picked up again.
    Reconnecting,
}

/// Why a session ended.
#[derive(Debug)]
pub enum Error {
    /// The PSP could not be opened.
    Usb(OpenError),
    /// The conversation with the PSP failed.
    Link(LinkError),
    /// The screen could not be captured.
    Capture(CaptureError),
    /// The virtual monitor could not be set up.
    Monitor(MonitorError),
    /// A picture could not be prepared.
    Convert(ConvertError),
    /// The virtual monitor is on the desktop but is not among the monitors that can be captured.
    NotCapturable,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usb(error) => error.fmt(f),
            Self::Link(error) => error.fmt(f),
            Self::Capture(error) => error.fmt(f),
            Self::Monitor(error) => error.fmt(f),
            Self::Convert(error) => error.fmt(f),
            Self::NotCapturable => {
                f.write_str("the virtual monitor is on the desktop but cannot be captured")
            }
        }
    }
}

impl std::error::Error for Error {}

macro_rules! from_error {
    ($($source:ty => $variant:ident),* $(,)?) => {
        $(impl From<$source> for Error {
            fn from(error: $source) -> Self {
                Self::$variant(error)
            }
        })*
    };
}

from_error! {
    OpenError => Usb,
    LinkError => Link,
    CaptureError => Capture,
    MonitorError => Monitor,
    ConvertError => Convert,
}

impl Error {
    /// Whether this is only the PSP not being ready: absent, or not answering. Such a failure is
    /// waited out, not reported as an end of the session.
    #[must_use]
    pub fn is_waiting_for_psp(&self) -> bool {
        matches!(
            self,
            Self::Usb(OpenError::NotFound)
                | Self::Link(LinkError::Pipe(
                    PipeError::Timeout | PipeError::Disconnected
                ))
        )
    }
}

/// Runs a session until `stop` is set or something that cannot be waited out goes wrong.
///
/// `report` is called from this thread with each [`Event`]. The virtual monitor, if one was
/// asked for, is taken off the desktop again when this returns.
///
/// # Errors
///
/// [`Error`] if the screen cannot be set up or captured, or the PSP fails in a way that is not
/// just its not being ready.
pub fn run(
    options: &Options,
    stop: &AtomicBool,
    mut report: impl FnMut(Event),
) -> Result<(), Error> {
    report(Event::Starting);

    // Kept for the whole session; dropping it removes the monitor from the desktop.
    let virtual_monitor = if options.virtual_monitor {
        Some(VirtualMonitor::attach(
            options.size.0,
            options.size.1,
            VIRTUAL_REFRESH_HZ,
        )?)
    } else {
        None
    };
    let index = match &virtual_monitor {
        Some(monitor) => capture::list_monitors()?
            .into_iter()
            .find(|candidate| candidate.name == monitor.name())
            .map(|found| found.index)
            .ok_or(Error::NotCapturable)?,
        None => options.monitor,
    };

    let first = Capturer::open(index)?;
    let mut source = Source {
        name: first.name().to_owned(),
        size: first.size(),
        capturer: Some(first),
        frame: Frame::default(),
        converter: Converter::new(options.quality),
        mapper: Mapper::new(options.controls),
        last_input: Instant::now(),
        virtual_monitor: options.virtual_monitor,
        last_check: Instant::now(),
        last_retry: Instant::now(),
    };

    while !stop.load(Ordering::SeqCst) {
        let Some(mut link) = connect(stop, &mut report)? else {
            break;
        };
        let ended = stream(&mut link, &mut source, stop, &mut report);
        // Whatever the way it ended, nothing the PSP was holding stays held on the PC.
        perform(source.mapper.release_all());
        match ended {
            Ok(()) => {}
            // The PSP stopped answering: the cable was pulled or its application was closed.
            // Wait for it and carry on with the same screen.
            Err(error) if error.is_waiting_for_psp() => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

/// Tries the PSP until it answers. `None` if `stop` was set first.
fn connect(
    stop: &AtomicBool,
    report: &mut impl FnMut(Event),
) -> Result<Option<Link<UsbPipe>>, Error> {
    let mut reported = false;
    while !stop.load(Ordering::SeqCst) {
        match open_link() {
            Ok(link) => return Ok(Some(link)),
            Err(error) if error.is_waiting_for_psp() => {
                if !reported {
                    report(Event::Waiting);
                    reported = true;
                }
                sleep_unless_stopped(CONNECT_RETRY, stop);
            }
            Err(error) => return Err(error),
        }
    }
    Ok(None)
}

fn open_link() -> Result<Link<UsbPipe>, Error> {
    let pipe = UsbPipe::open(TRANSFER_TIMEOUT)?;
    Ok(Link::open(pipe, HELLO_SETTLE)?)
}

/// Sleeps `total`, in short steps so that `stop` is noticed promptly.
fn sleep_unless_stopped(total: Duration, stop: &AtomicBool) {
    let pause = Duration::from_millis(50);
    let mut slept = Duration::ZERO;
    while slept < total && !stop.load(Ordering::SeqCst) {
        thread::sleep(pause);
        slept += pause;
    }
}

/// The screen being shown, kept across reconnections of the PSP.
struct Source {
    name: String,
    size: (u32, u32),
    /// `None` while the monitor is away; see [`stream`].
    capturer: Option<Capturer>,
    frame: Frame,
    converter: Converter,
    /// What is held on the PSP, so it can be released when the PSP goes away.
    mapper: Mapper,
    last_input: Instant,
    virtual_monitor: bool,
    last_check: Instant,
    last_retry: Instant,
}

/// Sends pictures until `stop` is set or a transfer fails.
fn stream(
    link: &mut Link<UsbPipe>,
    source: &mut Source,
    stop: &AtomicBool,
    report: &mut impl FnMut(Event),
) -> Result<(), Error> {
    let mut last_sent = Instant::now();
    let mut sent = 0_u32;
    let mut window = Instant::now();
    report(Event::Streaming {
        fps: 0,
        width: source.size.0,
        height: source.size.1,
    });

    while !stop.load(Ordering::SeqCst) {
        let Some(active) = source.capturer.as_mut() else {
            // The monitor is away: a change of resolution, Win+P, a mode change that ended the
            // capture. Keep the last picture on the PSP so it does not go into power saving,
            // and look for the monitor again. The old capture was dropped before this, which
            // Windows needs before it will describe the monitor as it is now.
            thread::sleep(CAPTURE_WAIT);
            if !source.frame.bgra.is_empty() && last_sent.elapsed() >= KEEP_ALIVE {
                let response = send_frame(link, &mut source.converter, &source.frame)?;
                handle_input(source, &response);
                last_sent = Instant::now();
            }
            if source.last_retry.elapsed() >= RETRY_EVERY {
                source.last_retry = Instant::now();
                reopen(source, report);
            }
            continue;
        };

        // Any error ends this capture: Windows reports a resolution change, Win+P and the like
        // in more than one way, and the cure is the same for all of them.
        let Ok(changed) = active.next_frame(duration_ms(CAPTURE_WAIT), &mut source.frame) else {
            source.capturer = None;
            report(Event::Reconnecting);
            continue;
        };
        if source.last_check.elapsed() >= RECHECK_EVERY {
            source.last_check = Instant::now();
            if active.check() != Check::Unchanged {
                source.capturer = None;
                report(Event::Reconnecting);
                continue;
            }
        }
        if source.frame.bgra.is_empty() || !(changed || last_sent.elapsed() >= KEEP_ALIVE) {
            continue;
        }
        let response = send_frame(link, &mut source.converter, &source.frame)?;
        handle_input(source, &response);
        last_sent = Instant::now();
        sent += 1;

        if window.elapsed() >= Duration::from_secs(1) {
            report(Event::Streaming {
                fps: sent,
                width: source.size.0,
                height: source.size.1,
            });
            sent = 0;
            window = Instant::now();
        }
    }
    Ok(())
}

/// Looks for the monitor again after it went away, and takes it up if it is back.
fn reopen(source: &mut Source, report: &mut impl FnMut(Event)) {
    if source.virtual_monitor {
        // Its name changes whenever the driver recreates it.
        if let Some(current) = monitor::find_virtual_monitor() {
            source.name = current;
        }
    }
    if let Ok(back) = Capturer::open_by_name(&source.name) {
        source.size = back.size();
        source.capturer = Some(back);
        report(Event::Streaming {
            fps: 0,
            width: source.size.0,
            height: source.size.1,
        });
    }
}

fn duration_ms(duration: Duration) -> u32 {
    u32::try_from(duration.as_millis()).unwrap_or(u32::MAX)
}

/// Converts `frame`, sends it to the PSP and returns what the PSP reports back.
fn send_frame(
    link: &mut Link<UsbPipe>,
    converter: &mut Converter,
    frame: &Frame,
) -> Result<Response, Error> {
    Ok(match converter.convert(frame)? {
        Picture::Jpeg(jpeg) => link.send_jpeg(jpeg)?,
        Picture::Raw(bgr) => link.send_raw(bgr)?,
    })
}

/// Does on the PC what the PSP's buttons and stick ask for.
fn handle_input(source: &mut Source, response: &Response) {
    let elapsed = source.last_input.elapsed();
    source.last_input = Instant::now();
    perform(source.mapper.update(response, elapsed));
}

/// Carries out the events. Windows may refuse one (a window with more rights than this program
/// is in front); that is not a reason to end the session, so it is let go.
fn perform(events: Vec<InputEvent>) {
    for event in events {
        let _ = inject::apply(&event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn psp_that_is_absent_or_silent_is_waited_for() {
        assert!(Error::Usb(OpenError::NotFound).is_waiting_for_psp());
        assert!(Error::Link(LinkError::Pipe(PipeError::Timeout)).is_waiting_for_psp());
        assert!(Error::Link(LinkError::Pipe(PipeError::Disconnected)).is_waiting_for_psp());
    }

    #[test]
    fn other_failures_end_the_session() {
        assert!(!Error::NotCapturable.is_waiting_for_psp());
        assert!(!Error::Monitor(MonitorError::NotFound).is_waiting_for_psp());
        assert!(!Error::Link(LinkError::RawSize(1)).is_waiting_for_psp());
        assert!(
            !Error::Link(LinkError::Pipe(PipeError::Other("denied".into()))).is_waiting_for_psp()
        );
    }

    #[test]
    fn defaults_show_the_recommended_size_at_medium_quality() {
        let options = Options::default();
        assert_eq!(options.size, (1366, 768));
        assert_eq!(options.quality, Quality::Medium);
        assert!(!options.virtual_monitor);
    }

    #[test]
    fn sleep_ends_early_when_stopped() {
        let stop = AtomicBool::new(true);
        let started = Instant::now();
        sleep_unless_stopped(Duration::from_secs(5), &stop);
        assert!(started.elapsed() < Duration::from_secs(1));
    }
}
