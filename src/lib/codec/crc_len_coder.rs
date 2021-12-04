use super::{crc16, crc16_for_u16, Decoder, Encoder};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::io::{self, Write};

#[derive(Debug)]
pub struct CRCLenCoder {
    state_read: StateRead,
    buff: BytesMut,
    adrr: u8,
    crc: u16,
    lenq: u16, //lenmessage
}

impl CRCLenCoder {
    const HEAD: u8 = 0x7E;//40; //(    //def: 0xFE
    const CONTROL: u8 = 0x03;
    const MAX_LEN: u16 = 1024;

    pub fn new() -> CRCLenCoder {
        let buff = BytesMut::new();
        let state_read = StateRead::Head;
        let adrr = 0;
        let crc = 0xFFFF;
        let lenq = 0;
        CRCLenCoder {
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

impl Encoder for CRCLenCoder {
    type Item = Bytes;

    fn encode(&mut self, data: Self::Item, dst: &mut BytesMut) -> Result<(), ()> {
        let mut n = data.remaining();
        let mut crc: u16 = 0xffff;

        if n > 65536 {
            n = 65536; //should throw errror...  Err(())
        }

        dst.put_u8(CRCLenCoder::HEAD);
        //Address
        dst.put_u8(self.adrr);
        //Control
        dst.put_u8(CRCLenCoder::CONTROL);
        //Options
        dst.put_u8(0x00);

        crc = crc16(crc, CRCLenCoder::HEAD);
        crc = crc16(crc, 0x00);
        crc = crc16(crc, CRCLenCoder::CONTROL);
        crc = crc16(crc, 0x00);
        

        //length
        dst.put_u16(n as u16);
        crc = crc16_for_u16(crc, n as u16);

        for i in 0..n {
            let value = data[i];
            dst.put_u8(data[i]);
            crc = crc16(crc, value);
        }
        dst.put_u16(crc);

        Ok(())
    }
}

impl Decoder for CRCLenCoder {
    type Item = Bytes;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, ()> {
        while src.has_remaining() {
            match self.state_read {
                StateRead::Head => {
                    if src.remaining() >= 4 {
                        let value = src.get_u8();
                        if value == CRCLenCoder::HEAD {
                            self.crc = crc16(self.crc, value);
                            self.adrr = src.get_u8();
                            let control = src.get_u8();
                            if control != CRCLenCoder::CONTROL {
                                self.reset_read();
                                break;
                            }
                            self.crc = crc16(self.crc, self.adrr);
                            self.crc = crc16(self.crc, control);
    
                            //Options
                            let options = src.get_u8();
                            self.crc = crc16(self.crc, options);
                            self.state_read = StateRead::Length;
    
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                    
                }
                StateRead::Length => {
                    if src.remaining() >= 2 {
                        let value = src.get_u8();
                        self.crc = crc16(self.crc, value);
                        
                        let value2 = src.get_u8();
                        self.lenq = ((value as u16) << 8) | value2 as u16;

                        self.state_read = StateRead::Data;
                        self.crc = crc16(self.crc, value2);
                    } else {
                        break;
                    }
                }
                StateRead::Data => {
                    let value = src.get_u8();
                    self.crc = crc16(self.crc, value);
                    self.lenq -= 1;
                    self.buff.put_u8(value);
                    //Gdy w trakcie sprawdzania danych, licznik się skończył, sprawdzana jest poprawność wiadomości
                    if self.lenq <= 0 {
                        self.state_read = StateRead::Crc;
                        break;
                    }
                }
                StateRead::Crc => {
                    let value = src.get_u8();
                    if src.has_remaining() {
                        let bsrc = src.get_u8();
                        let bsrcv2: u16 = ((value as u16) << 8) | bsrc as u16;
                        if self.crc == bsrcv2 {
                            let result = Ok(Some(self.buff.clone().freeze()));
                            self.reset_read();
                            return result;
                        } else {
                            self.reset_read();
                            break;
                        }
                    }
                }
            }
        }
        Ok(None)
    }
}

#[derive(Debug, Eq, PartialEq)]
enum StateRead {
    Head,
    Length,
    Data,
    Crc,
}
