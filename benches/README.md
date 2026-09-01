# Benchmark Harness — `criterion` Spec (Phase 2.4)

> **Purpose:** Turn "95% reduction / 60fps" from anecdote into a reproducible, hardware-pinned table that a hiring manager can trust and rerun with one command.

## 1. Quick Start (after harness lands)

```bash
cargo bench --features bench
# or filtered:
cargo bench --bench codec -- --sample-size 20
cargo bench --bench video_filter
open target/criterion/report/index.html
```

CI persists `bench.txt` and posts to `gh-pages` via `benchmark-action/github-action-benchmark`.

## 2. Harness Layout

```
benches/
├── codec.rs              # criterion main: encode/decode throughput + ratio at matched quality
├── video_filter.rs       # 1080p frame loop → asserts FPS ≥60
└── data/
    ├── kodak/            # 24 images 768×512 — SHA256 manifest, gitignored fetch via xtask
    ├── clic_valid/       # CLIC validation subset (or Tecnick 100)
    └── synthetic/        # high-density chart (checkerboard + text) to stress density fn
    └── MANIFEST.sha256
```

`Cargo.toml` additions:

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "codec"
harness = false

[[bench]]
name = "video_filter"
harness = false
```

`benches/codec.rs` skeleton:

```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use pixlzr_core::{encode, decode};

fn bench_codec(c: &mut Criterion) {
    let mut group = c.benchmark_group("pixlzr_codec");
    for (name, bytes) in datasets() { // kodak/clic/synthetic
        group.bench_with_input(BenchmarkId::new("encode", name), &bytes, |b, data| {
            b.iter(|| encode(data, &params_for_ssim(0.92)))
        });
        group.bench_with_input(BenchmarkId::new("decode", name), &bytes, |b, data| {
            let enc = encode(data, &params_for_ssim(0.92));
            b.iter(|| decode(&enc))
        });
    }
    group.finish();
}
criterion_group!(benches, bench_codec);
criterion_main!(benches);
```

## 3. Datasets — Fixed & Versioned

| Dataset | Content | Why | Source |
|---------|---------|-----|--------|
| Kodak | 24 × 768×512 photographic | Classic baseline, every codec reports it | https://r0k.us/graphics/kodak/ (store under `benches/data/kodak/`) |
| CLIC valid | ~102 images, varied res | Modern photographic, harder | https://clic.compression.cc/ |
| Synthetic high-density | 1 × 2048×2048 checkerboard + text | Stresses density function | Generate via `xtask gen-synthetic` |

- Store SHA256 in `benches/data/MANIFEST.sha256`; `xtask fetch-data` verifies.
- Do **not** commit large images to git — fetch in `ci.yml bench` job or via `git-lfs` if needed.

## 4. Baselines — Same Quality, Not Same File Size

Compare at **matched perceptual quality** (SSIM ≥0.92, PSNR ≥30 dB), then compare ratio/throughput.

| Baseline | Crate / tool | Quality sweep |
|----------|--------------|---------------|
| JPEG | `mozjpeg` or `jpeg-encoder` | q 60–95 |
| PNG | `oxipng` level 6 / `png` crate | level 6 (lossless — ratio will be poor, shows win) |
| WebP | `webp` crate (libwebp) | q 60–90 |
| AVIF (optional) | `ravif` | speed 6, q 50–70 |

Procedure per image:

1. Sweep quality param until SSIM ≥0.92 (±0.01).
2. At that quality, record compressed size and encode/decode ms.
3. Report ratio = original / compressed.

SSIM/PSNR computed via `ssim` crate or `dssim` — add as `dev-dependency` under `bench` feature.

## 5. Metrics to Report

Per dataset, per codec:

- **Compression ratio** at matched SSIM/PSNR (original / compressed).
- **Encode ms** (mean ± std, `criterion`).
- **Decode ms** (mean ± std).
- **Throughput MP/s** (megapixels per second).
- **FPS @1080p** (derived: `MP/s / 2.07` where 1080p = 1920×1080 ≈2.07 MP).
- **SSIM / PSNR** achieved.
- Optional: `Butteraugli` distance if `butteraugli` crate available.

## 6. Target Table (fill after first bench run)

Copy this table into `README.md#Benchmarks`:

| Dataset | PIXLZR (ratio @ SSIM 0.92) | JPEG | PNG | WebP | Enc ms | Dec ms | FPS 1080p | SSIM |
|---|---|---|---|---|---|---|---|---|
| Kodak (24× 768×512) | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ |
| CLIC valid | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ |
| Synthetic high-density | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ |

> **Acceptance bar for "95% reduction":** Show ≥20:1 ratio on Kodak at SSIM ≥0.92 where JPEG needs ≤5:1 for same quality — or reword claim to "Up to 95% vs BMP on high-density synthetic @ SSIM 0.90". Blanket "95%" without matching quality is trivial (vs BMP q=100).

Also add a small chart (generated via `criterion-table`):

```
cargo bench -- --output-format bencher | tee bench.txt
# benchmark-action renders a trend chart on gh-pages
```

## 7. Hardware Disclosure (mandatory)

Every bench run must log:

```
CPU: AMD Ryzen 7 5800X (8C/16T) — or: Apple M1, Intel i7-12700H, etc.
RAM: 32GB DDR4-3200
OS: Ubuntu 24.04 (kernel 6.8), governor=performance, turbo=on/off
rustc: 1.82.0 (cargo --version)
RUSTFLAGS: (e.g., -C target-cpu=native)
RAYON_NUM_THREADS: 8 (and 1 for single-thread baseline)
Dataset SHA256: <from MANIFEST.sha256>
```

Include this block in `benches/README.md` footer and in the `README.md#Benchmarks` reproducer snippet.

Add to `benches/codec.rs` header comment so `cargo bench -- --verbose` prints it.

## 8. Video 60fps Proof

`benches/video_filter.rs`:

```rust
// Decode + filter a 1080p30 clip (10s, 300 frames) and assert wall-clock FPS ≥60
let start = Instant::now();
let frames = decode_and_filter(&clip, &filter_params);
let elapsed = start.elapsed();
let fps = frames as f64 / elapsed.as_secs_f64();
assert!(fps >= 60.0, "expected ≥60fps, got {fps:.1}");
```

Report with/without `multithread` and attach `perf` flamegraph as CI artifact:

```yaml
- run: cargo bench --bench video_filter
- run: perf record -g cargo bench --bench video_filter && perf script | stackcollapse-perf | flamegraph > flame.svg
- uses: actions/upload-artifact@v4
  with: { name: flamegraph, path: flame.svg }
```

## 9. CI Bench Job (see `.github/workflows/ci.yml`)

- Runs only on `main` to avoid PR noise.
- Uses `benchmark-action/github-action-benchmark@v1` with `tool: cargo`, `output-file-path: bench.txt`, `auto-push: true` to maintain `gh-pages` bench history.
- Fails if `criterion` detects regression >10% (configurable).

## 10. Verification

```bash
cargo bench --features bench -- --sample-size 10  # quick smoke
ls target/criterion/report/index.html && echo "report ok"
cat bench.txt | head -20
# expect: hardware header + per-dataset rows with ratio/ssim/ms
```

---

*Copy this file to `benches/README.md` in `pixlzr`. Pair with `benches/codec.rs` + `benches/video_filter.rs` + `.github/workflows/ci.yml` bench job.*
