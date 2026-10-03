//! Images in a thread. aiball cites an upload as `/uploads/<sha>.<ext>`, a
//! path on its web server; tvty reads it — aiball's own file when it is on
//! this machine, else `/api/uploads/<sha>` through the socket — once, and
//! keeps it decoded.
//!
//! An image alone on its line (a pasted capture) is drawn by tvty itself: a
//! thumbnail in the compact panel, large full screen, a click opening the
//! viewer ([`segments`]). One inside a sentence goes to the kit's markdown as
//! a `data:` URL ([`inline`]). One missing or too large says so instead.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use base64::Engine as _;
use gpui_kit::{Image, ImageFormat};

use crate::aiball::{Aiball, Attachment, Thread};

/// Past this, an image stays out of the text.
const MAX_BYTES: usize = 5 * 1024 * 1024;

/// An upload read and decoded.
#[derive(Clone)]
pub struct Picture {
    /// The path the text cites, and what the text calls it.
    pub reference: String,
    pub alt: String,
    pub image: Arc<Image>,
    /// Its size in pixels.
    pub width: u32,
    pub height: u32,
    /// Where aiball keeps it on this machine, if it is here.
    pub path: Option<PathBuf>,
    /// The same, as a `data:` URL, for an image inside a sentence.
    data_url: String,
}

impl std::fmt::Debug for Picture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Picture({} {}x{})", self.reference, self.width, self.height)
    }
}

/// What an upload became once read.
#[derive(Clone)]
pub enum Entry {
    Loaded(Picture),
    TooLarge,
    Missing,
}

/// Uploads read already, by the path the texts cite: shared between reads.
pub type Cache = Arc<Mutex<HashMap<String, Entry>>>;

/// Reads the thread's images, and puts in its texts, as `data:` URLs, the
/// ones inside a sentence. Off the UI thread: it reads files and the socket.
pub fn inline(thread: &mut Thread, aiball: &Aiball, cache: &Cache) {
    let mut texts: Vec<&mut String> = thread.comments.iter_mut().filter_map(|c| c.body.as_mut()).collect();
    if let Some(body) = thread.ticket.body.as_mut() {
        texts.push(body);
    }
    let attachments = &thread.attachments;
    let entry = |reference: &str| -> Entry {
        if let Some(known) = cache.lock().ok().and_then(|c| c.get(reference).cloned()) {
            return known;
        }
        let read = read(reference, attachments.iter().find(|a| a.reference == reference), aiball);
        // A failed read (the bus busy, the file not written yet) is not
        // kept: the next read of the thread tries again.
        if !matches!(read, Entry::Missing) {
            if let Ok(mut cache) = cache.lock() {
                cache.insert(reference.to_string(), read.clone());
            }
        }
        read
    };
    for text in texts {
        if !text.contains("](/uploads/") {
            continue;
        }
        // Read them all: those alone on their line are drawn from the cache.
        for (_, reference) in cited(text) {
            entry(&reference);
        }
        *text = rewrite(text, &|reference| match entry(reference) {
            Entry::Loaded(p) => Link::Data(p.data_url),
            Entry::TooLarge => Link::TooLarge,
            Entry::Missing => Link::Missing,
        });
    }
}

fn read(reference: &str, attachment: Option<&Attachment>, aiball: &Aiball) -> Entry {
    if attachment.and_then(|a| a.bytes).is_some_and(|b| b as usize > MAX_BYTES) {
        return Entry::TooLarge;
    }
    let path = attachment
        .filter(|a| a.local)
        .and_then(|a| a.uri.as_deref())
        .and_then(|uri| uri.strip_prefix("file://"))
        .map(PathBuf::from);
    let bytes = match path.as_ref().and_then(|p| std::fs::read(p).ok()) {
        Some(bytes) => bytes,
        None => match aiball.upload_bytes(&api_ref(reference)) {
            Ok(bytes) => bytes,
            Err(error) => {
                log::info!("image {reference}: {error:#}");
                return Entry::Missing;
            }
        },
    };
    if bytes.len() > MAX_BYTES {
        return Entry::TooLarge;
    }
    let content_type = attachment
        .and_then(|a| a.content_type.clone())
        .unwrap_or_else(|| content_type_of(reference).to_string());
    let Some(format) = ImageFormat::from_mime_type(&content_type) else {
        log::info!("image {reference}: {content_type} is not an image tvty draws");
        return Entry::Missing;
    };
    let Ok((width, height)) = image::ImageReader::new(std::io::Cursor::new(&bytes))
        .with_guessed_format()
        .map_err(anyhow::Error::from)
        .and_then(|r| r.into_dimensions().map_err(anyhow::Error::from))
    else {
        log::info!("image {reference}: its size cannot be read");
        return Entry::Missing;
    };
    let data_url = format!("data:{content_type};base64,{}", base64::engine::general_purpose::STANDARD.encode(&bytes));
    Entry::Loaded(Picture {
        reference: reference.to_string(),
        alt: String::new(),
        image: Arc::new(Image::from_bytes(format, bytes)),
        width,
        height,
        path,
        data_url,
    })
}

