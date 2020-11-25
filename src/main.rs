use serialport::{SerialPortSettings, DataBits, FlowControl, Parity, StopBits};
use std::{str::FromStr, sync};
use std::thread;
use std::time::Duration;
use structopt::StructOpt;
use std::sync::Arc;


pub mod lib;
use lib::serial_controller::{ControllerMsg, SerialController};
use lib::modes::*;
use lib::args::*;

fn main() {
    let args = ArgCommands::from_args();
    println!("{:?}", args);
    let s = SerialPortSettings {
        baud_rate: 115200,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(1),
    };
    let mut serial = serialport::open_with_settings(&args.target, &s).expect("Can't open port.");
    let handler = SerialController::start_thread(serial, args.clone());

   match args.mode {
        ArgMode::Interactive => Interactive::new(handler, args).enter(),
        ArgMode::Loopshot{..} => LoopShot::new(handler, args).enter()
    };

    
}