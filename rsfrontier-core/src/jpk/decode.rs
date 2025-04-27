use core::panic;
use std::{
    io::{self, Cursor, Seek},
    ptr::read,
};

use byteorder::{LittleEndian, ReadBytesExt};

struct HuffmanState {
    flag: u8,
    flag_shift: i8,
    table_length: u16,
    table_offset: usize,
    data_offset: usize,
}

pub fn decode_jpk_raw(buf: &[u8], out: &mut Vec<u8>, size: usize) {
    todo!()
}

pub fn decode_jpk_huff_raw(buf: &[u8], out: &mut Vec<u8>, size: usize) {
    todo!()
}

fn consume_bit(cursor: &mut Cursor<&[u8]>, shift_idx: &mut i8, flag: &mut u8) -> u8 {
    *shift_idx -= 1;
    //If shift index less than 0 than we consumed all the bytes and we read a new flag
    if *shift_idx < 0 {
        *shift_idx = 7;
        *flag = cursor.read_u8().unwrap();
    }

    (*flag >> *shift_idx) & 1
}

fn backref_to_out(out: &mut Vec<u8>, offset: usize, length: usize, out_index: &mut usize) {
    for _ in 0..length {
        let byte = out[*out_index - offset - 1];
        out.push(byte);
        *out_index += 1;
    }
}

pub fn decode_jpk_lz(buf: &[u8], out: &mut Vec<u8>, size: usize) {
    let mut cursor = Cursor::new(buf);
    let mut flag: u8 = 0;
    let mut shift_idx: i8 = -1;
    let mut out_index: usize = 0;

    while out.len() < size {
        //Get a bit from the control flag
        let bit = consume_bit(&mut cursor, &mut shift_idx, &mut flag);

        //bit is 0 then we copy the byte to out_buf
        if bit == 0 {
            let byte = cursor.read_u8().unwrap();
            out.push(byte);
            out_index += 1;
        }

        //We have a backref
        if bit == 1 {
            let backref_type = consume_bit(&mut cursor, &mut shift_idx, &mut flag);
            if backref_type == 0 {
                //we're in a short ref
                let length = (consume_bit(&mut cursor, &mut shift_idx, &mut flag) << 1)
                    | consume_bit(&mut cursor, &mut shift_idx, &mut flag);
                let offset = cursor.read_u8().unwrap();
                backref_to_out(out, offset as usize, (length + 3) as usize, &mut out_index);
            }
            if backref_type == 1 {
                //We're in a long ref
                let high_byte = cursor.read_u8().unwrap();
                let low_byte = cursor.read_u8().unwrap();
                let length = (high_byte & 0xE0) >> 5;
                let offset: u16 = (((high_byte & 0x1F) as u16) << 8) | low_byte as u16;

                //case 1 length is not equal to 0
                if length != 0 {
                    backref_to_out(out, offset as usize, (length + 2) as usize, &mut out_index);
                }
                //Special cases
                if length == 0 {
                    let special_case_bit = consume_bit(&mut cursor, &mut shift_idx, &mut flag);
                    if special_case_bit == 0 {
                        let length = consume_bit(&mut cursor, &mut shift_idx, &mut flag) << 3
                            | consume_bit(&mut cursor, &mut shift_idx, &mut flag) << 2
                            | consume_bit(&mut cursor, &mut shift_idx, &mut flag) << 1
                            | consume_bit(&mut cursor, &mut shift_idx, &mut flag);
                        backref_to_out(
                            out,
                            offset as usize,
                            (length + 2 + 8) as usize,
                            &mut out_index,
                        );
                    }
                    if special_case_bit == 1 {
                        let temp = cursor.read_u8().unwrap();
                        if temp == 0xFF {
                            for _ in 0..offset + 0x1B {
                                out.push(cursor.read_u8().unwrap());
                                out_index += 1;
                            }
                        }
                        if temp != 0xFF {
                            backref_to_out(
                                out,
                                offset as usize,
                                temp as usize + 0x1A,
                                &mut out_index,
                            );
                        }
                    }
                }
            }
        }
    }
}

fn read_huff_byte(cursor: &mut Cursor<&[u8]>, hf: &mut HuffmanState) -> io::Result<u8> {
    let mut data = hf.table_length;

    while data >= 0x100 {
        hf.flag_shift -= 1;
        if hf.flag_shift < 0 {
            hf.flag_shift = 7;
            cursor.seek(std::io::SeekFrom::Start(hf.data_offset as u64))?;
            hf.data_offset += 1;
            hf.flag = cursor.read_u8()?;
        }
        let bit = (hf.flag >> hf.flag_shift) & 1;
        let offset = ((data as u64) * 2 - 0x200 + bit as u64) * 2 + hf.table_offset as u64;
        cursor.seek(std::io::SeekFrom::Start(offset))?;
        data = cursor.read_u16::<LittleEndian>()?;
    }

    Ok(data as u8)
}

