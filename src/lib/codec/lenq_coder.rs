use super::{Decoder, Encoder};
use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(Debug)]
pub struct LenqCoder {
    state_read: StateRead,
    buff: BytesMut,
    crc: u16,
    lenq: u16, //lenmessage
}

impl LenqCoder {
    const HEAD: u8 = 40; //(    //def: 0xFE
    const ESC: u8 = 43;
    const ESC_HEAD: u8 = 48; //0xFC;
    const MAX_LEN: u16 = 1024;

    pub fn new() -> LenqCoder {
        let buff = BytesMut::new();
        let state_read = StateRead::Head;
        let crc = 0;
        let lenq = 0;
        LenqCoder {
            state_read,
            buff,
            crc,
            lenq,
        }
    }
    pub fn reset_read(&mut self) {
        self.state_read = StateRead::Head;
        self.buff.clear();
        self.crc = 0;
        self.lenq = 0;
    }
}

impl Encoder for LenqCoder {
    type Item = Bytes;

    fn encode(&mut self, data: Self::Item, dst: &mut BytesMut) -> Result<(), ()> {
        let mut n = data.remaining();

        let mut crc: u16 = LenqCoder::HEAD.into();
        dst.put_u8(LenqCoder::HEAD);
        crc += 0x00;
        dst.put_u8(0x00);

        if n > 1024 {
            n = 1024;
        }
        let lenq = n as u16;
        crc += lenq;
        if crc == LenqCoder::HEAD as u16 {
            crc += 1;
        }

        dst.put_u16(lenq);
        dst.put_u16(crc);

        for i in 0..n {
            match data[i] {
                LenqCoder::HEAD => {
                    dst.put_u8(LenqCoder::ESC);
                    dst.put_u8(LenqCoder::ESC_HEAD);
                    crc += LenqCoder::ESC as u16;
                    crc += LenqCoder::ESC_HEAD as u16;
                }
                LenqCoder::ESC => {
                    dst.put_u8(LenqCoder::ESC);
                    dst.put_u8(LenqCoder::ESC);
                    crc += LenqCoder::ESC as u16;
                    crc += LenqCoder::ESC as u16;
                }
                val => {
                    dst.put_u8(val);
                    crc += val as u16;
                }
            }
        }
        dst.put_u16(crc);
        println!("{}",crc);
        Ok(())
    }
}

impl Decoder for LenqCoder {
    type Item = Bytes;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, ()> {
        
        while src.has_remaining() {
            
            let value = src.get_u8();
            
            if value == LenqCoder::HEAD {
                self.reset_read();
            }
            

            match self.state_read {
                StateRead::Head => {
                    if value == LenqCoder::HEAD {
                        self.crc = LenqCoder::HEAD as u16;
                        self.state_read = StateRead::Data;
                        //Read Options, lenq and check CRC
                        if src.remaining() > 5 {
                            self.crc += src.get_u8() as u16;
                            self.lenq = src.get_u16() as u16;
                            if self.lenq > 1024 {
                                self.lenq = 1024;
                            }
                            self.crc += self.lenq;
                            //check CRC in header
                            if self.crc != src.get_u16() {
                                self.reset_read();
                            }
                        } else {
                            self.reset_read();
                        }
                    }
                }
                StateRead::Esc => {
                    self.crc += value as u16;
                    self.state_read = StateRead::Data;
                    match value {
                        LenqCoder::ESC_HEAD => {
                            self.buff.put_u8(LenqCoder::HEAD);
                        }
                        LenqCoder::ESC => {
                            self.buff.put_u8(LenqCoder::ESC);
                        }
                        val => {
                            //invalid! reset state.
                            self.reset_read();
                        }
                    }
                }
                StateRead::Data => {
                    self.crc += value as u16;
                    self.lenq -= 1;
                    match value {
                        LenqCoder::HEAD => unreachable!(),
                        LenqCoder::ESC => {
                            self.state_read = StateRead::Esc;
                        }
                        val => {
                            self.buff.put_u8(val);
                        }
                    };
                }
                StateRead::Crc => {
                    if src.remaining() >= 1 {
                        let mut bsrc = src.get_u8();
                        if bsrc == LenqCoder::HEAD {
                            bsrc += 1;
                        }

                        let bsrc = (value as u16) << 8 | bsrc as u16;
                        if self.crc == bsrc as u16 {
                            
                            let result = Ok(Some(self.buff.clone().freeze()));
                            self.reset_read();
                            return result;
                        } else {
                            self.reset_read();
                        }
                    }
                }
            }
            //Gdy w trakcie sprawdzania danych, licznik się skończył, sprawdzana jest poprawność wiadomości
            if (self.state_read == StateRead::Data) && (self.lenq <= 0) {
                self.state_read = StateRead::Crc;
            }
        }

        Ok(None)
    }
}

#[derive(Debug, Eq, PartialEq)]
enum StateRead {
    Head,
    Data,
    Esc,
    Crc,
}
