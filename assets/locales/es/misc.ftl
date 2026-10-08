# La interfaz de tvty, en español. Los mismos archivos e ids que el
# inglés (assets/locales/en/), que es la referencia.
# Lo que queda: el menú y las marcas del terminal, el visor de imágenes,
# algunos avisos y errores.

## Avisos

misc-aiball-started = aiball no estaba en marcha: arrancado
misc-aiball-silent = aiball no responde en {$at}
misc-service-failed = el servicio de aiball no arrancó: {$error}
misc-no-service = {$why}; no hay servicio de aiball que arrancar
misc-filed = creado — {$title}
misc-filed-moderate = {$title} — un ticket nuevo por moderar
misc-filed-other = {$title} — un ticket nuevo
misc-something-new = algo nuevo
misc-held = {$said}: su bucle está retenido hasta que lo liberes
misc-go-to = ir a
misc-go-to-what = ir a {$what}
misc-not-a-ticket = {$typed} no es un ticket: un número, o el enlace #C. de un comentario
misc-copied = Copiado al portapapeles
misc-new-terminal = terminal nuevo
misc-no-folder = ninguna carpeta conocida para {$project}
misc-not-a-directory = {$folder} no es una carpeta
misc-not-idle = su Claude está trabajando: cuando esté en reposo
misc-stop-not-received = {$agent}: ninguno de sus bucles recibió la parada (no corre, o no donde aiball puede alcanzarlo)

## El terminal

misc-open-link = Abrir enlace
misc-copy-link = Copiar enlace
misc-copy = Copiar
misc-paste = Pegar
misc-frozen = congelado · selección
misc-frozen-tip = La pantalla queda congelada mientras hay texto seleccionado; la sesión sigue. Un clic aquí, Esc o una tecla la libera.
misc-size-taken = tamaño tomado por otro cliente · clic para recuperarlo
misc-session-ended = La sesión terminó.
misc-hub-session = Esta sesión corre en el hub de aiball, otra máquina: no se puede abrir desde esta. Sus tickets están en el panel.
misc-all-terminals = Todos los terminales
misc-gallery-hint = escribe para filtrar · flechas mueven · enter abre · esc cierra

## El modelo del agente, sus límites, sus rechazos

misc-price = {$input} / {$output} por M tokens
misc-model-out = ha salido {$name}
misc-prices-from = precios de {$catalog}
misc-limit = límite de uso alcanzado
misc-limit-resets = límite de uso alcanzado · se restablece {$resets}
misc-denials = { $count ->
    [one] {$count} llamada de herramienta rechazada por Claude Code en la última hora{$ago}{$why} — un agente rechazado se detiene ahí
   *[other] {$count} llamadas de herramienta rechazadas por Claude Code en la última hora{$ago}{$why} — un agente rechazado se detiene ahí
}
misc-denials-last = , la última hace {$ago}

## Las imágenes

misc-image-too-large = imagen demasiado grande para mostrarla aquí
misc-image-unavailable = imagen no disponible
misc-fitted = {" "}(ajustada)
misc-viewer-hint = rueda o + − zoom · 1 tamaño real · 0 ajustar · arrastrar para mover · Esc

## The usage arrow in the top bar

usage-ratio = ×{$ratio}
usage-points = {$points} pts
usage-wall = límite {$left}
usage-no-wall = sin límite
usage-window-five_hour = Ventana de 5 h
usage-window-seven_day = Semana
usage-tip-window = {$window}: {$used} % usado, {$expected} % a ritmo constante · se reinicia {$resets} · {$end}
usage-tip-wall = a este ritmo, la cuota se agota en {$left}
usage-tip-lasts = a este ritmo, dura hasta el reinicio
usage-tip-click = Un clic: la diferencia dicha de otra forma.
