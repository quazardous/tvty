# Die Oberfläche von tvty, auf Deutsch. Dieselben Dateien und ids wie
# Englisch (assets/locales/en/), die Referenz.
# Wörter, die die Oberflächen teilen.

# Prüft die Pluralformen (src/i18n.rs).
i18n-test-plural = { $count ->
    [one] {$count} Ticket
   *[other] {$count} Tickets
}

## Kurze Dauern: „jetzt“, „3 Min.“, „2 Std.“, „5 T.“

common-now = jetzt
common-minutes = {$n} Min.
common-hours = {$n} Std.
common-days = {$n} T.
common-seconds = {$n} s
