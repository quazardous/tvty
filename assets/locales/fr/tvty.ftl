# L'interface de tvty, en français. Mêmes ids que l'anglais
# (assets/locales/en/tvty.ftl), qui fait référence.

# Vérifie les pluriels (src/i18n.rs).
i18n-test-plural = { $count ->
    [one] {$count} ticket
   *[other] {$count} tickets
}

## Réglages > Apparence > Langue

setting-language = Langue
    .about = La langue de l'interface. Automatique : celle du système. Ce que les gens écrivent (tickets, commentaires, noms) reste tel quel.
setting-language-auto = Automatique ({$lang})
