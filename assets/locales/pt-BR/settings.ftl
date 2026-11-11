# A interface do tvty, em português do Brasil. Mesmos arquivos e mesmos
# ids que o inglês (assets/locales/en/), que é a referência; `tvty i18n`
# mede cada idioma.
# Configurações (src/settings.rs, src/options.rs, as páginas de opções de
# src/shell.rs). As palavras de uma configuração vão pela sua chave:
# appearance.terminal_font_size é setting-appearance-terminal_font_size,
# sua descrição .about, os dois estados de uma chave .on e .off.

## As configurações

setting-appearance-language = Idioma
    .about = O idioma da interface. O que as pessoas escrevem (tickets, comentários, nomes) fica como escrito.
setting-appearance-terminal_font_size = Fonte do terminal
    .about = O texto dos terminais. Ctrl+Shift+= / Ctrl+Shift+− / Ctrl+Shift+0 também.
setting-appearance-window_font_size = Texto da janela
    .about = Tudo ao redor dos terminais — listas, tickets, menus.
setting-notifications-max = Mostradas no máximo
    .about = No canto superior direito do terminal (inferior esquerdo sobre uma tela cheia), a mais recente no canto; as mais velhas abrem espaço.
setting-notifications-seconds = Segundos mostradas
    .about = Depois ela vai, a não ser que o ponteiro esteja sobre ela.
setting-notifications-own = Seus próprios gestos
    .about = Um ticket fechado, uma resposta publicada, um plano aceito: ditos assim que o aiball os tem. Uma recusa é sempre dita.
    .on = mostrados
    .off = ocultos
setting-notifications-copied = Copiado
    .about = Um breve "Copiado para a área de transferência" embaixo quando um texto vai para a área de transferência (não para uma mera seleção).
    .on = mostrado
    .off = oculto
setting-terminal-cursor = Cursor
    .about = No terminal que tem as teclas: piscando, fixo, ou como o programa pede. Fica aceso enquanto você digita e enquanto vem texto.
setting-terminal-scrollback = Linhas de histórico
    .about = O que cada terminal guarda para rolar de volta. Mais custa memória: uns 4 KB por linha a 160 colunas, então 5000 linhas dá uns 19 MB por terminal no máximo. O histórico de uma sessão tmux é do próprio tmux (seu history-limit).
setting-terminal-copy_unfreezes = Copiar
    .about = Uma seleção pausa a tela do terminal; uma cópia (Ctrl+Shift+C, o Copiar do menu) a solta e mostra a tela ao vivo de novo. Desligado, a tela fica pausada até uma colagem, Esc ou uma tecla digitada.
    .on = solta a tela
    .off = mantém pausada
setting-terminal-copy_joins = Copiar linhas cortadas pela tela
    .about = O Claude Code corta suas linhas longas à largura do terminal. Uma cópia (Ctrl+Shift+C, a seleção do clique do meio) as reúne: uma linha continua na seguinte quando a primeira palavra da seguinte não caberia, com a mesma indentação. É um palpite: Ctrl+Shift+Alt+C copia exatamente o que a tela mostra.
    .on = reunidas
    .off = como na tela
setting-terminal-paste_unfreezes = Colar
    .about = Uma seleção pausa a tela do terminal; uma colagem (Ctrl+V, Ctrl+Shift+V, o clique do meio) a solta e mostra a tela ao vivo de novo. Desligado, a tela fica pausada até Esc ou uma tecla digitada.
    .on = solta a tela
    .off = mantém pausada
setting-terminal-paste_scrolls_down = Colar, com o histórico aberto
    .about = Uma colagem num terminal rolado para trás no histórico o desce até o fundo, onde o texto entra. Desligado, a vista fica onde está.
    .on = desce ao fundo
    .off = fica onde está
setting-scroll-speed = Velocidade da roda
    .about = Multiplica a do próprio tvty: em 1, um entalhe dá umas três linhas de ticket, cinco linhas do histórico de um terminal.
setting-mouse-focus = Foco
    .about = O que dá o teclado ao terminal ou a uma caixa de escrever: um clique, ou o ponteiro passando sobre ele (como o foco segue o mouse de um gerenciador de janelas). Por padrão, como o sistema faz.
setting-appearance-theme = Janela
    .about = O tema de cores da janela. Ctrl+Shift+K percorre os temas.
setting-appearance-terminal_theme = Terminal
    .about = O dos próprios terminais, ou o da janela — um terminal escuro numa janela clara.
