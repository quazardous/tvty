# A interface do tvty, em português do Brasil. Mesmos arquivos e mesmos
# ids que o inglês (assets/locales/en/), que é a referência; `tvty i18n`
# mede cada idioma.
# As dicas (tips/<id>.md: o inglês delas é o do arquivo, um teste
# confere). {$key} é a tecla do comando da dica, {$key-font-reset} a
# tecla de font.reset: como estão, nunca traduzidas. **negrito** e
# `código` ficam.

tip-slider = **{$key}** mostra seus projetos como pilhas, o mais recente primeiro. Solte Ctrl para abrir o escolhido.
tip-gallery = **{$key}** mostra todos os terminais ao vivo, lado a lado. Digite para filtrar, setas para mover, Enter para abrir.
tip-gallery-filter = Cada miniatura é ao vivo: veja todos os seus agentes trabalhando de uma vez e abra o que precisa de você.
tip-notice = **{$key}** pula para a notificação mais recente: o terminal do agente, com seu ticket ao lado.
tip-goto = **{$key}** vai a um ticket: digite o número, ou cole o link `#C.` de um comentário.
tip-full-list = **{$key}** mostra todos os tickets em tela cheia, o crítico no topo.
tip-bulk = **Ctrl+clique** seleciona tickets, **Shift+clique** um intervalo: depois feche, adie ou marque todos de uma vez.
tip-afk = **{$key}** avança o modo AFK do agente mostrado: por conta própria ▶, pausado 10 minutos ‖, pausado até você soltar ■.
tip-remote-control = O **RC** acende na barra do agente enquanto o Claude dele está em Remote Control: acompanhe e responda-o pelo claude.ai. `/rc` liga.
tip-move-loop = O chip **… · controles** da barra do agente (host, tmux ou psmux): um clique pega ou larga os controles, ou move o loop para o outro lado.
tip-config-dir = O ícone depois da pasta do agente nomeia o config dir do Claude Code dele (a conta dele): um clique troca para outro, a conversa vai junto.
tip-critical = Um **!** vermelho marca o ticket crítico: o aberto que segura mais tickets. Mover é o que libera mais trabalho.
tip-references = Todo **#N** num ticket é um link, até para um ticket de outro projeto: um clique abre.
tip-comment-link = Clique na marca **#C.** de um comentário para copiar o link, depois cole em qualquer ticket ou na caixa de ir para.
tip-reply = **Ctrl+Enter** envia sua resposta; **sem notificar** publica sem avisar ninguém.
tip-preview = **Escrever / Prévia**: veja seu Markdown renderizado, imagens incluídas, antes de registrar.
tip-escape = **Esc** sai do campo em que você digita primeiro; um segundo **Esc** fecha a página.
tip-sections = Arraste o título de uma seção para redimensioná-la; um duplo clique no título reparte o espaço de novo.
tip-fullscreen = **{$key}** põe a janela em tela cheia, e volta.
tip-filter = **{$key}** filtra as sessões: digite, Enter abre, Esc limpa.
tip-new-project = **Novo projeto…** no menu (o ícone do app, no topo esquerdo) prepara uma pasta para o aiball e inicia o agente dela.
tip-restart = **Reiniciar o tvty**, no menu, reinicia só a janela: as sessões dos seus agentes continuam rodando.
tip-shortcuts = Todo atalho pode mudar: aqui em **Atalhos de teclado**, ou no `keymap.toml` na pasta de configuração do tvty.
tip-opacity = **Opacidade do terminal** abaixo de 100 % deixa a área de trabalho aparecer através dos terminais.
tip-aiball-board = **aiball**, no menu, abre o board do aiball no navegador: os tickets de todos os projetos.
tip-font = **{$key}** aumenta a fonte dos terminais, **{$key-font-smaller}** diminui, **{$key-font-reset}** volta ao que era.
tip-new-ticket = **{$key}** registra um ticket novo de onde você estiver, no projeto do último que você registrou.
tip-room = **{$key}** recolhe o painel de tickets, **{$key-sidebar-toggle}** a lista de projetos: todo o espaço para o terminal.
tip-themes = **{$key}** percorre os temas de cores; o **◐** da barra de título lista todos.
tip-wizard = O assistente mostra o que vai escrever antes de escrever qualquer coisa: nada muda antes de você configurar.
tip-file-and-exit = **Registrar e sair** registra o ticket sem abri-lo: você volta ao terminal, ou à lista completa.
tip-panel-pin = O **pino** no cabeçalho do painel de tickets: fixado, o painel fica ao lado do terminal; solto, ele se sobrepõe e o terminal mantém a largura.
tip-followers = Os **seguidores** são avisados de cada movimento deste ticket: ✕ encerra um, o menu suspenso adiciona. Um seguidor silenciado fica cinza.
tip-thread-order = **⇅** vira o thread de cabeça para baixo: mais recente primeiro ou por último, no painel e em tela cheia cada um do seu jeito.
tip-held-selection = Selecionar texto num terminal **pausa a tela** enquanto você copia: a saída do agente espera, nada rola para fora debaixo do mouse.
tip-workspaces = Um **workspace** guarda um conjunto de projetos e como cada agente deles roda, por conta própria ▶ ou pausado ■. **+ workspace** guarda o que roda agora sob um nome.
tip-workspace-open = O **Abrir** de um workspace inicia o que ele guarda e solta o que roda por conta própria; **Desligar** para as sessões dele, as compartilhadas com outros workspaces ficam.
tip-project-order = Arraste um projeto da lista sobre outro para pô-los na sua própria ordem; **⇅** alterna entre recentes primeiro, alfabética e a sua.
tip-language = O código do idioma na barra de título faz o tvty falar outro idioma: inglês, francês, espanhol, alemão ou português do Brasil.
tip-stop-session = Sobre um agente rodando na lista, **⏹** para o loop dele: um primeiro clique pergunta, um segundo para. Ele continua lá, para começar de novo.
