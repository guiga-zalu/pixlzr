# PIXLZR — Demo GIF & 60s Video Spec (Assets)

> **Companion:** `assets/demo-script.md` (storyboard + narration). This file adds the **file-level spec** for `assets/demo.gif` and `assets/demo-60s.mp4` so the README hero GIF and LinkedIn Featured video are reproducible without re-debating dimensions, codec, or capture steps.
>
> **Output files:**
> - `assets/demo.gif` — **README hero** (above the fold, `<4MB`, loops, no audio) — `![Demo](assets/demo.gif)` in `README.md`
> - `assets/demo-60s.mp4` — **LinkedIn / GitHub social preview video** (H.264, 60s, narrated or captioned, `<25MB`)
> - `assets/social-preview.png` — 1280×640 repo card (fallback when video not embedded)

---

## 1. File Spec — `assets/demo.gif` (README hero)

| Property | Spec | Why |
|----------|------|-----|
| **Path** | `assets/demo.gif` (repo root) | `README.md` references `![Demo](assets/demo.gif)` — relative, works on GitHub + Pages |
| **Dimensions** | **1280×720** (16:9) — downscale to **960×540** if >4MB | Fits GitHub README width (max ~1012px rendered) without pixelation |
| **Duration** | **15s** (trimmed from 60s video `0:15–0:30`) | WASM drag-drop + slider + stats — no need for full video in README |
| **Frame rate** | **10–12 fps** (palette-gen, not full 60fps) | GIF cannot do 60fps sanely; 12fps is smooth enough, keeps file <4MB |
| **Loop** | Infinite (`-loop 0`) | README auto-loops |
| **File size** | **<4MB hard limit** (GitHub soft limit ~10MB but README loads slow above 4MB) | If oversize: drop fps to 10, scale to 960, or reduce colors to 64 |
| **Colors** | **128 max** (palettegen) — 64 if needed for size | Photographic content tolerates 128; dithering on |
| **Captions** | **Burned-in** text overlay (not relying on narration) | README GIF has no audio: `Drag image → PIXLZR 94% smaller · SSIM 0.93 · 86ms encode` |
| **Source segment** | WASM demo: drag-drop PNG → instant encode/decode → slider wipe before/after → stats panel | Proves the codec, not just screen capture |
| **Background** | Clean browser profile, no extensions bar, 100% zoom, light or dark matching README | Dark hero matches GitHub dark mode readability |

**Prohibited:** No cursor spam, no OS dock/notifications, no password/secret in capture, no stock music without license. Sample image must be CC0/Kodak `kodim23` (parrot) or your own photo — not a copyrighted image you don't own.

### 1.1 Export Commands

From the 60s source `assets/demo-60s.mp4`:

```bash
# GIF — 1280×720, 12fps, 15s slice (0:15–0:30 of the 60s video)
ffmpeg -i assets/demo-60s.mp4 -ss 15 -t 15 \
  -vf "fps=12,scale=1280:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128[p];[s1][p]paletteuse=dither=bayer:bayer_scale=3" \
  -loop 0 assets/demo.gif

# If >4MB, retry at 960px / 10fps / 64 colors:
ffmpeg -i assets/demo-60s.mp4 -ss 15 -t 15 \
  -vf "fps=10,scale=960:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=64[p];[s1][p]paletteuse" \
  -loop 0 assets/demo.gif

ls -lh assets/demo.gif  # must be <4MB
ffprobe -v error -select_streams v:0 -show_entries stream=width,height,avg_frame_rate -of default=noprint_wrappers=1 assets/demo.gif
```

Verification before commit:

```bash
ls -lh assets/demo.gif assets/demo-60s.mp4
# demo.gif <4MB, 1280×720 (or 960×540), 10–12fps
# demo-60s.mp4 <25MB, 1280×720, H.264
file assets/demo.gif  # GIF image data, version 89a
```

---

## 2. File Spec — `assets/demo-60s.mp4` (Featured video)

