# La interfaz de tvty, en español. Los mismos archivos e ids que el
# inglés (assets/locales/en/), que es la referencia.
# El 📢 de un proyecto y sus opciones.

## El 📢

megaphone-no-project = Una consigna permanente y un foco de despertar son de un proyecto: muestra uno primero
megaphone-standing = Consigna permanente — {$project}
megaphone-standing-hint = Puesta al principio de cada despertar de sus agentes, evento o backlog. Deja una antes de ausentarte; bórrala al volver.
megaphone-prompt-placeholder = p. ej. primero depuración ligera, sin grandes cambios
megaphone-focus = Foco de despertar
megaphone-focus-hint = Solo estos tickets despiertan a los agentes del proyecto, backlog y eventos. 123, 456 deja solo esos; !789 deja todos menos ese. 123+ trae a sus hijos, 123++ a todos sus descendientes, +123 / ++123 a sus padres, 123~ sus tickets enlazados. Los eventos de fuera quedan sin leer hasta que lo borres.
megaphone-tickets-placeholder = p. ej. 2518, 2523++   o   !2180
megaphone-until-placeholder = hasta (opcional): 2026-09-30 18:00
megaphone-not-a-date = hasta: {$typed} no es una fecha (2026-09-30 18:00)
megaphone-not-a-time = hasta: {$typed} no es una hora válida aquí
megaphone-clear = Borrar
megaphone-save = Guardar
megaphone-said-nothing = {$project}: ya nada guía a sus agentes
megaphone-said-prompt = {$project}: sus agentes leen «{$prompt}» en cada despertar
megaphone-said-focus = {$project}: solo su foco despierta a sus agentes
megaphone-tip-standing = Consigna permanente: {$prompt}
megaphone-tip-focus = Foco de despertar: {$focus}

## Un mensaje a todos los agentes

megaphone-message = Mensaje a todos los agentes
megaphone-message-hint = Escrito ahora en cada sesión de agente en marcha, haga lo que haga. Enviar y retener también retiene cada bucle (no AFK ∞): ningún despertar empieza trabajo nuevo hasta que los liberes. Vacío, se envía el texto mostrado.
megaphone-no-loop = Ningún bucle de agente está en marcha.
megaphone-running = {$count} en marcha: {$names}
megaphone-release = Liberar
megaphone-send = Enviar
megaphone-send-hold = Enviar y retener
megaphone-typed-into = escrito en {$names}
megaphone-queued-for = en cola para {$names}
megaphone-held = retenido {$names}
megaphone-released = liberado {$names}
megaphone-hold-not-applied = retención no aplicada

## Las opciones de un proyecto

projectopts-global = Global
projectopts-a-project = un proyecto…
projectopts-failed = ajustes del proyecto
projectopts-choose = Elige un proyecto arriba.
projectopts-no-folder = aiball no conoce ninguna carpeta de este proyecto en esta máquina: ninguno de sus agentes trabaja aquí.
projectopts-asking = Preguntando a aiball…
projectopts-could-not-say = aiball no pudo decirlo: {$error}
projectopts-no-file = {$folder} no tiene .aiball.yaml, ni ninguna carpeta encima: se aplican los valores por defecto de aiball. Proyecto nuevo… lo configura.
projectopts-written-in = Escrito en {$file}.
projectopts-also-serves = También sirve a {$others}: un cambio aquí vale para ellos también.
projectopts-on-named = activado: {$name}
projectopts-none-set = {$project} no fija nada de la config del tablero: tiene los valores del tablero.
projectopts-all-keys = Todas sus claves
