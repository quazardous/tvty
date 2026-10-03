# La interfaz de tvty, en español. Los mismos archivos e ids que el
# inglés (assets/locales/en/), que es la referencia.
# El panel de tickets y lo que dice de un ticket (src/panel.rs,
# src/thread.rs, src/rowstate.rs).

## El panel

tickets-title = Tickets
tickets-no-project = No hay proyecto de aiball en este terminal.
tickets-none-open = Ningún ticket abierto.
tickets-loading = Cargando…
tickets-back = ← Tickets
tickets-fold-panel = Plegar el panel de tickets
tickets-pinned = Fijado: el panel queda junto al terminal. Sin fijar: se superpone a él
tickets-no-session = ninguna sesión abierta
tickets-megaphone = La consigna permanente del proyecto y el foco de despertar
tickets-new = + Nuevo
tickets-new-tip = Un ticket nuevo
tickets-full-list = La lista de tickets, a pantalla completa
tickets-sunk = hundido en el backlog de {$agent} hasta {$until} (en {$left}): su bucle no lo sacará antes, salvo que el hilo se mueva

## Las bandas de la lista

tickets-band-moderate = Por moderar
tickets-band-decide = Esperan tu decisión
tickets-band-working = Con agentes
tickets-band-open = Abiertos
tickets-band-closed = Cerrados

## Las marcas de una fila

tickets-pending-comments = { $count ->
    [one] {$count} comentario esperando moderación
   *[other] {$count} comentarios esperando moderación
}
tickets-unread = sin leer: hay algo nuevo para ti
tickets-you-spoke-last = hablaste el último
tickets-someone-spoke-last = otro habló el último
tickets-comments-tip = { $count ->
    [one] {$count} comentario — {$why}
   *[other] {$count} comentarios — {$why}
}
tickets-held-by = en manos de {$holder}
tickets-held-hot = en manos de {$holder}, activo en él hace poco
tickets-critical = { $holds ->
    [one] el ticket crítico del proyecto: retiene {$holds} ticket abierto
   *[other] el ticket crítico del proyecto: retiene {$holds} tickets abiertos
}
tickets-critical-quiet = , en silencio desde hace {$quiet}
tickets-step-resumes = paso — el agente retoma a las {$at}
tickets-critical-said = retiene {$holds}
tickets-critical-said-quiet = retiene {$holds} · en silencio {$quiet}
tickets-stage-rejected = rechazado
tickets-stage-closed-resolved = cerrado, resuelto
tickets-stage-closed = cerrado
tickets-stage-resolved = resuelto
tickets-stage-blocked = bloqueado
tickets-stage-snoozed = dormido
tickets-stage-pending = pendiente
tickets-stage-open = abierto
tickets-to-moderate = por moderar

## Lo que significa un glifo

tickets-glyph-escalation = un agente escala: necesita que actúes
tickets-glyph-plan = se propone un plan
tickets-glyph-resolution = se propone una resolución
tickets-glyph-wontfix = se propone cerrar sin corrección
tickets-glyph-stalled = el paso de un agente se quedó en silencio
tickets-glyph-step = un agente está en un paso (then: continue)
tickets-glyph-rejected = se rechazó el último plan o la última resolución
tickets-glyph-closed-resolved = cerrado, resuelto
tickets-glyph-closed = cerrado sin resolución

## Las decisiones

tickets-kind-plan = plan
tickets-kind-resolution = resolución
tickets-kind-wontfix = cierre sin corrección
tickets-kind-escalation = escalada
tickets-proposes-plan = {$who} propone un plan
tickets-proposes-resolution = {$who} propone una resolución
tickets-proposes-wontfix = {$who} propone cerrar sin corrección
tickets-proposes-escalation = {$who} escala
tickets-decision-pending = {$noun} · pendiente
tickets-decision-accepted = { $kind ->
    [resolution] {$noun} aceptada
    [escalation] {$noun} aceptada
   *[other] {$noun} aceptado
}
tickets-decision-rejected = { $kind ->
    [resolution] {$noun} rechazada
    [escalation] {$noun} rechazada
   *[other] {$noun} rechazado
}
tickets-decision-superseded = { $kind ->
    [resolution] {$noun} · reemplazada
    [escalation] {$noun} · reemplazada
   *[other] {$noun} · reemplazado
}
tickets-accept-close = Aceptar → cerrar
tickets-accept-wontfix = Aceptar → cerrar, sin corrección
tickets-accept-escalation = Hecho → aceptar
tickets-accept-plan = Aceptar → adelante
tickets-reject = Rechazar
tickets-approve = Aprobar
tickets-decided-once-approved = se decide una vez aprobado el ticket
tickets-reject-say-why = Para rechazar, di primero por qué aquí abajo.
tickets-waits-moderation = Este ticket espera moderación

## En qué punto está un ticket (la frase bajo su título)

