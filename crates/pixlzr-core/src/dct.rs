//! Orthonormal 8x8 DCT-II with JPEG zigzag coefficient order.
//!
//! The transform is the textbook separable orthonormal DCT:
//!
//! ```text
//! C[v][u] = sum_y sum_x B[y][x] * O[v][y] * O[u][x]
//! O[k][n] = alpha(k) * cos(pi * (2n + 1) * k / 16)
//! alpha(0) = sqrt(1/8), alpha(k) = sqrt(2/8)
//! ```
//!
//! Because `O` is orthonormal, the inverse is the transpose (`O^T C O`).

use std::sync::OnceLock;

/// Block edge length.
pub const N: usize = 8;

/// `ZIGZAG[i]` is the raster index of the `i`-th coefficient in zigzag order.
pub(crate) const ZIGZAG: [usize; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20,
    13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59,
    52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

fn basis() -> &'static [[f32; N]; N] {
    static BASIS: OnceLock<[[f32; N]; N]> = OnceLock::new();
    BASIS.get_or_init(|| {
        let mut table = [[0f32; N]; N];
        for (k, row) in table.iter_mut().enumerate() {
            let alpha = if k == 0 {
                (1.0 / N as f32).sqrt()
            } else {
                (2.0 / N as f32).sqrt()
            };
            for (n, cell) in row.iter_mut().enumerate() {
                let angle =
                    std::f32::consts::PI * (2.0 * n as f32 + 1.0) * k as f32 / (2.0 * N as f32);
                *cell = alpha * angle.cos();
            }
        }
        table
    })
}

/// Forward DCT followed by quantization, returned in zigzag order.
pub(crate) fn forward_quantized(block: &[f32; 64], step: f32) -> [i16; 64] {
    let basis = basis();
    // tmp[y][u] = sum_x B[y][x] * O[u][x]
    let mut tmp = [[0f32; N]; N];
    for y in 0..N {
        for (u, tmp_row) in tmp[y].iter_mut().enumerate() {
            let mut sum = 0f32;
            for (x, &b) in block[y * N..y * N + N].iter().enumerate() {
                sum += b * basis[u][x];
            }
            *tmp_row = sum;
        }
    }

    let mut out = [0i16; 64];
    for v in 0..N {
        for u in 0..N {
            let mut sum = 0f32;
            for y in 0..N {
                sum += tmp[y][u] * basis[v][y];
            }
            let quantized = (sum / step).round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            out[ZIGZAG[v * N + u]] = quantized;
        }
    }
    out
}

/// Dequantize zigzag coefficients and run the inverse DCT.
pub(crate) fn inverse(coeffs: &[i16; 64], step: f32) -> [f32; 64] {
    let basis = basis();
    let mut spectral = [[0f32; N]; N];
    for v in 0..N {
        for u in 0..N {
            spectral[v][u] = f32::from(coeffs[ZIGZAG[v * N + u]]) * step;
        }
    }

    // tmp[y][u] = sum_v C[v][u] * O[v][y]
    let mut tmp = [[0f32; N]; N];
    for y in 0..N {
        for u in 0..N {
            let mut sum = 0f32;
            for v in 0..N {
                sum += spectral[v][u] * basis[v][y];
            }
            tmp[y][u] = sum;
        }
    }

    let mut out = [0f32; 64];
    for y in 0..N {
        for x in 0..N {
            let mut sum = 0f32;
            for u in 0..N {
                sum += tmp[y][u] * basis[u][x];
            }
            out[y * N + x] = sum;
        }
    }
    out
}
