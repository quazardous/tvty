# La interfaz de tvty, en español. Los mismos archivos e ids que el
# inglés (assets/locales/en/), que es la referencia.
# Ajustes (src/settings.rs, src/options.rs, las páginas de ajustes de
# src/shell.rs). Las palabras de un ajuste van por su clave.

## Los ajustes

setting-appearance-language = Idioma
    .about = El idioma de la interfaz. Lo que escribe la gente (tickets, comentarios, nombres) queda tal cual.
setting-appearance-terminal_font_size = Fuente de los terminales
    .about = El texto de los terminales. También Ctrl+Mayús+= / Ctrl+Mayús+− / Ctrl+Mayús+0.
setting-appearance-window_font_size = Texto de la ventana
    .about = Todo lo que rodea a los terminales — listas, tickets, menús.
setting-notifications-max = Mostradas como máximo
    .about = En la esquina superior derecha del terminal (inferior izquierda sobre una pantalla completa), la más reciente en la esquina; las más antiguas le hacen sitio.
setting-notifications-seconds = Segundos mostrada
    .about = Luego se va, salvo que el puntero esté encima.
setting-notifications-own = Tus propios gestos
    .about = Un ticket cerrado, una respuesta publicada, un plan aceptado: dicho cuando aiball lo tiene. Un rechazo siempre se dice.
    .on = mostrados
    .off = ocultos
setting-notifications-copied = Copiado
    .about = Un breve «Copiado al portapapeles» abajo cuando un texto va al portapapeles (no para una simple selección).
    .on = mostrado
    .off = oculto
setting-terminal-scrollback = Líneas de historial
    .about = Lo que cada terminal guarda para volver atrás. Más cuesta memoria: unos 4 KB por línea a 160 columnas, así que 5000 líneas son unos 19 MB por terminal como mucho. El historial de una sesión tmux es el de tmux (su history-limit).
setting-scroll-speed = Velocidad de la rueda
    .about = Veces la de tvty: a 1, una muesca son unas tres filas de tickets, cinco líneas del historial de un terminal.
setting-mouse-focus = Foco
    .about = Lo que da el teclado al terminal o a un cuadro donde escribir: un clic, o el puntero al pasar por encima (como el foco de un gestor de ventanas sigue al ratón). Por defecto, como el sistema.
setting-appearance-theme = Ventana
    .about = El tema de colores de la ventana. Ctrl+Mayús+K los recorre.
setting-appearance-terminal_theme = Terminal
    .about = El propio de los terminales, o el de la ventana — un terminal oscuro en una ventana clara.
setting-appearance-terminal_opacity = Opacidad de los terminales
    .about = Por debajo del 100 %, el escritorio se ve a través del fondo de los terminales, en directo. Las listas, los tickets y los colores que fija un programa siguen opacos.
setting-appearance-terminal_blur = Desenfoque detrás
    .about = Detrás de un terminal transparente, el escritorio desenfocado. Solo donde el compositor lo ofrece (KDE); en otros sitios se ve nítido.
    .on = desenfocado
    .off = nítido
setting-updates-check = Buscar actualizaciones
    .about = Al arrancar y una vez al día, pregunta a GitHub si ha salido un Terminal Velocity más reciente, y lo dice una vez (no se envía nada más). El actualizador (Actualizaciones… en el menú) lo instala.
    .on = buscar
    .off = nunca
setting-tips-show = Mostrar consejos
    .about = «¿Sabías que…?»: un consejo corto una vez tras arrancar, y la primera vez que se abre una página; nunca de lo que ya usas. Consejos… en el menú los muestra todos.
    .on = mostrar
    .off = nunca
setting-sessions-order = Orden
    .about = Los proyectos en la lista, el carrusel y la galería: el último usado primero, como ctrl+tab; alfabético; o el tuyo, arrastrándolos en la lista. También el ⇅ en la cabecera de la lista.
setting-sessions-show_hub = Las sesiones del hub
    .about = Cuando esta máquina llega a aiball a través de un nodo proxy: listar también las sesiones que corren en el hub de aiball, aparte, bajo «en el hub». Se leen desde aquí (estado, tickets), nunca se abren: una sesión se conecta desde su propia máquina. Las sesiones de otros nodos nunca se listan.
    .on = listadas
    .off = no listadas
setting-sessions-on_quit = Al salir
    .about = Cuando tvty sale con bucles de Claude Code de esta máquina en marcha: preguntar, pararlos (a través de aiball: siguen pudiendo reiniciarse), o dejarlos en marcha.
setting-sessions-on_start = Al arrancar
    .about = Los bucles que tvty paró al salir: preguntar, reiniciarlos como estaban (retomando su conversación, uno retenido vuelve a estarlo), reiniciarlos de cero (arranque, luego solos), o dejarlos parados.
