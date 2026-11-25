//! Reversible RGBA <-> YCoCg colour transform.
//!
//! The transform is exact in integer arithmetic:
//!
//! ```text
//! Y  = (R + 2G + B) >> 2      G = Y - ((Cb + Cr) >> 2)
//! Cb = B - G                  R = Cr + G
//! Cr = R - G                  B = Cb + G
//! ```
//!
//! Alpha is not coded in format v1 and is restored as fully opaque.

/// Split RGBA8 pixels into full-resolution Y, Cb and Cr planes.
pub(crate) fn rgba_to_ycocg(rgba: &[u8], pixel_count: usize) -> (Vec<i16>, Vec<i16>, Vec<i16>) {
    let mut y = Vec::with_capacity(pixel_count);
    let mut cb = Vec::with_capacity(pixel_count);
    let mut cr = Vec::with_capacity(pixel_count);
    for i in 0..pixel_count {
        let px = &rgba[i * 4..i * 4 + 4];
        let r = i32::from(px[0]);
        let g = i32::from(px[1]);
        let b = i32::from(px[2]);
        y.push(((r + 2 * g + b) >> 2) as i16);
        cb.push((b - g) as i16);
        cr.push((r - g) as i16);
    }
    (y, cb, cr)
}

/// Merge reconstructed Y, Cb and Cr planes back into opaque RGBA8.
pub(crate) fn ycocg_to_rgba(y: &[f32], cb: &[f32], cr: &[f32]) -> Vec<u8> {
    let pixels = y.len();
    let mut rgba = Vec::with_capacity(pixels * 4);
    for i in 0..pixels {
        let y = y[i].round() as i32;
        let cb = cb[i].round() as i32;
        let cr = cr[i].round() as i32;
        let g = y - ((cb + cr) >> 2);
        let r = cr + g;
        let b = cb + g;
        rgba.push(clamp8(r));
        rgba.push(clamp8(g));
        rgba.push(clamp8(b));
        rgba.push(255);
    }
    rgba
}

#[inline]
fn clamp8(v: i32) -> u8 {
    v.clamp(0, 255) as u8
}
