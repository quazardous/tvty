# A interface do tvty, em português do Brasil. Mesmos arquivos e mesmos
# ids que o inglês (assets/locales/en/), que é a referência; `tvty i18n`
# mede cada idioma.
# Atalhos de teclado (src/keymap.rs, src/options.rs, src/shell.rs). As
# palavras de um comando vão pelo nome dele: notice.open é
# keys-notice-open.

## O que cada comando faz

keys-notice-open = Ir à notificação mais recente: o terminal do agente, com seu ticket
keys-slider-next = Slider: os grupos como pilhas, o mais recente primeiro; solte Ctrl para abrir
keys-slider-back = Slider, para trás
keys-tab-next = A aba seguinte, no grupo mostrado
keys-tab-back = A aba anterior, no grupo mostrado
keys-gallery-toggle = Galeria de todos os terminais; digite para filtrar, setas movem, Enter abre
keys-panel-toggle = Recolher ou estender o painel de tickets
keys-sidebar-toggle = Recolher ou estender a lista de projetos
keys-afk-cycle = O modo AFK do agente mostrado: auto → pausar 10 min → pausar, em vigor 3 s após a última tecla (um terminal sem agente recebe F9)
keys-sessions-filter = Filtrar as sessões: digite, Enter abre, setas movem, Esc limpa
keys-ticket-goto = Ir a um ticket: o número dele ou o link #C. de um comentário, Enter abre (o #… da barra de título)
keys-options-toggle = Configurações
keys-app-quit = Sair do tvty, como fechar a janela faz (loops rodando: parados ou mantidos, como escolhido)
keys-help-menu = Ajuda: sobre o tvty, sua documentação, novidades, um reinício
keys-window-fullscreen = A janela em tela cheia, ou volta
keys-font-bigger = Fonte do terminal maior
keys-font-smaller = Fonte do terminal menor
keys-font-reset = Fonte do terminal de volta ao padrão
keys-ticket-new = Um ticket novo (também + no painel e na lista completa)
keys-project-megaphone = O 📢 do projeto mostrado: sua instrução permanente e seu foco de despertar
keys-list-full = A lista de tickets, em tela cheia
keys-theme-next = Próximo tema de cores
keys-debug-inspector = O inspetor do GPUI: escolha um elemento, veja seu id e onde ele é criado (builds de debug)
keys-terminal-copy = Copiar a seleção, as linhas cortadas pela tela reunidas
keys-terminal-copy-raw = Copiar a seleção exatamente como na tela
keys-terminal-paste = Colar a área de transferência
keys-terminal-tab = Tab, para o programa (não o foco no próximo elemento)
keys-terminal-back-tab = Shift+Tab, para o programa

## As teclas que não são comandos: o que fazem, as teclas como .keys

keys-fixed-esc = Fecha a galeria, o slider, a lista de temas, as configurações, a lista completa
    .keys = Esc
keys-fixed-arrows = No slider e na galeria: mover ao cartão visto ali
    .keys = Setas
keys-fixed-select = Num terminal: selecionar texto · uma palavra · uma linha — copiado à seleção primária
    .keys = Arrastar · duplo clique · triplo clique
keys-fixed-middle-click = Num terminal: colar a seleção primária
    .keys = Clique do meio
keys-fixed-right-click = Num terminal: Copiar, Colar — num link, Abrir link, Copiar link
    .keys = Clique direito
keys-fixed-ctrl-click = Num terminal, num link: abri-lo
    .keys = Ctrl+clique
keys-fixed-wheel = Rolar o histórico
    .keys = Roda do mouse
## A página

keys-intro = Cada atalho é um comando, ligado num contexto: Terminal quando um terminal tem o foco, Janela em qualquer outro lugar. O atalho mais profundo vence; uma tecla sem atalho vai ao programa do terminal.
keys-how = Clique numa tecla para mudá-la, + para acrescentar, × para retirar. Guardado em {$file}, que você também pode editar.
keys-reset-all = Restaurar tudo
keys-not-applied = Não aplicado: {$error}
keys-context-window = Janela — em qualquer lugar do tvty, páginas em tela cheia incluídas
keys-context-workspace = Workspace — os terminais, a lista de sessões e o painel ao lado
keys-context-terminal = Terminal — quando um terminal tem o foco
keys-press = Pressione uma tecla… (Esc desiste)
keys-typing-refused = {$key} é digitação: num terminal continua do programa
keys-remove = retirar esta tecla
keys-the-program = o programa
keys-masked = {$what} — mascareada num terminal por {$by}
keys-no-key = sem tecla
keys-conflict = {$key} executa {$other}: tomá-la para {$name}?
keys-replace = Substituir
keys-cancel = Cancelar
keys-freed-terminal = Devolvida ao programa do terminal
keys-freed-focus = Devolvida ao que tem o foco
