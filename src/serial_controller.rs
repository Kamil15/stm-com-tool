use serialport::SerialPort;
use std::cell::RefCell;
use std::io::{self, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

pub struct SerialController {
    handler: JoinHandle<()>,
    pub tx: Sender<ControllerMsg>,
    pub rx: Receiver<ControllerMsg>,
}

impl SerialController {
    pub fn start_thread(serial: Box<dyn SerialPort>) -> SerialController {
        let (tx, thread_rx) = mpsc::channel();
        let (thread_tx, rx) = mpsc::channel();

        let handler = thread::spawn(move || {
            let thread_self = SerialControllerThread {
                tx: thread_tx,
                rx: thread_rx,
                serial: serial,
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

pub struct SerialControllerThread {
    tx: Sender<ControllerMsg>,
    rx: Receiver<ControllerMsg>,
    serial: Box<dyn SerialPort>,
}

impl SerialControllerThread {
    fn main_loop(mut self) {
        let mut r_bufer: [u8; 1024] = [0; 1024];
        loop {
            if let Ok(i) = self.serial.bytes_to_read() {
                if i > 0 {
                    let i = i as usize;
                    let x = &mut r_bufer[0..i];
                    let n = self.serial.read(&mut r_bufer[0..i]).unwrap_or(0);
                    if n > 1 {
                        let text = String::from_utf8_lossy(&r_bufer[..n]);
                        print!("{}", text);
                        std::io::stdout().flush().unwrap();
                    }
                }
            }

            if let Ok(x) = self.rx.try_recv() {
                if let ControllerMsg::Send(val) = x {
                    //let mut q = val.as_bytes();
                    let mut val = &(*val);
                    println!("{:?}", val);
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
