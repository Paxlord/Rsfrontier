use ecd::{decrypt_ecd, encrypt_ecd, is_buf_ecd};
use jpk::{create_jpk, decode_jpk, is_buf_jpk};
use magic::find_buf_extension;

pub mod ecd;
pub mod jpk;
pub mod magic;

pub struct UnpackedFile {
    pub name: String,
    pub ext: String,
    pub buffer: Vec<u8>,
}

pub enum PackType {
    Ecd,
    Jpk(u16),
}

pub fn unpack_buffer(filename: &str, buf: &[u8]) -> UnpackedFile {
    let mut current_buffer = buf.to_vec();

    loop {
        if is_buf_ecd(&current_buffer) {
            current_buffer = decrypt_ecd(&current_buffer);
            continue;
        }

        if is_buf_jpk(&current_buffer) {
            current_buffer = decode_jpk(&current_buffer);
            continue;
        }

        break;
    }

    let ext = find_buf_extension(&current_buffer);

    UnpackedFile {
        name: filename.to_string(),
        ext: ext.to_string(),
        buffer: current_buffer,
    }
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
