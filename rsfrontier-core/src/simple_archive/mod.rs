use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};

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
    use std::fs;

    use crate::simple_archive::is_buf_simple_archive;

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
}
