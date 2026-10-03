//! The interface's words, in the language chosen (Settings > Appearance >
//! Language) or the system's. Only tvty's own words: what people wrote
//! (tickets, comments, names) is shown as it is.
//!
//! The words live in `assets/locales/<lang>/<surface>.ftl` (Fluent), built
//! in: a folder per language, the same files in each, a file per surface
//! of the interface (`agentbar`, `tickets`…), `common` for words they share.
//! English is the reference: a word missing from another language is said
//! in English, and one missing from English shows its id.
//!
//! `tvty i18n` measures how far each language goes against English, and
//! how much of the interface asks for its words here rather than writing
//! them in the code (the CI prints it too).
//!
//! `TVTY_LANG` (en, fr, es) overrides the setting: the tests pin English with
//! it, whatever the machine speaks.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};

/// The languages tvty speaks: their code (their folder under
/// `assets/locales/`) and their name in themselves.
pub const LANGS: &[(&str, &str)] = &[("en", "English"), ("fr", "Français"), ("es", "Español")];

/// The words, built in: `<lang>/<surface>.ftl`.
#[derive(rust_embed::Embed)]
#[folder = "assets/locales/"]
struct Locales;

/// A language's files: their name (`agentbar`) and their words, by name.
fn files(code: &str) -> Vec<(String, String)> {
    let prefix = format!("{code}/");
    let mut files: Vec<_> = Locales::iter()
        .filter_map(|path| {
            let name = path.strip_prefix(prefix.as_str())?.strip_suffix(".ftl")?.to_string();
            let words = String::from_utf8(Locales::get(&path)?.data.into_owned()).ok()?;
            Some((name, words))
        })
        .collect();
    files.sort();
    files
}

/// The language spoken now (an index in `LANGS`).
static CURRENT: AtomicUsize = AtomicUsize::new(0);

fn bundles() -> &'static [FluentBundle<FluentResource>] {
    static BUNDLES: OnceLock<Vec<FluentBundle<FluentResource>>> = OnceLock::new();
    BUNDLES.get_or_init(|| {
        LANGS
            .iter()
            .map(|(code, _)| {
                let lang = code.parse().expect("a language code");
                let mut bundle = FluentBundle::new_concurrent(vec![lang]);
                // No isolation marks around a value: they show in a GPUI text.
                bundle.set_use_isolating(false);
                for (name, words) in files(code) {
                    let resource = FluentResource::try_new(words).unwrap_or_else(|(resource, errors)| {
                        log::warn!("i18n: {code}/{name}: {errors:?}");
                        resource
                    });
                    if let Err(errors) = bundle.add_resource(resource) {
                        log::warn!("i18n: {code}/{name}: {errors:?}");
                    }
                }
                bundle
            })
            .collect()
    })
}

/// The language a locale names ("fr_FR.UTF-8", "fr-FR", "fr": its first two
/// letters). English when tvty does not speak it.
fn spoken(locale: &str) -> usize {
    let code = locale.get(..2).unwrap_or("").to_ascii_lowercase();
    LANGS.iter().position(|(c, _)| *c == code).unwrap_or(0)
}

/// The language a choice names: the setting's (`None`: the system's), unless
/// `TVTY_LANG` says another.
fn chosen(choice: Option<&str>) -> usize {
    let env = std::env::var("TVTY_LANG").ok().filter(|l| !l.is_empty());
    let wanted = env.or_else(|| choice.map(str::to_string)).or_else(sys_locale::get_locale).unwrap_or_default();
    spoken(&wanted)
}

/// The system's language, as tvty speaks it (an index in `LANGS`).
pub fn system() -> usize {
    spoken(&sys_locale::get_locale().unwrap_or_default())
}

/// A language's name in itself ("Français").
pub fn name(index: usize) -> &'static str {
    LANGS[index].1
}

/// Speaks the language chosen (`None`: the system's); true when it changed.
pub fn set(choice: Option<&str>) -> bool {
    let index = chosen(choice);
    CURRENT.swap(index, Ordering::Relaxed) != index
}

