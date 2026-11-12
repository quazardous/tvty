# A interface do tvty, em português do Brasil. Mesmos arquivos e mesmos
# ids que o inglês (assets/locales/en/), que é a referência; `tvty i18n`
# mede cada idioma.
# O que resta: o menu do terminal e suas marcas, o visualizador de
# imagens, alguns avisos e erros que o usuário lê (src/shell.rs,
# src/terminal.rs, src/aiball.rs, src/images.rs, src/shell/viewer.rs,
# src/loops.rs, src/daemon.rs).

## Avisos

misc-aiball-started = o aiball não estava rodando: iniciado
misc-aiball-silent = o aiball não responde em {$at}
misc-service-failed = o serviço do aiball não iniciou: {$error}
misc-no-service = {$why}; não há serviço do aiball para iniciar
misc-filed = registrado — {$title}
misc-filed-moderate = {$title} — um ticket novo para moderar
misc-filed-other = {$title} — um ticket novo
misc-something-new = algo novo
misc-held = {$said}: seu loop está pausado até você soltá-lo
misc-go-to = ir para
misc-go-to-what = ir para {$what}
misc-not-a-ticket = {$typed} não é um ticket: um número, ou o link #C. de um comentário
misc-copied = Copiado para a área de transferência
misc-new-terminal = novo terminal
misc-no-folder = nenhuma pasta conhecida para {$project}
misc-not-a-directory = {$folder} não é um diretório
misc-not-idle = o Claude dele está trabalhando: assim que estiver ocioso
misc-stop-not-received = {$agent}: nenhum loop dele recebeu a parada (não roda, ou não onde o aiball o alcança)

## O terminal

misc-open-link = Abrir link
misc-copy-link = Copiar link
misc-copy = Copiar
misc-paste = Colar
misc-frozen = pausado · seleção
misc-frozen-tip = A tela está pausada enquanto um texto está selecionado; a sessão continua. Um clique aqui, Esc ou uma tecla a solta.
misc-size-taken = tamanho tomado por outro cliente · clique para tomá-lo de volta
misc-session-ended = A sessão terminou.
misc-hub-session = Esta sessão roda no hub do aiball, outra máquina: não pode ser aberta desta. Seus tickets estão no painel.
misc-all-terminals = Todos os terminais
misc-gallery-hint = digite para filtrar · setas movem · enter abre · esc fecha

## O modelo do agente, seus limites, suas recusas

misc-price = {$input} / {$output} por M de tokens
misc-model-out = {$name} saiu
misc-prices-from = preços de {$catalog}
misc-limit = limite de uso atingido
misc-limit-resets = limite de uso atingido · volta {$resets}
misc-denials = { $count ->
    [one] {$count} chamada de ferramenta recusada pelo Claude Code na última hora{$ago}{$why} — um agente recusado para aí
   *[other] {$count} chamadas de ferramenta recusadas pelo Claude Code na última hora{$ago}{$why} — um agente recusado para aí
}
misc-denials-last = , a última {$ago} atrás

## Imagens

misc-image-too-large = imagem grande demais para mostrar aqui
misc-image-unavailable = imagem indisponível
misc-fitted = {" "}(ajustada)
misc-viewer-hint = roda ou + − dá zoom · 1 tamanho real · 0 caber · arraste para mover · Esc

## A seta de uso na barra do topo

usage-ratio = ×{$ratio}
usage-points = {$points} pts
usage-wall = limite {$left}
usage-no-wall = sem limite
usage-window-five_hour = janela de 5 horas
usage-window-seven_day = Semana
usage-tip-window = {$window}: {$used} % usados, {$expected} % num ritmo constante · volta {$resets} · {$end}
usage-tip-wall = no ritmo da última hora, a cota acaba em {$left}
usage-tip-lasts = no ritmo da última hora, dura até a renovação
usage-trend = velocidade ×{$speed}
usage-tip-trend = velocidade recente: ×{$recent} o ritmo constante em 15 min, ×{$hour} na hora
usage-tip-read = lido {$ago} atrás
usage-tip-click = Um clique: a diferença dita de outro jeito.
usage-drawing-wall = Limite
usage-drawing-points = Diferença
usage-drawing-trend = Velocidade
