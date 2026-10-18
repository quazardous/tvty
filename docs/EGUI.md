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

## Gaps found

| Gap | Where in egui-pop | State |
|---|---|---|
| The shell always draws a menu bar | `App::shell` (egui-pop-api) | open |
| Side areas are egui panels with egui's frame, a 48 px floor, egui's resize handle | `workbench::show` | open |
| The dock's tab bar keeps egui_dock's default style | `workbench::draw_area` | open |
