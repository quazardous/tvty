# A interface do tvty, em português do Brasil. Mesmos arquivos e mesmos
# ids que o inglês (assets/locales/en/), que é a referência; `tvty i18n`
# mede cada idioma.
# Workspaces, e a lista que escolhe sessões (sair, começar de novo, um
# workspace guardado, aberto ou desligado) (src/shell/workspaces.rs,
# src/workspaces.rs).

## A lista de workspaces

workspaces-new = + workspace
workspaces-new-tip = guarda os grupos que rodam agora, escolhidos numa lista, sob um nome
workspaces-none = Nenhum workspace ainda: um workspace guarda uns grupos, e como cada sessão deles roda.
workspaces-default-name = Workspace
workspaces-its-name = seu nome
workspaces-act-open = Abrir
workspaces-act-open-tip = inicia o que está parado e põe cada sessão como mantida — pergunta primeiro
workspaces-act-shut = Desligar
workspaces-act-shut-tip = para suas sessões — pergunta primeiro, quais; um grupo que outro workspace também tem fica
workspaces-act-save = salvar
workspaces-act-save-tip = guarda-o de novo como as coisas estão, ou acrescenta um grupo — escolhido numa lista
workspaces-again-tip = o grupo mostrado agora, guardado de novo como roda agora
workspaces-add-tip = acrescenta o grupo mostrado agora, suas sessões como rodam
workspaces-act-rename = renomear
workspaces-act-rename-tip = outro nome; Enter confirma
workspaces-act-delete = excluir
workspaces-act-delete-sure = excluir: certeza?
workspaces-act-delete-tip = esquece o workspace; suas sessões não são tocadas
workspaces-no-project = sem projeto
workspaces-drop-tip = tira este grupo do workspace (suas sessões não são tocadas)
workspaces-now-stopped = parado
workspaces-now-runs = roda
workspaces-now-held = pausado
workspaces-kept-own = mantido por conta própria; agora {$now}
workspaces-kept-held = mantido pausado; agora {$now}
workspaces-open-sets = {" — "}Abrir põe como mantido

## No que uma sessão vai virar

workspaces-start-own = parada: para iniciar, por conta própria
workspaces-start-held = parada: para iniciar, pausada
workspaces-let-go = pausada: para soltar
workspaces-to-hold = roda por conta própria: para pausar
workspaces-as-kept = como mantida

## A lista que escolhe

workspaces-quit-title = Parar as sessões do Claude Code também?
workspaces-quit-summary = Ligado: paradas ao sair do tvty (continuam reiniciáveis). Desligado: seguem rodando.
workspaces-new-title = Um workspace novo
workspaces-new-summary = Ligado: guardadas nele, cada uma como roda agora (por conta própria, ou pausada).
workspaces-save-title = Guardar {$name} como as coisas estão agora
workspaces-save-summary = Ligado: nele, cada uma como roda agora. Desligado: tiradas dele.
workspaces-shut-title = Desligar {$name}: parar suas sessões?
workspaces-shut-summary = Ligado: paradas. Desligado: seguem rodando. Um grupo que outro workspace também tem fica desligado.
workspaces-open-title = Abrir {$name}
workspaces-open-summary = Ligado: feito. Desligado: fica como está.
workspaces-restart-title = Reiniciar as sessões que pararam?
workspaces-restart-summary = Ligado: reiniciadas, retomando a conversa. Como estavam: uma sessão pausada fica pausada de novo; do zero: cada uma arranca, depois roda por conta própria.
workspaces-host = host
workspaces-held-for-good = {" · "}pausada até soltarem
workspaces-held-while = {" · "}pausada por um tempo
workspaces-ran-on = {$place} · seguiu rodando: sua parada não pegou
workspaces-not-running = não roda: mantida como estava
workspaces-new-in-group = nova neste grupo
workspaces-not-in-yet = ainda não está nele
workspaces-also-in = também em {$others}
workspaces-some = {$ticked} de {$of}
workspaces-whole-group = o grupo inteiro
workspaces-name = Nome
workspaces-cancel = Cancelar
workspaces-keep = Manter
workspaces-quit-keep = Sair, mantendo todas rodando
workspaces-quit-stop = Sair, parando as {$count} ligadas
workspaces-shut-keep = Desligar, mantendo todas rodando
workspaces-shut-stop = Desligar, parando as {$count} ligadas
workspaces-open-do = Abrir, fazendo as {$count}
workspaces-not-now = Agora não
workspaces-restart-fresh = Reiniciar as {$count} do zero
workspaces-restart-as-were = Reiniciar as {$count} como estavam
workspaces-remember-quit = Lembrar esta escolha (parar todas, ou manter todas)
workspaces-remember-restart = Lembrar esta escolha (todas as vezes, todas elas)
workspaces-remember-tip = Configurações > Layout > Sessões muda

## O que foi feito

workspaces-none-runs = {$name}: nenhuma de suas sessões roda
workspaces-all-as-kept = {$name}: toda sessão está como mantida
workspaces-kept = { $count ->
    [one] workspace {$name} guardado: {$count} grupo
   *[other] workspace {$name} guardado: {$count} grupos
}
workspaces-shut-run-on = {$name} desligado: suas sessões seguem rodando
workspaces-shut-stopped = { $count ->
    [one] {$name} desligado: {$count} sessão parada
   *[other] {$name} desligado: {$count} sessões paradas
}
workspaces-shut = desligado
workspaces-no-loop = { $count ->
    [one] {$count} sessão em nenhum loop desta máquina: não parada
   *[other] {$count} sessões em nenhum loop desta máquina: não paradas
}
workspaces-nothing-to-start = {$agent}: nenhum loop nem pasta conhecidos para iniciá-lo
workspaces-astray = {$agent}: {$cwd} é a pasta de {$other}, não iniciado
workspaces-mode-failed = {$agent}: seu modo: {$error}
workspaces-opened = { $count ->
    [one] {$name} aberto: {$count} sessão posta
   *[other] {$name} aberto: {$count} sessões postas
}
workspaces-opening = abrindo {$name}
workspaces-added = {$project} está em {$name}
workspaces-shown-tip = o terminal mostrado assim que as sessões voltarem
workspaces-quit-later = ou depois:
workspaces-quit-later-tip = Parar as {$count} ligadas e sair em {$delay}: o diálogo conta regressivo enquanto isso; fechá-lo, ou sair por outro caminho, descarta
workspaces-quit-in = parando e saindo em {$left}
workspaces-shell = terminal · {$cwd}
workspaces-reopen-shells = Reabrir os terminais também