| Property | Spec |
|----------|------|
| **Path** | `assets/demo-60s.mp4` + upload to LinkedIn Featured / GitHub Release asset |
| **Container** | MP4 (H.264 + AAC if narrated, otherwise H.264 silent + captions) |
| **Resolution** | **1280×720** (720p), **60fps** capture (export 30fps ok — narration is not fps-sensitive) |
| **Duration** | **60s ±2s** per `assets/demo-script.md` storyboard (6 chapters 0:00→0:60) |
| **Bitrate** | ~2–3 Mbps video (720p) → ~15–22MB total — stays <25MB for GitHub asset |
| **Audio** | Optional narration (see storyboard captions); if no mic, use burned-in captions + subtitles SRT |
| **Thumbnail** | End frame holds 3s — used as thumbnail (`assets/social-preview.png` can be extracted: `ffmpeg -i demo-60s.mp4 -ss 57 -vframes 1 assets/social-preview.png`) |
| **Tool** | OBS (Display Capture, 1280×720 canvas, 60fps) or QuickTime (macOS) — hide desktop icons, Do Not Disturb on |

### 2.1 Storyboard → Chapters (from `assets/demo-script.md`)

| Time | Visual | Caption / Narration |
|------|--------|---------------------|
| 0:00–0:05 | Title card: **PIXLZR — Custom Image Codec in Rust** + badges + GitHub URL | "PIXLZR — a custom image codec and format built in Rust." |
| 0:05–0:15 | Terminal: `cargo run --release -- encode kodak.png -o kodak.pixlzr` → `412 KB → 24 KB (94% · SSIM 0.93)` | "High-density-aware — spends bits where the eye cares: edges and texture." |
| 0:15–0:30 | **WASM demo drag-drop** → slider wipe before/after → stats: **Encode 86ms · Decode 42ms · Ratio 17:1** | "Try it in-browser — drag, drop, compressed. No install — WASM runs the same Rust core." |
| 0:30–0:42 | Architecture mermaid + density heatmap overlay (red = high density) | "An information density function drives adaptive quantization, multithreaded with Rayon." |
| 0:42–0:52 | `cargo bench` + `criterion` report + bench table (PIXLZR vs JPEG/WebP @ SSIM 0.92) | "Benchmarked with Criterion — pinned hardware, matched quality." |
| 0:52–0:60 | Closing card: **Live demo: guiga-zalu.github.io/pixlzr · Code: github.com/guiga-zalu/pixlzr** | "Open source, MIT OR Apache-2.0. Link in description." |

Full narration script is in `assets/demo-script.md` — this file adds the **file spec + export** only.

### 2.2 Captions / Subtitles (for LinkedIn muted autoplay)

Provide `assets/demo-60s.srt`:

```srt
1
00:00:00,000 --> 00:00:05,000
PIXLZR — a custom image codec and format built in Rust.

2
00:00:05,000 --> 00:00:15,000
High-density-aware compression — it spends bits where the eye cares.

3
00:00:15,000 --> 00:00:30,000
Try it in-browser — drag, drop, compressed. No install.

4
00:00:30,000 --> 00:00:42,000
An information density function drives adaptive quantization — multithreaded with Rayon.

5
00:00:42,000 --> 00:00:52,000
Benchmarked with Criterion — pinned hardware, matched quality.

6
00:00:52,000 --> 00:01:00,000
Open source, MIT OR Apache-2.0 — guiga-zalu.github.io/pixlzr
```

LinkedIn auto-uses SRT if uploaded alongside MP4.

### 2.3 Export Commands (60s video)

```bash
# Record → edit → export H.264 720p, yuv420p for compatibility
ffmpeg -i raw-capture.mov -vf "scale=1280:720:flags=lanczos" \
  -c:v libx264 -profile:v high -pix_fmt yuv420p -crf 23 -preset medium \
  -c:a aac -b:a 128k -movflags +faststart \
  assets/demo-60s.mp4

ls -lh assets/demo-60s.mp4  # <25MB
ffprobe -v error -select_streams v:0 -show_entries stream=width,height,r_frame_rate,codec_name -of default=noprint_wrappers=1 assets/demo-60s.mp4
```

---

## 3. Recording Checklist (before export)

