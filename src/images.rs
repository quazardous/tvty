//! Images in a thread. aiball cites an upload as `/uploads/<sha>.<ext>`, a
//! path on its web server; the kit's markdown loads an image only from a
//! `data:` URL or over HTTP, and tvty talks to aiball over its socket. So an
//! image is read — from aiball's own file when it is on this machine, else
//! through the socket — and put in the text as a `data:` URL. One missing
//! or too large says so in the text instead.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use base64::Engine as _;

use crate::aiball::{Aiball, Attachment, Thread};

/// Past this, an image stays out of the text.
const MAX_BYTES: usize = 5 * 1024 * 1024;

/// What an upload's link becomes.
#[derive(Clone, Debug, PartialEq)]
pub enum Link {
    Data(String),
    TooLarge,
    Missing,
}

/// Images read already, by the path the texts cite: shared between reads.
pub type Cache = Arc<Mutex<HashMap<String, Link>>>;

/// Puts the thread's images in its texts. Off the UI thread: it reads files
/// and the socket.
pub fn inline(thread: &mut Thread, aiball: &Aiball, cache: &Cache) {
    let mut texts: Vec<&mut String> = thread.comments.iter_mut().filter_map(|c| c.body.as_mut()).collect();
    if let Some(body) = thread.ticket.body.as_mut() {
        texts.push(body);
    }
    let attachments = &thread.attachments;
    let link = |reference: &str| -> Link {
        if let Some(known) = cache.lock().ok().and_then(|c| c.get(reference).cloned()) {
            return known;
        }
        let read = read(reference, attachments.iter().find(|a| a.reference == reference), aiball);
        if let Ok(mut cache) = cache.lock() {
            cache.insert(reference.to_string(), read.clone());
        }
        read
    };
    for text in texts {
        if text.contains("](/uploads/") {
            *text = rewrite(text, &link);
        }
    }
}

fn read(reference: &str, attachment: Option<&Attachment>, aiball: &Aiball) -> Link {
    if attachment.and_then(|a| a.bytes).is_some_and(|b| b as usize > MAX_BYTES) {
        return Link::TooLarge;
    }
    let local = attachment
        .filter(|a| a.local)
        .and_then(|a| a.uri.as_deref())
        .and_then(|uri| uri.strip_prefix("file://"))
        .and_then(|path| std::fs::read(path).ok());
    let bytes = match local {
        Some(bytes) => bytes,
        None => match aiball.upload_bytes(&api_ref(reference)) {
            Ok(bytes) => bytes,
            Err(error) => {
                log::debug!("image {reference}: {error:#}");
                return Link::Missing;
            }
        },
    };
    if bytes.len() > MAX_BYTES {
        return Link::TooLarge;
    }
    let content_type = attachment
        .and_then(|a| a.content_type.clone())
        .unwrap_or_else(|| content_type_of(reference).to_string());
    Link::Data(format!(
        "data:{content_type};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
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

/// `text` with each image citing an upload (`![alt](/uploads/…)`) turned
/// into what `link` makes of it. Plain links to uploads stay as they are.
pub fn rewrite(text: &str, link: &dyn Fn(&str) -> Link) -> String {
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
                    Link::TooLarge => out.push_str("*(image too large to show here)*"),
                    Link::Missing => out.push_str("*(image unavailable)*"),
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
    use super::{Link, api_ref, rewrite};

    fn link(reference: &str) -> Link {
        match reference {
            "/uploads/a.png" => Link::Data("data:image/png;base64,AAA".into()),
            "/uploads/big.png" => Link::TooLarge,
            _ => Link::Missing,
        }
    }

    #[test]
    fn images_become_data_urls() {
        assert_eq!(
            rewrite("see ![shot](/uploads/a.png) here", &link),
            "see ![shot](data:image/png;base64,AAA) here"
        );
    }

    #[test]
    fn what_cannot_show_says_so() {
        assert_eq!(rewrite("![x](/uploads/big.png)", &link), "*(image too large to show here)*");
        assert_eq!(rewrite("a ![x](/uploads/gone.png) b", &link), "a *(image unavailable)* b");
    }

    #[test]
    fn an_upload_is_read_under_the_api() {
        assert_eq!(api_ref("/uploads/ab12.png"), "/api/uploads/ab12");
    }

    #[test]
    fn plain_links_stay() {
        let text = "the [log](/uploads/log.txt) and ![a](/uploads/a.png)";
        assert_eq!(rewrite(text, &link), "the [log](/uploads/log.txt) and ![a](data:image/png;base64,AAA)");
    }
}
