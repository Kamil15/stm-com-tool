use super::{crc16, crc16_for_u16, Decoder, Encoder};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::io::{self, Write};

#[derive(Debug)]
pub struct SlipLnqCoder {
    state_read: StateRead,
    buff: BytesMut,
    adrr: u8,
    crc: u16,
    lenq: u16, //lenmessage
}

impl SlipLnqCoder {
    const HEAD: u8 = 0x7E;//40; //(    //def: 0xFE
    const MAX_LEN: u16 = 1024;

    pub fn new() -> SlipLnqCoder {
        let buff = BytesMut::new();
        let state_read = StateRead::Head;
        let adrr = 0;
        let crc = 0xFFFF;
        let lenq = 0;
        SlipLnqCoder {
            state_read,
            buff,
            adrr,
            crc,
            lenq,
        }
    }
    pub fn reset_read(&mut self) {
        self.state_read = StateRead::Head;
        self.buff.clear();
        self.adrr = 0;
        self.crc = 0xFFFF;
        self.lenq = 0;
    }
}

impl Encoder for SlipLnqCoder {
    type Item = Bytes;

    fn encode(&mut self, data: Self::Item, dst: &mut BytesMut) -> Result<(), ()> {
        let mut n = data.remaining();
        let mut crc: u16 = 0xffff;


        Ok(())
    }
}

impl Decoder for SlipLnqCoder {
    type Item = Bytes;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, ()> {
        while src.has_remaining() {
            /*match self.state_read {
                StateRead::Slip => {
                    if src.remaining() >= 4 {

                    }
                    
                }
                StateRead::Data => {
                }
                StateRead::Crc => {
                }
            }*/
        }
        Ok(None)
    }
}

#[derive(Debug, Eq, PartialEq)]
enum StateRead {
    Slip,
    Head,
    Data,
    Crc,
}
