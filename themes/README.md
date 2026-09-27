# Bundled themes

These theme sets come from the collection of
[gpui-kit](https://github.com/longbridge/gpui-kit/tree/main/themes)
(Apache-2.0), in gpui-component's theme format — `eclipse.json` is tvty's
own, from the colours of Eclipse's dark theme. Each file names the author of
the palette it adapts:

| File | Palette | By |
|---|---|---|
| `catppuccin.json` | [Catppuccin](https://github.com/catppuccin/catppuccin) | Catppuccin |
| `eclipse.json` | [Eclipse Classic Dark](https://github.com/lorenzobilli/Eclipse-color-theme), Eclipse's dark theme as a VS Code one | Lorenzo Billi |
| `everforest.json` | [Everforest](https://github.com/sainnhe/everforest) | sainnhe |
| `flexoki.json` | [Flexoki](https://github.com/kepano/flexoki) | kepano |
| `gruvbox.json` | [Gruvbox](https://github.com/morhetz/gruvbox) | Pavel Pertsev |
| `solarized.json` | [Solarized](https://ethanschoonover.com/solarized) | Ethan Schoonover |
| `tokyonight.json` | [Tokyo Night](https://github.com/folke/tokyonight.nvim) | Folke Lemaitre |

tvty embeds them at build time. Your own themes, in the same format, go in
`~/.config/tvty/themes/`: they show in the theme list the next time it opens.
