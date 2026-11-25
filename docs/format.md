# PIXLZR Format v1 — Specification

Status: **implemented** by `crates/pixlzr-core` (native + WASM). This document is
normative for files with version byte `1`. Any mismatch between this document and
the code is a bug in one of them — the code is tested against this layout.

- Container: single blob, little-endian throughout.
- Model: lossy, 8×8 block DCT on reversible YCoCg, per-block adaptive quantization,
  zlib-compressed coefficient stream.
- Alpha: **not coded**. Decoders restore alpha as `255`.
- Colour: YCoCg is applied to non-linear (gamma-encoded) R'G'B' samples; no ICC
  profile, colour-space tag or metadata is stored.

---

## 1. Limits

| Constraint | Value |
|---|---|
| Width / height | `1..=8192` |
| Pixels (`width * height`) | `<= 16 777 216` (16.7 MP, e.g. 4096×4096) |
| Block size | `8` (the only value defined by v1) |
| Quality | `0..=100` (clamped by the encoder, informational) |
| Quant step | `4..=64` per block (Y), `4..=64` for chroma |

Encoders and decoders MUST reject blobs outside these limits with an error
(`BadDimensions`, `BadBlockSize`). They MUST NOT panic on malformed input.

---

## 2. Header

The header is 27 bytes, followed by a per-block step table. All integers are
unsigned little-endian.

| Offset | Size | Field | Value / meaning |
|---:|---:|---|---|
| 0 | 6 | `magic` | ASCII `PIXLZR` = `50 49 58 4C 5A 52` |
| 6 | 1 | `version` | `1` |
| 7 | 2 | `flags` | `0`; non-zero is reserved and rejected |
| 9 | 4 | `width` | image width in pixels |
| 13 | 4 | `height` | image height in pixels |
| 17 | 1 | `block_size` | `8` |
| 18 | 1 | `quality` | `0..=100`, informational (stored so tools can report the setting) |
| 19 | 4 | `blocks_x` | `ceil(width / 8)` |
| 23 | 4 | `blocks_y` | `ceil(height / 8)` |
| 27 | `blocks_x * blocks_y` | `steps` | one `u8` quant step per block, raster order |
| `27 + blocks_x*blocks_y` | rest | `payload` | zlib stream (§5) |

Validation rules:

- `blocks_x` and `blocks_y` MUST equal `ceil(width / 8)` and `ceil(height / 8)`.
  A header that disagrees is malformed (`BadDimensions`), not merely unusual.
- `flags` MUST be `0`. Future versions may reuse it; v1 decoders reject non-zero.
- A blob shorter than `27 + blocks_x*blocks_y` bytes is `Truncated`.
- There is **no CRC** in v1. Corruption is detected only if zlib or the
  coefficient-length check fails.

### Worked header example

A 13×9 fully black image at quality 50 encodes to exactly 54 bytes:

```text
50 49 58 4c 5a 52   magic "PIXLZR"
01                  version 1
00 00               flags 0
0d 00 00 00         width  = 13
09 00 00 00         height = 9
08                  block_size 8
32                  quality 50
02 00 00 00         blocks_x = 2  (ceil(13/8))
02 00 00 00         blocks_y = 2  (ceil(9/8))
04 04 04 04         steps: 4 blocks, all step 4
78 9c ed c0 ...     zlib payload (23 bytes)
```

Why step 4 for every block: density is zero for a black image, so
`round((4 + 0) * (1.2 - 0.6*0.5)) = round(3.6) = 4`. All 4 × 3 × 64 coefficients
are zero; the 1536-byte zero stream compresses to 23 bytes of zlib framing at
level 6. Full blob (54 bytes):

```text
50 49 58 4c 5a 52 01 00 00 0d 00 00 00 09 00 00 00 08 32 02 00 00 00 02 00
00 00 04 04 04 04 78 9c ed c0 01 01 00 00 00 82 20 ff af 6e 48 50 00 3c 1a
06 00 00 01
```

