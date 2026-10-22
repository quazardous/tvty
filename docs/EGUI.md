# tvty on egui-pop (experiment)

`egui/` rebuilds tvty's window as `tvty-egui` on
[egui-pop](https://github.com/quazardous/egui-pop), to measure how much of
tvty's look egui-pop lets an application control, down to the pixel.

- **Same data, side by side.** `egui/scripts/scene.py` dumps what tvty
  shows from an aiball daemon (the throwaway one: `make aiball-up`) as a
  scene; tvty-egui reads it (`TVTY_EGUI_SCENE`). Both windows run in wbox
  at 1280x800 and are compared region by region.
- **egui-pop's public API only.** tvty-egui does not paint around a gap:
  what egui-pop cannot express is changed in egui-pop (its tvty branch,
  checked out next door as `../egui-pop`), with an issue there labelled
  `tvty`.
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

GPUI draws a variable face (Adwaita Sans) at its default instance
whatever the weight asked: its bold is regular there. egui-pop sets the
face's `wght` axis, so tvty-egui drops the weight to draw what GPUI
draws; `TVTY_EGUI_WEIGHTS` keeps it, for a box whose UI font is static.

GPUI rounds a text's line box to whole pixels: `round(1.618 × size)`,
19, 23, 26 and 29 for 12, 14, 16 and 18. Laid out on fractions, a column
of rows drifts by a pixel every few rows. A line box (or a badge)
centred on a half pixel starts on the whole pixel above it.

GPUI centres a text's line box (1.618 × its size) by the font's own
metrics; egui centres its row. Text of 12 px lands on the same rows,
text of 14 px one row lower, the window's title two: a known difference,
left as it is.

## Where it stands

On this machine (GNOME, Adwaita Sans), share of pixels that differ by
more than 24 levels, same data on both sides: the startup screen 0.6 %,
the sessions list open 1.3 %, the full ticket list 4.7 % (its scroll bar
and the text's antialiasing). Every item sits on GPUI's pixel or one
off.

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
| Font families from tokens, weights, a size scale | egui-pop#5 | families and weights done (`egui_pop_core::fonts`), size scale open |
| A side area lying over the centre, with a shadow | egui-pop#6 | open |
| The title bar always centres the title | egui-pop#7 | done: `title-bar.align` |
| An application's own icons (SVG), tinted | egui-pop#8 | done: `icon_set`, crisp at any size |
| No line around the window; side areas keep egui's 8 px margins | — | done: `window.rest.border-*`, `side-area.padding-*` |
| A double line under the title bar (egui's panel line and the kit's) | — | done: the kit draws it alone |
| The window buttons' glyphs are egui-pop's | — | done: glyph roles `window-*`, `indicator-size` |
| A family name resolves to egui's bundled face (Ubuntu-Light) before the system's | egui-pop#59 | open; the theme leaves Ubuntu out |
| Scroll bars keep egui's default style: no kit component | egui-pop#60 | open |
| No collapsible sections (the sessions list's LIVE, IDLE, SHUT): no kit component nor widget | egui-pop#61 | open; tvty-egui paints the bands |
