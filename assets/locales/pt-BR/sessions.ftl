# A interface do tvty, em português do Brasil. Mesmos arquivos e mesmos
# ids que o inglês (assets/locales/en/), que é a referência; `tvty i18n`
# mede cada idioma.
# A lista de sessões, a linha de uma sessão, as abas e a barra de título
# (src/shell.rs, src/shell/loopstabs.rs, src/shell/tabs.rs, src/status.rs).

## O cabeçalho da lista

sessions-tab-sessions = Sessões
sessions-tab-workspaces = Workspaces
sessions-order-recent = o projeto usado por último primeiro, como o ctrl+tab vai; um clique: alfabética
sessions-order-alpha = alfabética; um clique: a sua ordem (arraste um projeto)
sessions-order-yours = a sua ordem: arraste um projeto; um clique: o projeto usado por último primeiro
sessions-new-project = + projeto
sessions-new-project-tip = uma pasta virada um projeto do aiball (aiball init), depois sua primeira sessão
sessions-new-terminal-tip = um terminal próprio, que sobrevive ao tvty
sessions-filter = Filtrar…  ctrl+shift+f
sessions-fold = Recolher a lista de sessões
sessions-unfold = Estender a lista de sessões

## Seus grupos

sessions-group-live = ativas
sessions-group-idle = ociosas
sessions-group-shut = desligadas
sessions-group-on-hub = no hub
sessions-none = Nenhuma sessão
sessions-none-found = Nenhuma sessão encontrada
sessions-first-project = Configure seu primeiro projeto
sessions-none-stopped = Nenhum loop parado
sessions-none-stopped-found = Nenhum loop parado encontrado
sessions-none-shut = Nenhum agente sem loop
sessions-none-shut-found = Nenhum agente encontrado
sessions-none-hub = Nenhuma sessão no hub
sessions-none-hub-found = Nenhuma sessão encontrada no hub
sessions-no-project = Sem projeto

## O cabeçalho de um projeto

sessions-heading-tip = seus tickets no painel, nenhuma sessão aberta; arraste-o sobre outro para movê-lo
sessions-new-session = + sessão
sessions-project-options = suas configurações: onde seus loops rodam, Remote Control, a configuração do board
sessions-project-terminal = um terminal na pasta do projeto, listado com ele

## A linha de uma sessão

sessions-open = aberto no tvty: seu terminal roda aqui
sessions-looped = {$mux}: seu Claude roda em claude-loop, aberto por {$mux} (não no host do aiball)
sessions-attached = { $others ->
    [one] {$others} outro cliente anexado (o terminal do claude-loop, outro tvty), {$typing} com os controles
   *[other] {$others} outros clientes anexados (o terminal do claude-loop, outro tvty), {$typing} com os controles
}
sessions-attached-copy = : aberto aqui como cópia
sessions-host-shell = um terminal no host do aiball, sem Claude
sessions-update = seu Claude Code instalou uma atualização: reinicie-o da barra dele
sessions-update-pending = seu reinício foi pedido: o Claude dele reinicia assim que estiver ocioso (o distintivo está na barra dele)
sessions-starting = iniciando…
sessions-start = ▶ iniciar
sessions-astray = ⚠ pasta de {$other}
sessions-astray-tip = {$other} trabalha em {$cwd}: iniciado aqui, este agente retomaria sua conversa. Não iniciado.
sessions-stop-ask = parar?
sessions-stop-tip = parar o loop de {$who}: pelo aiball, ele continua reiniciável (sob os parados) — um segundo clique o para
sessions-stopped = parou {$who}
sessions-stop-failed-loop = parar {$who}
sessions-forget-ask = esquecer?
sessions-forget-tip = esquecer {$who}: o aiball não o lista mais; sua pasta, seu .aiball.yaml e os tickets do projeto ficam — um segundo clique esquece
sessions-forgot = esqueceu {$who}
sessions-forget-failed = esquecer {$who}

## O estado de uma sessão, numa linha

sessions-offline = offline
sessions-offline-tip = seu loop não está conectado ao aiball
sessions-working = trabalhando
sessions-starting-state = iniciando
sessions-idle = ocioso
sessions-booting = dando boot
sessions-claude-working = o Claude está trabalhando
sessions-claude-starting = o Claude está iniciando
sessions-claude-idle = o Claude está ocioso
sessions-held-typing-for-good = pausado até soltarem (um humano digita nele): o loop não o acorda
sessions-typing = um humano digita nele: o loop espera
sessions-held-for-good = pausado até soltarem: o loop não o acorda
sessions-held-while = pausado por um tempo: o loop não o acorda
sessions-loop-drives = o loop o conduz por conta própria
sessions-mark-held-for-good = pausado até soltarem
sessions-mark-held-while = pausado por um tempo
sessions-mark-own = por conta própria

