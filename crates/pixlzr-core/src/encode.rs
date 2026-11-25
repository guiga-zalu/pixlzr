//! Encoder: RGBA8 -> density-adaptive quantized DCT blocks -> `.pixlzr` blob.

use crate::color;
use crate::dct;
use crate::format::{self, validate_encode_input, BLOCK_SAMPLES, PLANES};
use crate::PixlzrError;

/// Encode RGBA8 pixels into a PIXLZR format v1 blob.
///
/// `quality` is stored in the header and scales the quantization step; values
/// above 100 are clamped. Alpha is not coded and is restored as 255 on decode.
pub fn encode_rgba(
    width: u32,
    height: u32,
    rgba: &[u8],
    quality: u8,
) -> Result<Vec<u8>, PixlzrError> {
    let quality = quality.min(100);
    let mut info = validate_encode_input(width, height, rgba.len())?;
    info.quality = quality;

    let w = width as usize;
    let h = height as usize;
    let (y_plane, cb_plane, cr_plane) = color::rgba_to_ycocg(rgba, w * h);

    let block_count = info.block_count();
    let mut steps = Vec::with_capacity(block_count);
    let mut coeffs = Vec::with_capacity(block_count * PLANES * BLOCK_SAMPLES * 2);

    for by in 0..info.blocks_y as usize {
        for bx in 0..info.blocks_x as usize {
            let y_block = pad_block(&y_plane, w, h, bx, by);
            let step = quant_step(&y_block, quality);
            let chroma_step = (u16::from(step) * 2).min(64) as u8;
            steps.push(step);

            let cb_block = pad_block(&cb_plane, w, h, bx, by);
            let cr_block = pad_block(&cr_plane, w, h, bx, by);
            for (block, plane_step) in [
                (&y_block, step),
                (&cb_block, chroma_step),
                (&cr_block, chroma_step),
            ] {
                let quantized = dct::forward_quantized(block, f32::from(plane_step));
                for value in quantized {
                    coeffs.extend_from_slice(&value.to_le_bytes());
                }
            }
        }
    }

    let payload = miniz_oxide::deflate::compress_to_vec_zlib(&coeffs, 6);

    let mut out = Vec::with_capacity(format::HEADER_FIXED_LEN + steps.len() + payload.len());
    format::write_header(&mut out, &info);
    out.extend_from_slice(&steps);
    out.extend_from_slice(&payload);
    Ok(out)
}

/// Copy one 8x8 block out of a full-resolution plane, replicating edge pixels
/// for partial blocks.
fn pad_block(plane: &[i16], width: usize, height: usize, bx: usize, by: usize) -> [f32; 64] {
    let mut block = [0f32; 64];
    let base_x = bx * dct::N;
    let base_y = by * dct::N;
    for y in 0..dct::N {
        let src_y = (base_y + y).min(height - 1);
        for x in 0..dct::N {
            let src_x = (base_x + x).min(width - 1);
            block[y * dct::N + x] = f32::from(plane[src_y * width + src_x]);
        }
    }
    block
}

/// Map local luma gradient density and the quality knob to a quant step.
///
/// `density` is the mean of `|dx| + |dy|` over the padded 8x8 luma block,
/// normalised by 64 (a full 0..255 swing between neighbours saturates at 1).
fn quant_step(luma_block: &[f32; 64], quality: u8) -> u8 {
    let mut sum = 0f32;
    for y in 0..dct::N {
        for x in 0..dct::N {
            let center = luma_block[y * dct::N + x];
            if x + 1 < dct::N {
                sum += (luma_block[y * dct::N + x + 1] - center).abs();
            }
            if y + 1 < dct::N {
                sum += (luma_block[(y + 1) * dct::N + x] - center).abs();
            }
        }
    }
    let density_norm = (sum / (BLOCK_SAMPLES as f32 * 64.0)).clamp(0.0, 1.0);
    let base = 4.0 + density_norm * 28.0;
    let quality_factor = 1.2 - 0.6 * (f32::from(quality) / 100.0);
    (base * quality_factor).round().clamp(4.0, 64.0) as u8
}