tickets-you = tú
tickets-an-agent = un agente
tickets-closed-resolved-by = Cerrado, resuelto por {$who}
tickets-closed-resolved = Cerrado, resuelto
tickets-closed-unresolved = Cerrado sin resolución
tickets-yours-moderation = Te toca: este ticket espera moderación
tickets-your-proposal-plan = Tu plan espera una decisión
tickets-your-proposal-resolution = Tu resolución espera una decisión
tickets-your-proposal-wontfix = Tu cierre sin corrección espera una decisión
tickets-your-proposal-escalation = Tu escalada espera una decisión
tickets-yours-escalates = Te toca: {$who} escala — actúa, luego acepta
tickets-yours-decide-plan = Te toca: acepta o rechaza el plan de {$who}
tickets-yours-decide-resolution = Te toca: acepta o rechaza la resolución de {$who}
tickets-yours-decide-wontfix = Te toca: acepta o rechaza el cierre sin corrección de {$who}
tickets-step-quiet = el paso de {$agent} se quedó en silencio
tickets-on-step = {$agent} está en un paso
tickets-step-resumes-in = {" · "}retoma en {$span}
tickets-step-waits-on = {" · "}espera a #{$ticket}
tickets-yours-answer = Te toca: responde a {$who}
tickets-holds-you-spoke = {$holder} lo tiene: hablaste el último
tickets-yours-nobody = Te toca: nadie más está en él
tickets-their-turn-of = Turno de {$holder}: hablaste el último
tickets-their-turn = Les toca: hablaste el último
tickets-span-under-minute = menos de un minuto
tickets-span-minutes = {$n} min
tickets-span-hours = {$n} h

## El hilo

tickets-images = { $count ->
    [one] 🖼 imagen
   *[other] 🖼 {$count} imágenes
}

tickets-where-it-stands = En qué punto está · {$who}
tickets-edit = ✎ editar
tickets-edit-tip = Editar el título y el cuerpo
tickets-newest-first = ⇅ recientes primero
tickets-newest-last = ⇅ recientes al final
tickets-close-full = ✕  Esc
tickets-more = ⤢ más
tickets-comments-spoke = { $count ->
    [one] {$count} comentario · {$who} habló el último
   *[other] {$count} comentarios · {$who} habló el último
}
tickets-unfold-all = desplegar todo
tickets-fold-before-summary = plegar antes del resumen
tickets-fold = plegar
tickets-unfold = desplegar
tickets-snoozed-until = dormido hasta {$until}
tickets-tokens = {$count} tok
tickets-priority-tip = la prioridad: un clic la cambia
tickets-priority-label = Prioridad
tickets-rel-depends = depende de
tickets-rel-blocks = bloquea
tickets-rel-relates = relacionado con
tickets-rel-duplicates = duplica
tickets-rel-duplicated = duplicado por
tickets-rel-parent = padre de
tickets-rel-child = hijo de
tickets-answer = Responder
tickets-in-reply = ✓ en la respuesta
tickets-step = paso
tickets-resumes = retoma
tickets-resumes-on = con
tickets-resumes-or-on = o con
tickets-resumes-at = el agente retoma a las {$at}
tickets-resumes-when = el agente retoma cuando #{$ticket} se mueva (una respuesta, una decisión, un cierre)
tickets-or-when = o cuando #{$ticket} se mueva (una respuesta, una decisión, un cierre)

## Las acciones de un comentario

tickets-comment-actions = las acciones del comentario
tickets-edit-comment = Editar
tickets-delete = Borrar
tickets-really-delete = ¿Borrar de verdad?
tickets-classify-as = como {$noun}
tickets-no-decision = sin decisión
tickets-not-a-step = no es un paso
tickets-a-step = un paso
tickets-resurface = Volver a sacar
tickets-cancel = Cancelar
tickets-save = Guardar

## Escribir

tickets-reply-placeholder = Responder… (ctrl+enter envía)
tickets-editing = Editando #{$id}
tickets-edit-keys = ctrl+enter guarda · esc lo deja como estaba
tickets-title-first = Primero un título.
tickets-nothing-to-preview = Nada que previsualizar aún.
tickets-sending-answers = { $count ->
    [one] La respuesta contesta {$count} pregunta.
   *[other] La respuesta contesta {$count} preguntas.
}
tickets-close = Cerrar
tickets-reopen = Reabrir
tickets-wake = Despertar
tickets-snooze = Dormir ▾
tickets-snooze-for = Dormir durante
tickets-snooze-hour = 1 hora
tickets-snooze-day = un día
tickets-snooze-week = una semana
tickets-assign = Asignar ▾
tickets-assigned-menu = → {$who} ▾
tickets-assign-to = Asignar a
tickets-no-agent = el proyecto no tiene agente
tickets-claim-assigns = {$agent} lo tiene por un claim: un clic se lo asigna
tickets-unassign-tip = desasignar: se lo quita a {$holder}; nadie lo tiene, espera a quien lo tome
tickets-without-notifying = sin notificar
tickets-reply = Responder

## La columna de campos