setting-appearance-terminal_opacity = Opacidade do terminal
    .about = Abaixo de 100 %, a área de trabalho aparece através do fundo dos terminais, ao vivo. As cores que um programa define ficam opacas.
setting-appearance-panel_opacity = Opacidade do painel
    .about = Abaixo de 100 %, a área de trabalho aparece através da barra de título, das abas, dos lados recolhidos e do painel de tickets ao lado do terminal, ao vivo. O que abre sobre um terminal (a lista de sessões, menus, diálogos) fica opaco, para seu texto continuar legível.
setting-appearance-terminal_blur = Desfoque atrás
    .about = Atrás do que é transparente, a área de trabalho desfocada. Só onde o compositor oferece (KDE); em outros lugares aparece nítida.
    .on = desfocado
    .off = nítido
setting-updates-check = Verificar atualizações
    .about = No início e uma vez por dia, pergunta ao GitHub se um Terminal Velocity mais novo saiu, e o diz uma vez (nada mais é enviado). O atualizador (as Atualizações… do menu) o instala.
    .on = verificar
    .off = nunca
setting-usage-display = Seta de uso
    .about = Na barra do topo, a assinatura contra o ritmo que a esgotaria bem no fim da janela: uma seta vermelha para cima acima dele, uma verde para baixo abaixo, mais funda quanto maior a diferença. Sua figura: usado contra o esperado (×1,3), a diferença em pontos da cota (+12 pts), ou quando a cota acaba neste ritmo. Um clique na seta vai à próxima; Auto recomeça na última mostrada. Mostrada assim que um loop lê o uso do Claude Code (planos Pro e Max).
setting-tips-show = Mostrar dicas
    .about = "Você sabia?": uma dica curta uma vez após o início, e na primeira vez que uma página abre; nunca uma para o que você já usa. As Dicas… do menu mostram todas.
    .on = mostrar
    .off = nunca
setting-sessions-order = Ordem
    .about = Os projetos na lista, no slider e na galeria: o usado por último primeiro, como o ctrl+tab vai; alfabética; ou a sua, como você os arrasta na lista. Também o ⇅ no cabeçalho da lista.
setting-sessions-show_hub = As sessões do hub
    .about = Quando esta máquina alcança o aiball por um nó proxy: listar também as sessões que rodam no hub do aiball, à parte, sob "no hub". São lidas daqui (estado, tickets), nunca abertas: uma sessão é anexada da própria máquina dela. As sessões de outros nós nunca são listadas.
    .on = listadas
    .off = não listadas
setting-sessions-on_quit = Ao sair
    .about = Quando o tvty sai com loops do Claude Code desta máquina rodando: perguntar, pará-los (pelo aiball: continuam reiniciáveis), ou mantê-los rodando.
setting-sessions-on_start = Ao iniciar
    .about = Os loops que o tvty parou ao sair: perguntar, reiniciá-los como estavam (retomando suas conversas, um pausado pausado de novo), reiniciá-los do zero (arrancando, depois por conta própria), ou deixá-los parados.
setting-sessions-shells_on_start = Terminais ao iniciar
    .about = Os terminais abertos no tvty que não são do Claude (shells), dados como perdidos no início (o aiball ou a máquina reiniciou): perguntar, junto com os loops; reabri-los onde estavam (mesmo nome, pasta e lugar); ou deixá-los.
setting-tickets-panel_overlay = Lugar
    .about = Ao lado do terminal, que então fica mais estreito — ou sobre ele, como a lista de sessões à esquerda: o terminal mantém a largura toda e o painel cobre seu lado direito enquanto aberto.
    .on = sobre o terminal
    .off = ao lado do terminal
setting-tickets-newest_first = Mais recente primeiro, em tela cheia
    .about = Um ticket em tela cheia mostra a palavra mais nova do seu thread primeiro (também ⇅ no cabeçalho do ticket).
    .on = mais recente primeiro
    .off = mais recente por último
setting-tickets-panel_newest_first = Mais recente primeiro, no painel
    .about = Um ticket no painel mostra a palavra mais nova do seu thread primeiro; desligado, por último, junto à resposta (também ⇅ ali). O painel tem ordem própria: a da tela cheia não é a dele.
    .on = mais recente primeiro
    .off = mais recente por último
setting-tickets-summary_open = Onde está, aberto
    .about = No painel de tickets, o "Onde está" de um ticket mostra seu texto de uma vez; desligado, fica dobrado no título, a um clique. Em tela cheia sempre mostra.
    .on = aberto
    .off = dobrado
