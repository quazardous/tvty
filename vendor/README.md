# vendor

Crates tvty patches, each a copy of its published version with the smallest
change, until the fix is released upstream (`[patch.crates-io]` in the root
`Cargo.toml` points at them).

- `gpui_ce_linux` 0.1.0 (GPUI CE, Linux platform; Apache-2.0, see its
  `LICENSE-APACHE`): a Wayland keyboard event that comes before any keymap is
  ignored instead of panicking (`src/linux/wayland/client.rs`, the lines
  marked `tvty:`). Upstream: https://github.com/zed-industries/zed/issues/64660
- `gpui-ce` 0.2.2 (GPUI CE; Apache-2.0, see its `LICENSE-APACHE`), its
  examples, tests and benches left out: `ColorExt::blend` lays the other
  colour over with that colour's alpha, as Zed's GPUI does; it took the
  first colour's, so over an opaque one the themes' derived colours (a
  sidebar's, the hovers') came out see-through (`src/color.rs`, the lines
  marked `tvty:`).
  Two comments citing Zed's issues by number are written as links.
