# tvty on egui-pop (experiment)

`egui/` rebuilds tvty's window as `tvty-egui` on
[egui-pop](https://github.com/quazardous/egui-pop), to measure how much of
tvty's look egui-pop lets an application control, down to the pixel.

- **Same data, side by side.** `egui/scripts/scene.py` dumps what tvty
  shows from an aiball daemon (the throwaway one: `make aiball-up`) as a
  scene; tvty-egui reads it (`TVTY_EGUI_SCENE`). Both windows run in wbox
  at 1280x800 and are compared region by region.
- **egui-pop's public API only.** tvty-egui does not paint around a gap:
  what egui-pop cannot express is changed in egui-pop, with an issue
  there labelled `tvty`; `egui/Cargo.toml` pins egui-pop's main at a
  known commit, moved forward as those land.
- **The terminal grid is out of scope** for now: the window's chrome is the
  point.

```sh
python3 egui/scripts/scene.py dev/fake-aiball/home/sock > dev/scene.json
cd egui && TVTY_EGUI_SCENE=../dev/scene.json cargo run
```

`TVTY_EGUI_SIDEBAR=1` starts with the sessions list open (over the
centre, as GPUI lays it), `TVTY_EGUI_TIPS=0` without the tip card, as
the tests' tvty has tips off. `TVTY_EGUI_FULL=1` opens the full ticket list at start (the panel's ⤢
opens it too, Escape closes it). `egui/scripts/install-desktop` puts
"tvty-egui Dev" in the desktop's launcher, on the debug build and
`dev/scene.json`. The fake board moves on its own (a ticket stops being
"hot"): dump the scene again right before comparing.

## Comparing

`egui/wbox/` runs tvty-egui in the same compositor and size as
`dev/tvty-wbox/`. `egui/scripts/compare.py` puts two captures side by
side, cropped and enlarged, the differences framed in red and numbered;
the images the issues show live on the `egui-shots` branch (images only,
no shared history).

## Text

