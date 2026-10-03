# La config de aiball, tal como la muestra Ajustes > aiball: por su clave,
# estable (una clave que tvty no conoce muestra el inglés propio de aiball).
# Los puntos de una clave son guiones.

aiball-group-autopoll = Autopoll
aiball-group-backlog = Backlog
aiball-group-claude-loop = Bucle de Claude
aiball-group-defaults = Por defecto
aiball-group-earn = Ganar
aiball-group-on-repetitive-denied = Denegaciones repetidas
aiball-group-rules = Reglas
aiball-group-steps = Pasos
aiball-group-tickets = Tickets
aiball-group-updates = Actualizaciones
aiball-group-wait-credit = Crédito de espera

aiball-config-updates-check = Buscar actualizaciones
    .about = true (default) = al arrancar el daemon, y cuando alguien lo pide, lee la última versión de aiball en GitHub para que la bandeja del sistema, la extensión de GNOME y `aiball version` puedan avisar de una actualización. false = ninguna llamada saliente.
aiball-config-tickets-defaults-priority = Prioridad por defecto
    .about = La prioridad de un ticket nuevo creado sin una explícita. Fija un valor global; un proyecto puede cambiarlo.
aiball-config-tickets-defaults-broadcast-new = Tickets nuevos en difusión
    .about = Activado, un ticket nuevo sin alcance explícito se marca en difusión (se avisa a los seguidores del proyecto). Declarado; se aplicará cuando llegue quien lo use.
aiball-config-tickets-rules-summary-max = Límite del resumen (caracteres)
    .about = El summary_until más largo que un agente puede escribir en un comentario. Uno más largo se rechaza con una explicación y no se publica nada; nunca se recorta. Los humanos están exentos. 0 = sin límite.
aiball-config-tickets-rules-require-then = Comentarios de agente con then: o handback
    .about = Activado, el comentario de un agente sin then: debe fijar handback (true: devuelve el ticket, false: se lo queda), o se rechaza con una explicación y no se publica nada. Los humanos están exentos.
aiball-config-tickets-rules-require-commits = Comentarios de agente con commits
    .about = Activado, el comentario de un agente debe decir qué commits entrega (commits: ["<sha>"]) o que no entrega ninguno (commits: null o "none"), o se rechaza con una explicación. Un cliente anterior al campo recibe un aviso en vez de un rechazo hasta que se reconecte. Los humanos, el cierre y la reapertura están exentos.
aiball-config-tickets-steps-stale = Paso estancado tras
    .about = Un paso (then: continue) sin nada después durante este tiempo se señala en la bandeja de entrada: el trabajo que anunció se ha callado. 0 = nunca señalar.
aiball-config-tickets-steps-hot = Minutos que un paso deja su ticket arriba del backlog de su autor
    .about = Tras publicar un agente un paso (then: continue), su ticket encabeza el backlog de ese agente — justo después de los eventos, delante de cualquier otro ticket — durante este tiempo, contado desde el paso. Pasado ese tiempo, el ticket se ordena como cualquier otro. Solo cambia el orden; la marca visible «urgente» sigue su propia regla. 0 = un paso no da prioridad.
aiball-config-tickets-steps-max-wait = Espera más larga que puede declarar un paso
    .about = Lo máximo que un agente puede poner en resume_on.timer en un paso (then: continue). Una espera más larga se rechaza con este límite en el motivo — más allá, el trabajo ya no es un paso esperando una tarea: devuelve el ticket, o propón un plan.
aiball-config-tickets-wait-credit-enabled = Crédito de espera
    .about = true (default) = el resume_on.timer de un paso gasta el crédito de espera de un agente, ganado con pruebas de trabajo. false = las esperas son gratis y sin tope (como mucho tickets.step_after_max_minutes), no se gana, gasta ni devuelve nada, y las respuestas y los despertares no hablan de crédito.
aiball-config-tickets-wait-credit-refund = Crédito de espera: devolver una vuelta anticipada
    .about = true (default) = un agente que vuelve a hablar en un ticket antes de que acabe la espera de su paso recupera el resto de esa espera. false = una espera declarada se gasta entera.
aiball-config-tickets-wait-credit-earn-commit-max-age = Crédito de espera: commit más antiguo que aún gana
    .about = Un commit citado cuya fecha de commit es más antigua que esto no gana nada: el crédito premia el trabajo reciente.
