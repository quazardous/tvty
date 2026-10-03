# tvty's interface, in English: the reference. The other languages have
# the same files, with the same ids; `tvty i18n` measures them.
# Words the surfaces share.

# Checks the plurals (src/i18n.rs).
i18n-test-plural = { $count ->
    [one] {$count} ticket
   *[other] {$count} tickets
}
