//! A simple controller that keeps the SerialPort in a separate thread.

use super::codec::{crc_len_coder::CRCLenCoder, Decoder, Encoder};
use bytes::{*};
use serialport::*;
use serialport::SerialPort;
use std::io::{self, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

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
            let thread_self = SimpleSerialController {
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

    pub fn start_thread_codec(serial: Box<dyn SerialPort>, args: ArgCommands) -> SerialController {
        let (tx, thread_rx) = mpsc::channel();
        let (thread_tx, rx) = mpsc::channel();
        let encoder = Box::new(CRCLenCoder::new());
        let decoder = Box::new(CRCLenCoder::new());

        let handler = thread::spawn(move || {
            let thread_self = CodecSerialController {
                tx: thread_tx,
                rx: thread_rx,
                serial,
                encoder,
                decoder,
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

struct SimpleSerialController {
    tx: Sender<ControllerMsg>,
    rx: Receiver<ControllerMsg>,
    serial: Box<dyn SerialPort>,
    args: ArgCommands,
}

impl SimpleSerialController {
    fn main_loop(mut self) {
        let term = console::Term::stdout();
        let mut r_bufer: [u8; 512] = [0; 512];
        let mut r_vec = Vec::new();
        loop {
            if let Ok(i) = self.serial.bytes_to_read() {
                if i > 0 {
                    let i = i as usize;
                    let y = self.serial.read(&mut r_bufer[..i]).expect("lol");
                    self.serial.clear(ClearBuffer::All);
                    let text = String::from_utf8_lossy(&r_vec[..y]);
                    std::io::stdout().flush().unwrap();
                }
            }

            if let Ok(x) = self.rx.try_recv() {
                if let ControllerMsg::Send(val) = x {
                    let mut val = &(*val);
                    if self.args.verbose == true {
                        println!("{:?}", val);
                    }
                    self.serial.write_all(val).unwrap();
                }
            }
        }
    }
}

struct CodecSerialController {
    tx: Sender<ControllerMsg>,
    rx: Receiver<ControllerMsg>,
    serial: Box<dyn SerialPort>,
    encoder: Box<dyn Encoder<Item = Bytes>>,
    decoder: Box<dyn Decoder<Item = Bytes>>,
    args: ArgCommands,
}

impl CodecSerialController {
    fn main_loop(mut self) {
        let mut r_bufer: [u8; 1024] = [0; 1024];
        let mut code_buffer = BytesMut::with_capacity(1024);
        loop {
            if let Ok(i) = self.serial.bytes_to_read() {
                if i > 0 {
                    let i = i as usize;
                    let x = &mut r_bufer[0..i];
                    let i = self.serial.read(&mut r_bufer[0..i]).unwrap_or(0);
                    //code_buffer.put_slice(&r_bufer[0..i]);
                    //println!("{:?}", code_buffer);
                    let stry = String::from_utf8_lossy(&r_bufer[0..i]);
                    print!("{}", stry);
                    std::io::stdout().flush().unwrap();
                }
            }

            /*if let Some(value) = self.decoder.decode(&mut code_buffer).expect("error decoding") {
                print!("{}", String::from_utf8_lossy(value.as_ref()));
                std::io::stdout().flush().unwrap();
            }*/

            if let Ok(x) = self.rx.try_recv() {
                if let ControllerMsg::Send(val) = x {
                    let data = Bytes::from(Vec::from(val));
                    let mut dst = BytesMut::with_capacity(data.len()+20);
                    self.encoder.encode(data, &mut dst).expect("Errpr encode");
                    let encoded = dst.as_ref();
                    if self.args.verbose == true {
                        println!("input = {:?}", encoded);
                        println!("{}", String::from_utf8_lossy(encoded));
                        println!("----------------");
                    }
                    
                    self.serial.write_all(encoded).unwrap();
                }
            }
        }
    }
}

pub enum ControllerMsg {
    Send(Box<[u8]>),
    None,
}
