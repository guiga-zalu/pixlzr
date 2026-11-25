//! Criterion benchmarks for the PIXLZR v1 codec.
//!
//! The input is a deterministic 512x512 gradient with LCG noise mixed in, so
//! the payload is not trivially compressible and the numbers reflect real work.
//!
//! Run with:
//!
//! ```bash
//! cargo bench --features bench
//! ```

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use pixlzr_core::{decode_rgba, encode_rgba};

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;
const QUALITY: u8 = 80;

/// Deterministic gradient + pseudo-random noise (LCG, no external deps).
fn synthetic_rgba() -> Vec<u8> {
    let mut rgba = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
    let mut seed = 0x1234_5678u32;
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let noise = ((seed >> 16) & 0x3f) as i32 - 32;
            let r = (x * 255 / (WIDTH - 1)) as i32;
            let g = (y * 255 / (HEIGHT - 1)) as i32;
            let b = ((x * 3 + y * 5) % 256) as i32;
            rgba.push((r + noise).clamp(0, 255) as u8);
            rgba.push((g + noise / 2).clamp(0, 255) as u8);
            rgba.push((b + noise / 3).clamp(0, 255) as u8);
            rgba.push(255);
        }
    }
    rgba
}

fn bench_codec(c: &mut Criterion) {
    let rgba = synthetic_rgba();
    let blob = encode_rgba(WIDTH, HEIGHT, &rgba, QUALITY).expect("bench encode");

    let mut group = c.benchmark_group("codec");
    group.throughput(Throughput::Bytes(u64::from(WIDTH) * u64::from(HEIGHT) * 4));
    group.sample_size(10);

    group.bench_function("encode_512x512_q80", |b| {
        b.iter(|| encode_rgba(WIDTH, HEIGHT, black_box(&rgba), QUALITY).expect("bench encode"));
    });
    group.bench_function("decode_512x512_q80", |b| {
        b.iter(|| decode_rgba(black_box(&blob)).expect("bench decode"));
    });

    group.finish();
}

criterion_group!(benches, bench_codec);
criterion_main!(benches);
