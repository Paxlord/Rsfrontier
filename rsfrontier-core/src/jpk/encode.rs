use std::collections::VecDeque;

pub fn find_longest_match(
    in_buffer: &[u8],
    current_buffer: &[u8],
    min_length: usize,
) -> Option<(usize, usize)> {
    let mut best_match: Option<(usize, usize)> = None;
    let pattern = &current_buffer[0..min_length];

    for (i, slice) in in_buffer.windows(min_length).enumerate() {
        if slice == pattern {
            //we found a match now we try to expand
            let in_buffer_rest = &in_buffer[i + min_length..];
            let cur_buf_rest = &current_buffer[min_length..];

            let extended_length = in_buffer_rest
                .iter()
                .zip(cur_buf_rest)
                .take_while(|(a, b)| *a == *b)
                .count();

            let total_length = min_length + extended_length;
            if best_match.is_none_or(|(_, len)| total_length > len) {
                best_match = Some((i, total_length));
            }
        }
    }

    best_match
}

fn set_bit_flag(out_buffer: &mut Vec<u8>, flag_idx: &mut usize, shift_idx: &mut i8, bit_val: u8) {
    *shift_idx -= 1;

    if *shift_idx < 0 {
        *shift_idx = 7;
        out_buffer.push(0);
        *flag_idx = out_buffer.len() - 1;
    }

    if bit_val == 1 {
        out_buffer[*flag_idx] |= 1 << *shift_idx;
    }
}

pub fn encode_jpk_lz(decoded_buffer: &[u8]) -> Vec<u8> {
    let mut out_buffer: Vec<u8> = Vec::new();
    let mut flag_idx: usize = 0;
    let mut shift_idx: i8 = 0;

    let mut history: VecDeque<u8> = VecDeque::with_capacity(8192);

    let mut i: usize = 0;
    while i < decoded_buffer.len() {
        //We look for a sequence
        let max_search_len = std::cmp::min(280, decoded_buffer.len() - i);
        let sequence_match = find_longest_match(
            history.make_contiguous(),
            &decoded_buffer[i..i + max_search_len],
            3,
        );

        let encodable_match = sequence_match.and_then(|(match_start_idx, length)| {
            let relative_offset = history.len() - match_start_idx - 1;

            if relative_offset >= i {
                return None;
            }

            if relative_offset < 8191 {
                Some((relative_offset, length))
            } else {
                None
            }
        });

        //If we didn't find a sequence
        if encodable_match.is_none() {
            set_bit_flag(&mut out_buffer, &mut flag_idx, &mut shift_idx, 0);
            out_buffer.push(decoded_buffer[i]);
            history.push_back(decoded_buffer[i]);
            if history.len() > 8192 {
                history.pop_front();
            }
            i += 1;
        } else {
            let (relative_offset, length) = encodable_match.unwrap();
            //We found a backref, setting the current bit to 1
            set_bit_flag(&mut out_buffer, &mut flag_idx, &mut shift_idx, 1);

            if relative_offset < 256 && length <= 6 {
                //short backref, set bit to 0
                set_bit_flag(&mut out_buffer, &mut flag_idx, &mut shift_idx, 0);
                let bit_length = (length - 3) as u8;
                set_bit_flag(
                    &mut out_buffer,
                    &mut flag_idx,
                    &mut shift_idx,
                    bit_length >> 1,
                );
                set_bit_flag(
                    &mut out_buffer,
                    &mut flag_idx,
                    &mut shift_idx,
                    bit_length & 1,
                );
                out_buffer.push(relative_offset as u8);
            } else if relative_offset < 8192 && length <= 9 {
                set_bit_flag(&mut out_buffer, &mut flag_idx, &mut shift_idx, 1);
                //long ref mode
                let high_byte = (((length - 2) & 0x7) << 5) | ((relative_offset >> 8) & 0x1F);
                let low_byte = relative_offset as u8;

                out_buffer.push(high_byte as u8);
                out_buffer.push(low_byte);
            } else if relative_offset < 8192 && length <= 280 {
                set_bit_flag(&mut out_buffer, &mut flag_idx, &mut shift_idx, 1);
                //long ref mode
                let high_byte = (relative_offset >> 8) & 0x1F;
                let low_byte = relative_offset as u8;

                out_buffer.push(high_byte as u8);
                out_buffer.push(low_byte);

                if length <= 25 {
                    //write the special bit as 0
                    set_bit_flag(&mut out_buffer, &mut flag_idx, &mut shift_idx, 0);
                    let encoded_length = (length - 10) as u8;
                    //Write the length with the next 4 bits
                    set_bit_flag(
                        &mut out_buffer,
                        &mut flag_idx,
                        &mut shift_idx,
                        encoded_length >> 3 & 1,
                    );
                    set_bit_flag(
                        &mut out_buffer,
                        &mut flag_idx,
                        &mut shift_idx,
                        encoded_length >> 2 & 1,
                    );
                    set_bit_flag(
                        &mut out_buffer,
                        &mut flag_idx,
                        &mut shift_idx,
                        encoded_length >> 1 & 1,
                    );
                    set_bit_flag(
                        &mut out_buffer,
                        &mut flag_idx,
                        &mut shift_idx,
                        encoded_length & 1,
                    );
                } else {
                    //special case bit to 1
                    set_bit_flag(&mut out_buffer, &mut flag_idx, &mut shift_idx, 1);
                    //push the length as a full byte
                    out_buffer.push((length - 26) as u8);
                }
            } else {
                set_bit_flag(&mut out_buffer, &mut flag_idx, &mut shift_idx, 0);
                out_buffer.push(decoded_buffer[i]);
                history.push_back(decoded_buffer[i]);
                if history.len() > 8192 {
                    history.pop_front();
                }
                i += 1;
                continue;
            }

            for j in 0..length {
                history.push_back(decoded_buffer[i + j]);
                if history.len() > 8192 {
                    history.pop_front();
                }
            }

            i += length;
        }
    }

    out_buffer
}
