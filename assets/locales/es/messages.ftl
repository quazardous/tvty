# La interfaz de tvty, en español. Los mismos archivos e ids que el
# inglés (assets/locales/en/), que es la referencia.
# Las notificaciones y los pequeños diálogos.

## Lo que avisó

messages-proposes-plan = propone un plan
messages-proposes-close = propone cerrar
messages-proposes-wontfix = propone cerrar sin corrección
messages-escalates = escala
messages-new-ticket = un ticket nuevo
messages-new-comment = un comentario nuevo
messages-new-ticket-moderate = un ticket nuevo por moderar
messages-missed = { $more ->
    [0] {$what} · mientras tvty estaba cerrado
    [one] {$what} · y {$more} más mientras tvty estaba cerrado
   *[other] {$what} · y {$more} más mientras tvty estaba cerrado
}
messages-dismiss = cerrar

## Las actualizaciones

messages-update-out = Ha salido Terminal Velocity {$latest} (tienes {$running}): {$how}
messages-update-how = Actualizaciones… en el menú lo instala
messages-update-dev = esta es una versión de desarrollo: actualiza su checkout (Actualizaciones… en el menú dice cómo)
messages-aiball-old = aiball {$version} es más antiguo de lo que Terminal Velocity necesita ({$needs}): Actualizaciones… en el menú lo actualiza

## Una sesión terminada

messages-detached = — desconectada por otro cliente
messages-ended = — la sesión terminó
messages-attach-again = Volver a conectar
messages-starting = Arrancando…
messages-restart = Reiniciar
messages-close = Cerrar
messages-copies-failed = hacer de los otros terminales copias

## Salir, volver a arrancar

messages-stopping = Parando las sesiones de Claude Code, luego saliendo…
messages-quit-stop-failed = parar al salir de tvty
messages-ran-on = {$agents} siguió: la parada no tuvo efecto
messages-restarted-as-were = { $count ->
    [one] {$count} sesión reiniciada como estaba, retomando su conversación
   *[other] {$count} sesiones reiniciadas como estaban, retomando su conversación
}
messages-restarted-fresh = { $count ->
    [one] {$count} sesión reiniciada de cero, retomando su conversación
   *[other] {$count} sesiones reiniciadas de cero, retomando su conversación
}
messages-restart-failed = reinicio de las sesiones paradas
messages-restart-tvty-failed = reiniciar tvty

## Qué conversación retomar

messages-its-agent = su agente
messages-ago = hace {$time}
messages-some-time-ago = hace un tiempo
messages-last-one = La última, {$when}:
messages-says-nothing = (aún no dice nada)
messages-quoted = «{$said}»
messages-older = { $count ->
    [one] {$count} más antigua también: /resume en Claude Code elige entre ellas.
   *[other] {$count} más antiguas también: /resume en Claude Code elige entre ellas.
}
messages-new-conversation = Conversación nueva
messages-resume-last = Retomar la última conversación
messages-start = Arrancar {$who}
messages-resume-question = Claude Code ya tiene conversaciones en {$cwd}, ninguna de un bucle: ¿cuál retoma {$who}?

## La tarjeta de consejos

messages-tip-title = ¿Sabías que…?
messages-tips-browsing = Consejos · {$at} / {$of}
messages-not-now = Ahora no
messages-previous = ‹ Anterior
messages-next = Siguiente ›
messages-show-all-again = Mostrarlos todos de nuevo
messages-got-it = Entendido
messages-next-tip = Consejo siguiente
messages-turn-off = Desactivar los consejos
messages-tips-off = Consejos desactivados: Ajustes > Disposición > Consejos los reactiva
messages-tips-again = Cada consejo se mostrará de nuevo, uno a la vez
