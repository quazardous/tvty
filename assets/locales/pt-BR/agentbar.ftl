# A interface do tvty, em português do Brasil. Mesmos arquivos e mesmos
# ids que o inglês (assets/locales/en/), que é a referência; `tvty i18n`
# mede cada idioma.
# A barra do agente, sob seu terminal (src/shell/agentbar.rs).

## Quem conduz o loop

agentbar-afk-auto = auto
agentbar-afk-hold-10m = pausar 10 min
agentbar-afk-hold = pausar
agentbar-afk-boot = … boot
agentbar-afk-arming-tip = armado: o homenzinho mostra o modo escolhido com F9, em vigor 3 s após a última tecla — ▶ ou ‖ diz o que vale até lá
agentbar-afk-tip = quem conduz o loop: ▶ por conta própria, ‖ pausado para você (ou enquanto você digita); o homenzinho é o modo AFK — cinza: você está fora, o loop segue sozinho; os segundos de uma pausa de 10 min; ∞ pausado. F9 o percorre (auto → 10 min → ∞), em vigor 3 s após a última tecla; um clique escolhe

## O que o Claude dele faz

agentbar-working = trabalhando
agentbar-starting = iniciando
agentbar-idle = ocioso
agentbar-offline = offline
agentbar-reload = relançar o loop
agentbar-reload-tip = o Claude dele roda, mas nada o conduz (o kernel dele morreu com o aiball): relançado, seus despertares, barra e presença voltam; o Claude não é tocado
agentbar-reloading = relançando…
agentbar-reload-done = o loop {$name} conduz seu Claude de novo
agentbar-reload-failed = relançar o loop {$name}
# Depois do tempo desde o início do boot: " · faltam 12s".
agentbar-boot-left = {" · "}faltam {$seconds}s
agentbar-boot-tip = seu loop está iniciando: o Claude carrega, pode retomar sua conversa ou compactar; o loop o acorda assim que o boot acaba (30 s no mínimo, mais enquanto um retomar ou uma compactação aparece)
agentbar-state-tip = o que o Claude dele faz, e desde quando
agentbar-dialog = espera uma resposta

## Reiniciando o Claude dele

agentbar-restarting = reiniciando…
agentbar-restarting-tip = o Claude dele reinicia, retomando sua conversa
agentbar-restart-pending = reinício pendente
agentbar-restart-pending-tip = um reinício foi pedido: o Claude dele reinicia assim que estiver ocioso, retomando sua conversa; um clique o cancela
agentbar-update = atualização
agentbar-update-tip = o Claude Code dele instalou uma atualização: um clique pede para reiniciá-lo (já se ocioso, senão assim que estiver), retomando sua conversa
agentbar-restart-ask-busy = Reiniciar o Claude dele assim que estiver ocioso?
agentbar-restart-ask = Reiniciar o Claude dele agora?
agentbar-restart = Reiniciar
agentbar-restart-resumed = sua conversa é retomada
agentbar-cancel = Cancelar
agentbar-restart-done = o Claude de {$agent} reinicia assim que ocioso, retomando sua conversa
agentbar-restart-failed = reinício do Claude de {$agent}

## Onde seu loop roda, de quem são as mãos nele

agentbar-place-host = host
agentbar-hands-copy = cópia
agentbar-hands-controls = controles
agentbar-moving = movendo…
agentbar-runs-hosted = seu loop roda no host de sessões do aiball;
agentbar-runs-mux = seu loop roda em {$mux} (claude-loop);
agentbar-copy-tip = este terminal é uma cópia: você observa, nada do que digita chega ao Claude dele, e a sessão mantém seu tamanho.
agentbar-controls-tip = você tem os controles, compartilhados com qualquer outro cliente: o tamanho segue quem digita por último.
agentbar-others-attached = { $others ->
    [one] {$others} outro cliente anexado ({$typing} com os controles).
   *[other] {$others} outros clientes anexados ({$typing} com os controles).
}
agentbar-proxy-alive = O proxy de terminal à frente do Claude está vivo.
agentbar-place-click = Um clique: pegar ou largar os controles, mover o loop.
agentbar-others-type = o terminal do claude-loop também digita nele
agentbar-take-controls = Pegar os controles
agentbar-leave-copy = Largar por uma cópia
agentbar-close-others = Fechar os outros ({$others})
agentbar-close-others-tip = { $others ->
    [one] o outro cliente anexado a esta sessão a deixa (o terminal do claude-loop, outro Terminal Velocity); o Claude dele e este terminal seguem
   *[other] os {$others} outros clientes anexados a esta sessão a deixam (o terminal do claude-loop, outro Terminal Velocity); o Claude dele e este terminal seguem
}
agentbar-move-into = Mover para {$mux}
agentbar-move-to-host = Mover para o host
agentbar-move-tip-into = o Claude dele reinicia em {$mux}, retomando sua conversa
agentbar-move-tip-to-host = o Claude dele reinicia no host do aiball, retomando sua conversa
agentbar-move-interrupts = {" — "}ele trabalha agora: a mudança o interrompe
agentbar-moved-into = {$agent} movido para {$mux}, sua conversa retomada
agentbar-moved-to-host = {$agent} movido para o host do aiball, sua conversa retomada
agentbar-move-failed-into = mudança de {$agent} para {$mux}
agentbar-move-failed-to-host = mudança de {$agent} para o host do aiball
agentbar-hold-failed = pausa de {$agent}
agentbar-closed-others = { $count ->
    [one] fechado o outro cliente desta sessão
   *[other] fechados os {$count} outros clientes desta sessão
}
agentbar-closed-others-all = fechados os outros clientes desta sessão
agentbar-closed-others-asked = pedido ao host da sessão que feche seus outros clientes
agentbar-others-left = { $count ->
    [one] {$count} outro cliente ainda anexado: o {$mux} não o distingue deste ainda
   *[other] {$count} outros clientes ainda anexados: o {$mux} não os distingue deste ainda
}
agentbar-close-others-failed = fechar os outros clientes

