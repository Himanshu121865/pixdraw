// ── img2pixdraw ─────────────────────────────────────────────────
// CLI tool: convert any image file into a pixdraw .txt pixel-
// representation so it can be loaded into the editor.
//
// Why this exists:
//   Opendraw stores art as "R x y r g b" lines in a text file.
//   This tool bridges real images (photos, digital art) into that
//   format so they can be loaded into the editor.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use image::{GenericImageView, imageops::FilterType};

/// Default logical width when `--width` is not given.
const DEFAULT_WIDTH: u32 = 120;

/// Midpoint grey used in contrast adjustment.
const MID_GREY: f32 = 128.0;

/// Adjust contrast by pushing each channel away from mid-grey.
/// `factor = 1.0` → no change. `factor = 1.2` → more contrast.
fn apply_contrast(r: f32, g: f32, b: f32, factor: f32) -> (f32, f32, f32) {
    (
        (MID_GREY + (r - MID_GREY) * factor).clamp(0.0, 255.0),
        (MID_GREY + (g - MID_GREY) * factor).clamp(0.0, 255.0),
        (MID_GREY + (b - MID_GREY) * factor).clamp(0.0, 255.0),
    )
}

// ── CLI argument struct ─────────────────────────────────────────
// `#[derive(Parser)]` (from clap) generates all argument parsing
// automatically — no manual `match` on argv, no missing-help docs.
// Each doc comment on a field becomes the `--help` text for that
// flag. This is the standard approach for Rust CLI apps.
#[derive(Parser, Debug)]
#[command(
    name = "img2pixdraw",
    about = "Convert any image to pixdraw .txt format"
)]
struct Args {
    /// Source image (jpg, png, gif, bmp, …)
    #[arg(value_name = "INPUT")]
    input: PathBuf,

    /// Output .txt path (default: <input>.txt).
    /// Positional — clap treats any non-flag arg after INPUT as this.
    #[arg(value_name = "OUTPUT")]
    output: Option<PathBuf>,

    /// Logical pixel-art width. Height auto-scales to keep aspect.
    /// Smaller → chunkier; larger → smoother.
    #[arg(short, long, default_value_t = DEFAULT_WIDTH)]
    width: u32,

    /// Contrast enhancement. 1.0 = no change.
    #[arg(long, default_value = "1.0")]
    contrast: f32,

    /// Flip direction: "h" (horizontal) or "v" (vertical).
    #[arg(long, value_name = "h|v")]
    flip: Option<String>,

    /// Print colour frequency statistics to stderr.
    #[arg(long)]
    stats: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // ── Load image ──────────────────────────────────────────────
    let img = image::open(&args.input)
        .with_context(|| format!("Failed to open image {:?}", args.input))?;
    let (orig_w, orig_h) = img.dimensions();

    // ── Compute output dimensions (preserve aspect ratio) ───────
    let w = args.width.max(1);
    let h = (w as f64 * orig_h as f64 / orig_w as f64).round().max(1.0) as u32;

    // ── Resize and convert to RGB ───────────────────────────────
    // Lanczos3 produces sharper downsampling than bilinear or
    // nearest-neighbour, giving cleaner pixel-art colour blocks.
    let mut rgb = img.resize_exact(w, h, FilterType::Lanczos3).to_rgb8();

    // ── Optional contrast enhancement ───────────────────────────
    if (args.contrast - 1.0).abs() > f32::EPSILON {
        for pixel in rgb.pixels_mut() {
            let (r, g, b) = apply_contrast(
                pixel[0] as f32,
                pixel[1] as f32,
                pixel[2] as f32,
                args.contrast,
            );
            *pixel = image::Rgb([r as u8, g as u8, b as u8]);
        }
    }

    // ── Optional flip ───────────────────────────────────────────
    match args.flip.as_deref() {
        Some("h") => rgb = image::imageops::flip_horizontal(&rgb),
        Some("v") => rgb = image::imageops::flip_vertical(&rgb),
        Some(other) => anyhow::bail!("--flip must be 'h' or 'v', got '{other}'"),
        None => {}
    }

    // ── Determine output path ───────────────────────────────────
    let outpath = args.output.unwrap_or_else(|| {
        let mut p = args.input;
        p.set_extension("txt");
        p
    });

    // ── Write .txt ──────────────────────────────────────────────
    // BufWriter amortises the per-line write cost.
    let file = File::create(&outpath)
        .with_context(|| format!("Failed to create {}", outpath.display()))?;
    let mut writer = BufWriter::new(file);

    // Count unique colours for optional stats output.
    let mut color_counts: BTreeMap<(u8, u8, u8), u64> = BTreeMap::new();

    for y in 0..h {
        for x in 0..w {
            let pixel = rgb.get_pixel(x, y);
            let (r, g, b) = (pixel[0], pixel[1], pixel[2]);

            *color_counts.entry((r, g, b)).or_default() += 1;
            writeln!(writer, "R {x} {y} {r} {g} {b}")
                .with_context(|| format!("Failed to write to {}", outpath.display()))?;
        }
    }

    // Flush the BufWriter so we can detect I/O errors early.
    writer
        .flush()
        .with_context(|| format!("Failed to flush {}", outpath.display()))?;

    // ── Optional colour statistics ──────────────────────────────
    if args.stats {
        let total_pixels = (w * h) as f64;

        eprintln!();
        eprintln!("Unique colours: {}", color_counts.len());

        // Most frequent colours first.
        let mut pairs: Vec<_> = color_counts.into_iter().collect();
        pairs.sort_unstable_by_key(|(_, count)| std::cmp::Reverse(*count));

        for ((r, g, b), count) in pairs.iter().take(20) {
            let pct = *count as f64 / total_pixels * 100.0;
            eprintln!("  RGB({r:3},{g:3},{b:3}) {count:>5}px ({pct:>4.1}%)");
        }
        if pairs.len() > 20 {
            eprintln!("  ... and {} more", pairs.len() - 20);
        }
    }

    Ok(())
}
