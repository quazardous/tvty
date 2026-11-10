# A interface do tvty, em português do Brasil. Mesmos arquivos e mesmos
# ids que o inglês (assets/locales/en/), que é a referência; `tvty i18n`
# mede cada idioma.
# O painel de tickets e o que ele diz de um ticket (src/panel.rs,
# src/thread.rs, src/rowstate.rs).

## O painel

tickets-title = Tickets
tickets-no-project = Nenhum projeto do aiball neste terminal.
tickets-none-open = Nenhum ticket aberto.
tickets-loading = Carregando…
tickets-back = ← Tickets
tickets-fold-panel = Recolher o painel de tickets
tickets-unfold-panel = Estender o painel de tickets
tickets-pinned = Fixado: o painel fica ao lado do terminal. Não fixado: cobre o terminal
tickets-no-session = nenhuma sessão aberta
tickets-megaphone = A instrução permanente e o foco de despertar do projeto
tickets-new = + Novo
tickets-new-tip = Um ticket novo
tickets-full-list = A lista de tickets, em tela cheia
tickets-sunk = afundado no backlog de {$agent} até {$until} (em {$left}): seu loop não o traz antes, a não ser que o thread se mexa

## As faixas da lista

tickets-band-moderate = Para moderar
tickets-band-decide = Esperando você
tickets-band-working = Agentes nele
tickets-band-open = Abertos
tickets-band-closed = Fechados

## As marcas de uma linha

tickets-pending-comments = { $count ->
    [one] {$count} comentário esperando moderação
   *[other] {$count} comentários esperando moderação
}
tickets-unread = não lido: algo novo nele para você
tickets-you-spoke-last = você falou por último
tickets-someone-spoke-last = outra pessoa falou por último
tickets-comments-tip = { $count ->
    [one] {$count} comentário — {$why}
   *[other] {$count} comentários — {$why}
}
tickets-held-by = reivindicado por {$holder}
tickets-held-hot = reivindicado por {$holder}, ativo nele ultimamente
tickets-critical = { $holds ->
    [one] o ticket crítico do projeto: ele segura {$holds} ticket aberto
   *[other] o ticket crítico do projeto: ele segura {$holds} tickets abertos
}
tickets-critical-quiet = , quieto por {$quiet}
tickets-step-resumes = etapa — o agente retoma em {$at}
tickets-critical-said = segura {$holds}
tickets-critical-said-quiet = segura {$holds} · quieto {$quiet}
tickets-stage-rejected = rejeitado
tickets-stage-closed-resolved = fechado, resolvido
tickets-stage-closed = fechado
tickets-stage-resolved = resolvido
tickets-stage-blocked = bloqueado
tickets-stage-snoozed = adiado
tickets-stage-pending = pendente
tickets-stage-open = aberto
tickets-to-moderate = para moderar

## O que um glifo significa

tickets-glyph-escalation = um agente escala: ele precisa de você
tickets-glyph-plan = um plano é proposto
tickets-glyph-question = uma pergunta espera resposta
tickets-glyph-question-yours = uma pergunta espera sua resposta
tickets-glyph-resolution = uma resolução é proposta
tickets-glyph-wontfix = fechar sem correção é proposto
tickets-glyph-stalled = a etapa de um agente silenciou
tickets-glyph-step = um agente está numa etapa (then: continue)
tickets-glyph-rejected = o último plano ou resolução foi rejeitado
tickets-glyph-closed-resolved = fechado, resolvido
tickets-glyph-closed = fechado sem resolução

## Decisões

