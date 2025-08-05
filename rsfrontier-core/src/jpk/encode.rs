use std::{cmp::Reverse, collections::HashMap, hash::Hash};

use byteorder::{LittleEndian, WriteBytesExt};
use priority_queue::PriorityQueue;

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum HuffmanNode {
    Leaf {
        freq: usize,
        byte: u8,
    },
    Internal {
        freq: usize,
        left: Box<HuffmanNode>,
        right: Box<HuffmanNode>,
    },
}

pub struct BitWriter<'a> {
    buffer: &'a mut Vec<u8>,
    current_byte: u8,
    shift_idx: i8,
}

impl<'a> BitWriter<'a> {
    pub fn new(buffer: &'a mut Vec<u8>) -> Self {
        BitWriter {
            buffer,
            current_byte: 0,
            shift_idx: 8,
        }
    }

    pub fn write_bit(&mut self, bit: u8) {
        self.shift_idx -= 1;
        if self.shift_idx < 0 {
            self.buffer.push(self.current_byte);
            self.shift_idx = 7;
            self.current_byte = 0;
        }

        if bit == 1 {
            self.current_byte |= 1 << self.shift_idx;
        }
    }

    pub fn write_bits(&mut self, bits: u16, num: usize) {
        for i in (0..num).rev() {
            self.write_bit(((bits >> i) & 1) as u8);
        }
    }

    pub fn write_huff_byte(&mut self, byte: u8, code_map: &[(u16, u8); 256]) {
        let data = code_map[byte as usize];
        self.write_bits(data.0, data.1 as usize);
    }

    pub fn flush(&mut self) {
        if self.shift_idx != 8 {
            self.buffer.push(self.current_byte);
            self.current_byte = 0;
            self.shift_idx = 8;
        }
    }
}

