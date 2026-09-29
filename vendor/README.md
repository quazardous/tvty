# vendor

Crates tvty patches, each a copy of its published version with the smallest
change, until the fix is released upstream (`[patch.crates-io]` in the root
`Cargo.toml` points at them).

- `gpui-pre-linux` 0.3.6 (Zed's GPUI, Linux platform; Apache-2.0, see its
  `LICENSE-APACHE`): a Wayland keyboard event that comes before any keymap is
  ignored instead of panicking (`src/linux/wayland/client.rs`, the lines
  marked `tvty:`). Upstream: https://github.com/zed-industries/zed/issues/64660