tickets-kind-plan = plano
tickets-kind-resolution = resolução
tickets-kind-wontfix = fechamento sem correção
tickets-kind-escalation = escalonamento
tickets-proposes-plan = {$who} propõe um plano
tickets-proposes-resolution = {$who} propõe uma resolução
tickets-proposes-wontfix = {$who} propõe fechar sem correção
tickets-proposes-escalation = {$who} propõe um escalonamento
# $kind (plan, resolution, wontfix, escalation): para os idiomas cujas
# palavras concordam com o substantivo.
tickets-decision-pending = {$noun} · pendente
tickets-decision-accepted = { $kind ->
    [resolution] {$noun} aceita
   *[other] {$noun} aceito
}
tickets-decision-rejected = { $kind ->
    [resolution] {$noun} rejeitada
   *[other] {$noun} rejeitado
}
tickets-decision-withdrawn = { $kind ->
    [resolution] {$noun} · retirada
   *[other] {$noun} · retirado
}
tickets-decision-superseded = { $kind ->
    [resolution] {$noun} · superada
   *[other] {$noun} · superado
}
tickets-accept-close = Aceitar → fechar
tickets-accept-wontfix = Aceitar → fechar sem correção
tickets-accept-escalation = Feito → aceitar
tickets-accept-plan = Aceitar → seguir
tickets-reject = Rejeitar
tickets-approve = Aprovar
tickets-decided-once-approved = decidido assim que o ticket for aprovado
tickets-reject-say-why = Para rejeitar, diga o porquê abaixo primeiro.
tickets-waits-moderation = Este ticket espera moderação

## Onde um ticket está (a frase sob seu título)

tickets-you = você
tickets-an-agent = um agente
tickets-closed-resolved-by = Fechado, resolvido por {$who}
tickets-closed-resolved = Fechado, resolvido
tickets-closed-unresolved = Fechado sem resolução
tickets-yours-moderation = Seu: este ticket espera moderação
tickets-your-proposal-plan = Seu plano espera uma decisão
tickets-your-proposal-resolution = Sua resolução espera uma decisão
tickets-your-proposal-wontfix = Seu fechamento sem correção espera uma decisão
tickets-your-proposal-escalation = Seu escalonamento espera uma decisão
tickets-yours-escalates = Seu: {$who} escala — aja, depois aceite
tickets-yours-decide-plan = Seu: aceitar ou rejeitar o plano de {$who}
tickets-yours-decide-resolution = Seu: aceitar ou rejeitar a resolução de {$who}
tickets-yours-decide-wontfix = Seu: aceitar ou rejeitar o fechamento sem correção de {$who}
tickets-step-quiet = a etapa de {$agent} silenciou
tickets-on-step = {$agent} está numa etapa
tickets-step-resumes-in = {" · "}retoma em {$span}
tickets-step-waits-on = {" · "}espera o #{$ticket}
tickets-yours-answer = Seu: responda {$who}
tickets-holds-you-spoke = {$holder} o mantém: você falou por último
tickets-yours-nobody = Seu: ninguém mais está nele
tickets-their-turn-of = vez de {$holder}: você falou por último
tickets-their-turn = Vez deles: você falou por último
tickets-span-under-minute = menos de um minuto
tickets-span-minutes = {$n} min
tickets-span-hours = {$n} h

## O thread

tickets-images = { $count ->
    [one] 🖼 imagem
   *[other] 🖼 {$count} imagens
}

tickets-where-it-stands = Onde está · {$who}
tickets-edit = ✎ editar
tickets-edit-tip = Editar o título e o corpo
tickets-newest-first = ⇅ mais recente primeiro
tickets-newest-last = ⇅ mais recente por último
tickets-close-full = ✕  Esc
tickets-more = ⤢ mais
tickets-comments-spoke = { $count ->
    [one] {$count} comentário · {$who} falou por último
   *[other] {$count} comentários · {$who} falou por último
}
tickets-unfold-all = desdobrar tudo
tickets-fold-before-summary = dobrar antes do resumo
tickets-fold = dobrar
tickets-unfold = desdobrar
tickets-snoozed-until = adiado até {$until}
tickets-tokens = {$count} tok
tickets-priority-tip = a prioridade: um clique a muda
tickets-priority-label = Prioridade
tickets-rel-depends = depende de
tickets-rel-blocks = bloqueia
tickets-rel-relates = relaciona-se com
tickets-rel-duplicates = duplica
tickets-rel-duplicated = duplicado por
tickets-rel-parent = pai de
tickets-rel-child = filho de
tickets-step = etapa
tickets-resumes = retoma
tickets-resumes-on = em
tickets-resumes-or-on = ou em
tickets-resumes-at = o agente retoma em {$at}
tickets-resumes-when = o agente retoma quando o #{$ticket} se mexe (uma resposta, uma decisão, um fechamento)
tickets-or-when = ou quando o #{$ticket} se mexe (uma resposta, uma decisão, um fechamento)