/// A word in a given language, if that language has it.
fn said(index: usize, id: &str, args: Option<&FluentArgs>) -> Option<String> {
    let bundle = &bundles()[index];
    let (message, attribute) = match id.split_once('.') {
        Some((message, attribute)) => (message, Some(attribute)),
        None => (id, None),
    };
    let message = bundle.get_message(message)?;
    let pattern = match attribute {
        Some(attribute) => message.get_attribute(attribute)?.value(),
        None => message.value()?,
    };
    let mut errors = Vec::new();
    let text = bundle.format_pattern(pattern, args, &mut errors);
    if !errors.is_empty() {
        log::warn!("i18n: {} {id}: {errors:?}", LANGS[index].0);
    }
    Some(text.into_owned())
}

/// Whether English has the word `id`: a word built from a name (a page's,
/// a group's) that may have none.
pub fn has(id: &str) -> bool {
    said(0, id, None).is_some()
}

/// The word `id` (`message` or `message.attribute`) in the language spoken,
/// else in English, else its id. Use `t!`.
pub fn text(id: &str, args: Option<&FluentArgs>) -> String {
    let index = CURRENT.load(Ordering::Relaxed);
    said(index, id, args)
        .or_else(|| (index != 0).then(|| said(0, id, args)).flatten())
        .unwrap_or_else(|| {
            log::warn!("i18n: no word for {id}");
            id.to_string()
        })
}

/// The word `id` in the language spoken, its values given by name:
/// `t!("tickets-open", count = n)`.
#[macro_export]
macro_rules! t {
    ($id:expr $(,)?) => {
        $crate::i18n::text($id, None)
    };
    ($id:expr, $($name:ident = $value:expr),+ $(,)?) => {{
        let mut args = ::fluent_bundle::FluentArgs::new();
        $( args.set(stringify!($name), ::fluent_bundle::FluentValue::from($value)); )+
        $crate::i18n::text($id, Some(&args))
    }};
}

// ── Coverage: `tvty i18n` ────────────────────────────────────────────

/// Every message and attribute of a file, by id; Err: what does not read.
fn ids(words: &str) -> Result<Vec<String>, String> {
    let resource = FluentResource::try_new(words.to_string()).map_err(|(_, errors)| format!("{errors:?}"))?;
    let mut ids = Vec::new();
    for entry in resource.entries() {
        if let fluent_syntax::ast::Entry::Message(message) = entry {
            let id = message.id.name;
            if message.value.is_some() {
                ids.push(id.to_string());
            }
            ids.extend(message.attributes.iter().map(|a| format!("{id}.{}", a.id.name)));
        }
    }
    Ok(ids)
}

/// A language's ids, file by file, and its faults: a file that does not
/// read, an id said twice (the second is lost).
fn read(code: &str) -> (Vec<(String, Vec<String>)>, Vec<String>) {
    let mut faults = Vec::new();
    let mut seen = std::collections::HashMap::new();
    let mut read = Vec::new();
    for (name, words) in files(code) {
        match ids(&words) {
            Ok(ids) => {
                for id in &ids {
                    if let Some(first) = seen.insert(id.clone(), name.clone()) {
                        faults.push(format!("{code}/{name}.ftl: {id} is already in {code}/{first}.ftl"));
                    }
                }
                read.push((name, ids));
            }
            Err(e) => faults.push(format!("{code}/{name}.ftl: {e}")),
        }
    }
    (read, faults)
}

/// One language against English: the words it has, those it misses, those
/// English has not (left over) — as `file: id` — and file by file, its
/// words against English's.
struct Coverage {
    code: &'static str,
    has: usize,
    missing: Vec<String>,
    extra: Vec<String>,
    by_file: Vec<usize>,
}

fn coverage(english: &[(String, Vec<String>)], theirs: &[(String, Vec<String>)], code: &'static str) -> Coverage {
    let in_their = |file: &str, id: &String| theirs.iter().any(|(f, ids)| f == file && ids.contains(id));
    let mut c = Coverage { code, has: 0, missing: Vec::new(), extra: Vec::new(), by_file: Vec::new() };
    for (file, ids) in english {
        let has = ids.iter().filter(|id| in_their(file, id)).count();
        c.has += has;
        c.by_file.push(has);
        c.missing.extend(ids.iter().filter(|id| !in_their(file, id)).map(|id| format!("{file}: {id}")));
    }
    let in_english = |file: &str, id: &String| english.iter().any(|(f, ids)| f == file && ids.contains(id));
    for (file, ids) in theirs {
        c.extra.extend(ids.iter().filter(|id| !in_english(file, id)).map(|id| format!("{file}: {id}")));
    }
    c
}

