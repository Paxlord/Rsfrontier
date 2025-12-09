use rsfrontier_core::{PackType, pack_buffer, unpack_buffer};
use wasm_bindgen::prelude::*;

#[cfg(feature = "console_error_panic_hook")]
pub use console_error_panic_hook::set_once as set_panic_hook;

#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn unpack(data: &[u8], prefix_path: &str, depth: Option<u8>) -> Result<JsValue, JsValue> {
    let unpacked = unpack_buffer(prefix_path, data, depth);

    let result = js_sys::Array::new();

    for (path, buf) in unpacked {
        let obj = js_sys::Object::new();

        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("path"),
            &JsValue::from_str(&path.to_string_lossy()),
        )?;

        let uint8_array = js_sys::Uint8Array::from(&buf[..]);
        js_sys::Reflect::set(&obj, &JsValue::from_str("data"), &uint8_array)?;

        result.push(&obj);
    }

    Ok(result.into())
}

#[wasm_bindgen]
pub fn pack_with_ecd(data: &[u8]) -> Vec<u8> {
    pack_buffer(data, PackType::Ecd)
}

#[wasm_bindgen]
pub fn pack_with_jpk(data: &[u8], jpk_type: u16) -> Result<Vec<u8>, JsValue> {
    if ![0, 2, 3, 4].contains(&jpk_type) {
        return Err(JsValue::from_str(
            "Invalid JPK type. Valid types are 0, 2, 3, 4",
        ));
    }
    Ok(pack_buffer(data, PackType::Jpk(jpk_type)))
}

#[wasm_bindgen]
pub fn is_ecd_encrypted(data: &[u8]) -> bool {
    rsfrontier_core::ecd::is_buf_ecd(data)
}

#[wasm_bindgen]
pub fn is_jpk_compressed(data: &[u8]) -> bool {
    rsfrontier_core::jpk::is_buf_jpk(data)
}

#[wasm_bindgen]
pub fn decrypt_ecd(data: &[u8]) -> Vec<u8> {
    rsfrontier_core::ecd::decrypt_ecd(data)
}

#[wasm_bindgen]
pub fn encrypt_ecd(data: &[u8]) -> Vec<u8> {
    rsfrontier_core::ecd::encrypt_ecd(data)
}

#[wasm_bindgen]
pub fn decompress_jpk(data: &[u8]) -> Vec<u8> {
    rsfrontier_core::jpk::decode_jpk(data)
}

#[wasm_bindgen]
pub fn compress_jpk(data: &[u8], jpk_type: u16) -> Result<Vec<u8>, JsValue> {
    if ![0, 2, 3, 4].contains(&jpk_type) {
        return Err(JsValue::from_str(
            "Invalid JPK type. Valid types are 0, 2, 3, 4",
        ));
    }
    Ok(rsfrontier_core::jpk::create_jpk(data, jpk_type))
}
