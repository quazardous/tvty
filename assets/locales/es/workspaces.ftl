# La interfaz de tvty, en español. Los mismos archivos e ids que el
# inglés (assets/locales/en/), que es la referencia.
# Los espacios de trabajo, y la lista que elige sesiones.

## La lista de espacios

workspaces-new = + espacio
workspaces-new-tip = guarda los grupos que corren ahora, elegidos en una lista, bajo un nombre
workspaces-none = Aún no hay espacios: un espacio guarda algunos grupos, y cómo corre cada una de sus sesiones.
workspaces-default-name = Espacio
workspaces-its-name = su nombre
workspaces-act-open = Abrir
workspaces-act-open-tip = arranca lo que está parado y deja cada sesión como se guardó — preguntado antes
workspaces-act-shut = Cerrar
workspaces-act-shut-tip = para sus sesiones — preguntado antes, cuáles; un grupo que otro espacio también tiene se deja
workspaces-act-save = guardar
workspaces-act-save-tip = lo guarda de nuevo tal como está todo ahora, o añade un grupo — elegido en una lista
workspaces-again-tip = el grupo mostrado, guardado de nuevo tal como corre ahora
workspaces-add-tip = añade el grupo mostrado, sus sesiones tal como corren
workspaces-act-rename = renombrar
workspaces-act-rename-tip = otro nombre; Enter confirma
workspaces-act-delete = borrar
workspaces-act-delete-sure = borrar: ¿seguro?
workspaces-act-delete-tip = olvida el espacio; sus sesiones no se tocan
workspaces-no-project = sin proyecto
workspaces-drop-tip = saca este grupo del espacio (sus sesiones no se tocan)
workspaces-now-stopped = parada
workspaces-now-runs = corre
workspaces-now-held = retenida
workspaces-kept-own = guardada sola; ahora {$now}
workspaces-kept-held = guardada retenida; ahora {$now}
workspaces-open-sets = {" — "}Abrir la deja como se guardó

## Lo que será una sesión

workspaces-start-own = parada: a arrancar, sola
workspaces-start-held = parada: a arrancar, retenida
workspaces-let-go = retenida: a liberar
workspaces-to-hold = corre sola: a retener
workspaces-as-kept = como se guardó

## La lista que elige

workspaces-quit-title = ¿Parar también las sesiones de Claude Code?
workspaces-quit-summary = Activado: parada al salir tvty (siguen pudiendo reiniciarse). Desactivado: sigue corriendo.
workspaces-new-title = Un espacio nuevo
workspaces-new-summary = Activado: guardada en él, cada una tal como corre ahora (sola, o retenida).
workspaces-save-title = Guardar {$name} tal como está todo ahora
workspaces-save-summary = Activado: en él, cada una tal como corre ahora. Desactivado: sacada de él.
workspaces-shut-title = Cerrar {$name}: ¿parar sus sesiones?
workspaces-shut-summary = Activado: parada. Desactivado: sigue corriendo. Un grupo que otro espacio también tiene se deja desactivado.
workspaces-open-title = Abrir {$name}
workspaces-open-summary = Activado: hecho. Desactivado: se deja tal cual.
workspaces-restart-title = ¿Reiniciar las sesiones paradas al salir tvty?
workspaces-restart-summary = Activado: reiniciada, retomando su conversación. Como estaban: una sesión retenida vuelve a estarlo; de cero: cada una arranca, luego corre sola.
workspaces-host = host
workspaces-held-for-good = {" · "}retenida hasta nueva orden
workspaces-held-while = {" · "}retenida un rato
workspaces-ran-on = {$place} · siguió: su parada no tuvo efecto
workspaces-not-running = no corre: guardada como estaba
workspaces-new-in-group = nueva en este grupo
workspaces-not-in-yet = aún no está
workspaces-also-in = también en {$others}
workspaces-some = {$ticked} de {$of}
workspaces-whole-group = todo el grupo
workspaces-name = Nombre
workspaces-cancel = Cancelar
workspaces-keep = Guardar
workspaces-quit-keep = Salir, dejarlas todas corriendo
workspaces-quit-stop = Salir, parar las {$count} activadas
workspaces-shut-keep = Cerrar, dejarlas todas corriendo
workspaces-shut-stop = Cerrar, parar las {$count} activadas
workspaces-open-do = Abrir, hacer las {$count} activadas
workspaces-not-now = Ahora no
workspaces-restart-fresh = Reiniciar las {$count} de cero
workspaces-restart-as-were = Reiniciar las {$count} como estaban
workspaces-remember-quit = Recordar esta elección (pararlas todas, o dejarlas todas)
workspaces-remember-restart = Recordar esta elección (cada vez, todas)
workspaces-remember-tip = Ajustes > Disposición > Sesiones lo cambia

## Lo que se hizo

workspaces-none-runs = {$name}: ninguna de sus sesiones corre
workspaces-all-as-kept = {$name}: cada sesión está como se guardó
workspaces-kept = { $count ->
    [one] espacio {$name} guardado: {$count} grupo
   *[other] espacio {$name} guardado: {$count} grupos
}
workspaces-shut-run-on = {$name} cerrado: sus sesiones siguen
workspaces-shut-stopped = { $count ->
    [one] {$name} cerrado: {$count} sesión parada
   *[other] {$name} cerrado: {$count} sesiones paradas
}
workspaces-shut = cerrar
workspaces-no-loop = { $count ->
    [one] {$count} sesión en ningún bucle de esta máquina: no parada
   *[other] {$count} sesiones en ningún bucle de esta máquina: no paradas
}
workspaces-nothing-to-start = {$agent}: ni bucle ni carpeta conocidos para arrancarlo
workspaces-astray = {$agent}: {$cwd} es la carpeta de {$other}, no arrancado
workspaces-mode-failed = {$agent}: su modo: {$error}
workspaces-opened = { $count ->
    [one] {$name} abierto: {$count} sesión ajustada
   *[other] {$name} abierto: {$count} sesiones ajustadas
}
workspaces-opening = abriendo {$name}
workspaces-added = {$project} está en {$name}