setting-tickets-panel_overlay = Lugar
    .about = Junto al terminal, que queda más estrecho — o encima, como la lista de sesiones a la izquierda: el terminal conserva todo su ancho y el panel cubre su lado derecho mientras está abierto.
    .on = encima del terminal
    .off = junto al terminal
setting-tickets-newest_first = Recientes primero, a pantalla completa
    .about = Un ticket a pantalla completa muestra primero la palabra más reciente de su hilo (también ⇅ en la cabecera del ticket).
    .on = recientes primero
    .off = recientes al final
setting-tickets-panel_newest_first = Recientes primero, en el panel
    .about = Un ticket en el panel muestra primero la palabra más reciente de su hilo; si no, al final, junto a la respuesta (también ⇅ allí). El panel tiene su propio orden: el de la pantalla completa no es el suyo.
    .on = recientes primero
    .off = recientes al final
setting-tickets-summary_open = En qué punto está, abierto
    .about = En el panel de tickets, el «En qué punto está» de un ticket muestra su texto de inmediato; si no, queda plegado en su título, a un clic. A pantalla completa siempre se muestra.
    .on = abierto
    .off = plegado
setting-tickets-ctrl_enter_opens = Ctrl+Enter
    .about = En un ticket nuevo, Ctrl+Enter lo crea y vuelve a donde estabas («Crear y salir»), o lo crea y lo abre («Crear el ticket»). Los botones hacen cada uno lo suyo, diga lo que diga este ajuste.
    .on = lo crea y lo abre
    .off = lo crea y sale

## Las páginas y los grupos, por su nombre

settings-title-project = Proyecto
settings-title-appearance = Apariencia
settings-title-layout = Disposición
settings-title-ticket-list = Lista de tickets
settings-title-keyboard-shortcuts = Atajos de teclado
settings-title-aiball = aiball
settings-title-about = Acerca de
settings-title-language = Idioma
settings-title-sizes = Tamaños
settings-title-notifications = Notificaciones
settings-title-mouse = Ratón
settings-title-colours = Colores
settings-title-updates = Actualizaciones
settings-title-tips = Consejos
settings-title-sessions = Sesiones
settings-title-ticket-panel = Panel de tickets
settings-title-thread = Hilo
settings-title-new-ticket = Ticket nuevo
settings-title-sides = Lados
settings-title-legend = Leyenda
settings-title-window = Ventana
settings-title-workspace = Espacio de trabajo
settings-title-terminal = Terminal
settings-title-fixed-keys = Teclas fijas
settings-title-folder = Carpeta
settings-title-board = Tablero
settings-title-board-wide-only = Solo para todo el tablero
settings-title-per-project-only = Solo por proyecto
settings-title-in-aiball-yaml = En .aiball.yaml

## La página

settings-heading = Ajustes
settings-search-placeholder = Buscar…
settings-modified-hint = @modified: lo que cambiaste
settings-search = Búsqueda: {$text}
settings-close = ✕  Esc
settings-nothing-found = No se encontró nada. Busca un nombre, una palabra que diga, una tecla (ctrl+shift+b), un valor — o @modified.
settings-default = por defecto
settings-default-back = Por defecto
settings-back-to = volver a {$value}
settings-own-themes = Tus propios temas (en el formato de tema de gpui-component) van en ~/.config/tvty/themes/: aparecen aquí la próxima vez que se abra esta página.

## Las opciones

setting-language-auto = Automático ({$lang})
settings-ask = Preguntar
settings-order-recent = El más reciente primero
settings-order-alpha = Alfabético
settings-order-yours = El tuyo (arrastrado en la lista)
settings-quit-stop = Pararlos
settings-quit-keep = Dejarlos en marcha
settings-start-restart = Reiniciarlos como estaban
settings-start-fresh = Reiniciarlos de cero
settings-start-leave = Dejarlos parados
settings-focus-click = Clic
settings-focus-hover = Al pasar
settings-same-as-window = Como la ventana
settings-dark = Oscuros
settings-light = Claros
settings-theme-dark = OSCUROS
settings-theme-light = CLAROS
settings-theme-window = Ventana
settings-theme-terminal = Terminal
settings-theme-opacity = Opacidad de los terminales

## Los lados

settings-side-open = abierto
settings-side-folded = plegado
settings-side-said = {$state} · {$width} px
settings-side-list = Lista de proyectos
settings-side-list-about = Encima del terminal, a la izquierda. Arrastra su borde para cambiar su tamaño; su asa la pliega.
settings-side-panel = Panel de tickets
settings-side-panel-about = A la derecha del terminal. Arrastra su borde para cambiar su tamaño; su asa lo pliega.
settings-widths = Anchos
settings-widths-about = Volver a los valores por defecto: una lista de 290 px, un panel de un tercio de la ventana.
settings-reset = Restablecer

