use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Seek, Write};
use std::{str, string};

pub fn is_buf_mha(buf: &[u8]) -> bool {
    let magic: u32 = u32::from_le_bytes(
        buf.get(0..4)
            .unwrap_or_default()
            .try_into()
            .unwrap_or_default(),
    );
    magic == 23160941
}

fn read_null_terminated_string(buf: &[u8], offset: usize) -> &str {
    let sub_slice = &buf[offset..];
    let next_null = sub_slice.iter().position(|&byte| byte == 0).unwrap();
    let string_slice = &sub_slice[..next_null];
    std::str::from_utf8(string_slice).unwrap()
}

pub fn decode_mha_archive(buf: &[u8]) -> Vec<(&str, Vec<u8>)> {
    let mut out = Vec::new();
    let mut cursor = Cursor::new(buf);

    let _ = cursor.read_u32::<LittleEndian>().unwrap();
    let metadata_addy = cursor.read_u32::<LittleEndian>().unwrap();
    let file_count = cursor.read_u32::<LittleEndian>().unwrap();
    let string_start = cursor.read_u32::<LittleEndian>().unwrap();
    let _ = cursor.read_u32::<LittleEndian>().unwrap();
    let base_id = cursor.read_u16::<LittleEndian>().unwrap();
    let capacity = cursor.read_u16::<LittleEndian>().unwrap();

    for i in 0..file_count {
        let meta_data_start = (metadata_addy + (i * 20)) as u64;
        cursor
            .seek(std::io::SeekFrom::Start(meta_data_start))
            .unwrap();

        let file_name_off = cursor.read_u32::<LittleEndian>().unwrap();
        let file_data_off = cursor.read_u32::<LittleEndian>().unwrap() as usize;
        let file_size = cursor.read_u32::<LittleEndian>().unwrap() as usize;

        let file_name = read_null_terminated_string(buf, (string_start + file_name_off) as usize);
        let file_data = buf[file_data_off..file_data_off + file_size].to_vec();

        out.push((file_name, file_data));
    }

    let metadata_filebuf = format!("{},{}", base_id, capacity).as_bytes().to_vec();
    out.push((".metadata", metadata_filebuf));

    out
}

pub fn encode_mha_archive(files: Vec<(String, Vec<u8>)>, base_id: u16, capacity: u16) -> Vec<u8> {
    let mut out = Vec::new();

    let header_size = 24_usize;
    let mut files_buf: Vec<u8> = Vec::new();
    let mut string_buf: Vec<u8> = Vec::new();
    let mut metadata_buf: Vec<u8> = Vec::new();

    let nb_files = files.len();

    for (_, file_buf) in &files {
        let _ = files_buf.write(file_buf);
    }

    for (file_name, _) in &files {
        let string_bytes = file_name.as_bytes();
        let _ = string_buf.write(string_bytes);
        string_buf.push(0);
    }

    let mut relative_str_off = 0_usize;
    let mut relative_file_data_off = 0_usize;
    for (file_count, (file_name, file_buf)) in files.into_iter().enumerate() {
        let _ = metadata_buf.write_u32::<LittleEndian>(relative_str_off as u32);
        relative_str_off += file_name.len() + 1;
        let _ =
            metadata_buf.write_u32::<LittleEndian>((header_size + relative_file_data_off) as u32);
        relative_file_data_off += file_buf.len();
        let _ = metadata_buf.write_u32::<LittleEndian>(file_buf.len() as u32);
        let _ = metadata_buf.write_u32::<LittleEndian>(file_buf.len() as u32);
        let _ = metadata_buf.write_u32::<LittleEndian>((base_id as u32) + (file_count as u32));
    }

    let _ = out.write_u32::<LittleEndian>(23160941);
    let _ =
        out.write_u32::<LittleEndian>((header_size + files_buf.len() + string_buf.len()) as u32);
    let _ = out.write_u32::<LittleEndian>(nb_files as u32);
    let _ = out.write_u32::<LittleEndian>((header_size + files_buf.len()) as u32);
    let _ = out.write_u32::<LittleEndian>(string_buf.len() as u32);
    let _ = out.write_u16::<LittleEndian>(base_id);
    let _ = out.write_u16::<LittleEndian>(capacity);
    let _ = out.write(&files_buf);
    let _ = out.write(&string_buf);
    let _ = out.write(&metadata_buf);

    out
}
