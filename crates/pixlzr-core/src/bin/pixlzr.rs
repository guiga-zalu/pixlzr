//! Minimal PNG <-> PIXLZR command line tool.
//!
//! ```text
//! pixlzr encode <in.png> <out.pixlzr> [--quality N]
//! pixlzr decode <in.pixlzr> <out.png>
//! pixlzr info   <in.pixlzr>
//! ```
//!
//! Built only with the `cli` feature (which enables the `png` dependency).

use std::fs::{self, File};
use std::io::BufReader;
use std::process::ExitCode;
use std::time::Instant;

use pixlzr_core::{decode_rgba, encode_rgba, inspect_header};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("pixlzr: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = args.first() else {
        return Err(usage().to_string());
    };

    match command.as_str() {
        "-h" | "--help" | "help" => {
            println!("{}", usage());
            Ok(())
        }
        "version" | "--version" | "-V" => {
            println!("pixlzr {}", pixlzr_core::version_str());
            Ok(())
        }
        "encode" => {
            let (input, output, quality) = parse_encode_args(&args[1..])?;
            let start = Instant::now();
            let (width, height, rgba) = read_png_rgba(&input)?;
            let blob = encode_rgba(width, height, &rgba, quality)
                .map_err(|err| format!("encode failed: {err}"))?;
            let elapsed = start.elapsed();
            fs::write(&output, &blob).map_err(|err| format!("cannot write {output}: {err}"))?;
            let raw = f64::from(width) * f64::from(height) * 4.0;
            println!(
                "encoded {width}x{height} q{quality} -> {output} ({} bytes, raw ratio {:.2}:1, {:.1} ms)",
                blob.len(),
                raw / blob.len() as f64,
                elapsed.as_secs_f64() * 1000.0
            );
            Ok(())
        }
        "decode" => {
            let [input, output] = expect_paths::<2>(&args[1..], "decode")?;
            let blob = fs::read(input).map_err(|err| format!("cannot read {input}: {err}"))?;
            let start = Instant::now();
            let (image, rgba) =
                decode_rgba(&blob).map_err(|err| format!("decode failed: {err}"))?;
            let elapsed = start.elapsed();
            write_png_rgba(output, image.width, image.height, &rgba)?;
            println!(
                "decoded {}x{} -> {output} ({:.1} ms)",
                image.width,
                image.height,
                elapsed.as_secs_f64() * 1000.0
            );
            Ok(())
        }
        "info" => {
            let [input] = expect_paths::<1>(&args[1..], "info")?;
            let blob = fs::read(input).map_err(|err| format!("cannot read {input}: {err}"))?;
            let header =
                inspect_header(&blob).map_err(|err| format!("cannot parse {input}: {err}"))?;
            println!("dimensions: {}x{}", header.width, header.height);
            println!("blocks: {}x{} (8x8)", header.blocks_x, header.blocks_y);
            println!("quality: {}", header.quality);
            println!("file size: {} bytes", blob.len());
            Ok(())
        }
        other => Err(format!("unknown command `{other}`\n\n{}", usage())),
    }
}

fn usage() -> &'static str {
    "usage:
  pixlzr encode <in.png> <out.pixlzr> [--quality N]
  pixlzr decode <in.pixlzr> <out.png>
  pixlzr info   <in.pixlzr>

  --quality N  0..=100 (default 80; higher keeps more detail)"
}

fn expect_paths<'a, const N: usize>(
    args: &'a [String],
    command: &str,
) -> Result<[&'a str; N], String> {
    if args.len() != N {
        return Err(format!(
            "`{command}` expects {N} path argument(s), got {}\n\n{}",
            args.len(),
            usage()
        ));
    }
    let mut paths = [""; N];
    for (slot, arg) in paths.iter_mut().zip(args) {
        *slot = arg.as_str();
    }
    Ok(paths)
}

fn parse_encode_args(args: &[String]) -> Result<(String, String, u8), String> {
    let mut paths = Vec::new();
    let mut quality = 80u8;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--quality" | "-q" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--quality requires a value".to_string())?;
                quality = value
                    .parse::<u8>()
                    .map_err(|_| format!("invalid quality `{value}` (expected 0..=100)"))?;
                if quality > 100 {
                    return Err(format!("invalid quality {quality} (expected 0..=100)"));
                }
            }
            flag if flag.starts_with('-') => {
                return Err(format!("unknown option `{flag}`\n\n{}", usage()));
            }
            path => paths.push(path.to_string()),
        }
    }
    match paths.as_slice() {
        [input, output] => Ok((input.clone(), output.clone(), quality)),
        _ => Err(format!(
            "`encode` expects <in.png> <out.pixlzr>, got {} path(s)\n\n{}",
            paths.len(),
            usage()
        )),
    }
}

fn read_png_rgba(path: &str) -> Result<(u32, u32, Vec<u8>), String> {
    let file = File::open(path).map_err(|err| format!("cannot open {path}: {err}"))?;
    let mut decoder = png::Decoder::new(BufReader::new(file));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder
        .read_info()
        .map_err(|err| format!("{path}: {err}"))?;

    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(|err| format!("{path}: {err}"))?;
    if info.bit_depth != png::BitDepth::Eight {
        return Err(format!(
            "{path}: unsupported bit depth {:?} (8-bit expected after expansion)",
            info.bit_depth
        ));
    }

    let pixels = &buffer[..info.buffer_size()];
    let count = (info.width * info.height) as usize;
    let mut rgba = Vec::with_capacity(count * 4);
    match info.color_type {
        png::ColorType::Rgba => rgba.extend_from_slice(pixels),
        png::ColorType::Rgb => {
            for i in 0..count {
                let px = &pixels[i * 3..i * 3 + 3];
                rgba.extend_from_slice(&[px[0], px[1], px[2], 255]);
            }
        }
        png::ColorType::Grayscale => {
            for &value in pixels {
                rgba.extend_from_slice(&[value, value, value, 255]);
            }
        }
        png::ColorType::GrayscaleAlpha => {
            for i in 0..count {
                let px = &pixels[i * 2..i * 2 + 2];
                rgba.extend_from_slice(&[px[0], px[0], px[0], px[1]]);
            }
        }
        other => {
            return Err(format!("{path}: unsupported colour type {other:?}"));
        }
    }

    Ok((info.width, info.height, rgba))
}

fn write_png_rgba(path: &str, width: u32, height: u32, rgba: &[u8]) -> Result<(), String> {
    let file = File::create(path).map_err(|err| format!("cannot create {path}: {err}"))?;
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .map_err(|err| format!("{path}: {err}"))?;
    writer
        .write_image_data(rgba)
        .map_err(|err| format!("{path}: {err}"))?;
    writer.finish().map_err(|err| format!("{path}: {err}"))?;
    Ok(())
}
