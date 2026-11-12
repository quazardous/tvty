# A interface do tvty, em português do Brasil. Mesmos arquivos e mesmos
# ids que o inglês (assets/locales/en/), que é a referência; `tvty i18n`
# mede cada idioma.
# O que é um ticket, como campos: compartilhado pelo painel de tickets e
# pelo ticket novo (src/ui/fields.rs, src/composer.rs, src/ui/ticket_text.rs).

## Os valores do aiball, ditos (o aiball continua com os seus)

fields-intent-request = pedido
fields-intent-question = pergunta
fields-intent-fyi = informativo
fields-intent-feature = funcionalidade
fields-intent-panic = pânico
fields-priority-urgent = urgente
fields-priority-high = alta
fields-priority-normal = normal
fields-priority-low = baixa
fields-level-task = tarefa
fields-level-milestone = marco
fields-level-roadmap = roadmap
fields-scope-internal = interno
fields-scope-default = padrão
fields-scope-broadcast = broadcast

## Sob o escopo

fields-scope-note-internal = não notifica ninguém além de quem é mencionado
fields-scope-note-broadcast = notifica também os seguidores do projeto
fields-scope-note-default = notifica os inscritos no ticket e os donos do projeto

## Escrevendo

fields-take-out = retirar
fields-write = Escrever
fields-preview = Prévia
fields-title = Título
fields-body = Sobre o que é… (@ menciona, ctrl+v cola uma imagem)

## De onde vem uma configuração

fields-from-file = do .aiball.yaml
fields-from-global = da configuração global do aiball
fields-from-mcp = do .mcp.json
fields-from-env = do ambiente do aiball
fields-from-default = padrão
