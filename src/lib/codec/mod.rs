//! Codecs that allow encoding and decoding bytes with a simple interface.
//! Example:
//! ```
//! let mut x = lib::codec::simple_coder::SimpleCoder::new();
//! let mut buffor = BytesMut::with_capacity(1024);
//! loop {
//!    let mut string = String::new();
//!    let i = std::io::stdin().read_line(&mut string).unwrap();
//!
//!    let string = String::from(&string[..i-2]);
//!    buffor.extend_from_slice(string.as_bytes());
//!    println!("Before State: {:?}\n{:?}\n{:?}",x.state_read,x.buff, buffor);
//!    let res = x.decode( &mut buffor);
//!    println!("{:?}",res);
//!    println!("State: {:?}\n{:?}\n{:?}",x.state_read,x.buff, buffor);
//! }
//! ```
//! 

use bytes::{BytesMut};

pub mod simple_coder;
pub mod lenq_coder;
pub mod crc_len_coder;
pub mod slip_lng_coder;

pub trait Encoder {
    type Item;

    fn encode(&mut self, data: Self::Item, dst: &mut BytesMut) -> Result<(), ()>;
}

pub trait Decoder {
    type Item;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, ()>;
}


const POLY: u16 = 0x1021;

/// https://people.cs.umu.se/isak/snippets/crc-16.c
pub fn isak_crc16(data_p: &[u8]) -> u16 {
	let length = data_p.len();
	let mut data: u16;
	let mut crc: u16 = 0xffff;
	
	if length == 0 {
		return 0;
	}
	
	for ptr_i in 0..length {
		data = 0xff & data_p[ptr_i] as u16;
		for i in 0..8 {
			if (crc & 0x0001) ^ (data & 0x0001) > 0  {
				//crc = (crc >> 1) ^ POLY;
			} else {
				crc >>= 1
			}
			
			data >>= 1;
		}
	}
	crc = !crc;
	data = crc;
	crc = (crc << 8) | (data >> 8 & 0xff);
	return crc;
}

/// CRC-CCITT (0xFFFF) - Works properly. (Tested)
pub fn full_crc16(data_p: &[u8]) -> u16 {
	let length = data_p.len();
	let mut crc: u16 = 0xffff;
	
	if length == 0 {
		return 0;
	}
	
	for ptr_i in 0..length {
		crc = crc ^ (data_p[ptr_i] as u16) << 8;
		
		for _ in 0..8 {
			if crc & 0x8000 > 0 {
				crc = crc << 1 ^ POLY;
			} else {
				crc = crc << 1;
			}
		}
	}

	return crc as u16;
}

/// CRC-CCITT (base on input) - Works properly. (Tested)
pub fn crc16(crc: u16, byte: u8) -> u16 {
	let mut crcValue = crc;
	let mut newByte = byte;
	for i in 0..8 {
		if ((crcValue & 0x8000) >> 8) ^ ((newByte as u16) & 0x80) > 0{
			crcValue = (crcValue << 1)  ^ POLY;
		}else{
			crcValue = crcValue << 1;
		}
		newByte <<= 1;
	}
	
	return crcValue;
}

pub fn crc16_for_u16(crc: u16, byte: u16) -> u16 {
	let mut crcValue = crc;
	let left = byte.checked_shr(8).unwrap_or(0) as u8;
	let right = byte as u8;
	crcValue = crc16(crcValue, left);
	crcValue = crc16(crcValue, right);
	return crcValue;
}