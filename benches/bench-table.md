# PIXLZR — Reproducible Benchmark Table Template (Phase 2.4)

> **Goal:** Paste this table into `README.md#Benchmarks` after the first `cargo bench` run. Every number ties to a `criterion` report and a pinned hardware disclosure — so "95% reduction / 60fps" stops being marketing and becomes citable engineering.
>
> **File:** `benches/bench-table.md` — fill `_todo_` after running `cargo bench --features bench` per `benches/README.md` and `docs/pixlzr-bench-runbook.md`.
>
> **Matching rule:** Compare at **matched perceptual quality** — sweep baseline quality until **SSIM ≥0.92 (±0.01) and PSNR ≥30 dB**, then compare ratio/throughput. Never compare file sizes at mismatched quality.

---

## 0. How to Fill (3 steps)

1. Run benches pinned (`docs/pixlzr-bench-runbook.md`):
   ```bash
   RAYON_NUM_THREADS=8 cargo bench --features bench -- --output-format bencher | tee bench.txt
   RAYON_NUM_THREADS=1 cargo bench --features bench -- --sample-size 20 | tee bench-single.txt
   # then:
   cargo bench -- --verbose > bench-hardware.log  # capture header below
   ```
2. For each dataset × codec, record **ratio** (original/compressed at matched SSIM), **enc ms**, **dec ms** (criterion mean ± std), **MP/s**, **FPS 1080p**, **PSNR**, **SSIM**.
3. Copy the filled `Markdown Table §2` into `README.md#Benchmarks` and commit `bench.txt` + this file.

Datasets are fixed + SHA256-pinned — see `benches/README.md §3` and `benches/data/MANIFEST.sha256`. Do **not** re-pick images per run.

---

## 1. Hardware Disclosure (mandatory — paste from `bench-hardware.log`)

> Every bench row is meaningless without this block. Include verbatim in README below the table and in `target/criterion/report/index.html` header comment.

```
Hardware Spec — PIXLZR Bench Run
─────────────────────────────────
Date:             2026-__-__  (ISO 8601)
CPU:              AMD Ryzen 7 5800X (8C/16T)  — or: Intel i7-12700H / Apple M1 / etc.
RAM:              32GB DDR4-3200 (or LPDDR5-6400)
OS:               Ubuntu 24.04 (kernel 6.8.x), governor=performance, turbo=on  (or macOS 15.x)
rustc:            1.82.0  (rustc --version; cargo --version)
RUSTFLAGS:        -C target-cpu=native  (or empty — state explicitly)
RAYON_NUM_THREADS: 8  (and 1 for single-thread baseline run)
Dataset SHA256:   <from benches/data/MANIFEST.sha256 — paste 8-char prefix per dataset>
Criterion:        0.5.x  (html_reports)
Commit:           <git rev-parse --short HEAD>
Notes:            e.g., laptop on AC, no thermal throttling observed; WASM not included in this table
```

Add to `benches/codec.rs` header so `cargo bench -- --verbose` prints it. CI bench job sets `RAYON_NUM_THREADS=8` and uploads `bench.txt` as artifact.

---

## 2. Benchmark Table — Markdown (copy-paste into README)

### 2.1 Image Codec — Ratio & Latency at Matched Quality (SSIM ≥0.92, PSNR ≥30 dB)

| Dataset | PIXLZR ratio | PIXLZR enc ms | PIXLZR dec ms | JPEG ratio | PNG ratio | WebP ratio | MP/s (enc) | MP/s (dec) | FPS 1080p¹ | PSNR (dB) | SSIM | Notes |
|---------|:------------:|:-------------:|:-------------:|:----------:|:---------:|:----------:|:----------:|:----------:|:----------:|:---------:|:----:|-------|
| **Kodak** (24× 768×512, 9.4 MP total) | _todo_ | _todo_ ± _ | _todo_ ± _ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | matched q: JPEG q=_ WebP q=_ |
| **CLIC valid** (~50 images, ~45 MP total) | _todo_ | _todo_ ± _ | _todo_ ± _ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | subset SHA=_ |
| **Synthetic high-density** (1× 2048×2048, 4.2 MP) | _todo_ | _todo_ ± _ | _todo_ ± _ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | checkerboard+text, λ=0.8 |

> ¹ **FPS 1080p** derived: `FPS = MP/s (dec) / 2.07` where 1080p = 1920×1080 ≈ 2.07 MP. Shows whether "60fps" holds. Report both `enc` and `dec` MP/s; FPS uses decode throughput.

**How ratio is computed:** `ratio = original_bytes / compressed_bytes` at the quality that hits the SSIM/PSNR gate above. Original = raw RGB24 (`width·height·3`) or PNG-decoded bytes — state which in Notes; be consistent across baselines. For PNG baseline, ratio will be ~1.5–3× (lossless) — that is expected and shows the win is not trivial.

**Acceptance bar for the CV claim "95% reduction":** Requires **≥20:1 ratio** on Kodak at SSIM ≥0.92 where JPEG needs ≤5:1 for same quality. If not met, reword in README/CV to: `"Up to 95% vs BMP on high-density synthetic @ SSIM 0.90 (see benches/bench-table.md)"` — more defensible, still strong.

### 2.2 Video Filter — 60fps Proof (1080p30 clip, 10s / 300 frames)

| Mode | Threads | Frames | Wall clock (s) | FPS (frames/wall) | Filter kernel | Pass (≥60) | Notes |
|------|---------|--------|----------------|-------------------|---------------|------------|-------|
| PIXLZR decode+filter | 8 (Rayon) | 300 | _todo_ | _todo_ | e.g., density-aware blur | _todo_ yes/no | `benches/video_filter.rs` |
| PIXLZR decode+filter | 1 | 300 | _todo_ | _todo_ | same | _todo_ | single-thread baseline |
| No filter (decode only) | 8 | 300 | _todo_ | _todo_ | — | — | isolates filter cost |