aiball-config-tickets-wait-credit-earn-commits-per-comment = Crédito de espera: commits contados por comentario
    .about = El máximo de commits que un comentario puede citar para ganar crédito; los que pasan de ahí no ganan nada y la respuesta lo dice.
aiball-config-tickets-wait-credit-start = Crédito de espera inicial de un agente
    .about = Cada agente empieza cada proyecto con este crédito de espera, para que un agente nuevo pueda esperar una primera compilación. Los temporizadores de los pasos (resume_on.timer) gastan el crédito; las pruebas de trabajo lo ganan.
aiball-config-tickets-wait-credit-max = Crédito de espera máximo de un agente
    .about = Un saldo nunca pasa de esto: lo que lo haría pasar no se abona, y un saldo que ya lo supera se recorta. 0 = sin tope.
aiball-config-tickets-wait-credit-floor = Espera mínima de un paso, aun sin crédito
    .about = Sin crédito suficiente, la espera de un paso se limita al saldo pero nunca por debajo de esto, para que un agente sin crédito no vuelva en bucle. Nunca deja el saldo por debajo de cero. Un paso que pide 0 (seguir de inmediato) siempre se concede.
aiball-config-tickets-wait-credit-earn-resolved = Crédito de espera por un ticket resuelto, con commit
    .about = Se gana una vez por ticket, para el agente cuya resolución se aceptó, si citó un commit en ese ticket (commits: [...]) antes de cerrarse.
aiball-config-tickets-wait-credit-earn-resolved-no-commit = Crédito de espera por un ticket resuelto, sin commit
    .about = Se gana una vez por ticket, para el agente cuya resolución se aceptó sin haber citado ningún commit en ese ticket: una resolución sin código vale menos.
aiball-config-tickets-wait-credit-earn-wontfix = Crédito de espera por un ticket cerrado sin corrección
    .about = Se gana una vez por ticket, para el agente cuyo cierre sin corrección (wontfix) se aceptó.
aiball-config-tickets-wait-credit-earn-lines-per-minute = Líneas cambiadas por minuto de crédito de un commit
    .about = Un commit que un agente cita en una respuesta (commits: [...]) gana un minuto por cada tantas líneas cambiadas, leídas en la copia de trabajo del agente. Una vez por commit.
aiball-config-tickets-wait-credit-earn-commit-max = Crédito de espera máximo por commit
    .about = El tope de lo que gana un solo commit, por grande que sea su diff.
aiball-config-tickets-wait-credit-earn-commit-min = Crédito de espera mínimo por commit
    .about = Lo que gana un commit citado con al menos una línea cambiada, por pequeño que sea su diff: una corrección corta también es trabajo. 0 = solo cuenta la tarifa por línea.
aiball-config-tickets-backlog-claim-protect = Minutos que se protege la toma de un agente activo
    .about = Cuánto resiste una toma (claim) a la de otro agente, contado desde la última acción de quien lo tiene en el ticket — trabajar en él mantiene la protección. La toma de otro agente dentro de ese plazo se rechaza; pasado, el ticket se puede quitar, y el hilo lo registra. Una asignación siempre gana a una toma. 0 = sin protección.
aiball-config-tickets-backlog-rest = Reposo de un ticket tras un despertar por backlog
    .about = Tras nombrar un ticket un despertar por backlog, cuánto tiempo queda fuera de los siguientes despertares por backlog del agente mientras nadie más lo mueva. Un ticket bloqueado reposa `blocked_multiplier` veces más, un ticket cuya última acción es un paso solo `after_step`. Un bucle arrancado con CL_BACKLOG_COOLDOWN_SEC aplica ese valor. 0 = sin reposo.
aiball-config-tickets-backlog-depth = Hasta dónde despierta el backlog a un agente
    .about = El nivel de backlog más profundo por el que un despertar puede llamar a un agente, cuando no tiene ningún evento que leer. Los tickets críticos, urgentes y accionables siempre lo despiertan. followup: también un ticket donde alguien respondió pero lo retiene una decisión pendiente del propio agente. waiting: también un ticket donde el agente habló el último y nada se ha movido desde entonces. blocked: también un ticket retenido por una dependencia abierta (vuelve de todos modos cuando se cierra lo que lo bloquea). Un ticket por debajo del ajuste sigue en el backlog, visible; solo que no despierta a nadie.
