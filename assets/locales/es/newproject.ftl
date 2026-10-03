# La interfaz de tvty, en español. Los mismos archivos e ids que el
# inglés (assets/locales/en/), que es la referencia.
# El asistente de proyecto nuevo. Lo que aiball dice haber hecho se
# muestra tal como lo dice aiball.

newproject-title = Proyecto nuevo
newproject-step-folder = Carpeta
newproject-step-identity = Quién trabaja en él
newproject-step-done = Configurado
newproject-step-next = Y después

## La carpeta

newproject-which-folder = ¿Qué carpeta se vuelve el proyecto?
newproject-folder-placeholder = la carpeta del proyecto, p. ej. ~/dev/app
newproject-choose-folder = Elegir la carpeta del proyecto
newproject-browse = Examinar…
newproject-folder-empty = La carpeta donde trabajará el agente: la raíz del proyecto.
newproject-folder-missing = Esa carpeta no existe.
newproject-folder-file = Es un archivo, no una carpeta.
newproject-folder-checking = Preguntando a aiball…
newproject-folder-project = Ya es un proyecto de aiball: {$name} ({$file}). Siguiente muestra lo que dice, para configurarlo de nuevo.
newproject-folder-configured = Ya configurado para aiball ({$file}). Siguiente muestra lo que dice, para configurarlo de nuevo.
newproject-folder-git = Un repositorio git: listo.
newproject-folder-ready = Listo (no es un repositorio git).
newproject-open-running = Abrir {$label}, en marcha
newproject-open = Abrir {$name}
newproject-next = Siguiente →
newproject-back = ← Atrás

## Quién trabaja en él

newproject-project-placeholder = proyecto
newproject-agent-placeholder = agente
newproject-field-project = proyecto
newproject-field-agent = agente
newproject-bad-project = El nombre del proyecto: letras, cifras, -, _ y .
newproject-bad-agent = El nombre del agente: letras, cifras, -, _ y .
newproject-filled-from = Rellenado desde {$file}: cambia lo que quieras, lo que hay aquí es lo que se configura.
newproject-crew = un agente de equipo
newproject-crew-about = Junto al responsable del proyecto, en los tickets que se le dan; si no: el responsable.
newproject-host = sus bucles en el host de aiball
newproject-host-about = Dónde corren los bucles arrancados en esta carpeta; si no: en {$mux}.
newproject-rc = Remote Control
newproject-rc-about = Su Claude se puede retomar desde claude.ai y la app de Claude.
newproject-private = un proyecto privado
newproject-private-about = aiball le sirve su kit privado (sin tickets públicos, sin seguidores).
newproject-noclaim = sin claim
newproject-noclaim-about = El agente solo trabaja en los tickets que se le asignan, nunca toma uno del pool.
newproject-in = En {$folder}, aiball:
newproject-will-file = {$will} {$file}
newproject-will-created = crea
newproject-will-added = añade su entrada a
newproject-will-rewritten = reescribe su entrada en
newproject-will-patched = actualiza
newproject-will-overwrote = sobrescribe
newproject-will-kept = conserva
newproject-joins = {$name} ya está en el tablero: esta carpeta se le une (otra carpeta, o un agente de equipo).
newproject-then-sets = luego fija {$what} en .aiball.yaml
newproject-files = .mcp.json: el servidor MCP de aiball para Claude Code · .aiball.yaml: proyecto, agente, rol, dónde corren sus bucles.
newproject-set-up = Configurarlo
newproject-setting-up = Configurando…
newproject-unsaved = configurado, pero dónde corren sus bucles y su Remote Control no se guardaron: {$error}
newproject-set-in = fijado {$what} en {$file}

## Configurado

newproject-done = {$name} está configurado. aiball dijo:
newproject-failed = aiball no pudo configurarlo:
newproject-nothing-said = (no dijo nada)

## Y después

newproject-next-start = Arrancar el agente
newproject-next-start-host = «Arrancar su primera sesión» arranca el Claude Code de {$agent} en el host de aiball, en la carpeta del proyecto; su terminal se abre aquí, sus tickets al lado.
newproject-next-start-mux = «Arrancar su primera sesión» arranca el Claude Code de {$agent} en {$mux}, en la carpeta del proyecto; su terminal se abre aquí, sus tickets al lado.
newproject-next-mcp = Aceptar el servidor MCP de aiball
newproject-next-mcp-about = En su primer arranque en esta carpeta, Claude Code pregunta si usar el servidor MCP que declara .mcp.json (aiball): acéptalo. Sin él, el agente no puede ni leer el tablero ni responder a sus tickets. ¿Rechazado por error? /mcp en Claude Code lo activa.
newproject-next-skill = Instalar el skill de aiball
newproject-next-skill-about = Claude Code aún no tiene el skill de aiball en esta máquina: `aiball init skill`, una vez, lo instala — el agente conoce entonces los buenos gestos del tablero.
newproject-next-work = Darle trabajo
newproject-next-work-about = «+ Nuevo» en el panel de tickets crea un ticket en {$name}; el agente lo toma en su próximo despertar, y sus planes y preguntas vuelven como notificaciones.
newproject-close = Cerrar
newproject-start-first = Arrancar su primera sesión

newproject-step-mcp-created = {$path}: creado, con la entrada de aiball
newproject-step-mcp-added-others = {$path}: entrada de aiball añadida, los demás servidores conservados
newproject-step-mcp-added = {$path}: entrada de aiball añadida
newproject-step-mcp-rewritten = {$path}: entrada de aiball reescrita en su forma actual
newproject-step-mcp-kept = {$path}: entrada de aiball ya presente, conservada (force la sobrescribe)
newproject-step-file-created = {$path}: creado ({$set})
newproject-step-file-overwrote = {$path}: sobrescrito ({$set})
newproject-step-file-kept = {$path}: ya presente, conservado (force lo sobrescribe)
newproject-step-consumer = {$path}: identidad fijada ({$set})
newproject-step-type-kept = {$path}: tipo de proyecto ya {$value}
newproject-step-type = {$path}: tipo de proyecto {$value}
newproject-step-type-was = {$path}: tipo de proyecto {$value} (antes: {$previous})
newproject-step-deny = {$path}: herramientas de código denegadas ({$set})
