//! PIXLZR — density-aware 8x8 block image codec (format v1).
//!
//! Pipeline:
//!
//! 1. RGBA8 -> reversible YCoCg (integer, exact).
//! 2. Per 8x8 block, orthonormal DCT-II on Y, Cb and Cr.
//! 3. Quantize with a step derived from local luma gradient density and the
//!    `quality` knob (dense regions keep finer steps).
//! 4. Zigzag-scan i16 coefficients, zlib-compress the stream.
//!
//! The same code path powers the native CLI and the `wasm`-feature exports, so
//! there is no separate "demo" codec. See `docs/format.md` for the byte layout.
//!
//! Format v1 is lossy, single-threaded and drops alpha (restored as opaque).

mod color;
mod dct;
mod decode;
mod encode;
mod format;

pub use decode::decode_rgba;
pub use encode::encode_rgba;
pub use format::{
    blocks_along, header_len, inspect, inspect_header, raw_coeff_len, HeaderInfo, Image,
    BLOCK_SIZE, HEADER_FIXED_LEN, MAGIC, MAX_DIMENSION, MAX_PIXELS, VERSION,
};

use std::fmt;

/// Errors returned by the codec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PixlzrError {
    /// The first six bytes are not `PIXLZR`.
    BadMagic,
    /// The format version is not supported by this build.
    UnsupportedVersion(u8),
    /// Reserved header flags are non-zero.
    UnsupportedFlags(u16),
    /// The blob (or its coefficient stream) ended early.
    Truncated { expected: usize, actual: usize },
    /// Dimensions are zero, exceed the limits, or disagree with the block grid.
    BadDimensions { width: u32, height: u32 },
    /// Block size is not 8 (format v1 only defines 8).
    BadBlockSize(u8),
    /// The zlib payload could not be inflated.
    Zlib(String),
    /// The RGBA buffer is shorter than `width * height * 4`.
    InvalidImageData { expected: usize, actual: usize },
}

impl fmt::Display for PixlzrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadMagic => write!(f, "not a PIXLZR file (bad magic)"),
            Self::UnsupportedVersion(v) => {
                write!(
                    f,
                    "unsupported PIXLZR format version {v} (expected {VERSION})"
                )
            }
            Self::UnsupportedFlags(flags) => {
                write!(f, "unsupported PIXLZR header flags 0x{flags:04x}")
            }
            Self::Truncated { expected, actual } => {
                write!(
                    f,
                    "truncated input: expected {expected} bytes, got {actual}"
                )
            }
            Self::BadDimensions { width, height } => {
                write!(f, "invalid dimensions {width}x{height}")
            }
            Self::BadBlockSize(size) => {
                write!(f, "unsupported block size {size} (expected {BLOCK_SIZE})")
            }
            Self::Zlib(reason) => write!(f, "zlib payload error: {reason}"),
            Self::InvalidImageData { expected, actual } => {
                write!(
                    f,
                    "RGBA buffer too short: expected {expected} bytes, got {actual}"
                )
            }
        }
    }
}

impl std::error::Error for PixlzrError {}

/// Crate version.
pub fn version_str() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(feature = "wasm")]
mod wasm_exports {
    use super::{decode_rgba as decode_impl, encode_rgba as encode_impl, inspect, version_str};
    use crate::PixlzrError;
    use wasm_bindgen::prelude::*;

    fn to_js(err: PixlzrError) -> JsValue {
        JsValue::from_str(&err.to_string())
    }

    /// Encode RGBA8 pixels (`w * h * 4` bytes) to a PIXLZR blob.
    #[wasm_bindgen(js_name = encode_rgba)]
    pub fn encode_rgba_wasm(
        width: u32,
        height: u32,
        rgba: &[u8],
        quality: u8,
    ) -> Result<Vec<u8>, JsValue> {
        encode_impl(width, height, rgba, quality).map_err(to_js)
    }

