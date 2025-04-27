use std::collections::HashMap;

pub const MAGIC_TO_EXTENSION: &[(u32, &str)] = &[
    (542327876, "dds"),
    (0x000B0000, "ftxt"),
    (846751303, "gfx2"),
    (0x1A524B4A, "jkr"),
    (0x5367674F, "ogg"),
    (7302512, "pmo"),
    (0x474e5089, "png"),
    (1213027374, "tmh"),
];

pub fn get_exstension(magic: u32) -> Option<&'static str> {
    MAGIC_TO_EXTENSION
        .iter()
        .find(|(m, _)| *m == magic)
        .map(|&(_, ext)| ext)
}
