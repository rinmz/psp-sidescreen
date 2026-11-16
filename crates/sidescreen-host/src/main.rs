//! Command line of the PSP Sidescreen host.
//!
//! - `sidescreen-host monitors` lists the monitors that can be shown on the PSP.
//! - `sidescreen-host detach` takes the virtual monitor off the desktop, if a run left it there.
//! - `sidescreen-host probe` checks that the PSP can be reached.
//! - `sidescreen-host send <file.jpg>` shows a 480x272 JPEG on it.
//! - `sidescreen-host run [--virtual] [--size WxH] [--monitor N] [--quality NAME]` shows a
//!   monitor live; the quality is lossless, high, medium or low. With `--virtual` the PSP gets a
//!   monitor of its own, as an extra screen.

use std::error::Error;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use sidescreen_host::capture;
use sidescreen_host::dpi;
use sidescreen_host::link::Link;
use sidescreen_host::monitor;
use sidescreen_host::session::{self, Event, Options};
use sidescreen_host::usb::UsbPipe;

/// How long one transfer may take before the PSP is considered stuck.
const TRANSFER_TIMEOUT: Duration = Duration::from_millis(1000);
/// Pause PSPdisp makes after the first bytes of the hello.
const HELLO_SETTLE: Duration = Duration::from_millis(500);

const USAGE: &str = "usage: sidescreen-host monitors | detach | probe | send <file.jpg> | \
run [--virtual] [--size WxH] [--monitor N] [--quality lossless|high|medium|low]";

fn main() -> ExitCode {
    // Before anything asks Windows about monitors; see the `dpi` module.
    dpi::make_process_aware();

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

fn parse_size(text: &str) -> Result<(u32, u32), Box<dyn Error>> {
    let (width, height) = text
        .split_once('x')
        .ok_or_else(|| format!("size must look like 1366x768, not {text}"))?;
    Ok((width.parse()?, height.parse()?))
}

fn parse_run_options(arguments: &[&str]) -> Result<Options, Box<dyn Error>> {
    let mut options = Options::default();
    let mut iter = arguments.iter();
    while let Some(argument) = iter.next() {
        if *argument == "--virtual" {
            options.virtual_monitor = true;
            continue;
        }
        let value = iter
            .next()
            .ok_or_else(|| format!("{argument} needs a value"))?;
        match *argument {
            "--monitor" => options.monitor = value.parse()?,
            "--quality" => options.quality = value.parse()?,
            "--size" => options.size = parse_size(value)?,
            other => return Err(format!("unknown option {other}").into()),
        }
    }
    Ok(options)
}

fn run(arguments: &[&str]) -> Result<(), Box<dyn Error>> {
    let options = parse_run_options(arguments)?;

    // Ctrl+C ends the session instead of the process, so the virtual monitor is taken off the
    // desktop on the way out.
    let stop = Arc::new(AtomicBool::new(false));
    let handler_flag = Arc::clone(&stop);
    ctrlc::set_handler(move || handler_flag.store(true, Ordering::SeqCst))?;

    println!("press Ctrl+C to stop");
    session::run(&options, &stop, |event| match event {
        Event::Starting => {}
        Event::Waiting => {
            println!("waiting for the PSP; open Sidescreen on it and choose Start using USB");
        }
        Event::Streaming {
            fps: 0,
            width,
            height,
        } => println!("showing {width}x{height}"),
        Event::Streaming { fps, .. } => println!("{fps} frames/s"),
        Event::Reconnecting => println!("the screen changed; picking it up again"),
    })?;
    println!("stopped");
    Ok(())
}
