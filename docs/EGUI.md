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

On egui-pop's main of 2026-11-07 evening (after the fixes for
egui-pop#249 to egui-pop#260), this machine's UI font Noto Sans, share of pixels that differ by
more than 24 levels, same data on both sides: the startup screen 1.3 %
(the text's vertical offset; egui-pop#250's tokens are not set yet), the
full ticket list 11.6 % (most of it the rows' ages, taken minutes apart,
and a row under the pointer). The full list's side column takes GPUI's
default width (a third of the page, at most 460); its scroll bar is the
kit's, set to GPUI's (6 px, #4b4b4b, 4 px from the edge and from both
ends). The sessions list open: its sections are egui-pop's (`sections`,
the kit's `section-*` components set to GPUI's), sized by their content,
the count after the title, every row and band on GPUI's. The folded
strips are bare, as GPUI's. The scene holds the daemon's shells
(`session.list`), and reads the test env's tmux server and state when
`scripts/test-env` is sourced.

The tickets panel on a project (a terminal open): its header with the
project's name and alerts, then the tickets by band in egui-pop's
`sections`, rows 56 high (`stack.rows`), 13.8 % of the panel's pixels:
the bands' counts, heights and scroll bars as GPUI's; what differs is
GPUI's open band holding fewer rows before it scrolls, the rows' ages,
and a tooltip on GPUI's capture. `section-band.height` is 27 for GPUI's
28: the band takes a pixel more than its token.

tvty's menu (its icon, F1): egui-pop's menus, one command per entry, set
by the theme to GPUI's rows (31 apart, 12 from the edge), width (336),
separators (edge to edge) and no icon gutter; the notes ("sessions
kept", "the board") and the ↗ of links are egui-pop's. 9.6 % of the
menu's pixels: keys and notes at GPUI's 12 (`menu-item.key-font-size`);
the ↗ stays at the rows' size (egui-pop#351).

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
| A folded strip always shows an icon per panel: no bare grip | egui-pop#249 | done: `AppBuilder::bare_strips` |
| No token for a family's vertical metrics (baseline offset, line height) | egui-pop#250 | done: `font.baseline-offset.*`; not set here yet |
| A scroll bar's thumb runs to the ends of its track: no inset along it | egui-pop#251 | done: `scroll-bar.margin-track` |
| Sections always fill the panel: folded bands go to its bottom | egui-pop#252 | done: `Sections::fill(false)` |
| A section's count is always at the band's end | egui-pop#253 | done: `section-count.placement: after-title` |
| A section's scroll bar keeps the kit's default colours | egui-pop#254 | done: section bars in the `scroll-bar` scope |
| Title bar buttons and section drags respond only sometimes (GNOME, Wayland) | egui-pop#256 | done: the stale press of egui-pop#270, checked on mutter |
| Menus: keys and notes at the labels' size; no token for a smaller one | egui-pop#269 | done: `menu-item.key-font-size` |
| After a window move by the title bar, nothing hovers until a click | egui-pop#270 | done: hovers right after a move, checked in labwc |
| Shell panels: no least width set while running, nor taken from the content | egui-pop#271 | done: `AppHandle::set_shell_panel_min_size`; not used here yet |
| Tooltips: no kit component (placement, padding, radius, size) | egui-pop#272 | done: `tooltip`, `TooltipExt::tip`; GPUI's frame, padding and text |
| Sections sized by content: title drags stop once the sections above are at their content's height (`TVTY_EGUI_FILL=1` fills, to compare) | egui-pop#332 | done: egui_stack 0.25.7 |
| Tooltips at the pointer: no horizontal offset; at the end edge pushed back under the pointer instead of flipping | egui-pop#350 | open |
| Menus: the link mark ↗ keeps the rows' size when keys are smaller | egui-pop#351 | open |
| Layers: a side area lying over the centre covers the app's overlays (the tips card); no per-window z-order group | egui-pop#352 | open; the card moves itself to the top |
| Shell panels: no least or largest width set by the application | egui-pop#258 | done: `shell_panel_min_size`, `shell_panel_max_size`, `centre_min_size` |
| Menus: row height, padding, width, separators not themeable; no note on an item | egui-pop#260 | done: `menu-item.height`, `padding-x`, `width`, `separator-inset`, `CommandSpec::menu_note`, `opens_link` |