/// The string literals of a Rust source, outside comments, with their line
/// (raw strings read as plain ones: a rough reading, for an estimate).
fn literals(source: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut chars = source.char_indices().peekable();
    let mut line = 1;
    while let Some((at, c)) = chars.next() {
        match c {
            '\n' => line += 1,
            '/' if source[at..].starts_with("//") => {
                while chars.peek().is_some_and(|(_, c)| *c != '\n') {
                    chars.next();
                }
            }
            // '"' and '\"': a char, not a string.
            '\'' if source[at..].starts_with("'\"'") || source[at..].starts_with("'\\\"'") => {
                while chars.next().is_some_and(|(_, c)| c != '\'') {}
            }
            '"' => {
                let mut text = String::new();
                while let Some((_, c)) = chars.next() {
                    match c {
                        '\\' => {
                            if let Some((_, n)) = chars.next() {
                                text.push(n);
                            }
                        }
                        '"' => break,
                        '\n' => {
                            line += 1;
                            text.push(c);
                        }
                        _ => text.push(c),
                    }
                }
                out.push((line, text));
            }
            _ => {}
        }
    }
    out
}

/// Whether a literal reads as words for people rather than code: a space
/// between letters, no path, no format of code. A guess, for an estimate.
fn reads_as_words(text: &str, line: &str) -> bool {
    let quiet = ["log::", "named(", "assert", "panic!", "expect(", "debug!", "#[", "tvty-ctl", "ctl("];
    text.contains(' ')
        && text.chars().filter(|c| c.is_alphabetic()).count() >= 3
        && !text.contains("::")
        && !text.starts_with('-')
        && !quiet.iter().any(|q| line.contains(q))
}

/// One source file: the words it asks for (`t!`), those still written in it,
/// and the ids English has not.
#[derive(Default)]
struct Migration {
    asked: usize,
    written: usize,
    unknown: Vec<String>,
}

fn migration(source: &str, english: &[String]) -> Migration {
    // Its tests are not the interface.
    let source = source.split("#[cfg(test)]\nmod tests").next().unwrap_or(source);
    let lines: Vec<&str> = source.lines().collect();
    let mut m = Migration::default();
    for (at, _) in source.match_indices("t!(\"") {
        // `t!(` alone, not the end of another macro's name (`format!(`).
        if source[..at].ends_with(|c: char| c.is_alphanumeric() || c == '_') {
            continue;
        }
        let rest = &source[at + 4..];
        let id = &rest[..rest.find('"').unwrap_or(0)];
        m.asked += 1;
        if !english.iter().any(|e| e == id) {
            m.unknown.push(id.to_string());
        }
    }
    m.written = literals(source)
        .iter()
        .filter(|(line, text)| reads_as_words(text, lines.get(line - 1).copied().unwrap_or("")))
        .count();
    m
}

/// The sources read for the migration: tvty's `src`, where it was built.
fn sources() -> Vec<(String, String)> {
    let root = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src"));
    let mut files = Vec::new();
    let mut stack = vec![root.clone()];
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            stack.extend(std::fs::read_dir(&path).into_iter().flatten().flatten().map(|e| e.path()));
            continue;
        }
        if path.extension().is_none_or(|e| e != "rs") || path.ends_with("i18n.rs") {
            continue;
        }
        if let Ok(source) = std::fs::read_to_string(&path) {
            let name = path.strip_prefix(&root).unwrap_or(&path).display().to_string();
            files.push((name, source));
        }
    }
    files.sort();
    files
}

fn percent(part: usize, whole: usize) -> f64 {
    if whole == 0 { 100. } else { part as f64 * 100. / whole as f64 }
}

