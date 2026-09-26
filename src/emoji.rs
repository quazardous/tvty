//! Colour emoji in the terminals. GPUI colours an emoji only in a font drawn
//! in bitmaps (CBDT) or COLR v0, and its fallback picks the system's emoji
//! font, which on Fedora is COLR v1: grey outlines. So the terminal draws
//! emoji itself: a colour emoji font from the user's fonts (see
//! [`crate::fonts`]) is rasterised here, each emoji once, and painted as an
//! image over its two cells.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use gpui_kit::*;
use swash::scale::image::Content;
use swash::scale::{Render, ScaleContext, Source, StrikeWith};
use swash::shape::ShapeContext;
use swash::FontRef;

/// The pixel size emoji are drawn at, scaled to their cells when painted.
const DRAWN_AT: f32 = 128.;

/// The colour emoji font, if the user has one: its bytes, for good.
fn font_bytes() -> Option<&'static [u8]> {
    static FONT: OnceLock<Option<&'static [u8]>> = OnceLock::new();
    *FONT.get_or_init(|| {
        let dir = crate::fonts::dir()?;
        let found = std::fs::read_dir(&dir).ok()?.flatten().map(|e| e.path()).find_map(|path| {
            let bytes = std::fs::read(&path).ok()?;
            let font = FontRef::from_index(&bytes, 0)?;
            // A colour font in bitmaps, the kind drawn here.
            let colour = font.table(swash::tag_from_bytes(b"CBDT")).is_some();
            colour.then_some(bytes)
        })?;
        log::info!("colour emoji drawn from the user's fonts");
        Some(Box::leak(found.into_boxed_slice()) as &'static [u8])
    })
}

fn font() -> Option<FontRef<'static>> {
    FontRef::from_index(font_bytes()?, 0)
}

/// Whether a cell's text is an emoji this font draws: wide, or asking for
/// emoji presentation (VS16), and a glyph in the font.
pub fn is_emoji(text: &str, wide: bool) -> bool {
    let Some(first) = text.chars().next() else { return false };
    if first.is_ascii() || !(wide || text.contains('\u{FE0F}')) {
        return false;
    }
    font().is_some_and(|font| font.charmap().map(first) != 0)
}

/// The emoji as an image, drawn once and kept.
pub fn image(text: &str) -> Option<Arc<RenderImage>> {
    static DRAWN: OnceLock<Mutex<HashMap<String, Option<Arc<RenderImage>>>>> = OnceLock::new();
    let drawn = DRAWN.get_or_init(Default::default);
    if let Some(image) = drawn.lock().ok()?.get(text) {
        return image.clone();
    }
    let image = draw(text);
    drawn.lock().ok()?.insert(text.to_string(), image.clone());
    image
}

fn draw(text: &str) -> Option<Arc<RenderImage>> {
    let font = font()?;
    // Shaped, so that sequences (flags, skin tones, joined emoji) become
    // their one glyph.
    let mut shaping = ShapeContext::new();
    let mut shaper = shaping.builder(font).size(DRAWN_AT).build();
    shaper.add_str(text);
    let mut glyph = None;
    shaper.shape_with(|cluster| {
        if glyph.is_none() {
            glyph = cluster.glyphs.first().map(|g| g.id);
        }
    });
    let glyph = glyph.filter(|g| *g != 0)?;
    let mut scaling = ScaleContext::new();
    let mut scaler = scaling.builder(font).size(DRAWN_AT).build();
    let image = Render::new(&[Source::ColorBitmap(StrikeWith::BestFit)]).render(&mut scaler, glyph)?;
    if image.content != Content::Color || image.placement.width == 0 || image.placement.height == 0 {
        return None;
    }
    // GPUI draws BGRA.
    let mut data = image.data;
    for pixel in data.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    let buffer = ::image::RgbaImage::from_raw(image.placement.width, image.placement.height, data)?;
    Some(Arc::new(RenderImage::new([::image::Frame::new(buffer)])))
}

/// Where an emoji of `cell` height sits over `width` of cells from `origin`:
/// square, as large as the cells allow, centred.
pub fn bounds(origin: Point<Pixels>, width: Pixels, height: Pixels) -> Bounds<Pixels> {
    let side = width.min(height);
    Bounds::new(
        point(origin.x + (width - side) / 2., origin.y + (height - side) / 2.),
        size(side, side),
    )
}