pub fn find_longest_match(
    in_buffer: &[u8],
    hash_chain: &HashMap<u32, usize>,
    current_buffer: &[u8],
    min_length: usize,
) -> Option<(usize, usize)> {
    let mut best_match: Option<(usize, usize)> = None;
    if (current_buffer.len() < min_length) || (in_buffer.len() < min_length) {
        return None;
    }
    let pattern = &current_buffer[0..min_length];
    let pattern_hash = calculate_hash(pattern);

    if let Some(i) = hash_chain.get(&pattern_hash) {
        let in_buffer_rest = &in_buffer[*i + min_length..];
        let cur_buf_rest = &current_buffer[min_length..];

        let extended_length = in_buffer_rest
            .iter()
            .zip(cur_buf_rest)
            .take_while(|(a, b)| *a == *b)
            .count();

        let total_length = min_length + extended_length;
        //We found a good enough match returning this one
        if total_length >= 32 {
            best_match = Some((*i, total_length));
            return best_match;
        }

        if best_match.is_none_or(|(_, len)| total_length > len) {
            best_match = Some((*i, total_length));
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

pub fn calculate_hash(buf: &[u8]) -> u32 {
    if buf.len() < 3 {
        return 0;
    }

    u32::from_ne_bytes([buf[0], buf[1], buf[2], 0])
}

pub fn encode_jpk_lz(decoded_buffer: &[u8]) -> Vec<u8> {
    let mut out_buffer: Vec<u8> = Vec::new();
    let mut flag_idx: usize = 0;
    let mut shift_idx: i8 = 0;

    let mut pattern_dict: HashMap<u32, usize> = HashMap::new();

    let mut i: usize = 0;
    while i < decoded_buffer.len() {
        //We look for a sequence
        let max_search_len = std::cmp::min(280, decoded_buffer.len() - i);
        let sequence_match = find_longest_match(
            decoded_buffer,
            &pattern_dict,
            &decoded_buffer[i..i + max_search_len],
            3,
        );

        let encodable_match = sequence_match.and_then(|(match_start_idx, length)| {
            let relative_offset = i - match_start_idx - 1;

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
            if i >= 3 && i + 3 <= decoded_buffer.len() {
                let hash = calculate_hash(&decoded_buffer[i..i + 3]);
                pattern_dict.insert(hash, i);
            }
            if i >= 8192 {
                let old_pos = i - 8192;
                let old_hash = calculate_hash(&decoded_buffer[old_pos..old_pos + 3]);
                if let Some(index) = pattern_dict.get(&old_hash) {
                    if *index == old_pos {
                        pattern_dict.remove(&old_hash);
                    }
                }
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
            }

            for j in 0..length {
                if i + j >= 3 && i + j + 3 <= decoded_buffer.len() {
                    let hash = calculate_hash(&decoded_buffer[i + j..i + j + 3]);
                    pattern_dict.insert(hash, i + j);
                }
                if i + j >= 8192 {
                    let old_pos = i + j - 8192;
                    let old_hash = calculate_hash(&decoded_buffer[old_pos..old_pos + 3]);
                    if let Some(index) = pattern_dict.get(&old_hash) {
                        if *index == old_pos {
                            pattern_dict.remove(&old_hash);
                        }
                    }
                }
            }

            i += length;
        }
    }

    out_buffer
}

fn count_frequencies(data_bytes: &[u8]) -> [usize; 256] {
    let mut freq = [0usize; 256];

    for byte in data_bytes {
        let val = *byte as usize;
        freq[val] += 1;
    }

    freq
}

fn build_huffman_tree(frequencies: [usize; 256]) -> HuffmanNode {
    let mut pq = PriorityQueue::new();

    for (i, freq) in frequencies.iter().enumerate() {
        if *freq == 0 {
            continue;
        }

        pq.push(
            HuffmanNode::Leaf {
                freq: *freq,
                byte: i as u8,
            },
            Reverse(*freq),
        );
    }

    while pq.len() > 1 {
        let node_0 = pq.pop().unwrap();
        let node_1 = pq.pop().unwrap();

        let internal_freq = node_0.1.0 + node_1.1.0;

        let internal_node = HuffmanNode::Internal {
            freq: internal_freq,
            left: Box::new(node_0.0),
            right: Box::new(node_1.0),
        };

        pq.push(internal_node, Reverse(internal_freq));
    }

    let huffman_root = pq.pop().unwrap();
    huffman_root.0
}

fn generate_code_map(
    huffman_node: &HuffmanNode,
    current_code: u16,
    current_length: u8,
    code_map: &mut [(u16, u8); 256],
) {
    match huffman_node {
        HuffmanNode::Leaf { freq: _, byte } => {
            code_map[*byte as usize] = (current_code, current_length);
        }
        HuffmanNode::Internal {
            freq: _,
            left,
            right,
        } => {
            generate_code_map(left, current_code << 1, current_length + 1, code_map);
            generate_code_map(right, (current_code << 1) | 1, current_length + 1, code_map);
        }
    }
}

fn assign_indexes<'a>(
    huffman_node: &'a HuffmanNode,
    depth: usize,
    next_idx: &mut usize,
    out_map: &mut HashMap<&'a HuffmanNode, usize>,
) {
    match huffman_node {
        HuffmanNode::Leaf { freq: _, byte: _ } => {}
        HuffmanNode::Internal {
            freq: _,
            left,
            right,
        } => {
            if depth == 0 {
                out_map.insert(huffman_node, 510);
            } else {
                out_map.insert(huffman_node, *next_idx);
                *next_idx += 1;
            }

            assign_indexes(left, depth + 1, next_idx, out_map);
            assign_indexes(right, depth + 1, next_idx, out_map);
        }
    }
}

fn serialize_jpk_table(huffman_root: &HuffmanNode) -> Vec<u16> {
    let mut jpk_table: Vec<u16> = vec![0; 510];
    let mut index_map: HashMap<&HuffmanNode, usize> = HashMap::new();
    let mut next_index: usize = 0x100;
    assign_indexes(huffman_root, 0, &mut next_index, &mut index_map);

    for (node, &index) in &index_map {
        if let HuffmanNode::Internal {
            freq: _,
            left,
            right,
        } = node
        {
            let pair_start = (index - 0x100) * 2;

            match &**left {
                HuffmanNode::Leaf { freq: _, byte } => {
                    jpk_table[pair_start] = *byte as u16;
                }

                internal_left if index_map.contains_key(internal_left) => {
                    jpk_table[pair_start] = index_map[internal_left] as u16;
                }

                _ => {
                    eprintln!("error map")
                }
            }

            match &**right {
                HuffmanNode::Leaf { freq: _, byte } => {
                    jpk_table[pair_start + 1] = *byte as u16;
                }

                internal_right if index_map.contains_key(internal_right) => {
                    jpk_table[pair_start + 1] = index_map[internal_right] as u16;
                }

                _ => {
                    eprintln!("error map")
                }
            }
        }
    }

    jpk_table
}

pub fn encode_jpk_huff_lz(buffer: &[u8]) -> Vec<u8> {
    //First we encode the buffer with lz compression
    let lz_buffer = encode_jpk_lz(buffer);
    //Then we count the byte frequencies of our buffer
    let frequencies = count_frequencies(&lz_buffer);

    //From this we build the huffman tree and our code table
    let huffman_root = build_huffman_tree(frequencies);
    let mut code_table: [(u16, u8); 256] = [(0, 0); 256];
    generate_code_map(&huffman_root, 0, 0, &mut code_table);

    //We serialize the jpk table
    let jpk_table = serialize_jpk_table(&huffman_root);

    //Preparing the out buffer
    let mut out: Vec<u8> = Vec::new();

    //Writing table length
    out.write_u16::<LittleEndian>(510).unwrap();

    //Writing a little endian u16 for each entry in the table
    for table_entry in jpk_table {
        out.write_u16::<LittleEndian>(table_entry).unwrap();
    }

    //Creating our bit_writer
    let mut bit_writer = BitWriter::new(&mut out);

    //For each byte in our lz_buffer
    for byte in lz_buffer {
        //Writing it in the bit stream as an huffman byte
        bit_writer.write_huff_byte(byte, &code_table);
    }

    bit_writer.flush();

    out
}

pub fn encode_jpk_huff(buffer: &[u8]) -> Vec<u8> {
    let frequencies = count_frequencies(buffer);

    //From this we build the huffman tree and our code table
    let huffman_root = build_huffman_tree(frequencies);
    let mut code_table: [(u16, u8); 256] = [(0, 0); 256];
    generate_code_map(&huffman_root, 0, 0, &mut code_table);

    //We serialize the jpk table
    let jpk_table = serialize_jpk_table(&huffman_root);

    //Preparing the out buffer
    let mut out: Vec<u8> = Vec::new();

    //Writing table length
    out.write_u16::<LittleEndian>(510).unwrap();

    //Writing a little endian u16 for each entry in the table
    for table_entry in jpk_table {
        out.write_u16::<LittleEndian>(table_entry).unwrap();
    }

    //Creating our bit_writer
    let mut bit_writer = BitWriter::new(&mut out);

    //For each byte in our lz_buffer
    for byte in buffer {
        //Writing it in the bit stream as an huffman byte
        bit_writer.write_huff_byte(*byte, &code_table);
    }

    bit_writer.flush();

    out
}
