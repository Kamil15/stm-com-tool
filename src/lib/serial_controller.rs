//! A simple controller that keeps the SerialPort in a separate thread.

use serialport::SerialPort;
use std::io::{self, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use super::args::ArgCommands;

/// Thread Handler and communication interface with the SerialPort thread.
pub struct SerialController {
    handler: JoinHandle<()>,
    pub tx: Sender<ControllerMsg>,
    pub rx: Receiver<ControllerMsg>,
}

impl SerialController {
    /// Create a new thread with the running SerialPort
    pub fn start_thread(serial: Box<dyn SerialPort>, args: ArgCommands) -> SerialController {
        let (tx, thread_rx) = mpsc::channel();
        let (thread_tx, rx) = mpsc::channel();

        let handler = thread::spawn(move || {
            let thread_self = SerialControllerThread {
                tx: thread_tx,
                rx: thread_rx,
                serial,
                args,
            };
            thread_self.main_loop();
        });

        SerialController {
            handler: handler,
            tx: tx,
            rx: rx,
        }
    }
}

struct SerialControllerThread {
    tx: Sender<ControllerMsg>,
    rx: Receiver<ControllerMsg>,
    serial: Box<dyn SerialPort>,
    args: ArgCommands,
}

impl SerialControllerThread {
    fn main_loop(mut self) {
        let term = console::Term::stdout();
        let mut r_bufer: [u8; 1024] = [0; 1024];
        loop {
            if let Ok(i) = self.serial.bytes_to_read() {
                if i > 0 {
                    let i = i as usize;
                    let x = &mut r_bufer[0..i];
                    let i = self.serial.read(&mut r_bufer[0..i]).unwrap_or(0);
                    let text = String::from_utf8_lossy(&r_bufer[..i]);
                    print!("{}", text);
                    std::io::stdout().flush().unwrap();
                }
            }

            if let Ok(x) = self.rx.try_recv() {
                if let ControllerMsg::Send(val) = x {
                    let mut val = &(*val);
                    if self.args.debug == true {
                        println!("{:?}", val);
                    }
                    self.serial.write_all(val).unwrap();
                }
            }
        }
    }
}

pub enum ControllerMsg {
    Send(Box<[u8]>),
    None,
}
