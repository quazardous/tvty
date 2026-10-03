# La interfaz de tvty, en español. Los mismos archivos e ids que el
# inglés (assets/locales/en/), que es la referencia.
# Los consejos (tips/<id>.md: su inglés es el del archivo, un test lo
# comprueba). {$key} es la tecla del comando del consejo, {$key-font-reset}
# la de font.reset: tal cual, nunca traducidas. **negrita** y `code` se quedan.

tip-slider = **{$key}** muestra tus proyectos como pilas, el más reciente primero. Suelta Ctrl para abrir el elegido.
tip-gallery = **{$key}** muestra todos los terminales en directo, uno al lado de otro. Escribe para filtrar, las flechas para moverte, Enter para abrir.
tip-gallery-filter = Cada miniatura está en directo: mira trabajar a todos tus agentes a la vez y abre el que te necesita.
tip-notice = **{$key}** salta a la notificación más reciente: el terminal del agente, con su ticket al lado.
tip-goto = **{$key}** va a un ticket: escribe su número o pega el enlace `#C.` de un comentario.
tip-full-list = **{$key}** muestra todos los tickets a pantalla completa, el crítico arriba.
tip-bulk = **Ctrl+clic** elige tickets, **Mayús+clic** un rango: luego ciérralos, pospónlos o márcalos todos a la vez.
tip-afk = **{$key}** hace girar el modo AFK del agente mostrado: él solo ▶, retenido 10 minutos ‖, retenido hasta que lo liberes ■.
tip-remote-control = **RC** se enciende en la barra del agente mientras su Claude está en Remote Control: síguelo y respóndele desde claude.ai. `/rc` lo activa.
tip-move-loop = La pastilla **… · mandos** de la barra del agente (host, tmux o psmux): un clic toma o deja los mandos, o mueve el bucle al otro lado.
tip-critical = Un **!** rojo marca el ticket crítico: el abierto que retiene a más tickets. Moverlo libera más trabajo.
tip-references = Cada **#N** en un ticket es un enlace, incluso a un ticket de otro proyecto: un clic lo abre.
tip-comment-link = Haz clic en la marca **#C.** de un comentario para copiar su enlace, luego pégalo en cualquier ticket o en el cuadro «ir a».
tip-reply = **Ctrl+Enter** envía tu respuesta; **sin notificar** la publica sin avisar a nadie.
tip-preview = **Escribir / Vista previa**: ve tu Markdown renderizado, imágenes incluidas, antes de crearlo.
tip-escape = **Esc** sale primero del campo donde escribes; un segundo **Esc** cierra la página.
tip-sections = Arrastra el título de una sección para cambiar su tamaño; un doble clic en un título vuelve a repartir el espacio.
tip-fullscreen = **{$key}** pone la ventana a pantalla completa, y la devuelve.
tip-filter = **{$key}** filtra las sesiones: escribe, Enter abre, Esc borra.
tip-new-project = **Proyecto nuevo…** en el menú (el icono de la app, arriba a la izquierda) prepara una carpeta para aiball y arranca su agente.
tip-restart = **Reiniciar tvty**, en el menú, reinicia solo la ventana: las sesiones de tus agentes siguen corriendo.
tip-shortcuts = Cada atajo se puede cambiar: aquí en **Atajos de teclado**, o en `keymap.toml` en la carpeta de configuración de tvty.
tip-opacity = **Opacidad de los terminales** por debajo del 100 % deja ver tu escritorio a través de los terminales.
tip-aiball-board = **aiball**, en el menú, abre el tablero de aiball en tu navegador: los tickets de todos los proyectos.
tip-font = **{$key}** agranda la fuente de los terminales, **{$key-font-smaller}** la achica, **{$key-font-reset}** la devuelve a como estaba.
tip-new-ticket = **{$key}** crea un ticket nuevo desde cualquier sitio, en el proyecto donde creaste el último.
tip-room = **{$key}** pliega el panel de tickets, **{$key-sidebar-toggle}** la lista de proyectos: todo el espacio para el terminal.
tip-themes = **{$key}** recorre los temas de color; el **◐** de la barra de título los lista todos.
tip-wizard = El asistente muestra lo que va a escribir antes de escribir nada: nada cambia hasta que lo configuras.
tip-file-and-exit = **Crear y salir** crea el ticket sin abrirlo: vuelves al terminal, o a la lista completa.
tip-panel-pin = La **chincheta** en la cabecera del panel de tickets: fijado, el panel queda junto al terminal; sin fijar, se superpone a él y el terminal conserva su ancho.
tip-followers = Los **seguidores** reciben aviso de cada movimiento de este ticket: ✕ quita uno, la lista desplegable añade uno. Un seguidor silenciado aparece en gris.
tip-thread-order = **⇅** da la vuelta al hilo: recientes primero o al final, en el panel y a pantalla completa, cada uno a su manera.
tip-held-selection = Seleccionar texto en un terminal lo **mantiene quieto** mientras copias: la salida del agente espera, nada se desplaza bajo el ratón.
tip-workspaces = Un **espacio** guarda un conjunto de proyectos y cómo corre cada uno de sus agentes, solo ▶ o retenido ■. **+ espacio** guarda lo que corre ahora bajo un nombre.
tip-workspace-open = **Abrir** en un espacio arranca lo que guarda y libera lo que en él corre solo; **Cerrar** para sus sesiones, dejando las que comparten otros espacios.