## As marcas da lista recolhida

sessions-mark-tip = {$agent}: {$said}
sessions-mark-limit = {$agent}: {$said} — {$limit}

## "+ sessão"

sessions-form-cwd = diretório de trabalho
sessions-form-agent = agente
sessions-form-where = onde seu Claude trabalha
sessions-form-host-here = o host do aiball o inicia aqui
sessions-form-loop-here = o claude-loop inicia aqui
sessions-form-no-dir = não existe tal diretório
sessions-form-crew = um agente de equipe, ao lado do loop principal
sessions-form-on-host = no host do aiball, sem {$mux}
sessions-form-cancel = Cancelar
sessions-form-start = Iniciar
sessions-a-loop = um loop
sessions-astray-said = {$cwd} é a pasta de {$other}: {$agent} retomaria sua conversa
sessions-start-failed = iniciar
sessions-start-host-failed = iniciar no host
sessions-started-host = iniciado {$agent} no host do aiball em {$cwd}
sessions-started = iniciado {$agent} em {$cwd}
sessions-runs-already = {$agent} já roda: aberto como cópia

## Os distintivos: de um projeto, de um agente

sessions-badge-critical = o ticket crítico, #{$ticket}, está aqui: ele segura mais tickets abertos — um clique o abre
sessions-badge-decisions = { $count ->
    [one] {$count} ticket esperando sua decisão
   *[other] {$count} tickets esperando sua decisão
}
sessions-badge-unread = { $count ->
    [one] {$count} ticket com algo novo para você
   *[other] {$count} tickets com algo novo para você
}
sessions-light-backlog = { $count ->
    [one] backlog: {$count} ticket para ele olhar
   *[other] backlog: {$count} tickets para ele olhar
}
sessions-light-backlog-unknown = backlog: desconhecido até seu loop dizer
sessions-light-critical = ele mantém o ticket crítico: o que segura mais tickets abertos
sessions-light-events = eventos: {$count} não vistos ainda — pings, respostas, decisões esperando por ele
sessions-hub-mark = No hub do aiball, outra máquina: lido daqui (seu estado, seus tickets), não aberto — uma sessão é anexada da própria máquina dela.

## As abas

sessions-tab-name = seu nome
sessions-rename-failed = renomear o terminal
sessions-stop-failed = parar o terminal
sessions-rename-tip = renomeá-lo (ou um duplo clique, F2)
sessions-close-shell = fechar: para este terminal
sessions-close-tab = fechar a aba: o Claude dela segue
sessions-tab-shell-tip = um terminal no host do aiball, sem Claude; um duplo clique (ou F2) o renomeia
sessions-tab-tip = ctrl+pgup / ctrl+pgdn: a aba anterior, seguinte
sessions-new-tab-project = um terminal na pasta do projeto
sessions-new-tab-home = um terminal no diretório home

## A janela

sessions-pick = Escolha um terminal à esquerda · ctrl+shift+space mostra todos
sessions-on-hub = {$agent} — no hub
sessions-window-hub-project = {$name} — {$project} · {$agent} · no hub
sessions-window-hub = {$name} — {$agent} · no hub
sessions-bus-down = o bus do aiball caiu: o tvty reconecta; as listas podem estar atrasadas enquanto isso
sessions-bus-failing = assinar {$what} falhou: tentado de novo, lido inteiro; as listas podem estar atrasadas enquanto isso
sessions-menu = Menu: sobre, ajuda, reiniciar, sair
sessions-goto = Ir a um ticket: seu número ou o link #C. de um comentário, Enter · ctrl+shift+g
sessions-themes = Os temas de cores — o próximo
sessions-message-all = Uma mensagem a todos os agentes rodando: enviar, enviar e pausar, soltar as pausas
sessions-settings = Configurações
sessions-user = como quem o tvty age no board do aiball
sessions-hold-held = pausada: sua próxima partida fica pausada até você soltá-la; um clique a deixa rodar por conta própria
sessions-hold-free = por conta própria: sua próxima partida roda por conta própria; um clique a pausa até você soltá-la
sessions-hold-failed = pausar ou soltar {$agent}
sessions-play-again = Reiniciar {$agent}, na conversa dele
sessions-play-new = Iniciar o Claude Code em {$cwd}
