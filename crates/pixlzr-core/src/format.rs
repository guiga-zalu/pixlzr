//! Format v1: header, per-block quant steps and payload framing.
//!
//! See `docs/format.md` at the repository root for the byte-level spec.

use crate::PixlzrError;

/// File magic: `PIXLZR`.
pub const MAGIC: [u8; 6] = *b"PIXLZR";
/// Current on-disk format version.
pub const VERSION: u8 = 1;
/// The only block size defined by format v1.
pub const BLOCK_SIZE: u8 = 8;
/// Number of samples per 8x8 block, per plane.
pub const BLOCK_SAMPLES: usize = 64;
/// Planes stored per block: Y, Cb, Cr.
pub const PLANES: usize = 3;
/// Size of the fixed part of the header, before the per-block step table.
pub const HEADER_FIXED_LEN: usize = 27;

/// Hard limit on a single side, independent of [`MAX_PIXELS`].
pub const MAX_DIMENSION: u32 = 8192;
/// Hard limit on `width * height` (16.7 MP, i.e. 4096x4096).
pub const MAX_PIXELS: u64 = 16_777_216;

/// Dimensions of a PIXLZR image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
}

/// Parsed header metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeaderInfo {
    pub width: u32,
    pub height: u32,
    pub quality: u8,
    pub blocks_x: u32,
    pub blocks_y: u32,
    pub block_size: u8,
}

impl HeaderInfo {
    /// Dimensions only.
    pub fn image(&self) -> Image {
        Image {
            width: self.width,
            height: self.height,
        }
    }

    /// Total number of blocks in the image.
    pub fn block_count(&self) -> usize {
        self.blocks_x as usize * self.blocks_y as usize
    }
}

/// Number of blocks along an axis: `ceil(axis / 8)`.
pub fn blocks_along(axis: u32) -> u32 {
    axis.div_ceil(BLOCK_SIZE as u32)
}

/// Bytes occupied by the fixed header plus the per-block step table.
pub fn header_len(info: &HeaderInfo) -> usize {
    HEADER_FIXED_LEN + info.block_count()
}

/// Bytes of the uncompressed coefficient stream for `info`.
pub fn raw_coeff_len(info: &HeaderInfo) -> usize {
    info.block_count() * PLANES * BLOCK_SAMPLES * 2
}

/// Validate dimensions supplied to the encoder.
pub(crate) fn validate_encode_input(
    width: u32,
    height: u32,
    rgba_len: usize,
) -> Result<HeaderInfo, PixlzrError> {
    if width == 0 || height == 0 {
        return Err(PixlzrError::BadDimensions { width, height });
    }
    if width > MAX_DIMENSION
        || height > MAX_DIMENSION
        || u64::from(width) * u64::from(height) > MAX_PIXELS
    {
        return Err(PixlzrError::BadDimensions { width, height });
    }
    let expected = (u64::from(width) * u64::from(height) * 4) as usize;
    if rgba_len < expected {
        return Err(PixlzrError::InvalidImageData {
            expected,
            actual: rgba_len,
        });
    }
    Ok(HeaderInfo {
        width,
        height,
        quality: 0,
        blocks_x: blocks_along(width),
        blocks_y: blocks_along(height),
        block_size: BLOCK_SIZE,
    })
}

/// Parse and validate a format v1 header. Does not touch the payload.
pub fn inspect_header(blob: &[u8]) -> Result<HeaderInfo, PixlzrError> {
    if blob.len() < MAGIC.len() {
        return Err(PixlzrError::Truncated {
            expected: HEADER_FIXED_LEN,
            actual: blob.len(),
        });
    }
    if blob[..MAGIC.len()] != MAGIC {
        return Err(PixlzrError::BadMagic);
    }
    if blob.len() < HEADER_FIXED_LEN {
        return Err(PixlzrError::Truncated {
            expected: HEADER_FIXED_LEN,
            actual: blob.len(),
        });
    }

    let version = blob[6];
    if version != VERSION {
        return Err(PixlzrError::UnsupportedVersion(version));
    }

    let flags = u16::from_le_bytes([blob[7], blob[8]]);
    if flags != 0 {
        return Err(PixlzrError::UnsupportedFlags(flags));
    }

    let width = u32::from_le_bytes([blob[9], blob[10], blob[11], blob[12]]);
    let height = u32::from_le_bytes([blob[13], blob[14], blob[15], blob[16]]);
    let block_size = blob[17];
    let quality = blob[18];
    let blocks_x = u32::from_le_bytes([blob[19], blob[20], blob[21], blob[22]]);
    let blocks_y = u32::from_le_bytes([blob[23], blob[24], blob[25], blob[26]]);

    if block_size != BLOCK_SIZE {
        return Err(PixlzrError::BadBlockSize(block_size));
    }
    if width == 0
        || height == 0
        || width > MAX_DIMENSION
        || height > MAX_DIMENSION
        || u64::from(width) * u64::from(height) > MAX_PIXELS
        || blocks_x != blocks_along(width)
        || blocks_y != blocks_along(height)
    {
        return Err(PixlzrError::BadDimensions { width, height });
    }

    let info = HeaderInfo {
        width,
        height,
        quality,
        blocks_x,
        blocks_y,
        block_size,
    };

    if blob.len() < header_len(&info) {
        return Err(PixlzrError::Truncated {
            expected: header_len(&info),
            actual: blob.len(),
        });
    }

    Ok(info)
}

/// Read dimensions from a PIXLZR blob without decoding the payload.
pub fn inspect(blob: &[u8]) -> Result<Image, PixlzrError> {
    inspect_header(blob).map(|header| header.image())
}

/// Write the fixed header fields (including width/height/blocks) for `info`.
pub(crate) fn write_header(out: &mut Vec<u8>, info: &HeaderInfo) {
    out.extend_from_slice(&MAGIC);
    out.push(VERSION);
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&info.width.to_le_bytes());
    out.extend_from_slice(&info.height.to_le_bytes());
    out.push(info.block_size);
    out.push(info.quality);
    out.extend_from_slice(&info.blocks_x.to_le_bytes());
    out.extend_from_slice(&info.blocks_y.to_le_bytes());
}