/// `/uploads/<sha>.<ext>` → `/api/uploads/<sha>`: aiball's documented
/// reference form, and its API route.
fn api_ref(reference: &str) -> String {
    let file = reference.trim_start_matches("/uploads/");
    format!("/api/uploads/{}", file.split('.').next().unwrap_or(file))
}

fn content_type_of(reference: &str) -> &'static str {
    match reference.rsplit('.').next().unwrap_or("").to_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}

/// The images a text cites at the end of its lines, `(alt, reference)`, in
/// order: those tvty draws.
fn cited(text: &str) -> Vec<(String, String)> {
    text.lines().filter_map(trailing_images).flat_map(|(_, images)| images).collect()
}

/// A line that ends with images (`Look: ![a](…) ![b](…)`): the words before
/// them, and the images. `None` when the line has none at its end.
fn trailing_images(line: &str) -> Option<(String, Vec<(String, String)>)> {
    let trimmed = line.trim_end();
    // The images start at the first `![` from which the rest is only images.
    let mut from = trimmed.len();
    let mut found = None;
    while let Some(at) = trimmed[..from].rfind("![") {
        match images_of_line(&trimmed[at..]) {
            Some(images) => {
                found = Some((at, images));
                from = at;
            }
            None => break,
        }
    }
    let (at, images) = found?;
    Some((trimmed[..at].trim_end().to_string(), images))
}

/// The images of a line, when the line holds nothing else: `None` otherwise.
fn images_of_line(line: &str) -> Option<Vec<(String, String)>> {
    let mut rest = line.trim();
    let mut found = Vec::new();
    while !rest.is_empty() {
        let inner = rest.strip_prefix("![")?;
        let (alt, after) = inner.split_once("](")?;
        let (reference, tail) = after.split_once(')')?;
        if !reference.starts_with("/uploads/") || alt.contains(['[', ']']) {
            return None;
        }
        found.push((alt.to_string(), reference.to_string()));
        rest = tail.trim_start();
    }
    (!found.is_empty()).then_some(found)
}

/// A piece of a text, as tvty draws it.
pub enum Segment {
    /// Markdown, for the kit.
    Text(String),
    /// Images alone on their line(s): drawn by tvty.
    Pictures(Vec<Picture>),
    /// What could not be shown, and why.
    Note(String),
}

/// A text cut into markdown and the images alone on their line, read from
/// `cache` (see [`inline`]).
pub fn segments(text: &str, cache: &Cache) -> Vec<Segment> {
    let known = cache.lock().map(|c| c.clone()).unwrap_or_default();
    let mut out = Vec::new();
    let mut prose = String::new();
    for line in text.lines() {
        match trailing_images(line) {
            Some((words, images)) => {
                // The words stay text; the images go under them. A quote's
                // mark alone (`> ![…](…)`) is no words: no empty quote.
                if !words.trim_start_matches('>').trim().is_empty() {
                    prose.push_str(&words);
                    prose.push('\n');
                }
                if !prose.trim().is_empty() {
                    out.push(Segment::Text(std::mem::take(&mut prose)));
                }
                prose.clear();
                let mut row = Vec::new();
                for (alt, reference) in images {
                    match known.get(&reference) {
                        Some(Entry::Loaded(p)) => row.push(Picture { alt, ..p.clone() }),
                        Some(Entry::TooLarge) => out.push(Segment::Note(crate::t!("misc-image-too-large"))),
                        _ => out.push(Segment::Note(crate::t!("misc-image-unavailable"))),
                    }
                }
                if !row.is_empty() {
                    match out.last_mut() {
                        Some(Segment::Pictures(before)) => before.extend(row),
                        _ => out.push(Segment::Pictures(row)),
                    }
                }
            }
            None => {
                prose.push_str(line);
                prose.push('\n');
            }
        }
    }
    if !prose.trim().is_empty() {
        out.push(Segment::Text(prose));
    }
    out
}

