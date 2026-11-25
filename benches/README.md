# Benchmark Harness — `criterion` Notes

> **Status:** `benches/codec.rs` is implemented (encode/decode on a deterministic
> 512×512 gradient+noise image) and compiles in CI via
> `cargo bench --no-run --features bench`. **No numbers are published yet** — the
> README table stays `_pending measurement_` until both a full run and pinned
> hardware disclosure exist.
>
> The dataset/baseline sections below (Kodak/CLIC downloads, JPEG/PNG/WebP
> comparisons, `video_filter.rs`, hardware templates) are **plans, not shipped
> code**. Do not run `cargo bench --bench video_filter` — that benchmark does not
> exist.

> **Purpose:** Turn "95% reduction / 60fps" from anecdote into a reproducible, hardware-pinned table that a hiring manager can trust and rerun with one command.

## 1. Quick Start

```bash
cargo bench --features bench
# or filtered:
cargo bench --bench codec -- --sample-size 20
```

CI persists `bench.txt` and posts to `gh-pages` via `benchmark-action/github-action-benchmark`.

## 2. Harness Layout

```
benches/
└── codec.rs              # criterion: encode/decode at q80 on a generated 512x512 image
```

Planned (not implemented): `video_filter.rs`, vendored Kodak/CLIC datasets, and
JPEG/PNG/WebP baseline sweeps. The current benchmark generates its input in code,
so there are no external data files or SHA256 manifests to fetch.

`Cargo.toml` additions (actual):

```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "codec"
harness = false
```

`benches/codec.rs` (actual, abridged):

```rust
use pixlzr_core::{decode_rgba, encode_rgba};

fn synthetic_rgba() -> Vec<u8> { /* gradient + deterministic LCG noise */ }

fn bench_codec(c: &mut Criterion) {
    let rgba = synthetic_rgba();
    let blob = encode_rgba(512, 512, &rgba, 80).unwrap();
    let mut group = c.benchmark_group("codec");
    group.throughput(Throughput::Bytes(512 * 512 * 4));
    group.bench_function("encode_512x512_q80", |b| {
        b.iter(|| encode_rgba(512, 512, black_box(&rgba), 80).unwrap())
    });
    group.bench_function("decode_512x512_q80", |b| {
        b.iter(|| decode_rgba(black_box(&blob)).unwrap())
    });
    group.finish();
}
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

Include this block in the README once a real run exists. `benches/codec.rs`
currently prints no hardware header; do not publish numbers without one.

## 8. Video 60fps Proof (planned — not implemented)

A future `benches/video_filter.rs` would look like:

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
- Alerts (comment) on a >150% regression; it does not fail the build
  (`fail-on-alert: false` in `.github/workflows/ci.yml`).

## 10. Verification

```bash
cargo bench --features bench -- --sample-size 10  # quick smoke
cat bench.txt | head -20
# expect: criterion timing lines for encode_512x512_q80 and decode_512x512_q80
```

---

*Current implementation: `benches/codec.rs` only. The 60 fps video path and the
baseline comparisons above are future work; treat their snippets as design notes.*
