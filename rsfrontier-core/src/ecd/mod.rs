use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

pub struct EcdHeader {
    magic: u32,
    index: u16,
    version: u16,
    file_size: u32,
    crc32: u32,
}

const RAND_BUFFER_ECD: [u8; 48] = [
    0x4A, 0x4B, 0x52, 0x2E, 0x00, 0x00, 0x00, 0x01, // "JKR."
    0x00, 0x01, 0x0D, 0xCD, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x0D, 0xCD, 0x00, 0x00, 0x00, 0x01,
    0x00, 0x01, 0x0D, 0xCD, 0x00, 0x00, 0x00, 0x01, 0x00, 0x19, 0x66, 0x0D, 0x00, 0x00, 0x00, 0x03,
    0x7D, 0x2B, 0x89, 0xDD, 0x00, 0x00, 0x00, 0x01,
];

fn load_uint_32(buffer: &[u8], offset: usize) -> u32 {
    let bytes = &buffer[offset..offset + 4];
    u32::from_be_bytes(bytes.try_into().expect("Slice with incorrect length"))
}

fn get_rnd_ecd(index: usize, seed: &mut u32) -> u32 {
    let multiplier_offset = 8 * index;
    let increment_offset = multiplier_offset + 4;

    let multiplier = load_uint_32(&RAND_BUFFER_ECD, multiplier_offset);
    let increment = load_uint_32(&RAND_BUFFER_ECD, increment_offset);

    *seed = seed.wrapping_mul(multiplier).wrapping_add(increment);
    *seed
}

pub fn decrypt_ecd(buffer: &[u8]) -> Vec<u8> {
    let mut cursor = Cursor::new(buffer);
    let mut out_vec = Vec::new();

    let header = EcdHeader {
        magic: cursor.read_u32::<LittleEndian>().unwrap(),
        index: cursor.read_u16::<LittleEndian>().unwrap(),
        version: cursor.read_u16::<LittleEndian>().unwrap(),
        file_size: cursor.read_u32::<LittleEndian>().unwrap(),
        crc32: cursor.read_u32::<LittleEndian>().unwrap(),
    };

    let mut rnd = header.crc32.rotate_right(16) | 1;
    let mut xorpad = get_rnd_ecd(header.index as usize, &mut rnd);
    let mut r8 = xorpad as u8;

    for _ in 0..header.file_size as usize {
        xorpad = get_rnd_ecd(header.index as usize, &mut rnd);

        let data: u8 = cursor.read_u8().unwrap();
        let mut r11 = (data ^ r8) as u32;
        let mut r12 = (r11 >> 4) & 0xFF;

        for _ in 0..8 {
            let r10 = xorpad ^ r11;
            r11 = r12;
            r12 ^= r10;
            r12 &= 0xFF;
            xorpad >>= 4;
        }

        r8 = ((r12 & 0xF) | ((r11 & 0xF) << 4)) as u8;
        out_vec.push(r8);
    }

    out_vec
}

pub fn encrypt_ecd(buffer: &[u8]) -> Vec<u8> {
    let mut out_buf: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(buffer);

    let file_size = buffer.len();
    let crc32 = crc32fast::hash(buffer);
    let index: u16 = 4;

    out_buf.write_u32::<LittleEndian>(442786661).unwrap(); //magic
    out_buf.write_u16::<LittleEndian>(4).unwrap(); //index;
    out_buf.write_u16::<LittleEndian>(31739).unwrap(); //Version/Unk
    out_buf.write_u32::<LittleEndian>(file_size as u32).unwrap();
    out_buf.write_u32::<LittleEndian>(crc32).unwrap();

    let mut rnd = crc32.rotate_right(16) | 1;
    let mut xorpad = get_rnd_ecd(index as usize, &mut rnd);
    let mut r8 = xorpad as u8;

    for _ in 0..file_size {
        xorpad = get_rnd_ecd(index as usize, &mut rnd);
        let data = cursor.read_u8().unwrap();

        let mut r11 = 0;
        let mut r12 = 0;

        for _ in 0..8 {
            let r10 = xorpad ^ r11;
            r11 = r12;
            r12 ^= r10;
            r12 &= 0xFF;
            xorpad >>= 4;
        }

        let mut dig2 = data as u32;
        let mut dig1 = (dig2 >> 4) & 0xFF;
        dig1 ^= r11;
        dig2 ^= r12;
        dig1 ^= dig2;

        let mut rr = ((dig2 & 0xF) | ((dig1 & 0xF) << 4)) as u8;
        rr ^= r8;
        out_buf.push(rr);
        r8 = data;
    }

    out_buf
}

pub fn is_buf_ecd(buffer: &[u8]) -> bool {
    let magic = u32::from_le_bytes(buffer[0..4].try_into().unwrap());
    magic == 442786661
}

#[cfg(test)]
mod test {
    use std::fs;

    use crate::ecd::decrypt_ecd;

    use super::encrypt_ecd;

    #[test]
    fn decrypting_ecd() {
        let encrypted_buf = fs::read("./tests/data/mhfdat.bin").unwrap();
        let decrypted_buf = fs::read("./tests/data/mhfdat_decrypted_only.bin").unwrap();

        let custom_decrypted_buf = decrypt_ecd(&encrypted_buf);

        assert_eq!(custom_decrypted_buf, decrypted_buf);
    }

    #[test]
    fn encrypting_ecd() {
        let encrypted_buf = fs::read("./tests/data/mhfdat.bin").unwrap();
        let decrypted_buf = fs::read("./tests/data/mhfdat_decrypted_only.bin").unwrap();

        let custom_encrypted_buf = encrypt_ecd(&decrypted_buf);

        assert!(
            custom_encrypted_buf == encrypted_buf,
            "The buffers don't match"
        );
    }
}
