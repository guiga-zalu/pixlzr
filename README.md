# PIXLZR - Custom Image Codec & Format in Rust

English | [Português (BR)](README.pt-BR.md)

<!-- TODO Phase 2.3/2.4: badges will turn green after CI + Pages land -->
![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)
![CI](https://github.com/guiga-zalu/pixlzr/actions/workflows/ci.yml/badge.svg)
![Pages](https://github.com/guiga-zalu/pixlzr/actions/workflows/pages.yml/badge.svg)
![Benchmarks](https://img.shields.io/badge/benchmarks-todo-lightgrey)

> **One-liner:** High-density-aware image compression - custom format, multithreaded Rust encoder/decoder, and 60fps real-time video filtering. Built to preserve detail where it matters.

**🎮 Live Demo (WASM):** https://guiga-zalu.github.io/pixlzr/ - _drag-drop any PNG/JPEG → PIXLZR encode → decode in-browser, no install. [How it works](docs/wasm-demo-spec.md)._

![Demo](assets/demo.gif)
<!-- TODO Phase 2.3: replace with 15s screen capture - drag image → PIXLZR encode/decode + ratio/SSIM overlay. Until WASM lands this 404 is expected and intentional. -->
<!-- GIF spec: 1280×720, 15s, <4MB, 10–12fps, 128 colors, before/after split + stats panel. See assets/demo-script.md (storyboard) + assets/demo-gif-spec.md (file spec, ffmpeg export, Featured integration). -->

---

## What is PIXLZR?

PIXLZR is an experimental image codec that uses an **information density function** to allocate bits where the eye is most sensitive (edges, high-frequency regions), trading uniform quality for perceptually better results at high compression.

- **Custom format** with encoder/decoder (2023) - see [Architecture](docs/architecture.md) and [Format Spec](docs/format.md)
- **Rust core** (2022 port), **multithreaded pipeline** (2024, Rayon)
- **Video path:** real-time filtering at 60fps (1080p target - see [Benchmarks](#benchmarks))
- Viewer patch for an OSS image viewer to natively open `.pixlzr` files

> **Status:** Research prototype - API may change. Benchmarks vs JPEG/PNG/WebP at matched SSIM are on the roadmap (see below).

---

## Quick Start

```bash
# Requires Rust stable (1.75+)
git clone https://github.com/guiga-zalu/pixlzr.git
cd pixlzr
cargo install .

pixlzr --help
# example (CLI is illustrative - adjust to your binary name/args):
pixlzr -i input.png -o output.pixlzr
pixlzr -i output.pixlzr -o restored.png
```

Or via WASM demo: open https://guiga-zalu.github.io/pixlzr/ and drop an image - see [WASM Demo Spec](docs/wasm-demo-spec.md).

---

## Architecture

```
Input image
↓
Detail density map
↓
Quantization
↓
Multiple types of entropy encoding
↓
.pixlzr
↓
Entropy decoding -> Decode
↓
Dequantize + reconstruct → Output
```

Full pipeline, threading model and density function → [`docs/architecture.md`](docs/architecture.md).

---

## Format Spec (stub)

- Magic: `PIXLZR` + 3 octets of semantic versioning
- Header: width, height, colour space, line starts
- Blocks: Block header + payload

Full spec → [`docs/format.md`](docs/format.md) _(todo Phase 2)_.

---

## Benchmarks

> **Claims on the CV/old README ("95% reduction", "60fps") are anecdotal until this table is filled with `criterion` runs on pinned hardware.** This section exists to make the gap explicit and verifiable.

| Dataset | PIXLZR (ratio @ SSIM 0.92) | JPEG | PNG | WebP | Enc ms | Dec ms | FPS 1080p |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Kodak (24× 768×512) | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ |
| CLIC valid | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ |
| Synthetic high-density | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ | _todo_ |

**How to reproduce (Phase 2.4):**

```bash
cargo bench --features bench
# hardware: CPU model, cores, RAM, OS, rustc --version, RAYON_NUM_THREADS
```

Table template: [`benches/bench-table.md`](benches/bench-table.md) (reproducible table, hardware header, PSNR/SSIM + MP/s + FPS 1080p, vs JPEG/PNG/WebP at matched quality) · Runbook: [`docs/pixlzr-bench-runbook.md`](../docs/pixlzr-bench-runbook.md) or `docs/pixlzr-bench-runbook.md` in pipeline repo (criterion, hardware spec, `RAYON_NUM_THREADS`, `gnuplot`)

See [`benches/README.md`](benches/README.md) for harness spec and `docs/pixlzr-audit.md §3` for full bench design (datasets, SSIM/PSNR matching, hardware disclosure).

---

## Roadmap

- [x] 2021 - density-function codec prototype
- [x] 2022 - Rust port
- [x] 2023 - custom format + encoder/decoder + viewer patch
- [x] 2024 - multithreaded pipeline + 60fps video filter
- [ ] More internal block formats (delta, transform (DCT, Haar), composed)
- [ ] Fractal tree block format
- [ ] **P0** - MIT OR Apache-2.0, topics, EN README, badges, Pages placeholder (this patch)
- [ ] **Phase 2.3** - WASM live demo on GitHub Pages ([spec](docs/wasm-demo-spec.md))
- [ ] **Phase 2.4** - `criterion` harness vs JPEG/PNG/WebP, CI (`fmt`/`clippy`/`test`/`bench`) ([spec](benches/README.md))

---

## Contributing

Issues and PRs welcome. Please run before submitting:

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test --all-features
```

CI enforces the same - see [`.github/workflows/ci.yml`](.github/workflows/ci.yml).

---

## License

Dual-licensed under **MIT OR Apache-2.0** - see [`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).

`SPDX-License-Identifier: MIT OR Apache-2.0` - Copyright (c) 2021–2026 Guilherme Zaluchi

---

## Citation / Author

Guilherme Zaluchi - [@guiga-zalu](https://github.com/guiga-zalu) · [LinkedIn](https://linkedin.com/in/guilherme-alves-c-zaluchi) · SP, Brasil
