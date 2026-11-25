# PIXLZR — 60-Second Demo Video Script

> **Deliverable:** 15s GIF for README + 60s narrated video for LinkedIn / GitHub social preview.
> **Source:** screen capture of the local `www/` WASM demo (same page as
> `https://guiga-zalu.github.io/pixlzr/`) and the real CLI.
> **Tools:** OBS or QuickTime screen capture, 1280×720, 60fps capture, export H.264 + GIF via `ffmpeg`.
>
> **Honesty rule for this recording:** only show numbers that are visible on
> screen from the actual run. Do not hard-code ratio/PSNR/timing values in
> captions. There are no published benchmark numbers yet, and the codec is
> single-threaded.

## Storyboard (60s)

| Time | Visual | Narration / Caption |
|------|--------|---------------------|
| 0:00–0:05 | Title card: **PIXLZR — Custom Image Codec in Rust** + badges (Rust, MIT/Apache-2.0) + GitHub URL | "PIXLZR — a custom image codec and format built in Rust." |
| 0:05–0:15 | Terminal: `./target/release/pixlzr encode kodak.png kodak.pixlzr --quality 80` → the tool's real `encoded WxH q80 -> … (N bytes, raw ratio R:1, T ms)` line | "Each 8×8 block gets a quantization step from its local luma gradient — smooth blocks keep finer steps to avoid banding, busy blocks are quantized more coarsely." |
| 0:15–0:30 | Browser WASM demo: drag-drop PNG → encode/decode. Before/after panes. Stats panel shows the real **Encode ms · Decode ms · file KB -> blob KB (ratio:1)** line | "Try it in-browser — drag, drop, compressed. No install: WASM runs the same Rust core as the CLI." |
| 0:30–0:42 | Architecture diagram from `docs/architecture.md` (YCoCg → DCT → density → quantize → zlib) | "Reversible YCoCg, an 8×8 DCT, and a per-block step table written into the file. Single-threaded in v1." |
| 0:42–0:52 | Terminal: `cargo bench --no-run --features bench` then a short `cargo bench --features bench` run showing Criterion's own timing lines | "A Criterion harness is included. Published matched-quality numbers are still pending — I won't quote numbers I haven't measured on pinned hardware." |
| 0:52–0:60 | Closing card: **Live demo: guiga-zalu.github.io/pixlzr** · **Code: github.com/guiga-zalu/pixlzr** · "Author: Guilherme Zaluchi — Data Engineer · Rust · Ribeirão Pires" | "Open source, MIT OR Apache-2.0. Link in description." |

## GIF Cut (15s, for README `assets/demo.gif`)

- Trim to the WASM drag-drop segment (roughly 0:15–0:30 of the 60s recording).
- No audio; burned-in caption should use the numbers shown by the demo at
  capture time, e.g. **"Drag image → PIXLZR encode/decode · ratio and timings in the stats line"**.
- Export:

```bash
ffmpeg -i demo-60s.mp4 -ss 15 -t 15 -vf "fps=12,scale=1280:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128[p];[s1][p]paletteuse" -loop 0 assets/demo.gif
# target <4MB; if oversize reduce fps to 10 or scale to 960
```

## Recording Checklist

- [ ] Clean browser profile (no extensions bar), zoom 100%, dark or light consistent with README.
- [ ] Sample image: Kodak `kodim23` (parrot, high detail) or another image you have the right to publish.
- [ ] Show the actual demo stats line and actual CLI output — do not edit numbers into the frame.
- [ ] 1280×720 capture; hide OS dock/notifications.
- [ ] Add subtitles (SRT) for LinkedIn autoplay (muted by default).
- [ ] End frame holds 3s for thumbnail.

## LinkedIn Post Template (after publishing)

> Shipped a working PIXLZR v1: custom density-aware image format, Rust codec,
> CLI (`encode` / `decode` / `info`), and an in-browser WASM demo you can drag an
> image onto — live at guiga-zalu.github.io/pixlzr.
> 8×8 DCT + reversible YCoCg + per-block quantization driven by local luma gradient.
> Single-threaded in v1; matched-quality benchmarks and multithreading are next.
> Code: github.com/guiga-zalu/pixlzr #Rust #ImageCompression

## Verification

```bash
ls -lh assets/demo.gif assets/demo-60s.mp4
# demo.gif <4MB, 1280×720, 10–12fps
# demo-60s.mp4 <25MB, 1280×720, H.264, 60s ±2s
ffprobe -v error -select_streams v:0 -show_entries stream=width,height,r_frame_rate -of default=noprint_wrappers=1 assets/demo.gif
```

---

*This file is the single source for the video/GIF storyboard. Keep numbers out of
the script until they come from a committed run.*
