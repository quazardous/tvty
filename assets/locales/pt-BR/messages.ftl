# A interface do tvty, em português do Brasil. Mesmos arquivos e mesmos
# ids que o inglês (assets/locales/en/), que é a referência; `tvty i18n`
# mede cada idioma.
# Notificações e os diálogos pequenos: uma sessão que terminou, sair, a
# conversa a retomar, o cartão das dicas (src/pings.rs, src/updates.rs,
# src/shell/ended.rs, src/shell/quit.rs, src/shell/resume.rs,
# src/shell/tips.rs).

## O que tocou

messages-proposes-plan = propõe um plano
messages-proposes-close = propõe fechar
messages-proposes-wontfix = propõe fechar sem correção
messages-escalates = escala
messages-new-ticket = um ticket novo
messages-new-comment = um comentário novo
messages-new-ticket-moderate = um ticket novo para moderar
messages-missed = { $more ->
    [0] {$what} · enquanto o tvty estava fechado
    [one] {$what} · e mais {$more} enquanto o tvty estava fechado
   *[other] {$what} · e mais {$more} enquanto o tvty estava fechado
}
messages-dismiss = dispensar

## Atualizações

messages-update-out = O Terminal Velocity {$latest} saiu (você roda o {$running}): {$how}
messages-update-how = as Atualizações… do menu o instalam
messages-update-dev = este é um build de desenvolvimento: atualize o checkout dele (as Atualizações… do menu dizem como)
messages-aiball-old = o aiball {$version} é mais antigo do que o Terminal Velocity precisa ({$needs}): as Atualizações… do menu o atualizam

## Uma sessão que terminou

messages-detached = — desanexada por outro cliente
messages-ended = — a sessão terminou
messages-attach-again = Anexar de novo
messages-starting = Iniciando…
messages-restart = Reiniciar
messages-close = Fechar
messages-copies-failed = tornar os outros terminais cópias

## Sair e começar de novo

messages-stopping = Parando as sessões do Claude Code, depois saindo…
messages-quit-stop-failed = parar na saída do tvty
messages-ran-on = {$agents} seguiram rodando: a parada não pegou
messages-restarted-as-were = { $count ->
    [one] {$count} sessão reiniciada como estava, retomando sua conversa
   *[other] {$count} sessões reiniciadas como estavam, retomando suas conversas
}
messages-restarted-fresh = { $count ->
    [one] {$count} sessão reiniciada do zero, retomando sua conversa
   *[other] {$count} sessões reiniciadas do zero, retomando suas conversas
}
messages-restart-failed = reinício das sessões paradas
messages-restart-tvty-failed = reiniciar o tvty

## Qual conversa retomar

messages-its-agent = seu agente
messages-ago = {$time} atrás
messages-some-time-ago = há um tempo
messages-last-one = A última, {$when}:
messages-says-nothing = (nada dito ainda)
messages-quoted = “{$said}”
messages-older = { $count ->
    [one] Mais {$count} antiga: /resume no Claude Code escolhe entre elas.
   *[other] Mais {$count} antigas: /resume no Claude Code escolhe entre elas.
}
messages-new-conversation = Conversa nova
messages-resume-last = Retomar a última conversa
messages-start = Iniciar {$who}
messages-resume-question = O Claude Code já tem conversas em {$cwd}, nenhuma de um loop: qual {$who} retoma?

## O cartão das dicas

messages-tip-title = Você sabia?
messages-tips-browsing = Dicas · {$at} / {$of}
messages-not-now = Agora não
messages-previous = ‹ Anterior
messages-next = Próxima ›
messages-show-all-again = Mostrar todas de novo
messages-got-it = Entendi
messages-next-tip = Próxima dica
messages-turn-off = Desligar as dicas
messages-tips-off = As dicas estão desligadas: Configurações > Layout > Dicas liga de novo
messages-tips-again = Cada dica vai mostrar de novo, uma de cada vez
messages-shells-reopened = {$count} terminal(is) reaberto(s) onde estavam
messages-shells-reopen-failed = reabrir os terminais