/// Every image a text shows alone on its line, in order: the viewer's.
pub fn pictures(text: &str, cache: &Cache) -> Vec<Picture> {
    segments(text, cache)
        .into_iter()
        .flat_map(|s| match s {
            Segment::Pictures(p) => p,
            _ => Vec::new(),
        })
        .collect()
}

/// What an upload's link becomes inside a sentence.
#[derive(Clone, Debug, PartialEq)]
pub enum Link {
    Data(String),
    TooLarge,
    Missing,
}

/// Every upload `text` cites as an image, wherever it stands.
fn every_image(text: &str) -> Vec<String> {
    let found = std::cell::RefCell::new(Vec::new());
    for line in text.split('\n') {
        rewrite_line(line, &|reference| {
            found.borrow_mut().push(reference.to_string());
            Link::Missing
        });
    }
    found.into_inner()
}

/// Reads into `cache` the images `text` cites that it has not read yet: a
/// preview's, of a text not sent (no attachment to go by: through aiball).
/// Off the UI thread. Answers whether any came.
pub fn load(text: &str, aiball: &Aiball, cache: &Cache) -> bool {
    let mut came = false;
    for reference in every_image(text) {
        if cache.lock().is_ok_and(|c| c.contains_key(&reference)) {
            continue;
        }
        let read = read(&reference, None, aiball);
        if !matches!(read, Entry::Missing) {
            if let Ok(mut cache) = cache.lock() {
                cache.insert(reference, read);
                came = true;
            }
        }
    }
    came
}

/// What a preview shows of `text`: its images inside a sentence as `data:`
/// URLs from `cache` — and, with `all`, those alone on their line too, for a
/// view that does not draw them itself. One not read yet says so.
pub fn preview(text: &str, cache: &Cache, all: bool) -> String {
    let known = cache.lock().map(|c| c.clone()).unwrap_or_default();
    let link = |reference: &str| match known.get(reference) {
        Some(Entry::Loaded(p)) => Link::Data(p.data_url.clone()),
        Some(Entry::TooLarge) => Link::TooLarge,
        _ => Link::Missing,
    };
    if !all {
        return rewrite(text, &link);
    }
    text.split('\n').map(|line| rewrite_line(line, &link)).collect::<Vec<_>>().join("\n")
}

/// `text` with each image inside a sentence (`… ![alt](/uploads/…) …`)
/// turned into what `link` makes of it. Images that end their line stay —
/// tvty draws them —, and so do plain links to uploads.
pub fn rewrite(text: &str, link: &dyn Fn(&str) -> Link) -> String {
    let mut out = String::with_capacity(text.len());
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        if trailing_images(line).is_some() {
            out.push_str(line);
        } else {
            out.push_str(&rewrite_line(line, link));
        }
    }
    out
}

