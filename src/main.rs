use serialport::{DataBits, FlowControl, Parity, SerialPortSettings, StopBits};
use std::time::Duration;
use structopt::StructOpt;

pub mod lib;
use bytes::*;
use lib::modes::*;
use lib::serial_controller::SerialController;
use lib::{args::*, codec::lenq_coder::LenqCoder, codec::*};

fn main() {
    let args = ArgCommands::from_args();
    //println!("{:?}", args);
    let s = SerialPortSettings {
        baud_rate: 115200,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(1),
    };
    let serial = serialport::open_with_settings(&args.target, &s).expect("Can't open port.");
    let handler = SerialController::start_thread_codec(serial, args.clone());
    //let handler = SerialController::start_thread(serial, args.clone());

    match args.mode {
        ArgMode::Interactive => Interactive::new(handler, args).enter(),
        ArgMode::Loopshot { .. } => LoopShot::new(handler, args).enter(),
        ArgMode::Oneshot {..} => OneShot::new(handler, args).enter(),
    };
    //Interactive::new(handler, args).enter();
    
}
