use std::io::{Cursor, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

pub fn decode_simple_archive(buf: &[u8]) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let mut cursor = Cursor::new(buf);

    let file_count = cursor.read_u32::<LittleEndian>().unwrap();

    for _ in 0..file_count {
        let file_offset = cursor.read_u32::<LittleEndian>().unwrap() as usize;
        let file_size = cursor.read_u32::<LittleEndian>().unwrap() as usize;

        let file_buf = buf[file_offset..file_offset + file_size].to_vec();
        out.push(file_buf);
    }

    out
}

pub fn encode_simple_archive(files: &[Vec<u8>]) -> Vec<u8> {
    let out = Vec::new();
    let mut cursor = Cursor::new(out);

    let file_count = files.len();
    cursor.write_u32::<LittleEndian>(file_count as u32).unwrap();

    let mut file_start_off = (4 + file_count * 8) as u32;

    for file in files {
        let buf_size = file.len();
        cursor.write_u32::<LittleEndian>(file_start_off).unwrap();
        cursor.write_u32::<LittleEndian>(buf_size as u32).unwrap();
        file_start_off += buf_size as u32;
    }

    for file in files {
        let _ = cursor.write(file).unwrap();
    }

    cursor.into_inner()
}

pub fn is_buf_simple_archive(buf: &[u8]) -> bool {
    let mut cursor = Cursor::new(buf);
    let file_count = cursor.read_u32::<LittleEndian>().unwrap();

    if file_count >= 9999 {
        return false;
    }

    let mut buf_size = 0;

    for _ in 0..file_count {
        let file_offset = cursor.read_u32::<LittleEndian>().unwrap() as usize;
        let file_size = cursor.read_u32::<LittleEndian>().unwrap() as usize;

        buf_size += file_size;

        if file_offset > buf.len() || file_offset + file_size > buf.len() {
            return false;
        }
    }

    let header_size = cursor.position() as usize;

    if buf_size + header_size != buf.len() {
        return false;
    }

    true
}

#[cfg(test)]
pub mod test {
    use std::{
        fs,
        io::{Cursor, Write},
    };

    use byteorder::{LittleEndian, WriteBytesExt};

    use crate::simple_archive::is_buf_simple_archive;

    use super::{decode_simple_archive, encode_simple_archive};

    #[test]
    fn simple_archive_scan() {
        let simple_archive = fs::read("./tests/data/em125_decrypt.pac").unwrap();
        assert!(is_buf_simple_archive(&simple_archive));
    }

    #[test]
    fn not_simple_archive_scan() {
        let not_simple_archive = fs::read("./tests/data/mhfdat_decrypt_decomp.bin").unwrap();
        assert!(!is_buf_simple_archive(&not_simple_archive));
    }

    #[test]
    fn simple_archive_roundtrip() {
        let simple_archive = fs::read("./tests/data/em125_decrypt.pac").unwrap();
        let files = decode_simple_archive(&simple_archive);

        let encoded = encode_simple_archive(&files);
        assert!(encoded == simple_archive, "the buffers are not equal");
    }
}