---

## 3. Colour transform (reversible YCoCg)

Input samples are 8-bit R, G, B. The transform is exact integer arithmetic
(the `>>` is an arithmetic shift, i.e. floor division by 4):

```text
Y  = (R + 2G + B) >> 2        G = Y - ((Cb + Cr) >> 2)
Cb = B - G                    R = Cr + G
Cr = R - G                    B = Cb + G
```

Ranges: `Y` ∈ `0..=255`, `Cb`/`Cr` ∈ `-255..=255`. After decoding, R/G/B are
clamped to `0..=255` (DCT quantization can overshoot). Alpha is ignored on
encode and written as `255` on decode.

Note: this is the "reversible" integer variant used by the v1 prototype. It is
not identical to the floating-point YCoCg-R lifting scheme.

---

## 4. Blocks, density and quantization

### 4.1 Block partition and padding

The image is split into `blocks_x × blocks_y` blocks of 8×8, visited in raster
order (row 0 left→right, then row 1, …). Partial blocks on the right/bottom are
padded by **replicating the edge pixel**:

```text
src_x = min(block_x * 8 + x, width  - 1)
src_y = min(block_y * 8 + y, height - 1)
```

Padding samples are used for the forward DCT but are **cropped** on decode, so
the output is exactly `width × height` pixels.

### 4.2 Density measure (Y plane only)

For the padded 8×8 luma block `Y[y][x]`, sum the forward differences to the right
and down (edge pairs that leave the block are skipped):

```text
S = Σ_{y=0..7} Σ_{x=0..6} |Y[y][x+1] - Y[y][x]|
  + Σ_{y=0..6} Σ_{x=0..7} |Y[y+1][x] - Y[y][x]|
d = clamp(S / 4096.0, 0.0, 1.0)
```

`4096 = 64 × 64` normalises "one full-scale 255 gradient per edge pair" to 1.
Flat regions approach `d = 0`; dense texture/noise approaches `d = 1`.

### 4.3 Quant step

```text
base  = 4 + 28 * d
qf    = 1.2 - 0.6 * (quality / 100)
stepY = clamp(round(base * qf), 4, 64)
stepC = clamp(2 * stepY, 4, 64)
```

The step is stored per block in the header table and applies to all three planes
of that block (chroma uses `stepC`). Higher `quality` lowers the step; higher
local density raises it. Because steps are explicit in the file, decoders never
need to re-run the density function.

---

## 5. Payload

### 5.1 Coefficient order

For each block in raster order, for each plane in the order **Y, Cb, Cr**, 64
quantized coefficients are written in **JPEG zigzag order**, each as a signed
16-bit little-endian integer (`i16`):

```text
for each block:
  for plane in [Y, Cb, Cr]:
    for i in 0..64:
      i16_le( coefficient[ zigzag[i] ] )
```

Uncompressed payload size is therefore exactly
`blocks_x * blocks_y * 3 * 64 * 2` bytes (384 bytes per block).

### 5.2 Zigzag map

`zigzag[i]` is the raster index (`v*8 + u`, where `v` is the vertical frequency)
of the i-th coefficient:

```text
 0  1  8 16  9  2  3 10 17 24 32 25 18 11  4  5
12 19 26 33 40 48 41 34 27 20 13  6  7 14 21 28
35 42 49 56 57 50 43 36 29 22 15 23 30 37 44 51
58 59 52 45 38 31 39 46 53 60 61 54 47 55 62 63
```

(The same sequence, written as one array, is `dct::ZIGZAG` in the source.)

### 5.3 Compression

The whole coefficient stream is compressed as **one zlib stream** (RFC 1950:
2-byte header, DEFLATE data, 4-byte Adler-32) using
`miniz_oxide::deflate::compress_to_vec_zlib(bytes, 6)` (level 6). The stream
extends to the end of the file; there is no trailing data.

Decoder requirements:

