# La interfaz de tvty, en español. Los mismos ids que el inglés
# (assets/locales/en/tvty.ftl), que es la referencia.

# Comprueba los plurales (src/i18n.rs).
i18n-test-plural = { $count ->
    [one] {$count} ticket
   *[other] {$count} tickets
}

## Ajustes > Apariencia > Idioma

setting-language = Idioma
    .about = El idioma de la interfaz. Automático: el del sistema. Lo que escribe la gente (tickets, comentarios, nombres) queda tal cual.
setting-language-auto = Automático ({$lang})
