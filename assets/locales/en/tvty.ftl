# tvty's interface, in English: the reference. The other languages are
# measured against it (`tvty i18n`). Ids say where the word shows:
# <surface>-<what>.

# Checks the plurals (src/i18n.rs).
i18n-test-plural = { $count ->
    [one] {$count} ticket
   *[other] {$count} tickets
}

## Settings > Appearance > Language

setting-language = Language
    .about = The interface's language. Automatic: the system's. What people write (tickets, comments, names) stays as written.
setting-language-auto = Automatic ({$lang})
