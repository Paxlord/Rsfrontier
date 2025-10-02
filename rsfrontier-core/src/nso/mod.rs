use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

use serde::Serialize;

use crate::simple_archive::decode_simple_archive;

#[derive(Debug, Serialize)]
#[repr(C)]
pub struct NsoPlacementEntry {
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    rot_x: f32,
    rot_y: f32,
    rot_z: f32,
    param1: u32,
    param2: u32,
    param_block: [u32; 4],
    param3: u16,
    param4: u16,
    param5: u16,
    pub resource_id: u16,
}

pub fn decode_nso_pac(buf: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut out_files = Vec::new();
    let mut cursor = Cursor::new(buf);

    let obj_def_offset = cursor.read_u32::<LittleEndian>().unwrap() as usize;
    let _unknown_1 = cursor.read_u32::<LittleEndian>().unwrap();
    let _unknown_2 = cursor.read_u32::<LittleEndian>().unwrap();
    let _unknown_3 = cursor.read_u32::<LittleEndian>().unwrap();
    let csev_offset = cursor.read_u32::<LittleEndian>().unwrap() as usize;
    let csev_size = cursor.read_u32::<LittleEndian>().unwrap() as usize;
    let resource_count = cursor.read_u32::<LittleEndian>().unwrap();

    for _ in 0..resource_count {
        let resource_id = cursor.read_u32::<LittleEndian>().unwrap();
        let data_offset = cursor.read_u32::<LittleEndian>().unwrap() as usize;
        let data_size = cursor.read_u32::<LittleEndian>().unwrap() as usize;

        if data_size > 0 {
            let file_data = buf[data_offset..data_offset + data_size].to_vec();
            let file_name = format!("resource_{:04}.nsores", resource_id);
            out_files.push((file_name, file_data));
        }
    }

    if obj_def_offset > 0 {
        let mut placement_cursor = Cursor::new(&buf[obj_def_offset..]);
        let _unknown_header = placement_cursor.read_u32::<LittleEndian>().unwrap();
        let placement_count = placement_cursor.read_u32::<LittleEndian>().unwrap();
        placement_cursor.set_position(16);

        let mut placements = Vec::new();
        for _ in 0..placement_count {
            let entry_bytes = &buf[obj_def_offset + placement_cursor.position() as usize..];
            let entry: &NsoPlacementEntry =
                unsafe { &*(entry_bytes.as_ptr() as *const NsoPlacementEntry) };
            placements.push(entry);
            placement_cursor.set_position(placement_cursor.position() + 60);
        }

        let json_data = serde_json::to_string_pretty(&placements)
            .unwrap()
            .into_bytes();
        out_files.push(("placements.json".to_string(), json_data));
    }

    if csev_size > 0 {
        let csev_data = buf[csev_offset..csev_offset + csev_size].to_vec();
        out_files.push(("stage_events.sev".to_string(), csev_data));
    }

    out_files
}

pub fn decode_nsores(buf: &[u8]) -> Vec<(String, Vec<u8>)> {
    let files = decode_simple_archive(buf);

    if files.is_empty() {
        return Vec::new();
    }

    let manifest_buf = &files[0];
    if manifest_buf.len() < 4 {
        return files
            .into_iter()
            .enumerate()
            .map(|(i, f)| (format!("{:04}.bin", i), f))
            .collect();
    }

    let mut out_files = Vec::new();
    let mut manifest_cursor = Cursor::new(manifest_buf);

    let _unknown = manifest_cursor.read_u16::<LittleEndian>().unwrap();
    let part_count = manifest_cursor.read_u16::<LittleEndian>().unwrap() as usize;

    if part_count != files.len() - 1 {
        return files
            .into_iter()
            .enumerate()
            .map(|(i, f)| (format!("{:04}.bin", i), f))
            .collect();
    }

    let mut part_types = Vec::with_capacity(part_count);
    for _ in 0..part_count {
        part_types.push(manifest_cursor.read_u8().unwrap());
    }

    for i in 1..files.len() {
        let file_data = files[i].clone();
        let part_type = part_types[i - 1];

        let type_name = match part_type {
            1 => "model",
            2 => "skeleton",
            3 => "textures",
            4 => "dsd",
            5 => "wall_collision",
            6 => "ground_collision",
            7 => "event_data",
            8..=11 => "animation",
            13 => "neo",
            _ => "unknown",
        };

        let file_name = format!("part_{:02}_type_{:02}_{}", i - 1, part_type, type_name);
        out_files.push((file_name, file_data));
    }

    out_files
}

pub fn is_buf_nso_pac(buf: &[u8]) -> bool {
    if buf.len() < 28 {
        return false;
    }

    let mut cursor = Cursor::new(buf);
    let obj_def_offset = cursor.read_u32::<LittleEndian>().unwrap() as usize;
    let _ = cursor.read_u32::<LittleEndian>().unwrap();
    let _ = cursor.read_u32::<LittleEndian>().unwrap();
    let _ = cursor.read_u32::<LittleEndian>().unwrap();
    let csev_offset = cursor.read_u32::<LittleEndian>().unwrap() as usize;
    let csev_size = cursor.read_u32::<LittleEndian>().unwrap() as usize;
    let resource_count = cursor.read_u32::<LittleEndian>().unwrap();

    if resource_count > 1000 {
        return false;
    }
    if obj_def_offset > 0 && obj_def_offset >= buf.len() {
        return false;
    }
    if csev_offset > 0 && (csev_offset + csev_size > buf.len()) {
        return false;
    }

    if 28 + (resource_count * 12) as usize > buf.len() {
        return false;
    }

    for _ in 0..resource_count {
        let _ = cursor.read_u32::<LittleEndian>().unwrap();
        let data_offset = cursor.read_u32::<LittleEndian>().unwrap() as usize;
        let data_size = cursor.read_u32::<LittleEndian>().unwrap() as usize;

        if data_offset > buf.len() || data_size > buf.len() {
            return false;
        }

        if data_size > 0 && (data_offset + data_size > buf.len()) {
            return false;
        }
    }

    true
}