setting-tickets-ctrl_enter_opens = Ctrl+Enter
    .about = Num ticket novo, o Ctrl+Enter registra e volta para onde você estava ("Registrar e sair"), ou registra e abre ("Registrar o ticket"). Os botões fazem cada um deles, diga isto o que disser.
    .on = registra e abre
    .off = registra e sai

## As páginas e os grupos, (settings-title-<name>)

settings-title-project = Projeto
settings-title-appearance = Aparência
settings-title-layout = Layout
settings-title-ticket-list = Lista de tickets
settings-title-keyboard-shortcuts = Atalhos de teclado
settings-title-aiball = aiball
settings-title-about = Sobre
settings-title-language = Idioma
settings-title-sizes = Tamanhos
settings-title-notifications = Notificações
settings-title-mouse = Mouse
settings-title-colours = Cores
settings-title-updates = Atualizações
settings-title-tips = Dicas
settings-title-usage = Uso
settings-title-sessions = Sessões
settings-title-ticket-panel = Painel de tickets
settings-title-thread = Thread
settings-title-new-ticket = Ticket novo
settings-title-sides = Lados
settings-title-legend = Legenda
settings-title-window = Janela
settings-title-workspace = Workspace
settings-title-terminal = Terminal
settings-title-fixed-keys = Teclas fixas
settings-title-name = Nome
settings-title-folder = Pasta
settings-title-board = Board
settings-title-board-wide-only = Só do board inteiro
settings-title-per-project-only = Só por projeto
settings-title-in-aiball-yaml = No .aiball.yaml

## A página

settings-heading = Configurações
settings-search-placeholder = Buscar…
settings-modified-hint = @modified: o que você mudou
settings-search = Buscar: {$text}
settings-close = ✕  Esc
settings-nothing-found = Nada encontrado. Busque um nome, uma palavra que ela diz, uma tecla (ctrl+shift+b), um valor — ou @modified.
settings-default = padrão
settings-default-back = Padrão
settings-back-to = voltar a {$value}
settings-own-themes = Seus próprios temas (o formato de tema do gpui-component) vão em ~/.config/tvty/themes/: aparecem aqui na próxima vez que esta página abrir.

## Escolhas

setting-language-auto = Automático: {$lang}
settings-ask = Perguntar
settings-order-recent = Mais recente primeiro
settings-cursor-blink = Piscando
settings-cursor-steady = Fixo
settings-cursor-program = Como o programa pede
settings-usage-auto = Auto: como no último clique
settings-usage-ratio = Usado contra o esperado (×1,3)
settings-usage-points = Diferença em pontos (+12 pts)
settings-usage-wall = Tempo até o limite (limite 1h40)
settings-usage-trend = Velocidade recente (velocidade ×1,8)
settings-order-alpha = Alfabética
settings-order-yours = A sua (arrastada na lista)
settings-quit-stop = Pará-los
settings-quit-keep = Mantê-los rodando
settings-start-restart = Reiniciá-los como estavam
settings-start-fresh = Reiniciá-los do zero
settings-start-leave = Deixá-los parados
settings-shells-reopen = Reabri-los
settings-shells-no = Deixá-los
settings-focus-click = Clique
settings-focus-hover = Ponteiro sobre
settings-same-as-window = Igual à janela
settings-dark = Escuro
settings-light = Claro
settings-theme-dark = ESCURO
settings-theme-light = CLARO
settings-theme-window = Janela
settings-theme-terminal = Terminal
settings-theme-opacity = Opacidade do terminal
settings-theme-panel-opacity = Opacidade do painel

## Lados

settings-side-open = aberto
settings-side-folded = recolhido
settings-side-said = {$state} · {$width} px
settings-side-list = Lista de projetos
settings-side-list-about = Sobre o terminal, à esquerda. Arraste sua borda para redimensioná-la; sua alça a recolhe.
settings-side-panel = Painel de tickets
settings-side-panel-about = À direita do terminal. Arraste sua borda para redimensioná-lo; sua alça o recolhe.
settings-widths = Larguras
settings-widths-about = De volta aos padrões: uma lista de 290 px, um painel um terço da janela.
settings-reset = Restaurar

## Sobre

