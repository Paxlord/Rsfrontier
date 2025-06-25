use std::{
    fs,
    path::{Path, PathBuf},
};

use ecd::{decrypt_ecd, encrypt_ecd, is_buf_ecd};
use jpk::{create_jpk, decode_jpk, is_buf_jpk, should_jpk_compress};
use magic::find_buf_extension;
use mha::{decode_mha_archive, encode_mha_archive, is_buf_mha};
use queues::{IsQueue, Queue};
use simple_archive::{decode_simple_archive, encode_simple_archive, is_buf_simple_archive};

pub mod ecd;
pub mod jpk;
pub mod magic;
pub mod mha;
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

pub enum FolderPackType {
    Simple,
    MHA(u16, u16),
}

fn recursive_unpack(
    current_buffer: &[u8],
    current_pathbuf: PathBuf,
    out: &mut Vec<(PathBuf, Vec<u8>)>,
) {
    let mut processed_buffer = current_buffer.to_vec();

    loop {
        if is_buf_ecd(&processed_buffer) {
            processed_buffer = decrypt_ecd(&processed_buffer);
            continue;
        }

        if is_buf_jpk(&processed_buffer) {
            processed_buffer = decode_jpk(&processed_buffer);
            continue;
        }

        if is_buf_simple_archive(&processed_buffer) {
            let in_buffers = decode_simple_archive(&processed_buffer);
            for (i, in_buf) in in_buffers.iter().enumerate() {
                let folder_name = format!("{:04}", i);
                let mut new_pathbuf = current_pathbuf.clone();
                new_pathbuf.push(folder_name);
                recursive_unpack(in_buf, new_pathbuf, out);
            }
            return;
        }

        if is_buf_mha(&processed_buffer) {
            println!("Found mha archive");
            let in_buffers = decode_mha_archive(&processed_buffer);
            for (name, file_buf) in in_buffers {
                dbg!(name);
                let mut new_pathbuf = current_pathbuf.clone();
                new_pathbuf.push(name);
                new_pathbuf.set_extension("");
                recursive_unpack(&file_buf, new_pathbuf, out);
            }
            return;
        }

        break;
    }

    let get_file_ext = find_buf_extension(&processed_buffer);
    let mut final_path_buf = current_pathbuf.clone();

    if let Some(file_name) = final_path_buf.file_name() {
        if file_name.to_string_lossy().starts_with(".") {
            final_path_buf.set_extension("");
        } else {
            final_path_buf.set_extension(get_file_ext);
        }
    }

    out.push((final_path_buf, processed_buffer));
}

pub fn recursive_pack(current_path: &Path) -> Queue<(PathBuf, Vec<u8>)> {
    let mut folder_queue: Queue<(PathBuf, Vec<u8>)> = Queue::new();
    if current_path.is_dir() {
        for entry in fs::read_dir(current_path).unwrap() {
            let entry = entry.unwrap();
            let entry_path = entry.path();

            //Skips metadata files
            if let Some(entry_name) = entry_path.file_name() {
                if entry_name.to_string_lossy().starts_with(".") {
                    continue;
                }
            }

            if entry_path.is_dir() {
                let mut sub_folder_queue = recursive_pack(&entry_path);
                let mut simple_archive_vec = Vec::new();
                while sub_folder_queue.size() > 0 {
                    let file = sub_folder_queue.remove().unwrap();
                    simple_archive_vec.push(file.1);
                }
                let simple_archive_buf = encode_simple_archive(&simple_archive_vec);
                let mut file_pathbuf = entry_path.clone();
                let file_ext = find_buf_extension(&simple_archive_buf);
                file_pathbuf.set_extension(file_ext);
                let _ = folder_queue.add((file_pathbuf, simple_archive_buf));
            } else {
                let file_buf = fs::read(&entry_path).unwrap();
                let mut file_pathbuf = entry_path.clone();
                if should_jpk_compress(&entry_path, &file_buf) {
                    let comp_buf = create_jpk(&file_buf, 4);
                    let file_ext = find_buf_extension(&comp_buf);
                    file_pathbuf.set_extension(file_ext);
                    let _ = folder_queue.add((file_pathbuf, comp_buf)).unwrap();
                } else {
                    let file_ext = find_buf_extension(&file_buf);
                    file_pathbuf.set_extension(file_ext);
                    let _ = folder_queue.add((file_pathbuf, file_buf)).unwrap();
                }
            }
        }
    }
    folder_queue
}

pub fn unpack_buffer(prefix_path: &str, buf: &[u8]) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out = Vec::new();
    let base_path = PathBuf::from(prefix_path);
    recursive_unpack(buf, base_path, &mut out);
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

pub fn pack_folder(folder_path: &Path, pack_type: FolderPackType) -> Vec<u8> {
    let mut folder_queue = recursive_pack(folder_path);

    match pack_type {
        FolderPackType::Simple => {
            let mut simple_archive_vec = Vec::new();
            while folder_queue.size() > 0 {
                let file = folder_queue.remove().unwrap();
                simple_archive_vec.push(file.1);
            }
            encode_simple_archive(&simple_archive_vec)
        }
        FolderPackType::MHA(base_file_id, capacity) => {
            let mut mha_vec = Vec::new();
            while folder_queue.size() > 0 {
                let file = folder_queue.remove().unwrap();
                if let Some(file_name) = file.0.file_name() {
                    mha_vec.push((file_name.to_string_lossy().to_string(), file.1));
                }
            }
            encode_mha_archive(mha_vec, base_file_id, capacity)
        }
    }
}
