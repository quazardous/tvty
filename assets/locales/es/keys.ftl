# La interfaz de tvty, en español. Los mismos archivos e ids que el
# inglés (assets/locales/en/), que es la referencia.
# Atajos de teclado (src/keymap.rs, src/options.rs, src/shell.rs). Las
# palabras de un comando van por su nombre.

## Lo que hace cada comando

keys-notice-open = Ir a la notificación más reciente: el terminal del agente, su ticket
keys-slider-next = Carrusel: los grupos en pilas, el más reciente primero; soltar Ctrl abre
keys-slider-back = Carrusel, hacia atrás
keys-tab-next = La pestaña siguiente, en el grupo mostrado
keys-tab-back = La pestaña anterior, en el grupo mostrado
keys-gallery-toggle = Galería de todos los terminales; escribir filtra, las flechas mueven, Enter abre
keys-panel-toggle = Plegar o desplegar el panel de tickets
keys-sidebar-toggle = Plegar o desplegar la lista de proyectos
keys-afk-cycle = El modo AFK del agente mostrado: auto → retener 10 min → retener, vigente 3 s después de la última pulsación (un terminal sin agente recibe F9)
keys-sessions-filter = Filtrar las sesiones: escribir, Enter abre, las flechas mueven, Esc borra
keys-ticket-goto = Ir a un ticket: su número o el enlace #C. de un comentario, Enter lo abre (el #… de la barra de título)
keys-options-toggle = Ajustes
keys-app-quit = Salir de tvty, como al cerrar su ventana (los loops en marcha: detenidos o conservados, según lo elegido)
keys-help-menu = Ayuda: acerca de tvty, su documentación, las novedades, un reinicio
keys-window-fullscreen = La ventana a pantalla completa, o volver
keys-font-bigger = Fuente de los terminales más grande
keys-font-smaller = Fuente de los terminales más pequeña
keys-font-reset = Fuente de los terminales a su tamaño por defecto
keys-ticket-new = Un ticket nuevo (también + en el panel y la lista a pantalla completa)
keys-project-megaphone = El 📢 del proyecto mostrado: su consigna permanente y su foco de despertar
keys-list-full = La lista de tickets, a pantalla completa
keys-theme-next = Tema de colores siguiente
keys-debug-inspector = El inspector de GPUI: elegir un elemento, ver su id y dónde se hace (versiones de depuración)
keys-terminal-copy = Copiar la selección
keys-terminal-paste = Pegar el portapapeles
keys-terminal-tab = Tab, al programa (no el foco al elemento siguiente)
keys-terminal-back-tab = Mayús+Tab, al programa

## Las teclas que no son comandos

keys-fixed-esc = Cerrar la galería, el carrusel, la lista de temas, los ajustes, la lista a pantalla completa
    .keys = Esc
keys-fixed-arrows = En el carrusel y la galería: ir a la tarjeta vista allí
    .keys = Flechas
keys-fixed-select = En un terminal: seleccionar texto · una palabra · una línea — copiado a la selección primaria
    .keys = Arrastrar · doble clic · triple clic
keys-fixed-middle-click = En un terminal: pegar la selección primaria
    .keys = Clic central
keys-fixed-right-click = En un terminal: Copiar, Pegar — en un enlace, Abrir enlace, Copiar enlace
    .keys = Clic derecho
keys-fixed-ctrl-click = En un terminal, en un enlace: abrirlo
    .keys = Ctrl+clic
keys-fixed-wheel = Desplazar el historial
    .keys = Rueda del ratón

## La página

keys-intro = Cada atajo es un comando, ligado en un contexto: Terminal cuando un terminal tiene el foco, Ventana en cualquier otro sitio. Gana la ligadura más profunda; una tecla ligada en ninguna parte va al programa del terminal.
keys-how = Haz clic en una tecla para cambiarla, + para añadir una, × para quitarla. Se guarda en {$file}, que también puedes editar.
keys-reset-all = Restablecer todo
keys-not-applied = No aplicado: {$error}
keys-context-window = Ventana — en cualquier sitio de tvty, páginas a pantalla completa incluidas
keys-context-workspace = Espacio de trabajo — los terminales, la lista de sesiones y el panel al lado
keys-context-terminal = Terminal — cuando un terminal tiene el foco
keys-press = Pulsa una tecla… (Esc abandona)
keys-typing-refused = {$key} sirve para escribir: en un terminal sigue siendo del programa
keys-remove = quitar esta tecla
keys-the-program = el programa
keys-masked = {$what} — tapado en un terminal por {$by}
keys-no-key = ninguna tecla
keys-conflict = {$key} lanza {$other}: ¿tomarla para {$name}?
keys-replace = Reemplazar
keys-cancel = Cancelar
keys-freed-terminal = Devuelta al programa del terminal
keys-freed-focus = Devuelta a lo que tiene el foco
