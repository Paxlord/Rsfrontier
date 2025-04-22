use std::io::Cursor;

use byteorder::ReadBytesExt;

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
        //println!("{:x?}", &out);

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
                                (temp + 0x1A) as usize,
                                &mut out_index,
                            );
                        }
                    }
                }
            }
        }
    }
}

pub fn decode_jpk_huff_lz(buf: &[u8], out: &mut Vec<u8>, size: usize) {
    todo!()
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Cursor};

    use crate::jpk::decode::consume_bit;

    use super::decode_jpk_lz;

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
}
