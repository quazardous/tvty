# La interfaz de tvty, en español. Los mismos archivos e ids que el
# inglés (assets/locales/en/), que es la referencia.
# Las palabras que comparten las superficies.

# Comprueba los plurales (src/i18n.rs).
i18n-test-plural = { $count ->
    [one] {$count} ticket
   *[other] {$count} tickets
}

## Duraciones cortas: «ahora», «3 min», «2 h», «5 d»

common-now = ahora
common-minutes = {$n} min
common-hours = {$n} h
common-days = {$n} d
