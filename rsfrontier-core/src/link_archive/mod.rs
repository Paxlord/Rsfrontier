use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

#[derive(Debug)]
pub struct LinkHeader {
    pub magic: u16,
    pub entry_count: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct LinkEntry {
    pub file_type: u16,
    pub file_id: u16,
}

#[derive(Debug)]
pub struct LinkArchive {
    pub entries: Vec<LinkEntry>,
    pub files: Vec<Vec<u8>>,
}

pub fn decode_link_archive(files: Vec<Vec<u8>>) -> Option<LinkArchive> {
    if files.is_empty() {
        return None;
    }

    let header_file = &files[0];
    let mut cursor = Cursor::new(header_file);

    let _ = cursor.read_u16::<LittleEndian>().ok()?;
    let entry_count = cursor.read_u16::<LittleEndian>().ok()?;

    if entry_count as usize != files.len() - 1 {
        return None;
    }

    let mut entries = Vec::with_capacity(entry_count as usize);
    for _ in 0..entry_count {
        let file_type = cursor.read_u16::<LittleEndian>().ok()?;
        let file_id = cursor.read_u16::<LittleEndian>().ok()?;
        entries.push(LinkEntry { file_type, file_id });
    }

    Some(LinkArchive {
        entries,
        files: files.into_iter().skip(1).collect(),
    })
}

pub fn is_buf_link_archive(buf: &[u8]) -> bool {
    if buf.len() < 8 {
        return false;
    }
    let mut cursor = Cursor::new(buf);
    let file_count = match cursor.read_u32::<LittleEndian>() {
        Ok(count) => count,
        Err(_) => return false,
    };

    if file_count == 0 || file_count > 1000 {
        return false;
    }

    let first_file_offset = match cursor.read_u32::<LittleEndian>() {
        Ok(offset) => offset as usize,
        Err(_) => return false,
    };
    let first_file_size = match cursor.read_u32::<LittleEndian>() {
        Ok(size) => size as usize,
        Err(_) => return false,
    };

    if first_file_offset + first_file_size > buf.len() {
        return false;
    }

    let header_file_data = &buf[first_file_offset..first_file_offset + first_file_size];
    if header_file_data.len() < 4 {
        return false;
    }

    let mut header_cursor = Cursor::new(header_file_data);
    let _ = header_cursor.read_u16::<LittleEndian>().unwrap_or(0);
    let entry_count = header_cursor.read_u16::<LittleEndian>().unwrap_or(0);

    if entry_count as u32 == file_count - 1 {
        return true;
    }

    false
}
