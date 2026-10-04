//! Colour themes. tvty uses gpui-component's theme system: themes are its
//! JSON theme sets, a few bundled (from gpui-kit's collection) and any the
//! user drops in `$XDG_CONFIG_HOME/tvty/themes/`. The active theme gives the
//! whole window its colours — tvty's own views read [`p`], the kit's widgets
//! read the kit's theme — and the terminals their palette, unless the
//! terminals have a theme of their own (a dark terminal in a light window).

use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};

use gpui_kit::component::scroll::ScrollbarMode;
use gpui_kit::component::{Theme, ThemeMode, ThemeRegistry};
use gpui_kit::*;

/// Themes shipped with tvty: `(file, contents)`.
const BUNDLED: &[(&str, &str)] = &[
    ("catppuccin", include_str!("../themes/catppuccin.json")),
    ("eclipse", include_str!("../themes/eclipse.json")),
    ("everforest", include_str!("../themes/everforest.json")),
    ("flexoki", include_str!("../themes/flexoki.json")),
    ("gruvbox", include_str!("../themes/gruvbox.json")),
    ("solarized", include_str!("../themes/solarized.json")),
    ("tokyonight", include_str!("../themes/tokyonight.json")),
];

/// tvty's colours, drawn from the active theme.
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub bg: Hsla,
    /// Side panels, cards, the title bar.
    pub surface: Hsla,
    pub hover: Hsla,
    /// The selected row, the chosen card.
    pub active: Hsla,
    pub border: Hsla,
    pub text: Hsla,
    pub muted: Hsla,
    pub accent: Hsla,
    pub danger: Hsla,
    pub warning: Hsla,
    pub success: Hsla,
    pub info: Hsla,
    /// The veil behind the slider and the gallery.
    pub veil: Hsla,
}

/// The terminals' colours: the 16 ANSI ones, then text, background, cursor
/// and selection, as `0xRRGGBB`.
#[derive(Clone, Copy, Debug)]
pub struct TerminalColours {
    pub ansi: [u32; 16],
    pub foreground: u32,
    pub background: u32,
    pub cursor: u32,
    pub selection: u32,
}

static PALETTE: LazyLock<RwLock<Palette>> = LazyLock::new(|| RwLock::new(palette_of(&Theme::default())));
static TERMINAL: LazyLock<RwLock<TerminalColours>> =
    LazyLock::new(|| RwLock::new(terminal_of(&Theme::default())));
/// The terminals' own theme; `None`: the window's.
static TERMINAL_THEME: RwLock<Option<SharedString>> = RwLock::new(None);

/// The window's text size (the kit's, which sets the rem): by default, its
/// bounds, and the one chosen, `None` for the default.
pub const WINDOW_FONT_DEFAULT: f32 = 16.;
pub const WINDOW_FONT_MIN: f32 = 12.;
pub const WINDOW_FONT_MAX: f32 = 22.;
static WINDOW_FONT: RwLock<Option<f32>> = RwLock::new(None);

/// Sets the window's text size (`None`: the default); answers the size kept.
pub fn set_window_font(size: Option<f32>, cx: &mut App) -> f32 {
    let size = size.map(|s| s.round().clamp(WINDOW_FONT_MIN, WINDOW_FONT_MAX));
    *WINDOW_FONT.write().unwrap() = size;
    let size = size.unwrap_or(WINDOW_FONT_DEFAULT);
    Theme::global_mut(cx).font_size = px(size);
    size
}

pub fn window_font() -> f32 {
    WINDOW_FONT.read().unwrap().unwrap_or(WINDOW_FONT_DEFAULT)
}

/// The active palette.
pub fn p() -> Palette {
    *PALETTE.read().unwrap()
}

pub fn terminal() -> TerminalColours {
    *TERMINAL.read().unwrap()
}

