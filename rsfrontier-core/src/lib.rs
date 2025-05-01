use std::path::PathBuf;

use ecd::{decrypt_ecd, encrypt_ecd, is_buf_ecd};
use jpk::{create_jpk, decode_jpk, is_buf_jpk};
use magic::{find_buf_extension, get_extension};
use simple_archive::{decode_simple_archive, is_buf_simple_archive};

pub mod ecd;
pub mod jpk;
pub mod magic;
pub mod simple_archive;

pub struct UnpackedFile {
    pub name: String,
    pub ext: String,
    pub buffer: Vec<u8>,
}

pub struct UnpackFolder {
    pub files: Vec<(String, UnpackedFile)>,
}

pub enum UnpackResult {
    File(UnpackedFile),
    Folder(Vec<(String, UnpackedFile)>),
}

pub enum PackType {
    Ecd,
    Jpk(u16),
}

fn recursive_unpack(
    current_buffer: &[u8],
    current_pathbuf: &PathBuf,
    out: &mut Vec<(PathBuf, Vec<u8>)>,
) {
    let mut processed_buffer = current_buffer.to_vec();

    loop {
        if is_buf_ecd(&processed_buffer) {
            println!("ECD detected, decrypting");
            processed_buffer = decrypt_ecd(&processed_buffer);
            continue;
        }

        if is_buf_jpk(&processed_buffer) {
            println!("JPK detected, decoding");
            processed_buffer = decode_jpk(&processed_buffer);
            continue;
        }

        if is_buf_simple_archive(&processed_buffer) {
            println!("Simple archive detected, unpacking");
            let in_buffers = decode_simple_archive(&processed_buffer);
            for (i, in_buf) in in_buffers.iter().enumerate() {
                let folder_name = format!("{:04}", i);
                let mut new_pathbuf = current_pathbuf.clone();
                new_pathbuf.push(folder_name);
                recursive_unpack(in_buf, &new_pathbuf, out);
            }
            return;
        }

        break;
    }

    let get_file_ext = find_buf_extension(&processed_buffer);
    let mut final_path_buf = current_pathbuf.clone();
    final_path_buf.set_extension(get_file_ext);

    out.push((final_path_buf, processed_buffer));
}

pub fn unpack_buffer(prefix_path: &str, buf: &[u8]) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out = Vec::new();
    let base_path = PathBuf::from(prefix_path);
    recursive_unpack(buf, &base_path, &mut out);
    out
}

pub fn pack_buffer(buf: &[u8], pack_type: PackType) -> Vec<u8> {
    let mut current_buffer = buf.to_vec();

    match pack_type {
        PackType::Ecd => {
            current_buffer = encrypt_ecd(&current_buffer);
        }
        PackType::Jpk(jpk_type) => {
            current_buffer = create_jpk(&current_buffer, jpk_type);
        }
    }

    current_buffer
}

#[cfg(test)]
mod tests {}