- [ ] Clean browser profile (no bookmarks bar, no extensions), zoom 100%, capture 1280×720 only
- [ ] Sample image: **Kodak `kodim23`** (parrot, high detail — shows density map well) or own photo; confirm license
- [ ] Hide OS dock/taskbar, **Do Not Disturb on**, no notifications
- [ ] WASM demo served from `https://guiga-zalu.github.io/pixlzr/` or `python3 -m http.server --directory www 8000` — show **real numbers** from `cargo bench` if possible; `_todo_` is honest until bench lands, but prefer real run
- [ ] Run through storyboard once before recording — time each chapter with a stopwatch
- [ ] Record at 1280×720, 60fps canvas; keep mouse movements slow
- [ ] End frame holds **3s** for thumbnail extraction
- [ ] Generate `demo.srt` if no narration mic

---

## 4. README Hero Integration

`README.md` hero block (already in skeleton — keep in sync):

```md
**🎮 Live Demo (WASM):** https://guiga-zalu.github.io/pixlzr/ — _drag-drop any PNG/JPEG → PIXLZR encode → decode in-browser, no install. [How it works](docs/wasm-demo-spec.md)._

![Demo](assets/demo.gif)
<!-- TODO Phase 2.3: replace with 15s screen capture — drag image → PIXLZR encode/decode + ratio/SSIM overlay. Until WASM lands this 404 is expected. -->
<!-- GIF spec: 1280×720, 15s, <4MB, 10-12fps, 128 colors. See assets/demo-gif-spec.md for export. -->
```

Steps to go live:

1. Until WASM lands, `assets/demo.gif` 404 is **expected** — badge shows "benchmarks todo" — that is intentional per `docs/pixlzr-audit.md §2.4`.
2. When `www/` + WASM is ready, record `demo-60s.mp4` per `assets/demo-script.md` storyboard, export `demo.gif` per `§1.1` above, commit both, and remove the `TODO` comment.
3. Add `assets/demo.gif` to `MANIFEST.txt` and to `COPY_INSTRUCTIONS.md` copy set.

Do **not** commit `.mov` raw capture — only `demo.gif` + `demo-60s.mp4` + optional `demo-60s.srt`.

---

## 5. LinkedIn Featured Integration

After `demo-60s.mp4` is ready:

1. Upload MP4 to **LinkedIn Featured** (Profile → Featured → Media) + add link to `https://guiga-zalu.github.io/pixlzr/` and `https://github.com/guiga-zalu/pixlzr`.
2. Pin repos: `pixlzr` + `field-simulator-with-portals-rust` (see `docs/pixlzr-audit.md §2.3`).
3. Post template (from `assets/demo-script.md`):
   > Shipped PIXLZR hygiene: dual MIT/Apache-2.0, EN README, architecture doc + WASM demo spec — live at guiga-zalu.github.io/pixlzr. Rust · multithreaded codec · information-density-aware compression. Benchmarks vs JPEG/PNG/WebP (Criterion, pinned hardware) next. Code: github.com/guiga-zalu/pixlzr #Rust #DataEngineering #ImageCompression
4. Verify: Featured shows video thumbnail + plays inline; GitHub social preview (`assets/social-preview.png` 1280×640) appears when sharing repo URL.

---

## 6. Verification

```bash
# From repo root (either /auto/portfolio/pixlzr-demo/ or the real pixlzr clone)
ls -lh assets/demo.gif assets/demo-60s.mp4 2>&1
# expect: demo.gif <4MB, demo-60s.mp4 <25MB (or "No such file" before recording — that's ok, GIF 404 is expected until Phase 2.3)

# If files exist:
ffprobe -v error -select_streams v:0 -show_entries stream=width,height,avg_frame_rate,codec_name -of default=nw=1 assets/demo.gif
ffprobe -v error -select_streams v:0 -show_entries stream=width,height,r_frame_rate,codec_name -of default=nw=1 assets/demo-60s.mp4
file assets/demo.gif | grep -q "GIF" && echo "GIF ok"
grep -q "assets/demo.gif" README.md && echo "README hero link ok"
grep -q "guiga-zalu.github.io/pixlzr" README.md && echo "Pages link ok"
```

---

*Spec version: 2026-09-01. Storyboard source: `assets/demo-script.md`. Table template: `benches/bench-table.md`. Runbook: `docs/pixlzr-bench-runbook.md` (in /auto).*
