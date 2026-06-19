
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use image::{GenericImageView, imageops::FilterType};

const DEFAULT_WIDTH: u32 = 120;

const MID_GREY: f32 = 128.0;

fn apply_contrast(r: f32, g: f32, b: f32, factor: f32) -> (f32, f32, f32) {
    (
        (MID_GREY + (r - MID_GREY) * factor).clamp(0.0, 255.0),
        (MID_GREY + (g - MID_GREY) * factor).clamp(0.0, 255.0),
        (MID_GREY + (b - MID_GREY) * factor).clamp(0.0, 255.0),
    )
}

#[derive(Parser, Debug)]
#[command(
    name = "img2pixdraw",
    about = "Convert any image to pixdraw .txt format"
)]
struct Args {
        #[arg(value_name = "INPUT")]
    input: PathBuf,

            #[arg(value_name = "OUTPUT")]
    output: Option<PathBuf>,

            #[arg(short, long, default_value_t = DEFAULT_WIDTH)]
    width: u32,

        #[arg(long, default_value = "1.0")]
    contrast: f32,

        #[arg(long, value_name = "h|v")]
    flip: Option<String>,

        #[arg(long)]
    stats: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

        let img = image::open(&args.input)
        .with_context(|| format!("Failed to open image {:?}", args.input))?;
    let (orig_w, orig_h) = img.dimensions();

        let w = args.width.max(1);
    let h = (w as f64 * orig_h as f64 / orig_w as f64).round().max(1.0) as u32;

                let mut rgb = img.resize_exact(w, h, FilterType::Lanczos3).to_rgb8();

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

        match args.flip.as_deref() {
        Some("h") => rgb = image::imageops::flip_horizontal(&rgb),
        Some("v") => rgb = image::imageops::flip_vertical(&rgb),
        Some(other) => anyhow::bail!("--flip must be 'h' or 'v', got '{other}'"),
        None => {}
    }

        let outpath = args.output.unwrap_or_else(|| {
        let mut p = args.input;
        p.set_extension("txt");
        p
    });

            let file = File::create(&outpath)
        .with_context(|| format!("Failed to create {}", outpath.display()))?;
    let mut writer = BufWriter::new(file);

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

        writer
        .flush()
        .with_context(|| format!("Failed to flush {}", outpath.display()))?;

        if args.stats {
        let total_pixels = (w * h) as f64;

        eprintln!();
        eprintln!("Unique colours: {}", color_counts.len());

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
