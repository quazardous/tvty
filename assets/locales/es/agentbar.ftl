# La interfaz de tvty, en español. Los mismos archivos e ids que el
# inglés (assets/locales/en/), que es la referencia.
# La barra del agente, bajo su terminal (src/shell/agentbar.rs).

## Quién lleva el bucle

agentbar-afk-auto = auto
agentbar-afk-hold-10m = retener 10 min
agentbar-afk-hold = retener
agentbar-afk-boot = … arranque
agentbar-afk-arming-tip = armado: el muñequito muestra el modo elegido con F9, vigente 3 s después de la última pulsación — ▶ o ‖ indica el vigente hasta entonces
agentbar-afk-tip = quién lleva el bucle: ▶ él solo, ‖ retenido para ti (o mientras escribes); el muñequito es el modo AFK — gris: estás ausente, el bucle va solo; los segundos de una retención de 10 min; ∞ retenido. F9 lo hace girar (auto → 10 min → ∞), vigente 3 s después de la última pulsación; un clic elige

## Lo que hace su Claude

agentbar-working = trabajando
agentbar-starting = arrancando
agentbar-idle = en reposo
agentbar-offline = desconectado
agentbar-boot-left = {" · "}quedan {$seconds} s
agentbar-boot-tip = su bucle arranca: Claude se carga, puede retomar su conversación o compactar; el bucle lo despierta cuando termina el arranque (30 s como mínimo, más mientras se vea una reanudación o una compactación)
agentbar-state-tip = lo que hace su Claude, y desde cuándo
agentbar-dialog = espera una respuesta

## Reiniciar su Claude

agentbar-restarting = reiniciando…
agentbar-restarting-tip = su Claude se reinicia, retomando su conversación
agentbar-restart-pending = reinicio pendiente
agentbar-restart-pending-tip = se ha pedido un reinicio: su Claude se reinicia en cuanto esté en reposo, retomando su conversación
agentbar-update = actualización
agentbar-update-tip = su Claude Code instaló una actualización: un clic pide reiniciarlo (enseguida si está en reposo, si no en cuanto lo esté), retomando su conversación
agentbar-restart-ask-busy = ¿Reiniciar su Claude en cuanto esté en reposo?
agentbar-restart-ask = ¿Reiniciar su Claude ahora?
agentbar-restart = Reiniciar
agentbar-restart-resumed = su conversación se retoma
agentbar-cancel = Cancelar
agentbar-restart-done = el Claude de {$agent} se reinicia en cuanto esté en reposo, retomando su conversación
agentbar-restart-failed = reinicio del Claude de {$agent}

## Dónde corre su bucle, quién tiene los mandos

agentbar-place-host = host
agentbar-hands-copy = copia
agentbar-hands-controls = mandos
agentbar-moving = moviendo…
agentbar-runs-hosted = su bucle corre en el host de sesiones de aiball;
agentbar-runs-mux = su bucle corre en {$mux} (claude-loop);
agentbar-copy-tip = este terminal es una copia: miras, nada de lo que escribes llega a su Claude, y la sesión conserva su tamaño.
agentbar-controls-tip = tienes los mandos, compartidos con cualquier otro cliente: el tamaño sigue a quien escribe el último.
agentbar-others-attached = { $others ->
    [one] {$others} otro cliente conectado ({$typing} con los mandos).
   *[other] {$others} otros clientes conectados ({$typing} con los mandos).
}
agentbar-proxy-alive = El proxy de terminal delante de Claude está vivo.
agentbar-place-click = Un clic: tomar o dejar los mandos, mover el bucle.
agentbar-others-type = el terminal de claude-loop también escribe en él
agentbar-take-controls = Tomar los mandos
agentbar-leave-copy = Dejar por una copia
agentbar-close-others = Cerrar los demás ({$others})
agentbar-close-others-tip = { $others ->
    [one] el otro cliente conectado a esta sesión la deja (el terminal de claude-loop, otro Terminal Velocity); su Claude y este terminal siguen
   *[other] los {$others} otros clientes conectados a esta sesión la dejan (el terminal de claude-loop, otro Terminal Velocity); su Claude y este terminal siguen
}
agentbar-move-into = Mover a {$mux}
agentbar-move-to-host = Mover al host
agentbar-move-tip-into = su Claude se reinicia en {$mux}, retomando su conversación
agentbar-move-tip-to-host = su Claude se reinicia en el host de aiball, retomando su conversación
agentbar-move-interrupts = {" — "}está trabajando: el traslado lo interrumpe
agentbar-moved-into = {$agent} movido a {$mux}, su conversación retomada
agentbar-moved-to-host = {$agent} movido al host de aiball, su conversación retomada
agentbar-move-failed-into = traslado de {$agent} a {$mux}
agentbar-move-failed-to-host = traslado de {$agent} al host de aiball
agentbar-hold-failed = retención de {$agent}
agentbar-closed-others = { $count ->
    [one] cerrado el otro cliente de esta sesión
   *[other] cerrados los {$count} otros clientes de esta sesión
}
agentbar-closed-others-all = cerrados los otros clientes de esta sesión
agentbar-closed-others-asked = se pidió al host de la sesión cerrar sus otros clientes
agentbar-others-left = { $count ->
    [one] {$count} otro cliente sigue conectado: {$mux} aún no sabe distinguirlo de este
   *[other] {$count} otros clientes siguen conectados: {$mux} aún no sabe distinguirlos de este
}
agentbar-close-others-failed = cerrar los otros clientes

## Remote Control, alertas, el prompt

agentbar-rc-on = Remote Control está activo: este Claude se puede retomar desde claude.ai y la app móvil
agentbar-rc-off = Remote Control está inactivo. /rc en la sesión lo activa; los bucles de una carpeta lo toman de los ajustes del proyecto
agentbar-trust = ¿confiar en esta carpeta?
agentbar-not-logged-in = sin sesión iniciada
agentbar-api-unreachable = API inaccesible
agentbar-link-down = enlace del bucle caído
agentbar-aiball-unreachable = aiball inaccesible
agentbar-prompt-input = el prompt de Claude está en pantalla, con texto aún no enviado
agentbar-prompt-empty = el prompt de Claude está en pantalla, vacío
agentbar-typing = un humano escribió en su terminal hace un momento: el bucle espera
agentbar-zen = zen
agentbar-zen-tip = modo zen: el bucle se calla

## Sus contadores

agentbar-all = todos:{$count}
agentbar-all-tip = a: todos los tickets abiertos del proyecto
agentbar-backlog = backlog:{$count}
agentbar-backlog-tip = b: su backlog, los tickets que debe mirar; un clic los lista
agentbar-events = evts:{$count}
agentbar-events-tip = e: sus eventos aún no vistos — pings, respuestas, decisiones que lo esperan
agentbar-holds = tiene:{$count}
agentbar-holds-tip = los tickets que tiene
agentbar-wake-tip = hay trabajo esperando al bucle: despierta a su Claude al terminar la cuenta atrás
agentbar-pending-tip = hay trabajo esperando al bucle (eventos o backlog)

## Su backlog, sobre la barra

agentbar-backlog-of = backlog de {$agent}
agentbar-reading = leyendo…
agentbar-backlog-error = backlog: {$error}
agentbar-backlog-empty = nada en su backlog
agentbar-tier-critical = crítico
agentbar-tier-hot = urgente
agentbar-tier-yours = suyo
agentbar-tier-decision = su decisión pendiente
agentbar-tier-waiting = espera a los demás
agentbar-tier-blocked = bloqueado
agentbar-tier-other = otro