    /// Decode a PIXLZR blob to RGBA8 pixels (`w * h * 4` bytes).
    #[wasm_bindgen(js_name = decode_rgba)]
    pub fn decode_rgba_wasm(blob: &[u8]) -> Result<Vec<u8>, JsValue> {
        decode_impl(blob).map(|(_image, rgba)| rgba).map_err(to_js)
    }

    /// Header-only dimensions as `[width, height]`.
    #[wasm_bindgen]
    pub fn image_info(blob: &[u8]) -> Result<Vec<u32>, JsValue> {
        inspect(blob)
            .map(|image| vec![image.width, image.height])
            .map_err(to_js)
    }

    /// Crate version string.
    #[wasm_bindgen]
    pub fn version() -> String {
        version_str().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIDTH: u32 = 256;
    const HEIGHT: u32 = 256;

    /// Smooth two-axis gradient (RGB) with opaque alpha.
    fn gradient_rgba(width: u32, height: u32) -> Vec<u8> {
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            for x in 0..width {
                let r = (x * 255 / (width - 1).max(1)) as u8;
                let g = (y * 255 / (height - 1).max(1)) as u8;
                let b = ((x + y) * 255 / (width + height - 2).max(1)) as u8;
                rgba.extend_from_slice(&[r, g, b, 255]);
            }
        }
        rgba
    }

    fn psnr(a: &[u8], b: &[u8]) -> f64 {
        assert_eq!(a.len(), b.len());
        let mut sum = 0f64;
        for (&x, &y) in a.iter().zip(b) {
            let d = f64::from(x) - f64::from(y);
            sum += d * d;
        }
        let mse = sum / a.len() as f64;
        if mse == 0.0 {
            f64::INFINITY
        } else {
            10.0 * (255.0 * 255.0 / mse).log10()
        }
    }

    #[test]
    fn roundtrip_gradient_quality_80() {
        let rgba = gradient_rgba(WIDTH, HEIGHT);
        let blob = encode_rgba(WIDTH, HEIGHT, &rgba, 80).expect("encode");
        let (image, decoded) = decode_rgba(&blob).expect("decode");

        assert_eq!(
            image,
            Image {
                width: WIDTH,
                height: HEIGHT
            }
        );
        assert_eq!(decoded.len(), (WIDTH * HEIGHT * 4) as usize);
        let psnr_db = psnr(&rgba, &decoded);
        let raw = (WIDTH * HEIGHT * 4) as f64;
        let ratio = raw / blob.len() as f64;
        println!(
            "gradient {WIDTH}x{HEIGHT} q80: {} bytes, ratio {ratio:.2}:1, PSNR {psnr_db:.2} dB",
            blob.len()
        );
        assert!(psnr_db > 30.0, "PSNR {psnr_db:.2} dB is not above 30 dB");
        assert!(ratio > 2.0, "ratio {ratio:.2}:1 is not above 2:1");
    }

    #[test]
    fn header_only_inspect() {
        let rgba = gradient_rgba(13, 9);
        let blob = encode_rgba(13, 9, &rgba, 50).expect("encode");
        assert_eq!(
            inspect(&blob).unwrap(),
            Image {
                width: 13,
                height: 9
            }
        );
        let header = inspect_header(&blob).unwrap();
        assert_eq!(header.blocks_x, 2);
        assert_eq!(header.blocks_y, 2);
        assert_eq!(header.quality, 50);
        assert_eq!(header.block_size, 8);
    }

    #[test]
    fn bad_magic_is_rejected() {
        let rgba = gradient_rgba(8, 8);
        let mut blob = encode_rgba(8, 8, &rgba, 80).expect("encode");
        blob[0] ^= 0xff;
        assert_eq!(inspect(&blob), Err(PixlzrError::BadMagic));
        assert_eq!(decode_rgba(&blob), Err(PixlzrError::BadMagic));
    }

