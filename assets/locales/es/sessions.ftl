# La interfaz de tvty, en español. Los mismos archivos e ids que el
# inglés (assets/locales/en/), que es la referencia.
# La lista de sesiones, la línea de una sesión, las pestañas y la barra de
# título (src/shell.rs, src/shell/loopstabs.rs, src/shell/tabs.rs, src/status.rs).

## La cabecera de la lista

sessions-tab-sessions = Sesiones
sessions-tab-workspaces = Espacios
sessions-order-recent = el proyecto usado por último primero, como ctrl+tab; un clic: alfabético
sessions-order-alpha = alfabético; un clic: el proyecto usado por último primero, como ctrl+tab
sessions-new-project = + proyecto
sessions-new-project-tip = una carpeta se vuelve proyecto de aiball (aiball init), luego su primera sesión
sessions-new-terminal-tip = un terminal propio, que sobrevive a tvty
sessions-filter = Filtrar…  ctrl+shift+f
sessions-fold = Plegar la lista de sesiones

## Sus grupos

sessions-group-live = activas
sessions-group-idle = paradas
sessions-group-shut = sin bucle
sessions-group-on-hub = en el hub
sessions-none = Ninguna sesión
sessions-none-found = Ninguna sesión encontrada
sessions-first-project = Configura tu primer proyecto
sessions-none-stopped = Ningún bucle parado
sessions-none-stopped-found = Ningún bucle parado encontrado
sessions-none-shut = Ningún agente sin bucle
sessions-none-shut-found = Ningún agente encontrado
sessions-none-hub = Ninguna sesión en el hub
sessions-none-hub-found = Ninguna sesión encontrada en el hub
sessions-no-project = Sin proyecto

## La cabecera de un proyecto

sessions-heading-tip = sus tickets en el panel, sin abrir sesión
sessions-new-session = + sesión
sessions-project-options = sus ajustes: dónde corren sus bucles, Remote Control, su config del tablero
sessions-project-terminal = un terminal en la carpeta del proyecto, listado con él

## La fila de una sesión

sessions-open = abierta en tvty: su terminal corre aquí
sessions-looped = {$mux}: su Claude corre en claude-loop, abierto a través de {$mux} (no en el host de aiball)
sessions-attached = { $others ->
    [one] {$others} otro cliente conectado (el terminal de claude-loop, otro tvty), {$typing} con los mandos
   *[other] {$others} otros clientes conectados (el terminal de claude-loop, otro tvty), {$typing} con los mandos
}
sessions-attached-copy = : abierta aquí como copia
sessions-host-shell = un terminal en el host de aiball, sin Claude
sessions-update = su Claude Code instaló una actualización: reinícialo desde su barra
sessions-starting = arrancando…
sessions-start = ▶ arrancar
sessions-astray = ⚠ carpeta de {$other}
sessions-astray-tip = {$other} trabaja en {$cwd}: arrancado aquí, este agente retomaría su conversación. No arrancado.
sessions-forget-ask = ¿olvidar?
sessions-forget-tip = olvidar a {$who}: aiball deja de listarlo; su carpeta, su .aiball.yaml y los tickets del proyecto se quedan — un segundo clic lo olvida
sessions-forgot = {$who} olvidado
sessions-forget-failed = olvidar a {$who}

## El estado de una sesión, en una línea

sessions-offline = desconectado
sessions-offline-tip = su bucle no está conectado a aiball
sessions-working = trabajando
sessions-starting-state = arrancando
sessions-idle = en reposo
sessions-booting = arrancando
sessions-claude-working = Claude está trabajando
sessions-claude-starting = Claude está arrancando
sessions-claude-idle = Claude está en reposo
sessions-held-typing-for-good = retenido hasta nueva orden (un humano escribe en él): el bucle no lo despierta
sessions-typing = un humano escribe en él: el bucle espera
sessions-held-for-good = retenido hasta nueva orden: el bucle no lo despierta
sessions-held-while = retenido un rato: el bucle no lo despierta
sessions-loop-drives = el bucle lo lleva solo
sessions-mark-held-for-good = retenido hasta nueva orden
sessions-mark-held-while = retenido un rato
sessions-mark-own = solo

## Las marcas de la lista plegada

sessions-mark-tip = {$agent}: {$said}
sessions-mark-limit = {$agent}: {$said} — {$limit}

## «+ sesión»

sessions-form-cwd = carpeta de trabajo
sessions-form-agent = agente
sessions-form-where = dónde trabaja su Claude
sessions-form-host-here = el host de aiball lo arranca aquí
sessions-form-loop-here = claude-loop arranca aquí
sessions-form-no-dir = esa carpeta no existe
sessions-form-crew = un agente de equipo, junto al bucle principal
sessions-form-on-host = en el host de aiball, sin {$mux}
sessions-form-cancel = Cancelar
sessions-form-start = Arrancar
sessions-a-loop = un bucle
sessions-astray-said = {$cwd} es la carpeta de {$other}: {$agent} retomaría su conversación
sessions-start-failed = arranque
sessions-start-host-failed = arranque en el host
sessions-started-host = {$agent} arrancado en el host de aiball en {$cwd}
sessions-started = {$agent} arrancado en {$cwd}
sessions-runs-already = {$agent} ya corre: abierto como copia

## Las pestañas

sessions-tab-name = su nombre
sessions-rename-failed = renombrar el terminal
sessions-stop-failed = parar el terminal
sessions-rename-tip = renombrarlo (o un doble clic, F2)
sessions-close-shell = cerrar: para este terminal
sessions-close-tab = cerrar la pestaña: su Claude sigue
sessions-tab-shell-tip = un terminal en el host de aiball, sin Claude; un doble clic (o F2) lo renombra
sessions-tab-tip = ctrl+repág / ctrl+avpág: la pestaña anterior, siguiente
sessions-new-tab-project = un terminal en la carpeta del proyecto
sessions-new-tab-home = un terminal en la carpeta personal

## La ventana

sessions-pick = Elige un terminal a la izquierda · ctrl+shift+espacio los muestra todos
sessions-on-hub = {$agent} — en el hub
sessions-window-hub-project = {$name} — {$project} · {$agent} · en el hub
sessions-window-hub = {$name} — {$agent} · en el hub
sessions-bus-down = el bus de aiball está caído: tvty se reconecta; las listas pueden ir con retraso mientras tanto
sessions-bus-failing = la suscripción a {$what} falló: reintentada, releída entera; las listas pueden ir con retraso mientras tanto
sessions-menu = Menú: acerca de, ayuda, reiniciar, salir
sessions-goto = Ir a un ticket: su número o el enlace #C. de un comentario, Enter · ctrl+shift+g
sessions-themes = Los temas de color — el siguiente
sessions-message-all = Un mensaje a todos los agentes en marcha: enviar, enviar y retener, liberar
sessions-settings = Ajustes
sessions-user = con qué nombre actúa tvty en el tablero de aiball