fn consume_huff_bit(
    cursor: &mut Cursor<&[u8]>,
    shift_idx: &mut i8,
    flag: &mut u8,
    hf: &mut HuffmanState,
) -> u8 {
    *shift_idx -= 1;
    //If shift index less than 0 than we consumed all the bytes and we read a new flag
    if *shift_idx < 0 {
        *shift_idx = 7;
        *flag = read_huff_byte(cursor, hf).unwrap();
    }

    (*flag >> *shift_idx) & 1
}

fn initialize_huffman_state(
    table_length: u16,
    table_offset: usize,
    data_offset: usize,
) -> HuffmanState {
    HuffmanState {
        flag: 0,
        flag_shift: -1,
        table_length,
        table_offset,
        data_offset,
    }
}

pub fn decode_jpk_huff_lz(buf: &[u8], out: &mut Vec<u8>, size: usize) {
    let mut cursor = Cursor::new(buf);
    let mut flag: u8 = 0;
    let mut shift_idx: i8 = -1;
    let mut out_index: usize = 0;

    let table_length = cursor.read_u16::<LittleEndian>().unwrap();
    let table_offset = cursor.position() as usize;
    let data_offset = table_offset + table_length as usize * 4 - 0x3fc;
    let mut hf_state = initialize_huffman_state(table_length, table_offset, data_offset);

    while out.len() < size {
        //Get a bit from the control flag
        let bit = consume_huff_bit(&mut cursor, &mut shift_idx, &mut flag, &mut hf_state);
        //bit is 0 then we copy the byte to out_buf
        if bit == 0 {
            let byte = read_huff_byte(&mut cursor, &mut hf_state).unwrap();
            out.push(byte);
            out_index += 1;
        }

        //We have a backref
        if bit == 1 {
            let backref_type =
                consume_huff_bit(&mut cursor, &mut shift_idx, &mut flag, &mut hf_state);
            if backref_type == 0 {
                //we're in a short ref
                let length =
                    (consume_huff_bit(&mut cursor, &mut shift_idx, &mut flag, &mut hf_state) << 1)
                        | consume_huff_bit(&mut cursor, &mut shift_idx, &mut flag, &mut hf_state);
                let offset = read_huff_byte(&mut cursor, &mut hf_state).unwrap();
                backref_to_out(out, offset as usize, (length + 3) as usize, &mut out_index);
            }
            if backref_type == 1 {
                //We're in a long ref
                let high_byte = read_huff_byte(&mut cursor, &mut hf_state).unwrap();
                let low_byte = read_huff_byte(&mut cursor, &mut hf_state).unwrap();
                let length = (high_byte & 0xE0) >> 5;
                let offset: u16 = (((high_byte & 0x1F) as u16) << 8) | low_byte as u16;

                //case 1 length is not equal to 0
                if length != 0 {
                    backref_to_out(out, offset as usize, (length + 2) as usize, &mut out_index);
                }
                //Special cases
                if length == 0 {
                    let special_case_bit =
                        consume_huff_bit(&mut cursor, &mut shift_idx, &mut flag, &mut hf_state);
                    if special_case_bit == 0 {
                        let length =
                            consume_huff_bit(&mut cursor, &mut shift_idx, &mut flag, &mut hf_state)
                                << 3
                                | consume_huff_bit(
                                    &mut cursor,
                                    &mut shift_idx,
                                    &mut flag,
                                    &mut hf_state,
                                ) << 2
                                | consume_huff_bit(
                                    &mut cursor,
                                    &mut shift_idx,
                                    &mut flag,
                                    &mut hf_state,
                                ) << 1
                                | consume_huff_bit(
                                    &mut cursor,
                                    &mut shift_idx,
                                    &mut flag,
                                    &mut hf_state,
                                );
                        backref_to_out(
                            out,
                            offset as usize,
                            (length + 2 + 8) as usize,
                            &mut out_index,
                        );
                    }
                    if special_case_bit == 1 {
                        let temp = read_huff_byte(&mut cursor, &mut hf_state).unwrap();
                        if temp == 0xFF {
                            for _ in 0..offset + 0x1B {
                                out.push(read_huff_byte(&mut cursor, &mut hf_state).unwrap());
                                out_index += 1;
                            }
                        }
                        if temp != 0xFF {
                            backref_to_out(
                                out,
                                offset as usize,
                                temp as usize + 0x1A,
                                &mut out_index,
                            );
                        }
                    }
                }
            }
        }
    }
}

