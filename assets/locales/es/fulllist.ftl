# La interfaz de tvty, en español. Los mismos archivos e ids que el
# inglés (assets/locales/en/), que es la referencia.
# La lista de tickets a pantalla completa, y sus acciones en grupo.

fulllist-title = Tickets — {$project}
fulllist-title-all = Tickets — todos los proyectos
fulllist-shown = { $count ->
    [one] {$count} mostrado
   *[other] {$count} mostrados
}
fulllist-new = + Ticket nuevo
fulllist-search = Buscar en los títulos…
fulllist-projects = Proyectos
fulllist-all-projects = Todos los proyectos
fulllist-bands = Bandas
fulllist-sort = Orden
fulllist-sort-activity = Última actividad
fulllist-sort-turn = De quién es el turno (bandas)
fulllist-sort-priority = Prioridad
fulllist-sort-created = Creación
fulllist-sort-number = Número
fulllist-filters = Filtros
fulllist-open = Abiertos
fulllist-all = Todos
fulllist-unread = Sin leer
fulllist-reading-small = leyendo…
fulllist-critical = Crítico — lo que más retiene
fulllist-all-group = Todos
fulllist-reading = Leyendo…
fulllist-none = Ningún ticket aquí.

## Una fila

fulllist-priority = prioridad: {$priority}
fulllist-rejected = rechazado
fulllist-milestone = hito {$title}
fulllist-assigned-to = asignado a {$who}
fulllist-claimed-by = tomado por {$who}
fulllist-spoke-last = {$who} habló el último
fulllist-you-spoke-last = hablaste el último
fulllist-blocked = bloqueado
fulllist-payload = payload
fulllist-by = por {$who} · {$date}
fulllist-hot = un agente estuvo activo en él hace poco

## Acciones en grupo

fulllist-selected = { $count ->
    [one] {$count} seleccionado
   *[other] {$count} seleccionados
}
fulllist-select-all = Seleccionar todo lo mostrado
fulllist-clear = Limpiar · Esc
fulllist-actions = Acciones
fulllist-actions-about = Cada una actúa sobre los tickets elegidos a los que se aplica: cuántos, a la derecha.
fulllist-working = Trabajando…
fulllist-count-of = {$count} de {$of}
fulllist-confirm = { $count ->
    [one] ¿{$action} {$count} ticket?
   *[other] ¿{$action} {$count} tickets?
}
fulllist-cancel = Cancelar
bulk-refused = , {$count} rechazado(s) ({$first})
bulk-approve = Aprobar
    .about = Deja pasar los tickets elegidos que esperan moderación
    .done = { $count ->
        [one] {$count} aprobado
       *[other] {$count} aprobados
    }
bulk-reject = Rechazar
    .about = Rechaza los tickets elegidos que esperan moderación
    .done = { $count ->
        [one] {$count} rechazado
       *[other] {$count} rechazados
    }
bulk-close = Cerrar
    .about = Cierra los tickets elegidos que están abiertos (pregunta antes)
    .done = { $count ->
        [one] {$count} cerrado
       *[other] {$count} cerrados
    }
bulk-reopen = Reabrir
    .about = Vuelve a abrir los tickets elegidos que están cerrados
    .done = { $count ->
        [one] {$count} reabierto
       *[other] {$count} reabiertos
    }
bulk-mark-read = Marcar leído
    .about = Marca leídos los tickets elegidos que tienen algo nuevo
    .done = { $count ->
        [one] {$count} marcado leído
       *[other] {$count} marcados leídos
    }
bulk-mark-unread = Marcar no leído
    .about = Marca no leídos los tickets elegidos ya leídos
    .done = { $count ->
        [one] {$count} marcado no leído
       *[other] {$count} marcados no leídos
    }
bulk-snooze = Dormir 3 días
    .about = Aparta los tickets abiertos elegidos durante 3 días: vuelven entonces
    .done = { $count ->
        [one] {$count} dormido
       *[other] {$count} dormidos
    }
bulk-unsnooze = Despertar
    .about = Trae ahora de vuelta los tickets elegidos que están dormidos
    .done = { $count ->
        [one] {$count} despertado
       *[other] {$count} despertados
    }
bulk-step = Marcar como paso
    .about = Marca la última palabra de los tickets abiertos elegidos como un paso: el agente sigue, nada que decidir
    .done = { $count ->
        [one] {$count} marcado como paso
       *[other] {$count} marcados como paso
    }
bulk-link = Enlazar
    .about = Enlaza los tickets elegidos: el más reciente se relaciona con cada uno de los otros
    .done = { $count ->
        [one] {$count} enlazado
       *[other] {$count} enlazados
    }
