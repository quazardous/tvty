# A interface do tvty, em português do Brasil. Mesmos arquivos e mesmos
# ids que o inglês (assets/locales/en/), que é a referência; `tvty i18n`
# mede cada idioma.
# A lista de tickets, em tela cheia, e suas ações em massa (src/fulllist.rs,
# src/bulk.rs). As palavras de uma ação em massa vão pela sua chave: seu
# rótulo, .about, e o que ela fez (.done).

fulllist-title = Tickets — {$project}
fulllist-title-all = Tickets — todos os projetos
fulllist-shown = { $count ->
    [0] nenhum mostrado
    [one] {$count} mostrado
   *[other] {$count} mostrados
}
fulllist-new = + Novo ticket
fulllist-search = Buscar títulos…
fulllist-projects = Projetos
fulllist-all-projects = Todos os projetos
fulllist-bands = Faixas
fulllist-sort = Ordenar
fulllist-sort-activity = Última atividade
fulllist-sort-turn = Vez de quem (faixas)
fulllist-sort-priority = Prioridade
fulllist-sort-created = Criação
fulllist-sort-number = Número
fulllist-filters = Filtros
fulllist-open = Abertos
fulllist-all = Todos
fulllist-unread = Não lidos
fulllist-reading-small = lendo…
fulllist-critical = Crítico — o que mais segura
fulllist-all-group = Todos
fulllist-reading = Lendo…
fulllist-none = Nenhum ticket aqui.

## Uma linha

fulllist-priority = prioridade: {$priority}
fulllist-rejected = rejeitado
fulllist-milestone = marco {$title}
fulllist-assigned-to = atribuído a {$who}
fulllist-claimed-by = reivindicado por {$who}
fulllist-spoke-last = {$who} falou por último
fulllist-you-spoke-last = você falou por último
fulllist-blocked = bloqueado
fulllist-payload = payload
fulllist-by = por {$who} · {$date}
fulllist-hot = um agente esteve ativo nele ultimamente

## Em massa

fulllist-selected = { $count ->
    [0] nenhum selecionado
    [one] {$count} selecionado
   *[other] {$count} selecionados
}
fulllist-select-all = Selecionar todos os mostrados
fulllist-clear = Limpar · Esc
fulllist-actions = Ações
fulllist-actions-about = Cada uma age sobre os tickets escolhidos a que serve: quantos, à direita.
fulllist-working = Trabalhando…
fulllist-count-of = {$count} de {$of}
fulllist-confirm = { $count ->
    [one] {$action} {$count} ticket?
   *[other] {$action} {$count} tickets?
}
fulllist-cancel = Cancelar
bulk-refused = , { $count ->
    [one] {$count} recusado
   *[other] {$count} recusados
} ({$first})
bulk-approve = Aprovar
    .about = Deixa passar os tickets escolhidos que esperam moderação
    .done = { $count ->
        [0] nenhum aprovado
        [one] {$count} aprovado
       *[other] {$count} aprovados
    }
bulk-reject = Rejeitar
    .about = Recusa os tickets escolhidos que esperam moderação
    .done = { $count ->
        [0] nenhum rejeitado
        [one] {$count} rejeitado
       *[other] {$count} rejeitados
    }
bulk-close = Fechar
    .about = Fecha os tickets escolhidos que estão abertos (pergunta primeiro)
    .done = { $count ->
        [0] nenhum fechado
        [one] {$count} fechado
       *[other] {$count} fechados
    }
bulk-reopen = Reabrir
    .about = Abre de novo os tickets escolhidos que estão fechados
    .done = { $count ->
        [0] nenhum reaberto
        [one] {$count} reaberto
       *[other] {$count} reabertos
    }
bulk-mark-read = Marcar como lidos
    .about = Marca como lidos os tickets escolhidos com algo novo
    .done = { $count ->
        [0] nenhum marcado como lido
        [one] {$count} marcado como lido
       *[other] {$count} marcados como lidos
    }
bulk-mark-unread = Marcar como não lidos
    .about = Marca como não lidos os tickets escolhidos já lidos
    .done = { $count ->
        [0] nenhum marcado como não lido
        [one] {$count} marcado como não lido
       *[other] {$count} marcados como não lidos
    }
bulk-snooze = Adiar…
    .about = Põe de lado os tickets abertos escolhidos pelo tempo que você escolher: eles voltam então
    .done = { $count ->
        [0] nenhum adiado
        [one] {$count} adiado
       *[other] {$count} adiados
    }
bulk-unsnooze = Retomar agora
    .about = Traz de volta agora os tickets escolhidos que estão adiados
    .done = { $count ->
        [0] nenhum retomado
        [one] {$count} retomado
       *[other] {$count} retomados
    }
bulk-step = Marcar como etapa
    .about = Marca a última palavra dos tickets abertos escolhidos como etapa: o agente segue, nada a decidir
    .done = { $count ->
        [0] nenhum marcado como etapa
        [one] {$count} marcado como etapa
       *[other] {$count} marcados como etapa
    }
bulk-link = Vincular
    .about = Vincula os tickets escolhidos: o mais novo se relaciona com cada um dos outros
    .done = { $count ->
        [0] nenhum vinculado
        [one] {$count} vinculado
       *[other] {$count} vinculados
    }