## Acerca de

settings-about-version = Versión
settings-about-aiball-at = aiball en
settings-about-acting-as = Actúa como
settings-about-bus = Bus de aiball
settings-about-bus-said = versión {$version}, como {$who}
settings-about-bus-said-kind = versión {$version}, como {$who} ({$kind})
settings-about-not-connected = no conectado
settings-about-live = Tablero en directo
settings-about-not-subscribed = sin suscripción
settings-about-subscriptions = { $count ->
    [one] {$count} suscripción en el bus
   *[other] {$count} suscripciones en el bus
}
settings-about-theme = Tema
settings-about-terminal-theme = Tema de los terminales
settings-about-the-windows = el de la ventana
settings-about-config = Ajustes, atajos, temas
settings-about-state = Disposición y espacio de trabajo
settings-about-fonts = Fuentes
settings-about-what = Un terminal nativo para trabajar con muchos agentes de código IA a la vez: sus terminales agrupados por proyecto, los tickets de cada proyecto al lado — el agente pregunta, tú decides, él sigue. Construido sobre aiball, que ejecuta los bucles de los agentes y su tablero.
settings-about-github = GitHub ↗
settings-about-aiball = aiball en GitHub ↗
settings-about-license = Licencia MIT ↗
settings-about-footer = Terminal Velocity (tvty). Licencia MIT. Temas incluidos: ver themes/README.md.

## La config de aiball

settings-aiball-failed = config de aiball
settings-aiball-unreadable = no se pudo leer la config de aiball: {$error}. Vuelve a elegir el alcance, arriba a la izquierda, para reintentar.
settings-aiball-reading = Leyendo la config de aiball…
settings-aiball-board = La config propia del tablero: lo que recibe cada proyecto salvo que diga otra cosa. Elige un proyecto, arriba a la izquierda, para la suya.
settings-aiball-project = Lo que {$project} dice sobre la config del tablero; ↺ devuelve una clave al valor del tablero.
settings-aiball-in-file = aiball lee estas claves del .aiball.yaml de cada proyecto, no de su config: cámbialas en ese archivo.
settings-aiball-global-only = aiball declara estas claves para todo el tablero: un valor para todos los proyectos, fijado en Global.
settings-aiball-project-only = aiball declara estas claves solo por proyecto: sin valor para todo el tablero; elige un proyecto arriba para fijarlas.
settings-aiball-open-global = Abrir Global
settings-aiball-protected = 🔒 protegida
settings-aiball-set-in-file = fijada en .aiball.yaml
settings-aiball-board-wide = para todo el tablero: fijada en Global
settings-aiball-per-project = por proyecto
settings-aiball-from-board = {" "}De la config del tablero.
settings-aiball-board-back = Tablero
settings-on = activado
settings-off = desactivado

## La leyenda de la lista de tickets

settings-legend-intro = La lista de tickets tal como aiball la calcula para ti: la banda, de quién es el turno y el glifo de estado son de aiball; la franja es de tvty.
settings-legend-order = Orden
settings-legend-band-moderate = el ticket, o comentarios en él, esperan moderación
settings-legend-band-decide = un plan, una resolución, un cierre sin corrección o una escalada espera tu decisión
settings-legend-band-working = un agente lo tiene o está en un paso
settings-legend-band-open = abierto, nada urgente
settings-legend-band = {$what}; el más reciente primero
settings-legend-glyph = Glifo de estado — en color cuando te espera, atenuado si no
settings-legend-always-blue = {$what} — siempre azul
settings-legend-always-amber = {$what} — siempre ámbar
settings-legend-always-red = {$what} — siempre rojo
settings-legend-stripe = Franja — de quién es el turno
settings-legend-stripe-solid = una decisión te espera, y es el último mensaje
settings-legend-stripe-dashed = una decisión te espera, pero la conversación siguió después
settings-legend-stripe-yours = tu turno: un agente te respondió
settings-legend-stripe-waiting = fina y punteada: tu palabra es la última, esperas
settings-legend-stripe-none = sin franja: la pelota la tiene el agente
settings-legend-stripe-tape = cinta de obra: el ticket espera moderación, nada avanza hasta que lo dejes pasar
settings-legend-rest = El resto
settings-legend-unread = comentarios, en azul: algo nuevo para ti, sin leer (su título en negrita también)
settings-legend-others-spoke = claro, su punta a la izquierda: otro habló el último
settings-legend-you-spoke = discreto, su punta a la derecha: hablaste el último
settings-legend-pending = comentarios esperando moderación
settings-legend-agent = agente
settings-legend-holder = el agente que lo tiene; la llama cuando estuvo activo hace poco
settings-legend-critical = el ticket crítico del proyecto: retiene 3 tickets abiertos
settings-legend-priority = prioridad: urgente, alta, baja (normal no muestra nada)
