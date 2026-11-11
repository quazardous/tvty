# A configuração do aiball, como Configurações > aiball a mostra: pela
# sua chave, estável (uma chave que o tvty não conhece mostra o inglês
# do próprio aiball). Pontos numa chave viram traços.
# Em português do Brasil: mesmos arquivos e mesmos ids que o inglês
# (assets/locales/en/), que é a referência; `tvty i18n` mede cada idioma.

aiball-group-autopoll = Autopoll
aiball-group-backlog = Backlog
aiball-group-claude-loop = Loop do Claude
aiball-group-defaults = Padrões
aiball-group-earn = Ganho
aiball-group-on-repetitive-denied = Em recusas repetidas
aiball-group-rules = Regras
aiball-group-steps = Etapas
aiball-group-tickets = Tickets
aiball-group-updates = Atualizações
aiball-group-wait-credit = Crédito de espera

aiball-config-updates-check = Verificar atualizações
    .about = true (padrão) = ao iniciar o daemon, e quando alguém pede, ele lê o release mais recente do aiball no GitHub para a bandeja, a extensão do GNOME e o `aiball version` poderem dizer que há atualização. false = nenhuma chamada externa.
aiball-config-tickets-defaults-priority = Prioridade padrão dos tickets
    .about = Prioridade aplicada a um ticket novo criado sem uma explícita. Defina um padrão global; um projeto pode substituí-lo.
aiball-config-tickets-defaults-broadcast-new = Broadcast de tickets novos por padrão
    .about = Ligado, um ticket novo sem escopo explícito é marcado broadcast (os seguidores do projeto são notificados). Declarado; a aplicação vem com seu consumidor.
aiball-config-tickets-rules-summary-max = Orçamento do resumo (caracteres)
    .about = O summary_until mais longo que um agente pode escrever num comentário. Um mais longo é recusado com uma explicação e nada é publicado; nunca é truncado. Humanos estão livres. 0 = sem limite.
aiball-config-tickets-rules-require-then = Comentários de agente exigem then: ou handback
    .about = Ligado, um comentário de agente sem then: deve definir handback (true: devolve o ticket, false: o mantém), ou é recusado com uma explicação e nada é publicado. Humanos estão livres.
aiball-config-tickets-rules-require-commits = Comentários de agente exigem commits
    .about = Ligado, um comentário de agente deve dizer quais commits entrega (commits: ["<sha>"]) ou que não entrega nenhum (commits: null ou "none"), ou é recusado com uma explicação. Um cliente anterior ao campo é avisado em vez de recusado até reconectar. Humanos, fechar e reabrir estão livres.
aiball-config-tickets-steps-stale = Etapa parada após
    .about = Uma etapa (then: continue) sem nada depois dela por este tempo é marcada na caixa de entrada: o trabalho que ela anunciou silenciou. 0 = nunca marcar.
aiball-config-tickets-steps-hot = Minutos em que uma etapa mantém seu ticket no topo do backlog do autor
    .about = Depois que um agente posta uma etapa (then: continue), seu ticket lidera o backlog daquele agente — logo após os eventos, à frente de todo outro ticket — por este tempo, contado da etapa. Passado, o ticket ordena como qualquer outro. Só muda a ordem; a marca visível de 'hot' tem regra própria. 0 = etapa nenhuma ganha prioridade.
aiball-config-tickets-steps-max-wait = Espera mais longa que uma etapa pode declarar
    .about = O máximo que um agente pode pôr em resume_on.timer numa etapa (then: continue). Uma espera mais longa é recusada com este limite no motivo — passando dele, o trabalho já não é uma etapa esperando um serviço: devolva o ticket, ou proponha um plano.
aiball-config-tickets-wait-credit-enabled = Crédito de espera
    .about = true (padrão) = o resume_on.timer de uma etapa gasta o crédito de espera do agente, ganho por prova de trabalho. false = esperas são grátis e sem teto (ainda no máximo tickets.steps.max_wait), nada é ganho, gasto ou devolvido, e respostas e despertares nada dizem de crédito.
aiball-config-tickets-wait-credit-refund = Crédito de espera: devolver uma volta antecipada
    .about = true (padrão) = um agente que falar num ticket de novo antes do fim da espera da sua etapa recebe de volta o resto dela. false = uma espera é gasta inteira uma vez declarada.