/// Loads the bundled and the user's themes, then applies `name` (or the
/// kit's default dark theme) and the terminals' own theme, if any.
pub fn init(name: Option<&str>, terminal: Option<&str>, cx: &mut App) {
    load(cx);
    *TERMINAL_THEME.write().unwrap() = terminal
        .filter(|n| ThemeRegistry::global(cx).themes().contains_key(*n))
        .map(SharedString::from);
    let name = known_or_default(name, cx);
    apply(&name, None, cx);
}

/// `name` when it is a known theme; otherwise the kit's default dark one.
pub fn known_or_default(name: Option<&str>, cx: &App) -> SharedString {
    name.filter(|n| ThemeRegistry::global(cx).themes().contains_key(*n))
        .map(SharedString::from)
        .unwrap_or_else(|| ThemeRegistry::global(cx).default_dark_theme().name.clone())
}

/// (Re)reads the themes: the bundled ones, then the user's directory — so a
/// theme file dropped or edited there shows the next time the list opens.
pub fn load(cx: &mut App) {
    let registry = ThemeRegistry::global_mut(cx);
    for (file, content) in BUNDLED {
        if let Err(error) = registry.load_themes_from_str(&with_visible_hovers(content)) {
            log::warn!("theme {file}: {error}");
        }
    }
    let Some(dir) = user_dir() else { return };
    let Ok(entries) = std::fs::read_dir(&dir) else { return };
    for path in entries.flatten().map(|e| e.path()) {
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let loaded = std::fs::read_to_string(&path)
            .map_err(anyhow::Error::from)
            .and_then(|content| registry.load_themes_from_str(&with_visible_hovers(&content)));
        if let Err(error) = loaded {
            log::warn!("theme {}: {error}", path.display());
        }
    }
}

/// The themes to choose from, by name: dark ones first.
pub fn names(cx: &App) -> Vec<(SharedString, bool)> {
    let mut themes: Vec<(SharedString, bool)> = ThemeRegistry::global(cx)
        .themes()
        .values()
        .map(|t| (t.name.clone(), t.mode.is_dark()))
        .collect();
    themes.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    themes
}

pub fn current(cx: &App) -> SharedString {
    Theme::global(cx).theme_name().clone()
}

/// The terminals' own theme; `None` when they follow the window's.
pub fn current_terminal() -> Option<SharedString> {
    TERMINAL_THEME.read().unwrap().clone()
}

/// Gives the terminals a theme of their own, or (`None`) the window's.
pub fn apply_terminal(name: Option<&str>, cx: &App) {
    *TERMINAL_THEME.write().unwrap() = name.map(SharedString::from);
    *TERMINAL.write().unwrap() = terminal_colours(cx);
}

fn terminal_colours(cx: &App) -> TerminalColours {
    let own = TERMINAL_THEME.read().unwrap().clone();
    match own.and_then(|name| ThemeRegistry::global(cx).themes().get(&name).cloned()) {
        Some(config) => {
            let mut theme = Theme::default();
            theme.apply_config(&config);
            terminal_of(&theme)
        }
        None => terminal_of(Theme::global(cx)),
    }
}

/// Makes `name` the active theme, for the kit's widgets and tvty's views.
pub fn apply(name: &str, window: Option<&mut Window>, cx: &mut App) {
    let Some(config) = ThemeRegistry::global(cx).themes().get(name).cloned() else {
        return;
    };
    let mode = config.mode;
    {
        let theme = Theme::global_mut(cx);
        match mode {
            ThemeMode::Dark => theme.dark_theme = config,
            ThemeMode::Light => theme.light_theme = config,
        }
    }
    Theme::change(mode, window, cx);
    // A theme may carry its own text size: the user's choice stays.
    Theme::global_mut(cx).font_size = px(window_font());
    // The kit's monospace font, where it uses one, is the terminal's.
    Theme::global_mut(cx).mono_font_family = crate::fonts::mono().into();
    // Scrollbars stay visible: they say where a long list stands.
    Theme::set_scrollbar_mode(ScrollbarMode::Always, cx);
    let theme = Theme::global(cx);
    *PALETTE.write().unwrap() = palette_of(theme);
    *TERMINAL.write().unwrap() = terminal_colours(cx);
}