Assert in bench: `assert!(fps >= 60.0, "expected ≥60fps, got {fps:.1}")`. Report with/without multithread. Attach `perf` flamegraph artifact on CI (see `benches/README.md §8`).

### 2.3 Throughput Summary (derived — for chart)

| Metric | PIXLZR | JPEG (mozjpeg) | PNG (oxipng) | WebP |
|--------|:------:|:--------------:|:------------:|:----:|
| Encode MP/s (mean) | _todo_ | _todo_ | _todo_ | _todo_ |
| Decode MP/s (mean) | _todo_ | _todo_ | _todo_ | _todo_ |
| Encode ms @ 1080p (single frame) | _todo_ | _todo_ | _todo_ | _todo_ |
| Decode ms @ 1080p (single frame) | _todo_ | _todo_ | _todo_ | _todo_ |

Use for `gnuplot` / `criterion-table` chart — x=codec, y=MP/s at fixed SSIM.

---

## 3. Per-Codec Quality Sweep Log (fill one row per quality tested)

For each baseline, sweep quality until the gate is hit. Keep this log so reviewers see you did not cherry-pick.

| Codec | Quality param | Size (KB) | PSNR (dB) | SSIM | Hit gate? (SSIM≥0.92 & PSNR≥30) |
|-------|---------------|-----------|-----------|------|--------------------------------|
| PIXLZR (`q_base=0.80, λ=0.8`) | q_base 0.80 | _todo_ | _todo_ | _todo_ | _ |
| JPEG (`mozjpeg` q) | 75 | _todo_ | _todo_ | _todo_ | _ |
| JPEG | 85 | _todo_ | _todo_ | _todo_ | _ |
| WebP | 75 | _todo_ | _todo_ | _todo_ | _ |
| WebP | 80 | _todo_ | _todo_ | _todo_ | _ |
| PNG | level 6 (lossless) | _todo_ | inf (lossless) | 1.00 | n/a (no quality sweep) |

Pick the **first quality that passes the gate** for the main table above. If two pass, pick the smaller file (higher compression) — document choice in Notes.

---

## 4. Reproducibility Checklist

- [ ] `benches/data/MANIFEST.sha256` verified via `cargo xtask fetch-data` or `sha256sum -c benches/data/MANIFEST.sha256`
- [ ] `cargo bench --features bench` run **twice**: `RAYON_NUM_THREADS=8` and `RAYON_NUM_THREADS=1`
- [ ] `bench.txt` committed (or attached as CI artifact) — contains `criterion` mean ± std + hardware header
- [ ] `target/criterion/report/index.html` opens and shows per-benchmark plots
- [ ] Flamegraph generated (optional but recommended): `cargo bench --bench video_filter` + `perf` → `flame.svg` uploaded as artifact
- [ ] This file's `§2` table copied into `README.md#Benchmarks` — no drift
- [ ] `gnuplot` chart updated (if using): `gnuplot benches/plot.gnu` → `assets/bench-chart.png` (<100KB)
- [ ] LinkedIn Featured + GitHub Pages link to the filled table (not the `_todo_` version)

---

## 5. Example — Filled Row (illustrative, do NOT cite — replace with real runs)

> Remove this section after first real run. It shows the shape of a credible result.

| Dataset | PIXLZR ratio | PIXLZR enc ms | PIXLZR dec ms | JPEG ratio | PNG ratio | WebP ratio | MP/s (enc) | MP/s (dec) | FPS 1080p | PSNR | SSIM |
|---------|--------------|---------------|---------------|------------|-----------|------------|------------|------------|-----------|------|------|
| Kodak | 17.2 | 86 ± 4.2 | 42 ± 1.8 | 4.8 | 1.9 | 7.1 | 4.6 | 9.4 | 4.5 | 31.2 | 0.93 |

Hardware: `Ryzen 7 5800X (8C/16T), 32GB DDR4-3200, Ubuntu 24.04, rustc 1.82.0, RAYON_NUM_THREADS=8, governor=performance`.

Interpretation: PIXLZR 17.2× at SSIM 0.93 vs JPEG 4.8× at SSIM 0.92 — ~3.6× denser at matched quality on this dataset. Not "95% vs BMP" but defensible and strong.

---

## 6. Gnuplot Stub (optional chart)

Save as `benches/plot.gnu`:

```gnuplot
set terminal pngcairo size 960,540 enhanced font "Inter,10"
set output "assets/bench-chart.png"
set title "PIXLZR vs JPEG/PNG/WebP — Ratio at SSIM 0.92 (Kodak)"
set style data histogram
set style histogram clustered gap 1
set style fill solid 0.9 border -1
set boxwidth 0.9
set xtics rotate by -15
set ylabel "Compression ratio (higher = smaller file)"
set yrange [0:*]
plot "benches/bench-table.md" using 2:xtic(1) title "PIXLZR" , \
     "" using 5:xtic(1) title "JPEG", \
     "" using 7:xtic(1) title "WebP"
# For real use, export benches/data.csv from bench.txt and plot that CSV instead of parsing md.
```

Simpler: use `criterion-table` or `benchmark-action` chart on `gh-pages` — the md table is the source of truth, the PNG is just for README inline.

---

*Template version: 2026-09-01. Source spec: `benches/README.md` + `docs/pixlzr-bench-runbook.md`. Update this file, not README, when re-running benches — then copy `§2` into README.*
