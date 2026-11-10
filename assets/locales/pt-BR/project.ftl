# A interface do tvty, em português do Brasil. Mesmos arquivos e mesmos
# ids que o inglês (assets/locales/en/), que é a referência; `tvty i18n`
# mede cada idioma.
# O 📢 de um projeto (instrução permanente, foco de despertar, uma
# mensagem a todos os agentes) e suas opções (src/shell/megaphone.rs,
# src/shell/projectopts.rs).

## O 📢

megaphone-no-project = Uma instrução permanente e um foco de despertar são de um projeto: mostre um primeiro
megaphone-standing = Instrução permanente — {$project}
megaphone-standing-hint = Posta à cabeça de cada despertar dos seus agentes, evento e backlog igualmente. Deixe uma antes de se afastar; limpe-a quando voltar.
megaphone-prompt-placeholder = ex.: primeiro bugs leves, sem mudanças grandes
megaphone-focus = Foco de despertar
megaphone-focus-hint = Só estes tickets acordam os agentes do projeto, backlog e eventos. 123, 456 mantém só esses; !789 mantém todos menos ele. 123+ traz os filhos, 123++ todos os descendentes, +123 / ++123 os pais, 123~ seus tickets vinculados. Eventos fora ficam não lidos até você limpá-lo.
megaphone-tickets-placeholder = ex.: 2518, 2523++   ou   !2180
megaphone-until-placeholder = até (opcional): 2026-09-30 18:00
megaphone-not-a-date = até: {$typed} não é uma data (2026-09-30 18:00)
megaphone-not-a-time = até: {$typed} não é um horário daqui
megaphone-clear = Limpar
megaphone-save = Salvar
megaphone-said-nothing = {$project}: nada guia seus agentes agora
megaphone-said-prompt = {$project}: seus agentes leem “{$prompt}” a cada despertar
megaphone-said-focus = {$project}: só o seu foco acorda seus agentes
megaphone-tip-standing = Instrução permanente: {$prompt}
megaphone-tip-focus = Foco de despertar: {$focus}

## Uma mensagem a todos os agentes

megaphone-message = Mensagem a todos os agentes
megaphone-message-hint = Digitada em cada sessão de agente rodando agora, faça o que estiver fazendo. Enviar e pausar também pausa todos os loops (menos AFK ∞): nenhum despertar começa trabalho novo até você soltá-los. Vazio, o texto mostrado é enviado.
megaphone-no-loop = Nenhum loop de agente está rodando.
megaphone-running = {$count} rodando: {$names}
megaphone-release = Soltar as pausas
megaphone-send = Enviar
megaphone-send-hold = Enviar e pausar
megaphone-typed-into = digitado em {$names}
megaphone-queued-for = na fila para {$names}
megaphone-held = pausados {$names}
megaphone-released = soltos {$names}
megaphone-hold-not-applied = pausa não aplicada

## As opções de um projeto

projectopts-global = Global
projectopts-a-project = um projeto…
projectopts-failed = configurações do projeto
projectopts-choose = Escolha um projeto acima.
projectopts-no-folder = o aiball não conhece pasta deste projeto nesta máquina: nenhum agente dele trabalha aqui.
projectopts-asking = Perguntando ao aiball…
projectopts-could-not-say = o aiball não pôde dizer: {$error}
projectopts-no-file = {$folder} não tem .aiball.yaml, nem pasta alguma acima: valem os padrões do aiball. Novo projeto… configura.
projectopts-written-in = Escrito em {$file}.
projectopts-also-serves = Ele também serve {$others}: uma mudança aqui também é deles.
projectopts-on-named = em: {$name}
projectopts-none-set = {$project} não define nenhuma configuração do board: ele tem os valores do board.
projectopts-all-keys = Todas as suas chaves
projectopts-restart-needed = { $count ->
    [one] {$loops} ainda roda no valor antigo: assume o novo assim que reiniciar, numa conversa nova.
   *[other] {$loops} ainda rodam no valor antigo: assumem o novo assim que reiniciarem, numa conversa nova.
}
projectopts-restart = { $count ->
    [one] Reiniciá-lo
   *[other] Reiniciá-los
}
projectopts-restarting = Reiniciando…
projectopts-restarted = loops reiniciados
projectopts-restart-failed = reiniciar

# O projeto renomeado (o Nome das suas opções)
rename-placeholder = novo nome
rename-about = Renomeia {$project} em toda parte de uma vez: seu nome no aiball (seus tickets acompanham), seu .aiball.yaml, e sua pasta, a menos que ela fique. Seu agente mantém o nome. O tvty pergunta ao aiball primeiro o que ele faria.
rename-move-folder = Mover a pasta também
rename-check = Conferir
rename-checking = Perguntando ao aiball…
rename-renaming = Renomeando…
rename-folder-moves = A pasta muda: {$from} → {$to}
rename-folder-stays = A pasta fica onde está.
rename-stop = Pará-lo
rename-apply = Renomear para {$new}
rename-done = Projeto {$old} renomeado {$new}
rename-failed = Renomear {$project}
rename-stop-failed = Parar {$name}
rename-loop-unknown = o tvty não conhece este loop
rename-no-root = o aiball não conhece pasta deste projeto nesta máquina.
rename-which-root = o aiball conhece este projeto em várias pastas: qual?
rename-choose-root = A pasta a renomear:
