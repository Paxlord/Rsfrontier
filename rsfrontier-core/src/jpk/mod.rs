use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use core::fmt;
use decode::{decode_jpk_huff_lz, decode_jpk_huff_raw, decode_jpk_lz, decode_jpk_raw};
use std::io::{Cursor, Error};

mod decode;
mod encode;

#[derive(Debug)]
pub enum JpkError {
    InvalidType(u16),
    IoError(Error),
}

impl fmt::Display for JpkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JpkError::InvalidType(val) => write!(f, "Invalid JPK Type Found {}", val),
            JpkError::IoError(err) => write!(f, "I/O Error {}", err),
        }
    }
}

impl std::error::Error for JpkError {}

impl From<Error> for JpkError {
    fn from(value: Error) -> Self {
        JpkError::IoError(value)
    }
}

impl From<JpkError> for Error {
    fn from(value: JpkError) -> Self {
        match value {
            JpkError::InvalidType(val) => Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid JPK Type value: {}", val),
            ),
            JpkError::IoError(e) => e,
        }
    }
}

pub struct JpkHeader {
    magic: u32,
    version: u16,
    comp_type: JpkType,
    start_offset: usize,
    out_size: usize,
}

pub fn parse_header(data: &[u8]) -> Result<JpkHeader, Error> {
    let mut cursor = Cursor::new(data);
    let header = JpkHeader {
        magic: cursor.read_u32::<LittleEndian>()?,
        version: cursor.read_u16::<LittleEndian>()?,
        comp_type: JpkType::try_from(cursor.read_u16::<LittleEndian>()?)?,
        start_offset: cursor.read_u32::<LittleEndian>()? as usize,
        out_size: cursor.read_u32::<LittleEndian>()? as usize,
    };
    Ok(header)
}

pub fn decode_jpk(data: &[u8]) -> Vec<u8> {
    let header = parse_header(data).unwrap();
    let file_data_off = header.start_offset as usize;
    let file_data = &data[file_data_off..];
    let mut final_buf: Vec<u8> = Vec::with_capacity(header.out_size as usize);

    match header.comp_type {
        JpkType::Raw => decode_jpk_raw(file_data, &mut final_buf, header.out_size as usize),
        JpkType::HuffmanRw => {
            decode_jpk_huff_raw(file_data, &mut final_buf, header.out_size as usize)
        }
        JpkType::Lz => decode_jpk_lz(file_data, &mut final_buf, header.out_size as usize),
        JpkType::Huffman => decode_jpk_huff_lz(file_data, &mut final_buf, header.out_size as usize),
    };

    final_buf
}

pub fn create_jpk(data: &[u8], comp_type: u16, level: u32) -> Vec<u8> {
    todo!()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JpkType {
    Raw = 0,
    HuffmanRw = 2,
    Lz = 3,
    Huffman = 4,
}

impl TryFrom<u16> for JpkType {
    type Error = JpkError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(JpkType::Raw),
            2 => Ok(JpkType::HuffmanRw),
            3 => Ok(JpkType::Lz),
            4 => Ok(JpkType::Huffman),
            _ => Err(JpkError::InvalidType(value)),
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use crate::jpk::encode::encode_jpk_lz;

    use super::{
        decode::{decode_jpk_huff, decode_jpk_huff_lz, decode_jpk_huff_raw, decode_jpk_lz},
        encode::encode_jpk_hfi,
        parse_header,
    };

    #[test]
    fn roundtrip_lz() {
        let encoded_file = fs::read("./tests/data/quest_ex_0_comp.bin").unwrap();
        let decomp_file = fs::read("./tests/data/quest_ex_0_uncomp.bin").unwrap();
        let file_header = parse_header(&encoded_file).unwrap();

        let mut decomp_buf: Vec<u8> = Vec::new();
        decode_jpk_lz(
            &encoded_file[file_header.start_offset..],
            &mut decomp_buf,
            file_header.out_size,
        );

        let comp_buf = encode_jpk_lz(&decomp_file);
        dbg!(comp_buf.len());
        //fs::write("./tests/data/out/comp.bin", comp_buf).unwrap();
        let mut comp_decomp_buf: Vec<u8> = Vec::new();
        decode_jpk_lz(&comp_buf, &mut comp_decomp_buf, file_header.out_size);

        assert_eq!(decomp_buf, decomp_file);
    }

    #[test]
    fn roundtrip_hfi() {
        let decomp_file = fs::read("./tests/data/mhfdat_decrypt_decomp.bin").unwrap();
        let size = decomp_file.len();
        println!("encoding data...");
        let huff_comp = encode_jpk_hfi(&decomp_file);
        println!("decoding data...");
        let mut huff_decomp = Vec::new();
        decode_jpk_huff_lz(&huff_comp, &mut huff_decomp, size);
        assert!(decomp_file == huff_decomp, "the buffers are not equal");
    }
}
