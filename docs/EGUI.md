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

## Comparing

`egui/wbox/` runs tvty-egui in the same compositor and size as
`dev/tvty-wbox/`. `egui/scripts/compare.py` puts two captures side by
side, cropped and enlarged, the differences framed in red and numbered;
the images the issues show live on the `egui-shots` branch (images only,
no shared history).

## Gaps found

Each has an issue on egui-pop (label `tvty`), under quazardous/egui-pop#1.

| Gap | Issue | State |
|---|---|---|
| The shell always draws a menu bar | egui-pop#2 | fixed: `AppBuilder::menu_bar(false)` |
| Side areas: egui's frame, a 48 px floor, egui's resize handle, no folded strip | egui-pop#3 | open |
| The dock's tab bar keeps egui_dock's default style | egui-pop#4 | open |
| Font families from tokens, weights, a size scale | egui-pop#5 | families and weights done (`egui_pop_core::fonts`), size scale open |
| A side area lying over the centre, with a shadow | egui-pop#6 | open |
| The title bar always centres the title | egui-pop#7 | open |
| An application's own icons (SVG), tinted | egui-pop#8 | open |
