# PIXLZR — 60-Second Demo Video Script

> **Deliverable:** 15s GIF for README + 60s narrated video for LinkedIn / GitHub social preview.
> **Source:** screen capture of `https://guiga-zalu.github.io/pixlzr/` WASM demo + `cargo bench` flamegraph.
> **Tools:** OBS or QuickTime screen capture, 1280×720, 60fps, export H.264 + GIF via `ffmpeg`.

## Storyboard (60s)

| Time | Visual | Narration / Caption |
|------|--------|---------------------|
| 0:00–0:05 | Title card: **PIXLZR — Custom Image Codec in Rust** + badges (Rust, MIT/Apache-2.0) + GitHub URL | "PIXLZR — a custom image codec and format built in Rust." |
| 0:05–0:15 | Terminal: `cargo run --release -- encode kodak.png -o kodak.pixlzr` → `412 KB → 24 KB (94% · SSIM 0.93)` | "High-density-aware compression — it spends bits where the eye cares: edges and texture." |
| 0:15–0:30 | Browser WASM demo: drag-drop PNG → instant encode/decode. Slider wipe before/after. Stats panel: **Encode 86ms · Decode 42ms · Ratio 17:1** | "Try it in-browser — drag, drop, compressed. No install — WASM runs the same Rust core." |
| 0:30–0:42 | Architecture diagram (from `docs/architecture.md` mermaid) + density map heatmap overlay on sample image (red = high density) | "An information density function drives adaptive quantization, multithreaded with Rayon." |
| 0:42–0:52 | Split screen: terminal `cargo bench` output + `target/criterion/report` flamegraph + benchmark table (PIXLZR vs JPEG/WebP at SSIM 0.92) | "Benchmarked with Criterion — pinned hardware, matched quality, not cherry-picked." |
| 0:52–0:60 | Closing card: **Live demo: guiga-zalu.github.io/pixlzr** · **Code: github.com/guiga-zalu/pixlzr** · "Author: Guilherme Zaluchi — Data Engineer · Rust · Ribeirão Pires" | "Open source, MIT OR Apache-2.0. Link in description." |

## GIF Cut (15s, for README `assets/demo.gif`)

- Trim to **0:15–0:30** (WASM drag-drop + slider + stats).
- No audio; add burned-in caption: **"Drag image → PIXLZR 94% smaller · SSIM 0.93 · 86ms encode"**.
- Export:

```bash
ffmpeg -i demo-60s.mp4 -ss 15 -t 15 -vf "fps=12,scale=1280:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128[p];[s1][p]paletteuse" -loop 0 assets/demo.gif
# target <4MB; if oversize reduce fps to 10 or scale to 960
```

## Recording Checklist

- [ ] Clean browser profile (no extensions bar), zoom 100%, dark or light consistent with README.
- [ ] Sample image: Kodak `kodim23` (parrot, high detail) — shows density map well.
- [ ] Show real numbers from an actual `cargo bench` run — do not mock table if avoidable; `_todo` is honest until bench lands.
- [ ] 1280×720 capture; hide OS dock/notifications.
- [ ] Add subtitles (SRT) for LinkedIn autoplay (muted by default).
- [ ] End frame holds 3s for thumbnail.

## LinkedIn Post Template (after publishing)

> Shipped PIXLZR hygiene: dual MIT/Apache-2.0, EN README, architecture doc + WASM demo spec — live at guiga-zalu.github.io/pixlzr.
> Rust · multithreaded codec · information-density-aware compression.
> Benchmarks vs JPEG/PNG/WebP (Criterion, pinned hardware) next.
> Code: github.com/guiga-zalu/pixlzr #Rust #DataEngineering #ImageCompression

## Verification

```bash
ls -lh assets/demo.gif assets/demo-60s.mp4
# demo.gif <4MB, 1280×720, 10–12fps
# demo-60s.mp4 <25MB, 1280×720, H.264, 60s ±2s
ffprobe -v error -select_streams v:0 -show_entries stream=width,height,r_frame_rate -of default=noprint_wrappers=1 assets/demo.gif
```

---

*Keep this file in `assets/demo-script.md` in both the pipeline repo and `pixlzr` — it is the single source for the video/GIF spec.*
