# Contributing

Thanks for your interest in Terminal Velocity!

## Reporting bugs

Open an issue with:

- What you tried to do.
- What you expected to happen.
- What actually happened (paste error messages or unexpected output; a
  screenshot helps for anything visual).
- Your environment: OS and desktop (GNOME, KDE, …; Wayland or X11), GPU,
  `rustc --version`, and your tmux version.

A short reproducible case is worth pages of prose.

## Getting help

For usage questions, open an issue and tag it `question`.

## Building and testing

```bash
cargo build          # Linux needs the xcb, xkbcommon, vulkan, fontconfig,
                     # freetype and alsa development packages (GPUI)
make check           # build, launch in a headless compositor, screenshot
```

tvty is tested without a desktop and without spending model tokens: a nested
headless compositor to drive and screenshot it, scripted stand-ins for agent
sessions, and a throwaway aiball daemon for the ticket panel. See
[`docs/TESTING.md`](docs/TESTING.md); `make help` lists the targets.

## Sending a pull request

1. Fork and branch off `main` (one feature per branch).
2. Make the change. Keep diffs focused — small PRs review fast.
3. Show it works: a screenshot for anything visible, numbers from
   `TVTY_STATS` for anything touching rendering speed.
4. Run `cargo fmt` and keep `cargo build` free of warnings.
5. Open the PR. Describe the **what** and the **why**; mechanical diff
   details belong in the commit messages.

## Commit messages

[Conventional Commits](https://www.conventionalcommits.org/) (`feat:`,
`fix:`, `docs:`, `chore:` …). Subject line ≤ 72 characters, imperative mood,
then a blank line and a body that explains the why.

## Code of conduct

Be kind and assume good faith.
