//! Command line of the PSP Sidescreen host.
//!
//! - `sidescreen-host monitors` lists the monitors that can be shown on the PSP.
//! - `sidescreen-host detach` takes the virtual monitor off the desktop, if a run left it there.
//! - `sidescreen-host probe` checks that the PSP can be reached.
//! - `sidescreen-host send <file.jpg>` shows a 480x272 JPEG on it.
//! - `sidescreen-host run [--virtual] [--size WxH] [--monitor N] [--quality NAME]` shows a
//!   monitor live; the quality is lossless, high, medium or low. With `--virtual` the PSP gets a monitor of its own, as an extra screen.

use std::error::Error;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use sidescreen_host::capture::{self, CaptureError, Capturer, Frame};
use sidescreen_host::convert::{Converter, Picture, Quality};
use sidescreen_host::link::Link;
use sidescreen_host::monitor::{self, VirtualMonitor};
use sidescreen_host::usb::UsbPipe;

/// How long one transfer may take before the PSP is considered stuck.
const TRANSFER_TIMEOUT: Duration = Duration::from_millis(1000);
/// Pause PSPdisp makes after the first bytes of the hello.
const HELLO_SETTLE: Duration = Duration::from_millis(500);
/// How long to wait for the screen to change before looking again.
const CAPTURE_WAIT_MS: u32 = 33;
/// A picture is sent again after this long without change, so the PSP knows the host is alive.
const KEEP_ALIVE: Duration = Duration::from_millis(500);
/// Quality when none is given.
const DEFAULT_QUALITY: Quality = Quality::Medium;
/// Size of the virtual monitor when none is given. Small text stays readable on the PSP's
/// 480x272 screen at this size, and the driver offers it.
const DEFAULT_SIZE: (u32, u32) = (1366, 768);
/// Refresh rate asked of the virtual monitor.
const VIRTUAL_REFRESH_HZ: u32 = 60;

const USAGE: &str = "usage: sidescreen-host monitors | detach | probe | send <file.jpg> | \
run [--virtual] [--size WxH] [--monitor N] [--quality lossless|high|medium|low]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = match args.as_slice() {
        ["monitors"] => monitors(),
        ["detach"] => detach(),
        ["probe"] => probe(),
        ["send", path] => send(path),
        ["run", options @ ..] => run(options),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn monitors() -> Result<(), Box<dyn Error>> {
    for monitor in capture::list_monitors()? {
        println!(
            "{}: {} {}x{} at {},{}",
            monitor.index, monitor.name, monitor.width, monitor.height, monitor.left, monitor.top
        );
    }
    Ok(())
}

fn detach() -> Result<(), Box<dyn Error>> {
    monitor::detach()?;
    println!("the virtual monitor is off the desktop");
    Ok(())
}

fn probe() -> Result<(), Box<dyn Error>> {
    let pipe = UsbPipe::open(TRANSFER_TIMEOUT)?;
    Link::open(pipe, HELLO_SETTLE)?;
    println!("PSP found and the link is open");
    Ok(())
}

fn send(path: &str) -> Result<(), Box<dyn Error>> {
    let jpeg = std::fs::read(path)?;
    let pipe = UsbPipe::open(TRANSFER_TIMEOUT)?;
    let mut link = Link::open(pipe, HELLO_SETTLE)?;
    let response = link.send_jpeg(&jpeg)?;
    println!(
        "frame sent; PSP reports buttons {:#06x}, stick {},{}",
        response.buttons, response.analog_x, response.analog_y
    );
    Ok(())
}

/// Options of `run`.
struct RunOptions {
    /// Give the PSP a virtual monitor of its own instead of showing an existing one.
    virtual_monitor: bool,
    /// Size of the virtual monitor.
    size: (u32, u32),
    /// Existing monitor to show when `virtual_monitor` is off.
    monitor: usize,
    quality: Quality,
}

fn parse_size(text: &str) -> Result<(u32, u32), Box<dyn Error>> {
    let (width, height) = text
        .split_once('x')
        .ok_or_else(|| format!("size must look like 1366x768, not {text}"))?;
    Ok((width.parse()?, height.parse()?))
}

fn parse_run_options(options: &[&str]) -> Result<RunOptions, Box<dyn Error>> {
    let mut parsed = RunOptions {
        virtual_monitor: false,
        size: DEFAULT_SIZE,
        monitor: 0,
        quality: DEFAULT_QUALITY,
    };
    let mut iter = options.iter();
    while let Some(option) = iter.next() {
        if *option == "--virtual" {
            parsed.virtual_monitor = true;
            continue;
        }
        let value = iter
            .next()
            .ok_or_else(|| format!("{option} needs a value"))?;
        match *option {
            "--monitor" => parsed.monitor = value.parse()?,
            "--quality" => parsed.quality = value.parse()?,
            "--size" => parsed.size = parse_size(value)?,
            other => return Err(format!("unknown option {other}").into()),
        }
    }
    Ok(parsed)
}

fn run(options: &[&str]) -> Result<(), Box<dyn Error>> {
    let options = parse_run_options(options)?;

    // Ctrl+C ends the loop instead of the process, so the virtual monitor is taken off the
    // desktop on the way out.
    let stop = Arc::new(AtomicBool::new(false));
    let handler_flag = Arc::clone(&stop);
    ctrlc::set_handler(move || handler_flag.store(true, Ordering::SeqCst))?;

    let pipe = UsbPipe::open(TRANSFER_TIMEOUT)?;
    let mut link = Link::open(pipe, HELLO_SETTLE)?;

    // Kept alive for the whole session; dropping it removes the monitor from the desktop.
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
            .ok_or("the virtual monitor is on the desktop but cannot be captured")?,
        None => options.monitor,
    };

    let mut capturer = Capturer::open(index)?;
    let (width, height) = capturer.size();
    println!("showing monitor {index} ({width}x{height}); press Ctrl+C to stop");

    let mut converter = Converter::new(options.quality);
    let mut frame = Frame::default();
    let mut last_sent = Instant::now();
    let mut sent = 0_u32;
    let mut window = Instant::now();

    while !stop.load(Ordering::SeqCst) {
        let changed = match capturer.next_frame(CAPTURE_WAIT_MS, &mut frame) {
            Ok(changed) => changed,
            Err(CaptureError::AccessLost) => {
                // A secure desktop or a mode change ends the duplication; start it again.
                std::thread::sleep(Duration::from_millis(250));
                capturer = Capturer::open(index)?;
                continue;
            }
            Err(error) => return Err(error.into()),
        };
        if frame.bgra.is_empty() || !(changed || last_sent.elapsed() >= KEEP_ALIVE) {
            continue;
        }
        match converter.convert(&frame)? {
            Picture::Jpeg(jpeg) => link.send_jpeg(jpeg)?,
            Picture::Raw(bgr) => link.send_raw(bgr)?,
        };
        last_sent = Instant::now();
        sent += 1;

        if window.elapsed() >= Duration::from_secs(1) {
            println!("{sent} frames/s");
            sent = 0;
            window = Instant::now();
        }
    }
    println!("stopped");
    Ok(())
}