## As ações de um comentário

tickets-comment-actions = as ações do comentário
tickets-edit-comment = Editar
tickets-delete = Excluir
tickets-really-delete = Excluir mesmo?
tickets-classify-as = como {$noun}
tickets-no-decision = sem decisão
tickets-not-a-step = não é etapa
tickets-a-step = uma etapa
tickets-resurface = Trazê-lo de volta
tickets-cancel = Cancelar
tickets-save = Salvar

## Escrevendo

tickets-reply-placeholder = Responder… (ctrl+enter envia)
tickets-editing = Editando o #{$id}
tickets-edit-keys = ctrl+enter salva · esc desfaz
tickets-title-first = Um título primeiro.
tickets-nothing-to-preview = Nada para prévia ainda.
tickets-close = Fechar
tickets-reopen = Reabrir
tickets-wake = Acordar
tickets-snooze = Adiar ▾
tickets-snooze-for = Adiar por
tickets-snooze-hour = 1 hora
tickets-snooze-day = um dia
tickets-snooze-week = uma semana
tickets-snooze-3days = 3 dias
tickets-snooze-month = um mês
tickets-assign = Atribuir ▾
tickets-assigned-menu = → {$who} ▾
tickets-assign-to = Atribuir a
tickets-no-agent = o projeto não tem agente
tickets-claim-assigns = {$agent} o mantém por reivindicação: um clique o atribui a ele
tickets-unassign-tip = desatribuir: tira-o de {$holder}; ninguém o mantém, ele espera quem o tome
tickets-without-notifying = sem notificar
tickets-option-choose-one = Escolha uma:
tickets-option-choose-several = Escolha uma ou mais:
tickets-option-recommended = recomendada
tickets-option-confidence-high = seu autor tem certeza
tickets-option-confidence-medium = seu autor tem razoável certeza
tickets-option-confidence-low = seu autor não tem certeza
tickets-reply-keep = Responder, manter a vez
tickets-reply-ways-tip = Outros jeitos de enviar (ctrl+shift+enter: manter a vez)
tickets-you-keep-the-hand = Você mantém a vez: nenhum agente entra nele até sua próxima resposta
tickets-keeps-the-hand = mantém a vez
tickets-keeps-the-hand-tip = Não acabou: este comentário não acorda agente algum, exceto um que mencione
tickets-reply = Responder

## A coluna de campos

