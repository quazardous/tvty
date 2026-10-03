# L'interface de tvty, en français. Mêmes fichiers et mêmes ids que
# l'anglais (assets/locales/en/), qui fait référence.
# Les mots que les surfaces partagent.

# Vérifie les pluriels (src/i18n.rs).
i18n-test-plural = { $count ->
    [one] {$count} ticket
   *[other] {$count} tickets
}

## Durées courtes : « maint. », « 3 min », « 2 h », « 5 j »

common-now = maint.
common-minutes = {$n} min
common-hours = {$n} h
common-days = {$n} j
common-seconds = {$n} s
