# PIXLZR — Density-Aware Image Codec in Rust + WASM

English | [Português (BR)](README.pt-BR.md)

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)
![CI](https://github.com/guiga-zalu/pixlzr/actions/workflows/ci.yml/badge.svg)
![Pages](https://github.com/guiga-zalu/pixlzr/actions/workflows/pages.yml/badge.svg)
![Benchmarks](https://img.shields.io/badge/benchmarks-pending-lightgrey)

> **One-liner:** an image codec with per-block adaptive quantization — 8×8 DCT blocks whose quant step is chosen from local luma gradient density. Smooth blocks keep finer steps (banding is visible there); busy blocks are quantized more coarsely (noise is masked). Custom format, Rust core, CLI, and an in-browser WASM demo that actually encodes and reconstructs the image.

**🎮 Live demo (WASM):** https://guiga-zalu.github.io/pixlzr/ — drag-drop a PNG/JPEG → `.pixlzr` encode → decode → before/after with ratio and timings. Everything runs locally; nothing is uploaded. Build/run instructions below.

> **Demo GIF:** not recorded yet. The storyboard and file spec live in [`assets/demo-script.md`](assets/demo-script.md) and [`assets/demo-gif-spec.md`](assets/demo-gif-spec.md).

---

## What is PIXLZR?

PIXLZR v1 is a small, honest, dependency-light image codec:

- **Format v1** — 27-byte header + per-block quant steps + one zlib stream. Byte-level spec: [`docs/format.md`](docs/format.md).
- **Real transform** — reversible integer YCoCg, orthonormal 8×8 DCT-II, zigzag scan, `i16` coefficients.
- **Density-adaptive quantization** — each block's step comes from the mean absolute luma gradient in that block: smooth blocks get finer steps (banding is visible in flats), busy blocks get coarser steps (quantization noise is masked by texture). The exact formula is in the spec and in code.
- **Native API + CLI** — `encode_rgba` / `decode_rgba` / `inspect`, plus `pixlzr encode|decode|info`.
- **WASM demo** — the same Rust code compiled to `wasm32-unknown-unknown`, driven by a vanilla ES-module front end (no bundler).
- **Tests + benches** — round-trip PSNR/ratio assertions, header fuzz-ish rejection tests, and Criterion benches.

**Explicitly not implemented yet** (do not assume otherwise):

- Multithreading / Rayon. Both the native path and the WASM demo are **single-threaded**. WASM is single-thread because the demo does not use `SharedArrayBuffer`.
- The 60 fps video filter, the image-viewer patch, and the learned α/β/γ density model from early design notes.
- SSIM, perceptual metrics, and matched-quality comparisons against JPEG/PNG/WebP.
- Lossless mode, alpha coding, metadata, streaming decode, and CRC.

The early design documents ([`docs/architecture.md`](docs/architecture.md), [`docs/wasm-demo-spec.md`](docs/wasm-demo-spec.md)) are kept for context, with status notes where they describe plans rather than shipped code.

---

## Quick start

```bash
# Requires stable Rust. No system image libraries for the core.
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features

# CLI (PNG <-> .pixlzr)
cargo build --release --features cli
./target/release/pixlzr encode input.png output.pixlzr --quality 80
./target/release/pixlzr info output.pixlzr
./target/release/pixlzr decode output.pixlzr restored.png

# Benchmarks (Criterion)
cargo bench --features bench
```

### WASM demo locally

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --locked   # or use your distro package

wasm-pack build crates/pixlzr-core --target web --out-dir ../../www/pkg -- --features wasm
python3 -m http.server --directory www 8000
# open http://localhost:8000 and drop an image
```

If `www/pkg/` is missing, the demo page shows the build command instead of a stack trace.

---

## Architecture

```
RGBA8 ──► reversible YCoCg ──► 8×8 DCT-II ──► quantize (step from local density)
                                                         │
                                                         ▼
                                          zigzag i16 coefficients ──► zlib
                                                         │
   .pixlzr  =  header + per-block steps + zlib payload ◄──┘

Decode: parse header → inflate → dequantize → IDCT → inverse YCoCg → clamp → RGBA8
```

Everything lives in `crates/pixlzr-core`; the crate has no I/O (the CLI does).
The WASM exports are thin wrappers over the same native functions, so the demo
cannot drift from the codec. Details: [`docs/architecture.md`](docs/architecture.md).

---

## Format spec

Header (little-endian):

| Field | Size |
|---|---|
| magic `PIXLZR` + version `1` | 7 bytes |
| flags (`0`) | 2 bytes |
| width, height | 4 bytes each |
| block size (`8`), quality | 1 byte each |
| blocks_x, blocks_y | 4 bytes each |
| per-block quant steps | `blocks_x * blocks_y` bytes |
| zlib stream of zigzag `i16` coefficients | rest of file |

Full normative spec, including the density/quantization formula, zigzag table,
padding rule, worked hex example and limitations: [`docs/format.md`](docs/format.md).

---

## Benchmarks

> **No benchmark numbers are published yet.** The table below is intentionally
> empty; rows will be filled only from Criterion runs on pinned hardware, at
> matched quality, with the raw output committed. The self-tests assert
> `PSNR > 30 dB` and `raw/encoded > 2:1` at quality 80 on a synthetic gradient.

| Dataset | PIXLZR (ratio @ matched quality) | JPEG | PNG | WebP | Enc ms | Dec ms | Notes |
|---|---|---|---|---|---|---|---|
| Kodak (24× 768×512) | _pending measurement_ | — | — | — | — | — | baseline comparisons not implemented |
| CLIC valid | _pending measurement_ | — | — | — | — | — | dataset not vendored |
| Synthetic 512×512 (gradient + LCG noise) | _pending measurement_ | — | — | — | — | — | `cargo bench --features bench` |

Reproduce the harness:

```bash
cargo bench --features bench
# criterion prints mean/median timing estimates; raw JSON under target/criterion/
```

The benchmark input is generated deterministically in `benches/codec.rs`
(gradient + LCG noise), so runs are comparable across machines. Template for
the eventual matched-quality table: [`benches/bench-table.md`](benches/bench-table.md);
harness notes: [`benches/README.md`](benches/README.md).

---

## Roadmap

- [x] 2021–2022 — density-function experiments, Rust port
- [x] 2023 — custom format prototype
- [x] **v1 (this repo state)** — real format + codec, reversible YCoCg, adaptive
  per-block quantization, native API, CLI (`encode`/`decode`/`info`), WASM
  exports, drag-drop demo, round-trip tests, Criterion benches, CI + Pages
- [ ] Lossless mode and alpha coding
- [ ] Matched-quality benchmarks vs JPEG/PNG/WebP (SSIM/PSNR gate)
- [ ] Multithreaded encode/decode (Rayon) and a streaming decoder
- [ ] Better entropy coding (rANS/arithmetic) and adaptive block sizes
- [ ] Viewer integration for `.pixlzr`

---

## Contributing

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo bench --no-run --features bench
```

CI runs the same checks (`.github/workflows/ci.yml`). Format changes must update
[`docs/format.md`](docs/format.md) in the same PR.

---

## License

Dual-licensed under **MIT OR Apache-2.0** — see [`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).

`SPDX-License-Identifier: MIT OR Apache-2.0` — Copyright (c) 2021–2026 Guilherme Alves Ceobaniuk Zaluchi

---

## Citation / Author

Guilherme Zaluchi — [@guiga-zalu](https://github.com/guiga-zalu) · [LinkedIn](https://linkedin.com/in/guilherme-alves-c-zaluchi) · Ribeirão Pires, SP (Remote)
