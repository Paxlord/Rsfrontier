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

pub fn get_extension(magic: u32) -> Option<&'static str> {
    MAGIC_TO_EXTENSION
        .iter()
        .find(|(m, _)| *m == magic)
        .map(|&(_, ext)| ext)
}

pub fn is_file_fmod(buf: &[u8]) -> bool {
    let header = u32::from_le_bytes(
        buf.get(0..4)
            .unwrap_or_default()
            .try_into()
            .unwrap_or_default(),
    );
    if header != 1 {
        return false;
    }

    let file_len = u32::from_le_bytes(
        buf.get(8..12)
            .unwrap_or_default()
            .try_into()
            .unwrap_or_default(),
    );
    if file_len != buf.len() as u32 {
        return false;
    }

    true
}

pub fn is_file_fskl(buf: &[u8]) -> bool {
    let header = u32::from_le_bytes(
        buf.get(0..4)
            .unwrap_or_default()
            .try_into()
            .unwrap_or_default(),
    );
    if header != 0xC0000000 {
        return false;
    }

    let file_len = u32::from_le_bytes(
        buf.get(8..12)
            .unwrap_or_default()
            .try_into()
            .unwrap_or_default(),
    );
    if file_len != buf.len() as u32 {
        return false;
    }

    true
}

pub fn find_buf_extension(buf: &[u8]) -> &str {
    if let Some(ext) = get_extension(u32::from_le_bytes(
        buf.get(0..4)
            .unwrap_or_default()
            .try_into()
            .unwrap_or_default(),
    )) {
        return ext;
    }

    if is_file_fskl(buf) {
        return "fskl";
    }

    if is_file_fmod(buf) {
        return "fmod";
    }

    "bin"
}
