use std::time::Duration;

use super::args::*;

use super::serial_controller::*;
use super::serial_controller::ControllerMsg::*;

pub trait ModeProgram {
    fn enter(&self);
}

pub struct Interactive {
    handler: SerialController,
    args: ArgCommands
}
pub struct LoopShot {
    handler: SerialController,
    interval: u64,
    command: String,
}

pub struct OneShot;

impl ModeProgram for Interactive {
    fn enter(&self) {
        let mut rbuffer = String::new();
        loop {
            let i = std::io::stdin().read_line(&mut rbuffer).expect("read_line");
    
            let rbuffer = Box::from(rbuffer[..i - 2].as_bytes());
            self.handler.tx.send(ControllerMsg::Send(rbuffer)).unwrap();
        }
    }
}

impl Interactive {
    pub fn new(handler: SerialController, args: ArgCommands) -> Interactive {
        Interactive {
            handler,
            args
        }
    }
}

impl ModeProgram for LoopShot {
    fn enter(&self) {
        loop {
            let cmd = Box::from(self.command.as_bytes());
            self.handler.tx.send(ControllerMsg::Send(cmd)).unwrap();
            std::thread::sleep(Duration::from_millis(self.interval));
        }
    }
}

impl LoopShot {
    pub fn new(handler: SerialController, args: ArgCommands) -> LoopShot {
        if let ArgMode::Loopshot{interval, command} = args.mode {
            LoopShot {
                handler,
                interval,
                command
            }
        } else {
            panic!("Bad args: {:?}", args);
        }
        
    }
}