aiball-config-tickets-backlog-blocked-multiplier = Multiplicador de espera para tickets bloqueados
    .about = Cuánto más tiempo un despertar por backlog deja un ticket BLOQUEADO (retenido por un depends_on abierto) fuera de los despertares, comparado con cualquier otro ticket. Debe seguir apareciendo para no olvidarse, pero nada se mueve en él entre dos despertares. 1 = la misma espera que los demás.
aiball-config-tickets-backlog-after-step = Espera del backlog tras un paso
    .about = Cuánto tiempo un despertar por backlog deja un ticket fuera de los despertares cuando su última acción es un paso (then: continue), en vez de la espera completa. Corta a propósito: el paso dice que hay trabajo ahora, y la pausa solo deja que la cola avance. 0 = nunca apartarlo.
aiball-config-claude-loop-on-repetitive-denied-threshold = Denegaciones antes de que responda el bucle
    .about = Cuántas llamadas a herramientas debe denegar a un agente el sistema de permisos de Claude Code en la última hora antes de que su bucle envíe el prompt de abajo (o la salida del comando). No se envía nada mientras ambos estén vacíos.
aiball-config-claude-loop-on-repetitive-denied-max-per-hour = Prompts por hora, como máximo
    .about = Como mucho estos prompts por hora por denegaciones repetidas, para que se corte un ciclo denegación, prompt, denegación. 0 = nunca enviar.
aiball-config-claude-loop-on-repetitive-denied-prompt = Prompt ante denegaciones repetidas
    .about = Se escribe tal cual en el prompt de Claude, como un despertar y solo cuando podría haberlo (sin retención AFK, zen, escritura, límite de uso, arranque ni ocupado). Vacío = nada (por defecto).
aiball-config-claude-loop-on-repetitive-denied-command = Comando que escribe el prompt
    .about = Un comando de shell ejecutado en la máquina del bucle, en su carpeta, durante 10 s como máximo. Recibe las denegaciones en JSON por stdin (agent, project, cwd, tool, reason, last_hour, recent); lo que imprime es el prompt. Gana al prompt de arriba. Vacío = ninguno.
aiball-config-autopoll-volatile = Autopoll: avisos únicos
    .about = true = avisa solo cuando llega un ping estrictamente más reciente (sin recordatorios por tiempo). false (default) = un recordatorio persistente vuelve a sonar tras throttle_seconds.
aiball-config-autopoll-throttle = Autopoll: ritmo de recordatorios
    .about = Ritmo de los recordatorios en segundos, ignorado con volatile=true. 0 = en cada Stop (molesto). Los pings nuevos y los tickets abiertos nuevos se saltan el límite.
aiball-config-autopoll-recent-tickets = Autopoll: títulos recientes incluidos
    .about = Hasta N títulos de tickets recientes sin leer en el motivo del hook, para que el agente sepa qué le espera antes de vaciarlo. 0 = solo el número.
aiball-config-autopoll-backlog = Autopoll: el backlog como disparador
    .about = true (default) = los tickets abiertos del alcance disparan notificaciones aun sin pings sin leer. false = solo contexto (se muestran en el motivo, nunca disparan).
aiball-config-autopoll-tone = Autopoll: tono
    .about = hint = cortés, fácil de ignorar. directive (default) = nombra la acción de forma explícita. imperative = último recurso si el agente insiste en pedir permiso.
aiball-config-assign-window-sec = Plazo de asignación
    .about = Cuánto dura una asignación o una toma antes de que el ticket vuelva a la reserva común.
aiball-config-hot-window-sec = Plazo «urgente»
    .about = Lo reciente que debe ser un movimiento para que un ticket siga urgente en la bandeja de entrada.
aiball-config-upstream-transport = Upstream: transporte
    .about = Cómo salen las llamadas upstream (GitHub / GitLab): gh (la credencial de la CLI), http, o auto (gh cuando funciona). El transporte propio de un acoplamiento gana.
aiball-config-upstream-sync = Upstream: vigilancia
    .about = pull (default) = un repo acoplado se vigila y sus cambios se anuncian; off = solo enlazado. La sincronización propia de un acoplamiento gana.