GPUI on Linux draws ".SystemUIFont" with the first installed font of its
own fallback stack (Helvetica, Segoe UI, Ubuntu, Adwaita Sans, Cantarell,
Noto Sans, DejaVu Sans, Arial), looked up by exact family name: Adwaita
Sans on GNOME, DejaVu Sans on a bare box. Glyphs it lacks (the panel's ⤢)
come from cosmic-text's fallbacks (Noto Sans Symbols...) or FreeSerif.
The theme lists the same stack, without Ubuntu: egui-pop takes that
name for egui's bundled Ubuntu-Light before any system font, where GPUI
skips it when Ubuntu is not installed.

GPUI draws a variable face (Adwaita Sans) at an instance of its own
whatever the weight asked: its bold comes out near 500 (regular with the
GPUI of October 2026). egui-pop sets the face's `wght` axis, so tvty-egui
caps the weight at 500 to draw what GPUI draws; `TVTY_EGUI_WEIGHTS` sets
another cap (400 for that older GPUI, `all` for a box whose UI font is
static).

GPUI rounds a text's line box to whole pixels: `round(1.618 × size)`,
19, 23, 26 and 29 for 12, 14, 16 and 18. Laid out on fractions, a column
of rows drifts by a pixel every few rows. A line box (or a badge)
centred on a half pixel starts on the whole pixel above it.

GPUI centres a text's line box (1.618 × its size) by the font's own
metrics; egui centres its row. Text of 12 px lands on the same rows,
text of 14 px one row lower, the window's title two: a known difference,
left as it is.

## Where it stands

On egui-pop's main of 2026-11-07, this machine's UI font Noto Sans, share
of pixels that differ by more than 24 levels, same data on both sides:
the startup screen 1.4 % (the folded strip's icon, egui-pop#249, and the
text's vertical offset, egui-pop#250), the full ticket list 11.6 % (most
of it the rows' ages, taken minutes apart, and a row under the pointer).
The full list's side column takes GPUI's default width (a third of the
page, at most 460); its scroll bar is the kit's, set to GPUI's (6 px,
#4b4b4b, 4 px from the edge): the same pixels across, 4 px longer at each
end (egui-pop#251). The sessions list open: its sections are egui-pop's
(`sections`, the kit's `section-*` components set to GPUI's), every row
on GPUI's, shifted 10 px right by the strip's icon (egui-pop#249); the
count sits at the band's end (egui-pop#253) and the folded IDLE and
SHUT at the panel's bottom (egui-pop#252), where GPUI has them after
LIVE. The scene now holds the
daemon's shells (`session.list`), and reads the test env's tmux server and
state when `scripts/test-env` is sourced.

The tickets panel on a project (a terminal open): its header with the
project's name and alerts, then the tickets by band in egui-pop's
`sections`, rows 56 high (`stack.rows`), 14.3 % of the panel's pixels:
every band and row where GPUI has them in the first section, then the
sections' heights differ (egui-pop#252: GPUI sizes a section to its rows
and shrinks it only when the stack does not fit), the count at the band's
end (egui-pop#253), and the sections' scroll bars in the kit's default
white (egui-pop#254). `section-band.height` is 27 for GPUI's 28: the band
takes a pixel more than its token.

A ticket opened in the panel (a pending plan with options, the reply
box), 11.1 % of the panel's pixels: drawn with the kit's `button`,
`radio`, `field`, `card` and `tag-chip`, the theme setting each to
gpui-component's look (measured on GPUI's capture); every block within
two pixels of GPUI's. What is left is mostly text antialiasing, and the
reply button, which GPUI splits ("Reply" and its ▾).

On 2026-10-22 (GNOME, Adwaita Sans): the startup screen 0.6 %, the
sessions list open 1.3 %, the full ticket list 4.7 %.

Earlier, on the capture box:

Startup screen (ref1), share of pixels that differ from GPUI's by more
than 24 levels: 0.67 %. Every item sits within a pixel of GPUI's; what
is left is text antialiasing (GPUI's is subpixel, coloured; egui's is
grey), the vertical text offset above, and the title bar's vertical
gradient (#111111 to #171717; a kit has no gradient term).

## Gaps found

Each has an issue on egui-pop (label `tvty`), under quazardous/egui-pop#1.
"Done" means on egui-pop's main.

| Gap | Issue | State |
|---|---|---|
| The shell always draws a menu bar | egui-pop#2 | done: `AppBuilder::menu_bar(false)` |
| Side areas: egui's frame, a 48 px floor, egui's resize handle, no folded strip | egui-pop#3 | done: `area-strip`, `area-grip`, `side-area`, `AppBuilder::shell_panel_size`, `folded` |
| The dock's tab bar keeps egui_dock's default style | egui-pop#4 | done: `tab`, `tab-bar` kit components |
| Font families from tokens, weights, a size scale | egui-pop#5 | done |
| A side area lying over the centre, with a shadow | egui-pop#6 | done |
| The title bar always centres the title | egui-pop#7 | done: `title-bar.align` |
| An application's own icons (SVG), tinted | egui-pop#8 | done: `icon_set`, crisp at any size |
| No line around the window; side areas keep egui's 8 px margins | — | done: `window.rest.border-*`, `side-area.padding-*` |
| A double line under the title bar (egui's panel line and the kit's) | — | done: the kit draws it alone |
| The window buttons' glyphs are egui-pop's | — | done: glyph roles `window-*`, `indicator-size` |
| A family name resolves to egui's bundled face (Ubuntu-Light) before the system's | egui-pop#59 | done; the theme still leaves Ubuntu out |
| Scroll bars keep egui's default style: no kit component | egui-pop#60 | done: `scroll-bar`, the full list uses it |
| No collapsible sections (the sessions list's LIVE, IDLE, SHUT): no kit component nor widget | egui-pop#61 | done: `sections`, the sessions list uses it |
| A folded strip always shows an icon per panel: no bare grip | egui-pop#249 | open |
| No token for a family's vertical metrics (baseline offset, line height) | egui-pop#250 | open |
| A scroll bar's thumb runs to the ends of its track: no inset along it | egui-pop#251 | open |
| Sections always fill the panel: folded bands go to its bottom | egui-pop#252 | open |
| A section's count is always at the band's end | egui-pop#253 | open |
| A section's scroll bar keeps the kit's default colours | egui-pop#254 | open |
| Title bar buttons and section drags respond only sometimes (GNOME, Wayland) | egui-pop#256 | open |
| Shell panels: no least or largest width set by the application | egui-pop#258 | open |