/// A theme set as read, its themes whose main button's hover is its own
/// colour (some give it with a touch of transparency only, and the button
/// then shows nothing under the pointer) given one: lighter on a dark theme,
/// darker on a light one. The text as it was when it cannot be read.
fn with_visible_hovers(content: &str) -> String {
    let Ok(mut set) = serde_json::from_str::<serde_json::Value>(content) else { return content.to_string() };
    let Some(themes) = set.get_mut("themes").and_then(serde_json::Value::as_array_mut) else { return content.to_string() };
    for theme in themes {
        let dark = theme.get("mode").and_then(serde_json::Value::as_str) == Some("dark");
        let Some(colors) = theme.get_mut("colors").and_then(serde_json::Value::as_object_mut) else { continue };
        if colors.contains_key("button.primary.hover.background") {
            continue;
        }
        let colour = |key: &str| colors.get(key).and_then(serde_json::Value::as_str).and_then(parse_hex);
        let (Some(base), Some(hover)) = (colour("primary.background"), colour("primary.hover.background")) else { continue };
        if !same_colour(base, hover) {
            continue;
        }
        let derived = Hsla { l: (base.l + if dark { 0.08 } else { -0.08 }).clamp(0., 1.), a: 1., ..base }.to_rgb();
        let byte = |v: f32| (v.clamp(0., 1.) * 255.).round() as u8;
        let hex = format!("#{:02x}{:02x}{:02x}", byte(derived.r), byte(derived.g), byte(derived.b));
        colors.insert("button.primary.hover.background".into(), hex.into());
    }
    set.to_string()
}

/// `#rrggbb` or `#rrggbbaa`, its alpha aside.
fn parse_hex(text: &str) -> Option<Hsla> {
    let digits = text.strip_prefix('#')?;
    let rgb = u32::from_str_radix(digits.get(..6)?, 16).ok()?;
    Some(Hsla::from(gpui_kit::rgb(rgb)))
}

/// Two colours a person would not tell apart (alpha aside).
fn same_colour(a: Hsla, b: Hsla) -> bool {
    let (a, b) = (a.to_rgb(), b.to_rgb());
    (a.r - b.r).abs() + (a.g - b.g).abs() + (a.b - b.b).abs() < 0.03
}

fn user_dir() -> Option<PathBuf> {
    Some(crate::config::dir(crate::config::Place::Config)?.join("themes"))
}

fn palette_of(theme: &Theme) -> Palette {
    Palette {
        bg: theme.background,
        surface: theme.sidebar,
        hover: theme.list_hover,
        active: theme.list_active,
        border: theme.border,
        text: theme.foreground,
        muted: theme.muted_foreground,
        // Not `primary`: some themes make it near white or black.
        accent: theme.blue,
        danger: theme.danger,
        warning: theme.warning,
        success: theme.success,
        info: theme.info,
        veil: theme.background.opacity(0.92),
    }
}

fn terminal_of(theme: &Theme) -> TerminalColours {
    let hex = |c: Hsla| {
        let c = c.to_rgb();
        let byte = |v: f32| (v.clamp(0., 1.) * 255.).round() as u32;
        byte(c.r) << 16 | byte(c.g) << 8 | byte(c.b)
    };
    let dark = theme.mode.is_dark();
    // Black and white follow the theme's light: on a light theme, "black"
    // text must stay dark and "white" stay light.
    let (black, white, bright_white) = if dark {
        (theme.muted, theme.foreground.opacity(0.85), theme.foreground)
    } else {
        (theme.foreground, theme.muted, theme.background)
    };
    let ansi = [
        black,
        theme.red,
        theme.green,
        theme.yellow,
        theme.blue,
        theme.magenta,
        theme.cyan,
        white,
        theme.muted_foreground,
        theme.red_light,
        theme.green_light,
        theme.yellow_light,
        theme.blue_light,
        theme.magenta_light,
        theme.cyan_light,
        bright_white,
    ]
    .map(|c| hex(blend(c, theme.background)));
    TerminalColours {
        ansi,
        foreground: hex(theme.foreground),
        background: hex(theme.background),
        cursor: hex(theme.caret),
        selection: hex(blend(theme.selection, theme.background)),
    }
}