settings-about-version = Versão
settings-about-aiball-at = aiball em
settings-about-acting-as = Agindo como
settings-about-bus = bus do aiball
settings-about-bus-said = versão {$version}, como {$who}
settings-about-bus-said-kind = versão {$version}, como {$who} ({$kind})
settings-about-not-connected = não conectado
settings-about-live = Board ao vivo
settings-about-not-subscribed = não inscrito
settings-about-subscriptions = { $count ->
    [0] nenhuma inscrição no bus
    [one] {$count} inscrição no bus
   *[other] {$count} inscrições no bus
}
settings-about-theme = Tema
settings-about-terminal-theme = Tema do terminal
settings-about-the-windows = o da janela
settings-about-config = Configurações, atalhos, temas
settings-about-state = Layout e workspace
settings-about-fonts = Fontes
settings-about-what = Um terminal nativo para trabalhar com muitos agentes de código com IA de uma vez: seus terminais agrupados por projeto, os tickets de cada projeto ao lado — o agente pergunta, você decide, ele segue. Construído sobre o aiball, que roda os loops dos agentes e seu board.
settings-about-github = GitHub ↗
settings-about-aiball = aiball no GitHub ↗
settings-about-license = Licença MIT ↗
settings-about-footer = Terminal Velocity (tvty). Licença MIT. Temas embutidos: veja themes/README.md.

## A configuração do aiball

settings-aiball-failed = configuração do aiball
settings-aiball-unreadable = a configuração do aiball não pôde ser lida: {$error}. Escolha o escopo de novo, no topo esquerdo, para tentar outra vez.
settings-aiball-reading = Lendo a configuração do aiball…
settings-aiball-board = A configuração do próprio board: o que todo projeto recebe, a não ser que diga o contrário. Escolha um projeto, no topo esquerdo, para a dele.
settings-aiball-project = O que {$project} diz sobre a configuração do board; o ↺ devolve uma chave ao valor do board.
settings-aiball-in-file = O aiball lê estas do .aiball.yaml de cada projeto, não do seu repositório de configuração: mude-as naquele arquivo.
settings-aiball-global-only = O aiball declara estas para o board inteiro: um valor para todo projeto, definido em Global.
settings-aiball-project-only = O aiball declara estas só por projeto: nenhum valor para o board inteiro; escolha um projeto acima para defini-las.
settings-aiball-open-global = Abrir Global
settings-aiball-protected = 🔒 protegida
settings-aiball-set-in-file = definida no .aiball.yaml
settings-aiball-board-wide = do board inteiro: definida em Global
settings-aiball-per-project = por projeto
settings-aiball-from-board = {" "}Da configuração do board.
settings-aiball-board-back = Board
settings-on = ligado
settings-off = desligado

## A legenda da lista de tickets

settings-legend-intro = A lista de tickets como o aiball a computa para você: a faixa, a vez de quem e o glifo de estado são do aiball; a listra é do tvty.
settings-legend-order = Ordem
settings-legend-band-moderate = o ticket, ou comentários nele, esperam moderação
settings-legend-band-decide = um plano, uma resolução, um wontfix ou um escalonamento esperam sua decisão
settings-legend-band-working = um agente o mantém ou está numa etapa
settings-legend-band-open = aberto, nada urgente
settings-legend-band = {$what}; o mais recente primeiro
settings-legend-glyph = Glifo de estado — colorido quando espera por você, apagado de outro modo
settings-legend-always-blue = {$what} — sempre azul
settings-legend-always-amber = {$what} — sempre âmbar
settings-legend-always-red = {$what} — sempre vermelho
settings-legend-stripe = Listra — vez de quem
settings-legend-stripe-solid = uma decisão espera por você, e é a última mensagem
settings-legend-stripe-dashed = uma decisão espera por você, mas a conversa seguiu depois dela
settings-legend-stripe-yours = sua vez: um agente respondeu você
settings-legend-stripe-waiting = fina e pontilhada: sua palavra é a última, você espera
settings-legend-stripe-none = sem listra: a bola está com o agente
settings-legend-stripe-tape = fita de obra: o ticket espera moderação, nada segue até você deixar passar
settings-legend-rest = O resto
settings-legend-unread = comentários, azul: algo novo nele para você, não lido (seu título em negrito também)
settings-legend-others-spoke = vivo, com a ponta à esquerda: outra pessoa falou por último
settings-legend-you-spoke = discreto, com a ponta à direita: você falou por último
settings-legend-pending = comentários esperando moderação
settings-legend-agent = agente
settings-legend-holder = o agente que o mantém; a chama quando esteve ativo ultimamente
settings-legend-critical = o ticket crítico do projeto: ele segura 3 tickets abertos
settings-legend-priority = prioridade: urgente, alta, baixa (normal não mostra nada)
