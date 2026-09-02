// pixlzr-core stub — real codec TODO
// Minimal valid crate that compiles for native and wasm32-unknown-unknown.
// Provides stub encode/decode APIs and wasm-bindgen exports when `wasm` feature is enabled.

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Native API — always available (no wasm-bindgen required).
pub fn encode(input: &[u8]) -> Vec<u8> {
    // stub: returns empty; real codec will implement density-aware compression
    let _ = input;
    Vec::new()
}

/// Native API — always available.
pub fn decode(input: &[u8]) -> Vec<u8> {
    // stub: returns empty; real codec will reconstruct image
    let _ = input;
    Vec::new()
}

/// Returns crate version (native).
pub fn version_str() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn encode_stub(input: &[u8]) -> Vec<u8> {
    // wasm export stub — mirrors native encode for now
    let _ = input;
    Vec::new()
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn decode_stub(input: &[u8]) -> Vec<u8> {
    // wasm export stub — mirrors native decode for now
    let _ = input;
    Vec::new()
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("pixlzr-core {name} v{}", env!("CARGO_PKG_VERSION"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_stub() {
        let data = b"hello";
        let enc = encode(data);
        let dec = decode(&enc);
        // stub returns empty vec — just verifies API compiles and runs
        assert_eq!(dec.len(), 0);
        assert_eq!(version_str(), "0.1.0");
    }

    #[test]
    fn encode_decode_accept_bytes() {
        assert_eq!(encode(b""), Vec::<u8>::new());
        assert_eq!(decode(b""), Vec::<u8>::new());
        assert_eq!(encode(b"abc"), Vec::<u8>::new());
    }
}