aiball-config-tickets-wait-credit-earn-commit-max-age = Crédito de espera: commit mais antigo que ainda ganha
    .about = Um commit citado cuja data é mais antiga que isto não ganha nada: o crédito recompensa trabalho fresco.
aiball-config-tickets-wait-credit-earn-commits-per-comment = Crédito de espera: commits contados por comentário
    .about = O máximo de commits que um comentário pode citar por crédito; os que passam não ganham nada, e a resposta o diz.
aiball-config-tickets-wait-credit-start = Crédito de espera com que um agente começa
    .about = Cada agente começa em cada projeto com este tanto de crédito de espera, para que um agente novo possa esperar uma primeira build. O crédito é gasto por temporizadores de etapa (resume_on.timer) e ganho por prova de trabalho.
aiball-config-tickets-wait-credit-max = Máximo de crédito de espera que um agente mantém
    .about = Um saldo nunca passa disto: o que o levaria além não é creditado, e um saldo já além é cortado. 0 = sem teto.
aiball-config-tickets-wait-credit-floor = Espera que uma etapa sempre recebe, mesmo sem crédito
    .about = Sem crédito, a espera de uma etapa é limitada ao saldo mas nunca abaixo disto, para que um agente sem crédito não volte em loop. Nunca leva o saldo abaixo de zero. Uma etapa pedindo 0 (seguir de imediato) é sempre concedida.
aiball-config-tickets-wait-credit-earn-resolved = Crédito de espera ganho por um ticket fechado resolvido, com commit
    .about = Ganho uma vez por ticket pelo agente cuja resolução foi aceita, quando citou um commit naquele ticket (commits: [...]) antes de fechá-lo.
aiball-config-tickets-wait-credit-earn-resolved-no-commit = Crédito de espera ganho por um ticket fechado resolvido, sem commit
    .about = Ganho uma vez por ticket pelo agente cuja resolução foi aceita quando não citou commit algum naquele ticket: uma resolução sem código vale menos.
aiball-config-tickets-wait-credit-earn-wontfix = Crédito de espera ganho por um ticket fechado wontfix
    .about = Ganho uma vez por ticket pelo agente cujo wontfix foi aceito.
aiball-config-tickets-wait-credit-earn-lines-per-minute = Linhas mudadas por minuto de crédito de espera vindo de um commit
    .about = Um commit que um agente cita numa resposta (commits: [...]) ganha um minuto por este tanto de linhas mudadas, lidas no checkout do agente. Uma vez por commit.
aiball-config-tickets-wait-credit-earn-commit-max = Máximo de crédito de espera que um commit ganha
    .about = O teto do que um único commit ganha, por maior que seja seu diff.
aiball-config-tickets-wait-credit-earn-commit-min = Mínimo de crédito de espera que um commit ganha
    .about = O que um commit citado com ao menos uma linha mudada ganha, por menor que seja seu diff: um conserto curto também é trabalho. 0 = só a taxa por linha conta.
aiball-config-tickets-backlog-claim-protect = Minutos em que a reivindicação de um agente a trabalho é protegida
    .about = Por quanto tempo uma reivindicação vale contra a de outro agente, contada da última ação do seu dono no ticket — trabalhar nela mantém a proteção viva. A reivindicação de outro agente dentro dessa janela é recusada; passado, o ticket pode ser tomado, e o thread registra. Uma atribuição sempre vence uma reivindicação. 0 = sem proteção.
aiball-config-tickets-backlog-rest = Quanto tempo um ticket descansa depois de um despertar de backlog
    .about = Depois que um despertar de backlog nomeou um ticket, quanto ele fica fora dos próximos despertares de backlog do agente enquanto ninguém mais se mexe nele. Um ticket bloqueado descansa `blocked_multiplier` vezes mais, um ticket cuja última ação é uma etapa só `after_step`. Um loop iniciado com CL_BACKLOG_COOLDOWN_SEC aplica aquele em vez deste. 0 = sem descanso.
aiball-config-tickets-backlog-depth = Até onde o backlog acorda um agente
    .about = O nível mais fundo de backlog que um despertar pode nomear para um agente, quando não tem evento a ler. Tickets críticos, hot e acionáveis sempre o acordam. followup: também um ticket onde alguém respondeu mas a decisão pendente do próprio agente o segura. waiting: também um ticket onde o agente falou por último e nada se mexeu desde. blocked: também um ticket seguro por uma dependência aberta (ele volta mesmo quando seu bloqueador fecha). Um ticket abaixo do nível fica no backlog, mostrado; só não acorda ninguém.