fn rewrite_line(text: &str, link: &dyn Fn(&str) -> Link) -> String {
    const START: &str = "](/uploads/";
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(START) {
        let Some(close) = rest[at + 2..].find(')').map(|c| at + 2 + c) else { break };
        let reference = &rest[at + 2..close];
        // The `![` that opens this image, if it is one.
        let open = rest[..at].rfind("![").filter(|o| !rest[o + 2..at].contains(['[', ']', '\n']));
        match open {
            Some(open) => {
                let alt = &rest[open + 2..at];
                out.push_str(&rest[..open]);
                match link(reference) {
                    Link::Data(url) => out.push_str(&format!("![{alt}]({url})")),
                    Link::TooLarge => out.push_str(&format!("*({})*", crate::t!("misc-image-too-large"))),
                    Link::Missing => out.push_str(&format!("*({})*", crate::t!("misc-image-unavailable"))),
                }
            }
            None => out.push_str(&rest[..=close]),
        }
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::{Cache, Entry, Link, api_ref, cited, every_image, images_of_line, preview, rewrite, trailing_images};

    fn link(reference: &str) -> Link {
        match reference {
            "/uploads/a.png" => Link::Data("data:image/png;base64,AAA".into()),
            "/uploads/big.png" => Link::TooLarge,
            _ => Link::Missing,
        }
    }

    #[test]
    fn an_image_inside_a_sentence_becomes_a_data_url() {
        assert_eq!(
            rewrite("see ![shot](/uploads/a.png) here", &link),
            "see ![shot](data:image/png;base64,AAA) here"
        );
        assert_eq!(rewrite("a ![x](/uploads/gone.png) b", &link), "a *(image unavailable)* b");
    }

    #[test]
    fn an_image_alone_on_its_line_stays_for_tvty() {
        let text = "Look:\n![shot](/uploads/a.png)\nthen";
        assert_eq!(rewrite(text, &link), text);
        assert_eq!(cited(text), vec![("shot".to_string(), "/uploads/a.png".to_string())]);
    }

    #[test]
    fn several_images_on_one_line_are_alone_too() {
        let line = "![a](/uploads/a.png) ![b](/uploads/b.png)";
        assert_eq!(images_of_line(line).map(|v| v.len()), Some(2));
        assert_eq!(images_of_line("see ![a](/uploads/a.png)"), None);
        assert_eq!(images_of_line("![a](https://x/y.png)"), None);
    }

    #[test]
    fn images_ending_a_line_are_tvtys_too() {
        let (words, images) = trailing_images("Le panneau : ![p](/uploads/a.png)").unwrap();
        assert_eq!((words.as_str(), images.len()), ("Le panneau :", 1));
        assert_eq!(trailing_images("![p](/uploads/a.png) then words"), None);
        let line = "see ![a](/uploads/a.png) here";
        assert_eq!(rewrite(line, &link), "see ![a](data:image/png;base64,AAA) here");
    }

    #[test]
    fn an_upload_is_read_under_the_api() {
        assert_eq!(api_ref("/uploads/ab12.png"), "/api/uploads/ab12");
    }

    #[test]
    fn plain_links_stay() {
        let text = "the [log](/uploads/log.txt) and ![a](/uploads/a.png) in a sentence";
        assert_eq!(
            rewrite(text, &link),
            "the [log](/uploads/log.txt) and ![a](data:image/png;base64,AAA) in a sentence"
        );
    }

    #[test]
    fn an_image_in_a_quote_is_drawn_with_no_empty_quote() {
        use super::{Entry, Picture, Segment, segments};
        use std::sync::{Arc, Mutex};
        let picture = Picture {
            reference: "/uploads/a.png".into(),
            alt: String::new(),
            image: Arc::new(gpui_kit::Image::from_bytes(gpui_kit::ImageFormat::Png, vec![])),
            width: 1,
            height: 1,
            path: None,
            data_url: String::new(),
        };
        let cache = Arc::new(Mutex::new(std::collections::HashMap::from([("/uploads/a.png".to_string(), Entry::Loaded(picture))])));
        let parts = segments("> ![shot](/uploads/a.png)", &cache);
        assert!(matches!(&parts[..], [Segment::Pictures(p)] if p.len() == 1), "{}", parts.len());
        // Words before it stay, quoted.
        let parts = segments("> Look: ![shot](/uploads/a.png)", &cache);
        assert!(matches!(&parts[..], [Segment::Text(t), Segment::Pictures(_)] if t.trim() == "> Look:"));
    }

    #[test]
    fn a_preview_finds_every_image_and_draws_those_it_read() {
        let text = "see ![a](/uploads/a.png) here\n![shot](/uploads/b.png)";
        assert_eq!(every_image(text), vec!["/uploads/a.png".to_string(), "/uploads/b.png".to_string()]);
        let cache = Cache::default();
        cache.lock().unwrap().insert("/uploads/big.png".into(), Entry::TooLarge);
        // Nothing read yet: said so, inside a sentence; alone on its line, left to the drawer.
        assert_eq!(preview(text, &cache, false), "see *(image unavailable)* here\n![shot](/uploads/b.png)");
        // For a view that draws nothing itself, the lone image too.
        assert_eq!(preview(text, &cache, true), "see *(image unavailable)* here\n*(image unavailable)*");
        assert_eq!(preview("![x](/uploads/big.png)", &cache, true), "*(image too large to show here)*");
    }
}