tickets-group-state = Estado
tickets-group-fields = Campos
tickets-group-people = Personas
tickets-group-links = Enlaces
tickets-group-project = Proyecto
tickets-group-tokens = Tokens
tickets-group-payload = Payload
tickets-lifecycle = situación
tickets-lifecycle-closed-resolved = cerrado, resuelto
tickets-lifecycle-closed = cerrado
tickets-lifecycle-moderation = espera moderación
tickets-lifecycle-open = abierto
tickets-snoozed = dormido
tickets-until = hasta {$date}
tickets-field-intent = intención
tickets-field-priority = prioridad
tickets-field-level = nivel
tickets-field-scope = alcance
tickets-field-tags = etiquetas
tickets-field-milestone = hito
tickets-field-reporter = autor
tickets-field-claimed = tomado por
tickets-field-assigned = asignado a
tickets-field-followers = seguidores
tickets-field-project = proyecto
tickets-filed = abierto el {$date}
tickets-lapsed = {$who} (caducado)
tickets-claim-until = {$who}, hasta {$date}
tickets-muted = silenciado: no se le notifica este ticket, ni siquiera por su rol — ✕ lo devuelve a lo que dice su rol
tickets-followers-tip = Quién sigue este ticket por elección propia (o lo silenció). Los propietarios del proyecto reciben avisos por su rol: no aparecen aquí.
tickets-sub-ticket = subticket
tickets-new-sub = + uno nuevo
tickets-sub-ticket-of = padre
tickets-sub-tickets = subtickets
tickets-remove-relation = quitar esta relación
tickets-relate = relacionar
tickets-relate-to = con…
tickets-relate-add = + un ticket
tickets-move-to = Mover a {$project}
tickets-in-out = entrada · salida
tickets-cache = caché
tickets-cache-said = {$written} escritos · {$read} leídos
tickets-payload = este ticket lleva un payload (ver la interfaz web)
tickets-pick-project = un proyecto
tickets-pick-project-search = un proyecto…
tickets-pick-tag = añadir una etiqueta
tickets-pick-tag-search = una etiqueta…
tickets-pick-milestone = ninguno
tickets-pick-milestone-search = un hito…
tickets-pick-reporter = el autor
tickets-pick-agent-or-you = un agente, o tú…
tickets-pick-follower = añadir uno
tickets-pick-intent-search = una intención…
tickets-pick-priority-search = una prioridad…
tickets-pick-level-search = un nivel…
tickets-pick-scope-search = un alcance…
tickets-pick-nobody = nadie
tickets-pick-agent-search = un agente…

## Lo que hizo un gesto (dicho en una notificación)

tickets-did-reply = respuesta publicada
tickets-did-with-reply = {$what}, con una respuesta
tickets-did-on = {$title} — {$what}
tickets-did-comment-edited = comentario editado
tickets-did-tagged = etiqueta {$name} añadida
tickets-did-tag-removed = etiqueta {$name} quitada
tickets-did-milestone-set = hito fijado
tickets-did-milestone-removed = hito quitado
tickets-did-reporter = autor: {$name}
tickets-did-assigned = asignado a {$name}
tickets-did-unassigned = desasignado
tickets-did-follows = {$name} lo sigue
tickets-did-unfollows = {$name} ya no lo sigue
tickets-did-intent = intención {$value}
tickets-did-priority = prioridad {$value}
tickets-did-level = nivel {$value}
tickets-did-scope = alcance {$value}
tickets-did-edited = título y cuerpo editados
tickets-did-related = relacionado con #{$target} ({$kind})
tickets-did-unrelated = relación con #{$target} quitada
tickets-did-moved = movido a {$project}
tickets-did-snoozed = dormido
tickets-did-woken = despertado
tickets-did-accepted = decisión aceptada
tickets-did-rejected = decisión rechazada
tickets-did-approved = aprobado
tickets-did-rejected-moderation = rechazado en moderación
tickets-did-closed = cerrado
tickets-did-reopened = reabierto
tickets-did-deleted = comentario borrado
tickets-did-classified = comentario clasificado como {$noun}
tickets-did-plain = comentario sin decisión
tickets-did-step-marked = paso marcado
tickets-did-step-unmarked = paso quitado
tickets-did-voted = voto registrado
tickets-did-resurfaced = comentario sacado de nuevo

## Los eventos del hilo

tickets-event-closed = cerró el ticket
tickets-event-reopened = reabrió el ticket
tickets-event-resolved = lo marcó resuelto
tickets-event-blocked = lo señaló para decidir
tickets-event-taken-over = tomó el claim
tickets-event-sub-added = añadió un subticket{$source}
tickets-event-referenced = lo referenció desde{$source}
tickets-event-dependency-closed = una dependencia se cerró{$source}
tickets-event-dependency-rejected = una dependencia fue rechazada{$source}
tickets-event-related-closed = un ticket relacionado se cerró{$source}
tickets-event-linked = enlazó{$source}