aiball-config-tickets-backlog-blocked-multiplier = Multiplicador do descanso de backlog para tickets bloqueados
    .about = Quanto mais um despertar de backlog mantém um ticket BLOCKED (seguro por um depends_on aberto) fora do pool de despertar, comparado a qualquer outro ticket. Ele precisa continuar aparecendo para não ser esquecido, mas nada se mexe nele entre dois despertares. 1 = mesmo descanso que os demais.
aiball-config-tickets-backlog-after-step = Descanso de backlog depois de uma etapa
    .about = Quanto tempo um despertar de backlog mantém um ticket fora do pool de despertar quando sua última ação é uma etapa (then: continue), em vez do descanso inteiro. Curto de propósito: a etapa diz que há trabalho agora, e a pausa só deixa a fila girar. 0 = nunca afundá-lo.
aiball-config-claude-loop-on-repetitive-denied-threshold = Recusas antes de o loop responder
    .about = Quantas chamadas de ferramenta o sistema de permissão do Claude Code precisa recusar a um agente na última hora antes de seu loop enviar o prompt abaixo (ou a saída do comando). Nada é enviado enquanto ambos estão vazios.
aiball-config-claude-loop-on-repetitive-denied-max-per-hour = Prompts enviados por hora, no máximo
    .about = No máximo este tanto de prompts por hora para recusas repetidas, para que um ciclo recusa, prompt, recusa pare. 0 = nunca enviar.
aiball-config-claude-loop-on-repetitive-denied-prompt = Prompt enviado em recusas repetidas
    .about = Digitado no prompt do Claude como está, como um despertar e só quando um despertar poderia ser (sem pausa AFK, zen, digitação, limite de uso, boot ou ocupado). Vazio = nada (o padrão).
aiball-config-claude-loop-on-repetitive-denied-command = Comando que escreve o prompt
    .about = Um comando de shell rodado na máquina do loop, na sua pasta, por 10 s no máximo. Recebe as recusas como JSON no stdin (agent, project, cwd, tool, reason, last_hour, recent); o que imprime é o prompt. Vence o prompt acima. Vazio = nenhum.
aiball-config-autopoll-volatile = Autopoll: lembretes de uma vez
    .about = true = notificar só quando um ping estritamente mais novo chega (sem lembretes por tempo). false (padrão) = lembrete persistente, redispara após throttle_seconds.
aiball-config-autopoll-throttle = Autopoll: cadência do lembrete
    .about = Cadência do lembrete em segundos, ignorada com volatile=true. 0 = a cada Stop (spam). Pings novos / tickets abertos novos furam a cadência.
aiball-config-autopoll-recent-tickets = Autopoll: títulos de tickets recentes a incluir
    .about = Até N títulos de tickets recentes não lidos no motivo do hook, para o agente saber o que espera antes de drenar. 0 = só a contagem.
aiball-config-autopoll-backlog = Autopoll: backlog como gatilho
    .about = true (padrão) = tickets abertos no escopo disparam notificações mesmo sem pings não lidos. false = só contexto (mostrado no motivo, nunca o gatilho).
aiball-config-autopoll-tone = Autopoll: tom
    .about = hint = polido, fácil de ignorar. directive (padrão) = nomeia a ação explicitamente. imperative = último recurso se o agente persiste em pedir padrões de permissão.
aiball-config-assign-window-sec = Janela de atribuição
    .about = Quanto tempo uma atribuição ou reivindicação vive antes de o ticket voltar à piscina comum.
aiball-config-hot-window-sec = Janela hot
    .about = Quão recente um movimento mantém um ticket hot na caixa de entrada.
aiball-config-upstream-transport = Upstream: transporte
    .about = Como as chamadas upstream (GitHub / GitLab) saem: gh (a credencial da CLI), http, ou auto (gh quando funciona). O transporte próprio de um acoplamento vence.
aiball-config-upstream-sync = Upstream: vigiar
    .about = pull (padrão) = um repo acoplado é vigiado e suas mudanças anunciadas; off = só vinculado. O sync próprio de um acoplamento vence.