1. Inflate the payload with an output size limit equal to
   `blocks_x * blocks_y * 384`, so a malformed/crafted stream cannot allocate
   unbounded memory.
2. Require the inflated length to equal that value exactly; otherwise return
   `Truncated` (or `Zlib` if the limit was exceeded).
3. Dequantize with `coefficient * step` (the same `step`/`stepC` used on encode)
   and run the inverse DCT.

---

## 6. DCT

The transform is the orthonormal 8×8 DCT-II, separable:

```text
O[k][n] = alpha(k) * cos(pi * (2n + 1) * k / 16)
alpha(0) = sqrt(1/8), alpha(k) = sqrt(2/8)

C[v][u] = Σ_y Σ_x B[y][x] * O[v][y] * O[u][x]     (forward)
B[y][x] = Σ_v Σ_u O[v][y] * C[v][u] * O[u][x]     (inverse, transpose)
```

Quantization: `q = round(C / step)`, clamped to `i16` range (`-32768..=32767`)
before serialization. This clamp is unreachable for legal 8-bit inputs, but is
part of the encoder for safety.

---

## 7. Decode procedure

1. Parse and validate the header (§2).
2. Read the step table (one byte per block).
3. Inflate the payload and check its exact length.
4. Allocate three float planes of `width * height`.
5. For each block in raster order, for each plane: dequantize, inverse DCT,
   copy only the in-image part of the 8×8 block into the plane.
6. Apply the inverse YCoCg transform per pixel, clamp RGB to `0..=255`,
   set alpha to `255`.
7. Return the RGBA8 buffer of `width * height * 4` bytes.

Header-only inspection (`inspect` / `image_info`) reads and validates everything
up to and including the step table, but does not inflate the payload.

---

## 8. Errors

`PixlzrError` variants (and their meaning):

| Variant | Condition |
|---|---|
| `BadMagic` | first six bytes are not `PIXLZR` |
| `UnsupportedVersion(u8)` | version byte is not `1` |
| `UnsupportedFlags(u16)` | reserved flags are non-zero |
| `Truncated { expected, actual }` | header/step table too short, or inflated coefficient stream has the wrong length |
| `BadDimensions { width, height }` | zero/oversized dimensions, or block grid disagrees with `ceil(w/8)`/`ceil(h/8)` |
| `BadBlockSize(u8)` | block size byte is not `8` |
| `Zlib(String)` | the zlib stream could not be inflated |
| `InvalidImageData { expected, actual }` | encoder received fewer than `w*h*4` RGBA bytes |

`PixlzrError` implements `Display` and `std::error::Error`. WASM exports convert
it to a JavaScript string (a thrown `JsValue`).

---

## 9. Known limitations (v1)

- **Lossy** at every quality; there is no lossless mode.
- **Alpha is discarded** (restored opaque).
- `quality` only scales the quantization step; it is not a calibrated
  perceptual-quality target and does not guarantee a bitrate.
- **No entropy coding beyond zlib**; a real codec would use arithmetic/rANS
  coding, adaptive scan order, or a context model.
- **No CRC/checksum** and no integrity guarantee beyond zlib.
- **Whole-file buffering**: encode/decode allocate the full image and coefficient
  stream in memory; there is no streaming or partial decode.
- **Single-threaded**: no Rayon/worker pool; the WASM demo cannot use threads.
- **8×8 fixed blocks**, no quadtree or adaptive block size.
- **No metadata**, colour profiles, animation or multi-image support.
- The density function is a simple luma-gradient mean; the α/β/γ "information
  density" model described in early design notes is **not** implemented.
- Format v1 stores `quality` but decoders must not rely on it for reconstruction;
  the per-block steps are authoritative.

---

## 10. Versioning

- The version byte gates all layout changes. A v2 may change field widths, add a
  checksum, or introduce adaptive block sizes, but must remain distinguishable
  from v1 by byte 6.
- v1 decoders reject unknown versions rather than guessing.
- The magic is `PIXLZR`; file extension is `.pixlzr` (not registered).