/// `tvty i18n [LANG] [--markdown]`: each language's coverage against
/// English, then how far the interface is migrated, file by file; with a
/// language, the words it misses. `--markdown` writes tables (the CI's job
/// summary). Exit code 1 on a real fault: a file that does not read, a
/// `t!` naming a word English has not.
pub fn report(args: &[String]) -> i32 {
    let markdown = args.iter().any(|a| a == "--markdown");
    let only = args.iter().find(|a| !a.starts_with('-'));
    let (english_files, mut faults) = read(LANGS[0].0);
    let english: Vec<String> = english_files.iter().flat_map(|(_, ids)| ids.clone()).collect();
    let mut out = String::new();
    let row = |out: &mut String, cells: &[String]| {
        if markdown {
            out.push_str(&format!("| {} |\n", cells.join(" | ")));
        } else {
            out.push_str(&format!("{:<24}{}\n", cells[0], cells[1..].iter().map(|c| format!("{c:>12}")).collect::<String>()));
        }
    };
    let head = |out: &mut String, title: &str, cells: &[String]| {
        if markdown {
            out.push_str(&format!("### {title}\n\n| {} |\n|{}\n", cells.join(" | "), "---|".repeat(cells.len())));
        } else {
            out.push_str(&format!("{title}\n"));
            row(out, cells);
        }
    };
    let cells = |names: &[&str]| names.iter().map(|n| n.to_string()).collect::<Vec<_>>();

    head(&mut out, "Languages", &cells(&["language", "coverage", "missing", "left over"]));
    let mut all = Vec::new();
    for (code, name) in LANGS {
        let (theirs, their_faults) = read(code);
        if *code != LANGS[0].0 {
            faults.extend(their_faults);
        }
        let c = coverage(&english_files, &theirs, code);
        row(
            &mut out,
            &[
                format!("{code} {name}"),
                format!("{:.0} %", percent(c.has, english.len())),
                c.missing.len().to_string(),
                c.extra.len().to_string(),
            ],
        );
        all.push(c);
    }

    out.push('\n');
    let mut columns = vec!["file".to_string(), "words".to_string()];
    columns.extend(all.iter().skip(1).map(|c| c.code.to_string()));
    head(&mut out, "Files", &columns);
    for (at, (file, ids)) in english_files.iter().enumerate() {
        let mut cells = vec![format!("{file}.ftl"), ids.len().to_string()];
        cells.extend(all.iter().skip(1).map(|c| format!("{:.0} %", percent(c.by_file[at], ids.len()))));
        row(&mut out, &cells);
    }

    let files = sources();
    if !files.is_empty() {
        out.push('\n');
        head(
            &mut out,
            "Migration (an estimate: words asked for, against words still written in the code)",
            &cells(&["file", "asked", "written", "migrated"]),
        );
        let (mut asked, mut written) = (0, 0);
        for (name, source) in &files {
            let m = migration(source, &english);
            faults.extend(m.unknown.iter().map(|id| format!("{name}: t!(\"{id}\"): English has no such word")));
            if m.asked + m.written == 0 {
                continue;
            }
            asked += m.asked;
            written += m.written;
            row(&mut out, &[name.clone(), m.asked.to_string(), m.written.to_string(), format!("{:.0} %", percent(m.asked, m.asked + m.written))]);
        }
        row(&mut out, &["all".into(), asked.to_string(), written.to_string(), format!("{:.0} %", percent(asked, asked + written))]);
    }

    if let Some(c) = only.and_then(|o| all.iter().find(|c| c.code == o)) {
        out.push_str(&format!("\n{} misses {} word(s):\n", c.code, c.missing.len()));
        for id in &c.missing {
            out.push_str(&format!("  {id}\n"));
        }
        if !c.extra.is_empty() {
            out.push_str(&format!("{} has {} word(s) English has not there:\n", c.code, c.extra.len()));
            for id in &c.extra {
                out.push_str(&format!("  {id}\n"));
            }
        }
    } else if let Some(o) = only {
        faults.push(format!("{o}: not a language tvty speaks"));
    }

    print!("{out}");
    if faults.is_empty() {
        return 0;
    }
    if markdown {
        println!("\n### Faults\n");
    } else {
        eprintln!();
    }
    for fault in &faults {
        if markdown {
            println!("- {fault}");
        } else {
            eprintln!("fault: {fault}");
        }
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every language's files read, say each word once, and have no word
    /// English has not (in the same file).
    #[test]
    fn every_language_reads() {
        let (english, faults) = read(LANGS[0].0);
        assert!(faults.is_empty(), "{faults:?}");
        for (code, _) in LANGS {
            let (theirs, faults) = read(code);
            assert!(faults.is_empty(), "{faults:?}");
            let c = coverage(&english, &theirs, code);
            assert!(c.extra.is_empty(), "{code} has words English has not: {:?}", c.extra);
        }
    }

    /// Every `t!("…")` of the sources names a word English has.
    #[test]
    fn every_word_asked_for_exists() {
        let english: Vec<String> = read(LANGS[0].0).0.into_iter().flat_map(|(_, ids)| ids).collect();
        let unknown: Vec<_> = sources()
            .iter()
            .flat_map(|(name, source)| migration(source, &english).unknown.into_iter().map(move |id| format!("{name}: {id}")))
            .collect();
        assert!(unknown.is_empty(), "words asked for and not in English: {unknown:?}");
    }

    /// The words whose id is built as tvty runs (`tickets-proposes-{kind}`):
    /// the sources' scan cannot see them.
    #[test]
    fn every_built_word_exists() {
        let english: Vec<String> = read(LANGS[0].0).0.into_iter().flat_map(|(_, ids)| ids).collect();
        let mut built = Vec::new();
        for kind in ["plan", "resolution", "wontfix", "escalation"] {
            built.push(format!("tickets-proposes-{kind}"));
            built.push(format!("tickets-your-proposal-{kind}"));
        }
        for kind in ["plan", "resolution", "wontfix"] {
            built.push(format!("tickets-yours-decide-{kind}"));
        }
        for stage in ["rejected", "closed-resolved", "closed", "resolved", "blocked", "snoozed", "pending", "open"] {
            built.push(format!("tickets-stage-{stage}"));
        }
        for group in ["live", "idle", "shut", "on-hub"] {
            built.push(format!("sessions-group-{group}"));
        }
        // Chosen in a match, then asked for: `t!(tip)`, `t!(mode)`.
        built.extend(
            ["sessions-new-tab-project", "sessions-new-tab-home", "sessions-mark-held-for-good", "sessions-mark-held-while", "sessions-mark-own"]
                .map(String::from),
        );
        for field in ["intent", "priority", "level", "scope"] {
            built.extend(crate::ui::fields::values(field).iter().map(|value| format!("fields-{field}-{value}")));
        }
        // Every setting, by its key; every page's and group's title.
        for setting in crate::settings::SCHEMA.0 {
            let id = format!("setting-{}", setting.key.replace('.', "-"));
            built.push(id.clone());
            built.push(format!("{id}.about"));
            if matches!(setting.kind, tvty_config::Kind::Toggle { .. }) {
                built.push(format!("{id}.on"));
                built.push(format!("{id}.off"));
            }
            built.push(crate::options::title_id(setting.page));
            built.push(crate::options::title_id(setting.group));
        }
        built.extend(crate::options::Section::ALL.map(|s| crate::options::title_id(s.title())));
        let missing: Vec<_> = built.iter().filter(|id| !english.contains(id)).collect();
        assert!(missing.is_empty(), "built ids English has not: {missing:?}");
    }

    #[test]
    fn written_words_are_told_from_code() {
        let source = "fn f() {\n    div().child(\"Stop them\")\n        .named(\"stop the loop\");\n    // \"a comment says\"\n    let c = '\"';\n    x(\"some-id\", \"a::path b\", t!(\"i18n-test-plural\"));\n}\n";
        let english = vec!["i18n-test-plural".to_string()];
        let m = migration(source, &english);
        assert_eq!((m.asked, m.written), (1, 1));
        assert!(m.unknown.is_empty());
    }

    #[test]
    fn a_word_falls_back_to_english_then_to_its_id() {
        assert_eq!(said(0, "i18n-test-plural", None).is_some(), true);
        assert_eq!(text("no-such-word", None), "no-such-word");
    }

    #[test]
    fn plurals_follow_the_language() {
        let mut args = FluentArgs::new();
        args.set("count", 1);
        assert_eq!(said(0, "i18n-test-plural", Some(&args)).unwrap(), "1 ticket");
        args.set("count", 0);
        assert_eq!(said(0, "i18n-test-plural", Some(&args)).unwrap(), "0 tickets");
        // French says the singular for none.
        assert_eq!(said(1, "i18n-test-plural", Some(&args)).unwrap(), "0 ticket");
    }

    #[test]
    fn a_language_is_read_from_a_locale() {
        assert_eq!(LANGS[spoken("fr_FR.UTF-8")].0, "fr");
        assert_eq!(LANGS[spoken("es-AR")].0, "es");
        assert_eq!(LANGS[spoken("de")].0, "en");
    }

    #[test]
    fn every_language_has_its_folder() {
        for (code, _) in LANGS {
            assert!(!files(code).is_empty(), "no assets/locales/{code}/");
        }
    }
}
