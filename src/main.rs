use serialport::*;
use std::sync;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;

mod serial_controller;
use serial_controller::{ControllerMsg, SerialController};

#[cfg(target_os = "linux")]
const DEFAULT_COM: &str = "NON";

#[cfg(target_os = "windows")]
const DEFAULT_COM: &str = "COM1";
#[derive(Debug, StructOpt)]
#[structopt(name = "COM_tool", about = "About.")]
struct ArgCommands {
    #[structopt(short, long, default_value = DEFAULT_COM)]
    target: String,

    #[structopt(short, long)]
    baud_rate: Option<u32>,
}

fn main() {
    let x = ArgCommands::from_args();
    println!("{}", x.target);
    let s = SerialPortSettings {
        baud_rate: 115200,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(1),
    };
    let mut serial = serialport::open_with_settings(&x.target, &s).expect("Can't open port.");
    let handler = SerialController::start_thread(serial);

    loop {
        let mut rbuffer = String::new();
        let i = std::io::stdin().read_line(&mut rbuffer).expect("read_line");

        let rbuffer = Box::from(rbuffer[..i - 2].as_bytes());
        handler.tx.send(ControllerMsg::Send(rbuffer)).unwrap();
    }
}