pub fn decode_jpk_huff(buf: &[u8]) -> Vec<u8> {
    let mut cursor = Cursor::new(buf);

    let table_length = cursor.read_u16::<LittleEndian>().unwrap();
    let table_offset = cursor.position() as usize;
    let data_offset = table_offset + table_length as usize * 4 - 0x3fc;

    let mut hf_state = initialize_huffman_state(table_length, table_offset, data_offset);

    let mut out_vec: Vec<u8> = Vec::new();

    while let Ok(byte) = read_huff_byte(&mut cursor, &mut hf_state) {
        out_vec.push(byte);
    }

    out_vec
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Cursor};

    use crate::jpk::decode::consume_bit;

    use super::{decode_jpk_huff, decode_jpk_huff_lz, decode_jpk_lz};

    #[test]
    fn consume_bit_test() {
        let mut cursor: Cursor<&[u8]> = Cursor::new(&[35, 0xCB, 0x12, 0x16]);
        let mut flag: u8 = 0b00100011;
        let mut shift_idx: i8 = -1;

        assert_eq!(consume_bit(&mut cursor, &mut shift_idx, &mut flag), 0);
        assert_eq!(shift_idx, 7);
        assert_eq!(consume_bit(&mut cursor, &mut shift_idx, &mut flag), 0);
        assert_eq!(consume_bit(&mut cursor, &mut shift_idx, &mut flag), 1);
        assert_eq!(consume_bit(&mut cursor, &mut shift_idx, &mut flag), 0);
        assert_eq!(consume_bit(&mut cursor, &mut shift_idx, &mut flag), 0);
        assert_eq!(consume_bit(&mut cursor, &mut shift_idx, &mut flag), 0);
        assert_eq!(consume_bit(&mut cursor, &mut shift_idx, &mut flag), 1);
        assert_eq!(consume_bit(&mut cursor, &mut shift_idx, &mut flag), 1);
        assert_eq!(consume_bit(&mut cursor, &mut shift_idx, &mut flag), 1);
        assert_eq!(shift_idx, 7);
        assert_eq!(flag, 0xCB);
    }

    #[test]
    fn decode_jpk_lz_test() {
        let decompressed_data = fs::read("./tests/data/quest_ex_0_uncomp.bin").unwrap();
        let compressed_data = fs::read("./tests/data/quest_ex_0_comp.bin").unwrap();
        let expected_size = 14640;
        let start_off = 16;

        let mut out: Vec<u8> = Vec::new();
        decode_jpk_lz(&compressed_data[start_off..], &mut out, expected_size);

        assert_eq!(out, decompressed_data);
    }

    #[test]
    fn decode_jpk_huff_lz_test() {
        let decompressed_data = fs::read("./tests/data/mhfdat_decrypt_decomp.bin").unwrap();
        let compressed_data = fs::read("./tests/data/mhfdat_decrypted_only.bin").unwrap();

        let mut out: Vec<u8> = Vec::new();
        decode_jpk_huff_lz(&compressed_data[16..], &mut out, 28722096);

        assert!(out == decompressed_data, "The buffers are not equal");
    }

    #[test]
    fn huff_byte_reading() {
        let compressed_data = fs::read("./tests/data/mhfdat_decrypted_only.bin").unwrap();

        let mut output = String::new();
        for (i, chunk) in compressed_data[16..0x410].chunks(2).enumerate() {
            let address = i * 2;
            let value = ((chunk[1] as u16) << 8) | (chunk[0] as u16);

            let bytes_str = chunk
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<String>>()
                .join(" ");

            // Output format: "address: ushort [ bytes ]"
            output.push_str(&format!("{:x}: {} [ {} ]\n", address, value, bytes_str));
        }

        fs::write("./tests/data/out/huff_file.txt", output.as_bytes());
    }

    #[test]
    fn huff_rw_decomp() {
        let decompressed_data = fs::read("./tests/data/mhfdat_decrypt_decomp.bin").unwrap();
        let compressed_data = fs::read("./tests/data/mhfdat_decrypted_only.bin").unwrap();
        let out_vec = decode_jpk_huff(&compressed_data[16..]);
        let mut decompressed_lz = Vec::new();
        decode_jpk_lz(&out_vec, &mut decompressed_lz, 28722096);

        assert!(
            decompressed_data == decompressed_lz,
            "the buffer are not equal"
        );
    }
}