/// The tips' own colour: a violet — not the accent's blue, nor a state's
/// green, yellow, orange or red. Fixed rather than a theme's magenta, which
/// some terminal themes wash out to a grey; lighter on a dark window,
/// darker on a light one.
pub fn tip() -> Hsla {
    if p().bg.l < 0.5 { hsla(270. / 360., 0.75, 0.74, 1.) } else { hsla(270. / 360., 0.6, 0.45, 1.) }
}

/// Where a dragged tab would go: an orange, apart from the accent's blue
/// that the tab being moved keeps.
pub fn drop_target() -> Hsla {
    if p().bg.l < 0.5 { hsla(28. / 360., 0.9, 0.6, 1.) } else { hsla(28. / 360., 0.85, 0.45, 1.) }
}

/// A restart asked and waiting: the theme's warning turned to orange, so
/// that it reads as the update's yellow, one step further.
pub fn pending() -> Hsla {
    let warning = p().warning;
    hsla(warning.h * 0.6, warning.s.max(0.7), warning.l, warning.a)
}

/// What was imported, not typed: a choice filled from a folder's
/// configuration. A teal, apart from the accent, the states and the tips.
pub fn imported() -> Hsla {
    if p().bg.l < 0.5 { hsla(172. / 360., 0.6, 0.55, 1.) } else { hsla(172. / 360., 0.75, 0.3, 1.) }
}

/// Readable text on a `background`: dark on light colours, light on dark.
pub fn on(background: Hsla) -> Hsla {
    let c = background.to_rgb();
    let luminance = 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
    if luminance > 0.55 {
        Hsla::from(Rgba { r: 0.08, g: 0.08, b: 0.08, a: 1. })
    } else {
        gpui_kit::white()
    }
}

/// A colour with its transparency folded onto `under`.
fn blend(colour: Hsla, under: Hsla) -> Hsla {
    let (c, u) = (colour.to_rgb(), under.to_rgb());
    let a = c.a;
    Rgba {
        r: c.r * a + u.r * (1. - a),
        g: c.g * a + u.g * (1. - a),
        b: c.b * a + u.b * (1. - a),
        a: 1.,
    }
    .into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_main_button_without_a_hover_of_its_own_gets_one() {
        let set = r##"{"name":"t","themes":[
            {"name":"Same","mode":"dark","colors":{"primary.background":"#1290C3","primary.hover.background":"#1290C3ee"}},
            {"name":"Own","mode":"dark","colors":{"primary.background":"#1290C3","primary.hover.background":"#40a0d0"}},
            {"name":"Set","mode":"light","colors":{"primary.background":"#007acc","primary.hover.background":"#007acc","button.primary.hover.background":"#005a9c"}}
        ]}"##;
        let fixed: serde_json::Value = serde_json::from_str(&super::with_visible_hovers(set)).unwrap();
        let hover = |i: usize| fixed["themes"][i]["colors"].get("button.primary.hover.background").and_then(|v| v.as_str()).map(str::to_string);
        // The same colour: a lighter one, on a dark theme.
        let derived = hover(0).expect("a hover given");
        let (base, given) = (super::parse_hex("#1290C3").unwrap(), super::parse_hex(&derived).unwrap());
        assert!(given.l > base.l, "{derived}");
        // A hover of its own, or one the theme already gives the button: kept.
        assert_eq!(hover(1), None);
        assert_eq!(hover(2).as_deref(), Some("#005a9c"));
    }
}
