//! Command line of the PSP Sidescreen host.
//!
//! `sidescreen-host probe` checks that the PSP can be reached; `sidescreen-host send <file.jpg>`
//! shows a 480x272 JPEG on it.

use std::process::ExitCode;
use std::time::Duration;

use sidescreen_host::link::Link;
use sidescreen_host::usb::UsbPipe;

/// How long one transfer may take before the PSP is considered stuck.
const TRANSFER_TIMEOUT: Duration = Duration::from_millis(1000);
/// Pause PSPdisp makes after the first bytes of the hello.
const HELLO_SETTLE: Duration = Duration::from_millis(500);

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["probe"] => probe(),
        ["send", path] => send(path),
        _ => {
            eprintln!("usage: sidescreen-host probe | send <file.jpg>");
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

fn probe() -> Result<(), Box<dyn std::error::Error>> {
    let pipe = UsbPipe::open(TRANSFER_TIMEOUT)?;
    Link::open(pipe, HELLO_SETTLE)?;
    println!("PSP found and the link is open");
    Ok(())
}

fn send(path: &str) -> Result<(), Box<dyn std::error::Error>> {
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
