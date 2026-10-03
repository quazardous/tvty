# L'interface de tvty, en français. Mêmes fichiers et mêmes ids que
# l'anglais (assets/locales/en/), qui fait référence.
# Les mots que les surfaces partagent.

# Vérifie les pluriels (src/i18n.rs).
i18n-test-plural = { $count ->
    [one] {$count} ticket
   *[other] {$count} tickets
}