tickets-group-state = Estado
tickets-group-fields = Campos
tickets-group-people = Pessoas
tickets-group-links = Vínculos
tickets-group-project = Projeto
tickets-group-tokens = Tokens
tickets-group-payload = Payload
tickets-lifecycle = ciclo de vida
tickets-lifecycle-closed-resolved = fechado, resolvido
tickets-lifecycle-closed = fechado
tickets-lifecycle-moderation = espera moderação
tickets-lifecycle-open = aberto
tickets-snoozed = adiado
tickets-until = até {$date}
tickets-field-intent = intenção
tickets-field-priority = prioridade
tickets-field-level = nível
tickets-field-scope = escopo
tickets-field-tags = tags
tickets-field-milestone = marco
tickets-field-reporter = autor
tickets-field-claimed = reivindicado por
tickets-field-assigned = atribuído a
tickets-field-followers = seguidores
tickets-field-project = projeto
tickets-filed = registrado {$date}
tickets-lapsed = {$who} (vencido)
tickets-claim-until = {$who}, até {$date}
tickets-muted = silenciado: não notificado deste ticket, nem pelo seu papel — ✕ o devolve ao que seu papel diz
tickets-followers-tip = Quem segue este ticket por escolha própria (ou o silenciou). Os donos do projeto são notificados pelo papel: não estão listados aqui.
tickets-sub-ticket = subticket
tickets-new-sub = + um novo
tickets-sub-ticket-of = subticket de
tickets-sub-tickets = subtickets
tickets-remove-relation = remover este vínculo
tickets-relate = vincular
tickets-relate-to = a…
tickets-relate-add = + um ticket
tickets-move-to = Mover para {$project}
tickets-in-out = entra · sai
tickets-cache = cache
tickets-cache-said = {$written} escritos · {$read} lidos
tickets-payload = este ticket carrega um payload (veja a interface web)
tickets-pick-project = um projeto
tickets-pick-project-search = um projeto…
tickets-pick-tag = acrescentar uma tag
tickets-pick-tag-search = uma tag…
tickets-pick-milestone = nenhum
tickets-pick-milestone-search = um marco…
tickets-pick-reporter = o autor
tickets-pick-agent-or-you = um agente, ou você…
tickets-pick-follower = acrescentar
tickets-pick-intent-search = uma intenção…
tickets-pick-priority-search = uma prioridade…
tickets-pick-level-search = um nível…
tickets-pick-scope-search = um escopo…
tickets-pick-nobody = ninguém
tickets-pick-agent-search = um agente…

## O que um gesto fez (dito numa notificação)

tickets-did-reply = resposta publicada
tickets-did-with-reply = {$what}, com uma resposta
tickets-did-on = {$title} — {$what}
tickets-did-comment-edited = comentário editado
tickets-did-tagged = tag {$name} acrescentada
tickets-did-tag-removed = tag {$name} removida
tickets-did-milestone-set = marco definido
tickets-did-milestone-removed = marco removido
tickets-did-reporter = autor agora {$name}
tickets-did-assigned = atribuído a {$name}
tickets-did-unassigned = desatribuído
tickets-did-follows = {$name} agora o segue
tickets-did-unfollows = {$name} não o segue mais
tickets-did-intent = intenção {$value}
tickets-did-priority = prioridade {$value}
tickets-did-level = nível {$value}
tickets-did-scope = escopo {$value}
tickets-did-edited = título e corpo editados
tickets-did-related = vinculado ao #{$target} ({$kind})
tickets-did-unrelated = vínculo com o #{$target} removido
tickets-did-moved = movido para {$project}
tickets-did-snoozed = adiado
tickets-did-woken = acordado
tickets-did-accepted = decisão aceita
tickets-did-rejected = decisão rejeitada
tickets-did-approved = aprovado
tickets-did-rejected-moderation = rejeitado na moderação
tickets-did-closed = fechado
tickets-did-reopened = reaberto
tickets-did-deleted = comentário excluído
tickets-did-classified = comentário virou {$noun}
tickets-did-plain = comentário tornado comum
tickets-did-step-marked = etapa marcada
tickets-did-step-unmarked = etapa desmarcada
tickets-did-voted = votado
tickets-did-resurfaced = comentário trazido de volta

## Os eventos do thread

tickets-event-closed = fechou o ticket
tickets-event-reopened = reabriu o ticket
tickets-event-resolved = marcou-o resolvido
tickets-event-blocked = marcou-o para ser decidido
tickets-event-taken-over = tomou a reivindicação
tickets-event-sub-added = acrescentou um subticket{$source}
tickets-event-referenced = referenciou-o de{$source}
tickets-event-dependency-closed = uma dependência fechou{$source}
tickets-event-dependency-rejected = uma dependência foi rejeitada{$source}
tickets-event-related-closed = um ticket vinculado fechou{$source}
tickets-event-linked = vinculou{$source}
tickets-waits-on = espera o #{$id}
tickets-waits-on-tip = A etapa do agente espera o #{$id}: {$title}{$state}
