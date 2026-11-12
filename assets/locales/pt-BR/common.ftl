# A interface do tvty, em português do Brasil. Mesmos arquivos e mesmos
# ids que o inglês (assets/locales/en/), que é a referência; `tvty i18n`
# mede cada idioma.
# Palavras compartilhadas pelas superfícies.

# Verifica os plurais (src/i18n.rs).
i18n-test-plural = { $count ->
    [one] {$count} ticket
   *[other] {$count} tickets
}

## Durações curtas: "agora", "3min", "2h", "5d"

common-now = agora
common-minutes = {$n}min
common-hours = {$n}h
common-days = {$n}d
common-seconds = {$n}s
