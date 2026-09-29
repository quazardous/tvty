//! Draws `assets/tvty.ico`, the Windows programs' icon, from `assets/tvty.svg`:
//! one PNG per size Windows asks for (the taskbar, the Start menu, Explorer's
//! views), in one .ico. Run it again when the SVG changes:
//!
//!     cargo run --example windows_icon

use std::path::Path;

const SIZES: [u32; 7] = [16, 24, 32, 48, 64, 128, 256];

fn main() -> anyhow::Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let svg = std::fs::read(root.join("assets/tvty.svg"))?;
    let tree = resvg::usvg::Tree::from_data(&svg, &resvg::usvg::Options::default())?;
    let pngs = SIZES
        .iter()
        .map(|&size| {
            let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size).expect("a size");
            let scale = size as f32 / tree.size().width();
            resvg::render(&tree, resvg::tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
            Ok((size, pixmap.encode_png()?))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let out = root.join("assets/tvty.ico");
    std::fs::write(&out, ico(&pngs))?;
    println!("wrote {}", out.display());
    Ok(())
}

/// An .ico holding PNG images (Windows Vista and later read them).
fn ico(pngs: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&[0, 0, 1, 0]);
    out.extend_from_slice(&(pngs.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * pngs.len() as u32;
    for (size, png) in pngs {
        // 256 is written 0.
        let side = if *size >= 256 { 0 } else { *size as u8 };
        out.extend_from_slice(&[side, side, 0, 0]);
        out.extend_from_slice(&1u16.to_le_bytes()); // planes
        out.extend_from_slice(&32u16.to_le_bytes()); // bits per pixel
        out.extend_from_slice(&(png.len() as u32).to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        offset += png.len() as u32;
    }
    for (_, png) in pngs {
        out.extend_from_slice(png);
    }
    out
}