## Remote Control, alertas, o prompt

agentbar-rc-on = Remote Control está ligado: este Claude pode ser tomado do claude.ai e do app móvel
agentbar-rc-off = Remote Control está desligado. /rc na sessão liga; os loops de uma pasta o recebem das configurações do projeto
agentbar-trust = confiar nesta pasta?
agentbar-not-logged-in = sem login
agentbar-api-unreachable = API inalcançável
agentbar-link-down = link do loop caído
agentbar-aiball-unreachable = aiball inalcançável
agentbar-prompt-input = o prompt do Claude está na tela, com texto ainda não enviado
agentbar-prompt-empty = o prompt do Claude está na tela, vazio
agentbar-typing = um humano digitou no terminal dele há um momento: o loop segura
agentbar-zen = zen
agentbar-zen-tip = modo zen: o loop fica quieto

## Seus contadores

agentbar-all = todos:{$count}
agentbar-all-tip = a: todos os tickets abertos do projeto
agentbar-backlog = backlog:{$count}
agentbar-backlog-tip = b: seu backlog, os tickets para ele olhar; um clique os lista
agentbar-events = eventos:{$count}
agentbar-events-tip = e: seus eventos ainda não vistos — pings, respostas, decisões esperando por ele
agentbar-holds = mantém:{$count}
agentbar-holds-tip = os tickets que ele mantém
agentbar-wake-tip = trabalho espera pelo loop: ele acorda seu Claude nele quando a contagem acaba
agentbar-pending-tip = trabalho espera pelo loop (eventos ou backlog)

## Seu backlog, sobre a barra

agentbar-backlog-of = backlog de {$agent}
agentbar-reading = lendo…
agentbar-backlog-error = backlog: {$error}
agentbar-backlog-empty = nada em seu backlog
agentbar-tier-critical = crítico
agentbar-tier-hot = hot
agentbar-tier-yours = seus
agentbar-tier-decision = sua decisão pendente
agentbar-tier-waiting = esperando por eles
agentbar-tier-blocked = bloqueado
agentbar-tier-other = outro

agentbar-info-resuming = retomando
agentbar-info-compacting = compactando
agentbar-info-wait = esperando
agentbar-info-interrupted = interrompido
agentbar-info-user = um humano no teclado
agentbar-info-picker-session = escolhendo uma sessão
agentbar-info-picker-mode = escolhendo um modo
agentbar-info-error-rate-limit = com limite de taxa
agentbar-info-error-overloaded = API sobrecarregada
agentbar-info-error-api = erro de API
agentbar-info-retry = tentativa {$attempt}
agentbar-restart-cancel-ask = Cancelar o reinício pendente do Claude dele?
agentbar-restart-cancel = Cancelar
agentbar-restart-keep = Manter
agentbar-restart-cancelled = o reinício do Claude de {$agent} foi cancelado; a atualização continua oferecida
agentbar-restart-none = nenhum reinício do Claude de {$agent} estava pendente
agentbar-restart-cancel-failed = cancelar o reinício do Claude de {$agent}
agentbar-restart-too-old = seu loop começou antes de o aiball poder cancelar um reinício: ele vai reiniciar
agentbar-config-tip = config dir do Claude Code: {$dir} ({$from}): sua conta, configurações e sessões. Clique para trocar para outro.
agentbar-config-tip-unread = config dir do Claude Code: sua conta, configurações e sessões. Clique para vê-lo e trocar.
agentbar-config-here = Roda em
agentbar-config-others = Trocar para
agentbar-config-none = Nenhum outro config dir nas pastas do aiball: defina um na página de opções de um projeto.
agentbar-config-default = padrão
agentbar-config-not-ready = nunca rodado
agentbar-config-not-ready-tip = Rode claude nele uma vez à mão antes (tema, login): um loop não pode passar por isso.
