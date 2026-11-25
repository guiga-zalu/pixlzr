//! Decoder: `.pixlzr` blob -> RGBA8 pixels.

use crate::color;
use crate::dct;
use crate::format::{self, Image, BLOCK_SAMPLES, PLANES};
use crate::PixlzrError;

/// Decode a PIXLZR format v1 blob into `(dimensions, RGBA8 pixels)`.
pub fn decode_rgba(blob: &[u8]) -> Result<(Image, Vec<u8>), PixlzrError> {
    let info = format::inspect_header(blob)?;
    let header_len = format::header_len(&info);
    let steps = &blob[format::HEADER_FIXED_LEN..header_len];
    let payload = &blob[header_len..];

    let expected = format::raw_coeff_len(&info);
    let coeffs = miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(payload, expected)
        .map_err(|err| PixlzrError::Zlib(format!("{err:?}")))?;
    if coeffs.len() != expected {
        return Err(PixlzrError::Truncated {
            expected,
            actual: coeffs.len(),
        });
    }

    let width = info.width as usize;
    let height = info.height as usize;
    let pixels = width * height;
    let mut planes = [vec![0f32; pixels], vec![0f32; pixels], vec![0f32; pixels]];

    let mut offset = 0usize;
    for by in 0..info.blocks_y as usize {
        for bx in 0..info.blocks_x as usize {
            let block_index = by * info.blocks_x as usize + bx;
            let step = steps[block_index];
            let chroma_step = (u16::from(step) * 2).min(64) as u8;

            for (plane, plane_step) in planes.iter_mut().zip([step, chroma_step, chroma_step]) {
                let mut quantized = [0i16; 64];
                for value in &mut quantized {
                    *value = i16::from_le_bytes([coeffs[offset], coeffs[offset + 1]]);
                    offset += 2;
                }
                let block = dct::inverse(&quantized, f32::from(plane_step));
                write_block(plane, width, height, bx, by, &block);
            }
        }
    }

    let rgba = color::ycocg_to_rgba(&planes[0], &planes[1], &planes[2]);
    Ok((info.image(), rgba))
}

/// Scatter the valid part of a decoded 8x8 block into the full-resolution plane.
fn write_block(
    plane: &mut [f32],
    width: usize,
    height: usize,
    bx: usize,
    by: usize,
    block: &[f32; 64],
) {
    let base_x = bx * dct::N;
    let base_y = by * dct::N;
    let max_y = dct::N.min(height - base_y);
    let max_x = dct::N.min(width - base_x);
    for y in 0..max_y {
        let row = (base_y + y) * width + base_x;
        plane[row..row + max_x].copy_from_slice(&block[y * dct::N..y * dct::N + max_x]);
    }
}

const _: () = {
    assert!(BLOCK_SAMPLES * PLANES * 2 == 384);
};