    #[test]
    fn truncated_input_errors_without_panicking() {
        let rgba = gradient_rgba(16, 16);
        let blob = encode_rgba(16, 16, &rgba, 80).expect("encode");

        // Shorter than the magic.
        assert!(matches!(
            decode_rgba(&blob[..3]),
            Err(PixlzrError::Truncated { .. })
        ));
        // Header cut in the middle.
        assert!(matches!(
            decode_rgba(&blob[..20]),
            Err(PixlzrError::Truncated { .. })
        ));
        // Header complete, payload missing.
        let hlen = header_len(&inspect_header(&blob).unwrap());
        assert!(matches!(
            decode_rgba(&blob[..hlen]),
            Err(PixlzrError::Zlib(_))
        ));
        // Payload clipped.
        assert!(matches!(
            decode_rgba(&blob[..hlen + 4]),
            Err(PixlzrError::Zlib(_))
        ));
    }

    #[test]
    fn rejects_bad_version_and_flags() {
        let rgba = gradient_rgba(8, 8);
        let mut blob = encode_rgba(8, 8, &rgba, 80).expect("encode");
        blob[6] = 9;
        assert_eq!(inspect(&blob), Err(PixlzrError::UnsupportedVersion(9)));

        let mut blob = encode_rgba(8, 8, &rgba, 80).expect("encode");
        blob[7] = 1;
        assert_eq!(inspect(&blob), Err(PixlzrError::UnsupportedFlags(1)));
    }

    #[test]
    fn rejects_inconsistent_block_grid() {
        let rgba = gradient_rgba(16, 16);
        let mut blob = encode_rgba(16, 16, &rgba, 80).expect("encode");
        // blocks_x claims 3 blocks for a 16px-wide image (should be 2).
        blob[19] = 3;
        assert_eq!(
            inspect(&blob),
            Err(PixlzrError::BadDimensions {
                width: 16,
                height: 16
            })
        );
    }

    #[test]
    fn rejects_short_rgba_buffer() {
        let err = encode_rgba(4, 4, &[0; 63], 80).unwrap_err();
        assert_eq!(
            err,
            PixlzrError::InvalidImageData {
                expected: 64,
                actual: 63
            }
        );
    }

    #[test]
    fn rejects_zero_and_oversized_dimensions() {
        assert!(matches!(
            encode_rgba(0, 4, &[], 80),
            Err(PixlzrError::BadDimensions { .. })
        ));
        assert!(matches!(
            encode_rgba(MAX_DIMENSION + 1, 1, &[], 80),
            Err(PixlzrError::BadDimensions { .. })
        ));
    }

    #[test]
    fn odd_sizes_roundtrip_and_crop() {
        for (w, h) in [(1, 1), (7, 3), (13, 9), (17, 17)] {
            let rgba = gradient_rgba(w, h);
            let blob = encode_rgba(w, h, &rgba, 90).expect("encode");
            let (image, decoded) = decode_rgba(&blob).expect("decode");
            assert_eq!(
                image,
                Image {
                    width: w,
                    height: h
                }
            );
            assert_eq!(decoded.len(), (w * h * 4) as usize);
            assert!(psnr(&rgba, &decoded) > 28.0);
        }
    }

    #[test]
    fn quality_is_clamped_and_informational() {
        let rgba = gradient_rgba(8, 8);
        let blob = encode_rgba(8, 8, &rgba, 200).expect("encode");
        assert_eq!(inspect_header(&blob).unwrap().quality, 100);
    }

    #[test]
    fn lower_quality_yields_smaller_blob() {
        let rgba = gradient_rgba(64, 64);
        // Add noise so quantization actually changes the entropy.
        let mut noisy = rgba.clone();
        for i in 0..(64 * 64) {
            let n = ((i * 37) % 41) as i16 - 20;
            for c in &mut noisy[i * 4..i * 4 + 3] {
                *c = (i16::from(*c) + n).clamp(0, 255) as u8;
            }
        }
        let high = encode_rgba(64, 64, &noisy, 95).unwrap().len();
        let low = encode_rgba(64, 64, &noisy, 10).unwrap().len();
        assert!(
            low < high,
            "q10 blob ({low}) should be smaller than q95 ({high})"
        );
    }

    #[test]
    fn version_str_matches_manifest() {
        assert_eq!(version_str(), env!("CARGO_PKG_VERSION"));
    }
}
