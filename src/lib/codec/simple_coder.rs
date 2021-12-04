use super::{Decoder, Encoder};
use bytes::{Buf, BufMut, Bytes, BytesMut};

pub struct SimpleCoder {
    state_read: StateRead,
    buff: BytesMut,
}

impl SimpleCoder {
    const HEAD: u8 = 40; //A    //def: 0xFE
    const END: u8 = 41; //Z     //def: 0xFF;
    const ESC: u8 = 43;
    const ESC_HEAD: u8 = 48; //0xFC;
    const ESC_END: u8 = 49; //0xFB;
    const MAX_LEN: u16 = 1024;

    pub fn new() -> SimpleCoder {
        let buff = BytesMut::new();
        let state_read = StateRead::Head;
        SimpleCoder { state_read, buff }
    }
    pub fn reset_read(&mut self) {
        self.state_read = StateRead::Head;
        self.buff.clear();
    }
}

impl Encoder for SimpleCoder {
    type Item = Bytes;

    fn encode(&mut self, data: Self::Item, dst: &mut BytesMut) -> Result<(), ()> {
        let data = data.as_ref();

        dst.put_u8(SimpleCoder::HEAD);

        for x in data {
            match *x {
                SimpleCoder::HEAD => {
                    dst.put_u8(SimpleCoder::ESC);
                    dst.put_u8(SimpleCoder::ESC_HEAD);
                }
                SimpleCoder::END => {
                    dst.put_u8(SimpleCoder::ESC);
                    dst.put_u8(SimpleCoder::ESC_END);
                }
                SimpleCoder::ESC => {
                    dst.put_u8(SimpleCoder::ESC);
                    dst.put_u8(SimpleCoder::ESC);
                }
                val => dst.put_u8(val),
            }
        }

        dst.put_u8(SimpleCoder::END);
        Ok(())
    }
}

impl Decoder for SimpleCoder {
    type Item = Bytes;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, ()> {
        let mut n = src.len();

        if n > 1024 {
            n = 1024;
        }

        for i in 0..n {
            let value = src.get_u8();

            match self.state_read {
                StateRead::Head => {
                    if value == SimpleCoder::HEAD {
                        self.state_read = StateRead::Data;
                    }
                }
                StateRead::Esc => {
                    self.state_read = StateRead::Data;
                    match value {
                        SimpleCoder::ESC_HEAD => {
                            self.buff.put_u8(SimpleCoder::HEAD);
                        }
                        SimpleCoder::ESC_END => {
                            self.buff.put_u8(SimpleCoder::END);
                        }
                        SimpleCoder::ESC => {
                            self.buff.put_u8(SimpleCoder::ESC);
                        }
                        val => {
                            //invalid!
                            self.state_read = StateRead::Head;
                            self.buff.clear();
                        }
                    }
                }
                StateRead::Data => {
                    match value {
                        SimpleCoder::HEAD => {
                            //new packed, restart
                            self.buff.clear();
                            self.state_read = StateRead::Head;
                        }
                        SimpleCoder::END => {
                            self.state_read = StateRead::Head;
                            let result = Ok(Some(self.buff.clone().freeze()));
                            self.buff.clear();
                            return result;
                        }
                        SimpleCoder::ESC => {
                            self.buff.reserve(1);
                            self.state_read = StateRead::Esc;
                        }
                        val => {
                            self.buff.reserve(1);
                            self.buff.put_u8(val);
                        }
                    };
                }
            }
        }
        if self.buff.len() > 1024 {
            self.state_read = StateRead::Head;
            self.buff.clear();
        }
        Ok(None)
    }
}

#[derive(Debug)]
enum StateRead {
    Head,
    Data,
    Esc,
}
