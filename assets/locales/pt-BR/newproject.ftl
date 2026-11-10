# A interface do tvty, em português do Brasil. Mesmos arquivos e mesmos
# ids que o inglês (assets/locales/en/), que é a referência; `tvty i18n`
# mede cada idioma.
# O assistente de projeto novo (src/shell/newproject.rs). O que o aiball
# diz ter feito aparece como o aiball diz.

newproject-title = Novo projeto
newproject-step-folder = Pasta
newproject-step-identity = Quem trabalha nele
newproject-step-advanced = Avançado
newproject-step-done = Configurado
newproject-step-next = E agora

## A pasta

newproject-which-folder = Qual pasta vira o projeto?
newproject-folder-placeholder = a pasta do projeto, ex. ~/dev/app
newproject-choose-folder = Escolher a pasta do projeto
newproject-browse = Navegar…
newproject-folder-empty = A pasta em que o agente vai trabalhar: a raiz do projeto.
newproject-folder-missing = Não existe tal pasta.
newproject-folder-file = Isto é um arquivo, não uma pasta.
newproject-folder-checking = Perguntando ao aiball…
newproject-folder-project = Já é um projeto do aiball: {$name} ({$file}). O Próximo mostra o que ele diz, para configurar de novo.
newproject-folder-configured = Já configurado para o aiball ({$file}). O Próximo mostra o que ele diz, para configurar de novo.
newproject-folder-git = Um repositório git: pronto.
newproject-folder-ready = Pronto (não é um repositório git).
newproject-open-running = Abrir {$label}, rodando
newproject-open = Abrir {$name}
newproject-next = Próximo →
newproject-back = ← Voltar
newproject-advanced = Configurações avançadas →
newproject-advanced-about = As outras configurações da pasta, como o aiball as descreve, e duas do projeto. Cada uma começa como a pasta diz, senão no padrão do aiball; nada é escrito antes de "Configurar".
newproject-prepare = prepará-lo de {$from}
newproject-prepare-about = Como o claude-config --fork do aiball: as instruções, skills, agentes e comandos vinculados aos de {$from}, para que suas atualizações alcancem ambos; settings.json e os plugins copiados; nunca a conta nem as sessões.
newproject-then-prepared = (preparado do que está em vigor)
newproject-prepared = preparado de {$from}: vinculados {$linked} · copiados {$copied}
newproject-prepared-nothing = preparado de {$from}: nada a levar para lá, o dir fica como o Claude o fará

## Quem trabalha nele

newproject-project-placeholder = projeto
newproject-agent-placeholder = agente
newproject-field-project = projeto
newproject-field-agent = agente
newproject-bad-project = O nome do projeto: letras, dígitos, -, _ e .
newproject-bad-agent = O nome do agente: letras, dígitos, -, _ e .
newproject-filled-from = Preenchido de {$file}: mude o que quiser, o que está aqui é o que será configurado.
newproject-crew = um agente de equipe
newproject-crew-about = Ao lado do líder do projeto, nos tickets que lhe derem; desligado: o líder.
newproject-host = seus loops no host do aiball
newproject-host-about = Onde os loops iniciados nesta pasta rodam; desligado: em {$mux}.
newproject-private = um projeto privado
newproject-private-about = O aiball o serve com seu kit privado (sem tickets públicos, sem seguidores).
newproject-noclaim = sem reivindicar
newproject-noclaim-about = O agente trabalha só nos tickets atribuídos a ele, nunca toma um do pool.
newproject-in = Em {$folder}, o aiball:
newproject-will-file = {$will} {$file}
newproject-will-created = cria
newproject-will-added = acrescenta sua entrada em
newproject-will-rewritten = reescreve sua entrada em
newproject-will-patched = atualiza
newproject-will-overwrote = sobrescreve
newproject-will-kept = mantém
newproject-joins = {$name} já está no board: esta pasta se junta a ele (outra pasta, ou um agente de equipe).
newproject-then-sets = então define {$what} no .aiball.yaml
newproject-files = .mcp.json: o servidor MCP do aiball para o Claude Code · .aiball.yaml: projeto, agente, papel, onde seus loops rodam.
newproject-set-up = Configurar
newproject-setting-up = Configurando…
newproject-unsaved = configurado, mas suas configurações não foram salvas: {$error}
newproject-set-in = definido {$what} em {$file}

## Configurado

newproject-done = {$name} está configurado. O aiball disse:
newproject-failed = o aiball não pôde configurar:
newproject-nothing-said = (nada dito)

## E agora

newproject-next-start = Iniciar o agente
newproject-next-start-host = "Iniciar sua primeira sessão" inicia o Claude Code de {$agent} no host do aiball, na pasta do projeto; seu terminal abre aqui, seus tickets ao lado.
newproject-next-start-mux = "Iniciar sua primeira sessão" inicia o Claude Code de {$agent} em {$mux}, na pasta do projeto; seu terminal abre aqui, seus tickets ao lado.
newproject-next-mcp = Aceitar o servidor MCP do aiball
newproject-next-mcp-about = No primeiro início nesta pasta, o Claude Code pergunta se usa o servidor MCP que o .mcp.json declara (aiball): aceite. Sem ele o agente não consegue ler o board nem responder seus tickets. Recusado sem querer? /mcp no Claude Code liga.
newproject-next-first-run = Rode o Claude uma vez no seu config dir
newproject-next-first-run-about = {$dir} não passou pela primeira execução do Claude (seu tema, seu login), que um loop não pode responder: rode CLAUDE_CONFIG_DIR={$path} claude uma vez num terminal, depois inicie a sessão.
newproject-next-skill = Instalar a skill do aiball
newproject-next-skill-about = O Claude Code ainda não tem skill do aiball nesta máquina: `aiball init skill`, uma vez, a instala — o agente então conhece os bons gestos do board.
newproject-next-work = Dê trabalho a ele
newproject-next-work-about = O "+ Novo" no painel de tickets registra um ticket em {$name}; o agente o pega no próximo despertar, e seus planos e perguntas voltam como notificações.
newproject-close = Fechar
newproject-start-first = Iniciar sua primeira sessão

newproject-step-mcp-created = {$path}: criado, com a entrada do aiball
newproject-step-mcp-added-others = {$path}: entrada do aiball acrescentada, os outros servidores mantidos
newproject-step-mcp-added = {$path}: entrada do aiball acrescentada
newproject-step-mcp-rewritten = {$path}: entrada do aiball escrita de novo na forma atual
newproject-step-mcp-kept = {$path}: entrada do aiball já lá, mantida (força a sobrescreve)
newproject-step-file-created = {$path}: criado ({$set})
newproject-step-file-overwrote = {$path}: sobrescrito ({$set})
newproject-step-file-kept = {$path}: já lá, mantido (força o sobrescreve)
newproject-step-consumer = {$path}: identidade definida ({$set})
newproject-step-type-kept = {$path}: tipo de projeto já {$value}
newproject-step-type = {$path}: tipo de projeto {$value}
newproject-step-type-was = {$path}: tipo de projeto {$value} (era {$previous})
newproject-step-deny = {$path}: ferramentas de código negadas ({$set})
