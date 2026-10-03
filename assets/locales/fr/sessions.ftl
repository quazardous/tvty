# L'interface de tvty, en français. Mêmes fichiers et mêmes ids que
# l'anglais (assets/locales/en/), qui fait référence.
# La liste des sessions, la ligne d'une session, les onglets et la barre de
# titre (src/shell.rs, src/shell/loopstabs.rs, src/shell/tabs.rs, src/status.rs).

## La tête de la liste

sessions-tab-sessions = Sessions
sessions-tab-workspaces = Espaces
sessions-order-recent = le projet utilisé en dernier d'abord, comme ctrl+tab ; un clic : alphabétique
sessions-order-alpha = alphabétique ; un clic : ton ordre (glisse un projet par son nom)
sessions-order-yours = ton ordre : glisse un projet par son nom ; un clic : le projet utilisé en dernier d'abord
sessions-new-project = + projet
sessions-new-project-tip = un dossier devient un projet aiball (aiball init), puis sa première session
sessions-new-terminal-tip = un terminal à part, qui survit à tvty
sessions-filter = Filtrer…  ctrl+shift+f
sessions-fold = Replier la liste des sessions

## Ses groupes

sessions-group-live = actives
sessions-group-idle = arrêtées
sessions-group-shut = sans boucle
sessions-group-on-hub = sur le hub
sessions-none = Aucune session
sessions-none-found = Aucune session trouvée
sessions-first-project = Créer ton premier projet
sessions-none-stopped = Aucune boucle arrêtée
sessions-none-stopped-found = Aucune boucle arrêtée trouvée
sessions-none-shut = Aucun agent sans boucle
sessions-none-shut-found = Aucun agent trouvé
sessions-none-hub = Aucune session sur le hub
sessions-none-hub-found = Aucune session trouvée sur le hub
sessions-no-project = Sans projet

## La tête d'un projet

sessions-heading-tip = ses tickets dans le panneau, sans ouvrir de session ; glisse-le sur un autre pour le déplacer
sessions-new-session = + session
sessions-project-options = ses réglages : où tournent ses boucles, Remote Control, sa config du board
sessions-project-terminal = un terminal dans le dossier du projet, listé avec lui

## La ligne d'une session

sessions-open = ouverte dans tvty : son terminal tourne ici
sessions-looped = {$mux} : son Claude tourne dans claude-loop, ouvert via {$mux} (pas sur l'hôte d'aiball)
sessions-attached = { $others ->
    [one] {$others} autre client attaché (le terminal de claude-loop, un autre tvty), {$typing} avec les commandes
   *[other] {$others} autres clients attachés (le terminal de claude-loop, un autre tvty), {$typing} avec les commandes
}
sessions-attached-copy = {" "}: ouverte ici en copie
sessions-host-shell = un terminal sur l'hôte d'aiball, sans Claude
sessions-update = son Claude Code a installé une mise à jour : redémarre-le depuis sa barre
sessions-starting = démarrage…
sessions-start = ▶ démarrer
sessions-astray = ⚠ dossier de {$other}
sessions-astray-tip = {$other} travaille dans {$cwd} : démarré ici, cet agent reprendrait sa conversation. Pas démarré.
sessions-stop-ask = arrêter ?
sessions-stop-tip = arrêter la boucle de {$who} : via aiball, elle reste redémarrable (dans les arrêtées) — un second clic l'arrête
sessions-stopped = {$who} arrêté
sessions-stop-failed-loop = arrêter {$who}
sessions-forget-ask = oublier ?
sessions-forget-tip = oublier {$who} : aiball ne le liste plus ; son dossier, son .aiball.yaml et les tickets du projet restent — un second clic oublie
sessions-forgot = {$who} oublié
sessions-forget-failed = oublier {$who}

## L'état d'une session, en une ligne

sessions-offline = hors ligne
sessions-offline-tip = sa boucle n'est pas connectée à aiball
sessions-working = au travail
sessions-starting-state = démarre
sessions-idle = au repos
sessions-booting = démarre
sessions-claude-working = Claude travaille
sessions-claude-starting = Claude démarre
sessions-claude-idle = Claude est au repos
sessions-held-typing-for-good = retenue jusqu'à nouvel ordre (un humain y tape) : la boucle ne le réveille pas
sessions-typing = un humain y tape : la boucle attend
sessions-held-for-good = retenue jusqu'à nouvel ordre : la boucle ne le réveille pas
sessions-held-while = retenue un moment : la boucle ne le réveille pas
sessions-loop-drives = la boucle le mène seule
sessions-mark-held-for-good = retenue jusqu'à nouvel ordre
sessions-mark-held-while = retenue un moment
sessions-mark-own = toute seule

## Les marques de la liste repliée

sessions-mark-tip = {$agent} : {$said}
sessions-mark-limit = {$agent} : {$said} — {$limit}

## « + session »

sessions-form-cwd = dossier de travail
sessions-form-agent = agent
sessions-form-where = où son Claude travaille
sessions-form-host-here = l'hôte d'aiball le démarre ici
sessions-form-loop-here = claude-loop démarre ici
sessions-form-no-dir = ce dossier n'existe pas
sessions-form-crew = un agent d'équipe, à côté de la boucle principale
sessions-form-on-host = sur l'hôte d'aiball, sans {$mux}
sessions-form-cancel = Annuler
sessions-form-start = Démarrer
sessions-a-loop = une boucle
sessions-astray-said = {$cwd} est le dossier de {$other} : {$agent} reprendrait sa conversation
sessions-start-failed = démarrage
sessions-start-host-failed = démarrage sur l'hôte
sessions-started-host = {$agent} démarré sur l'hôte d'aiball dans {$cwd}
sessions-started = {$agent} démarré dans {$cwd}
sessions-runs-already = {$agent} tourne déjà : ouvert en copie

## Les badges : ceux d'un projet, d'un agent

sessions-badge-critical = le ticket critique, #{$ticket}, est ici : il retient le plus de tickets ouverts — un clic l'ouvre
sessions-badge-decisions = { $count ->
    [one] {$count} ticket attend ta décision
   *[other] {$count} tickets attendent ta décision
}
sessions-badge-unread = { $count ->
    [one] {$count} ticket avec du nouveau pour toi
   *[other] {$count} tickets avec du nouveau pour toi
}
sessions-light-backlog = { $count ->
    [one] backlog : {$count} ticket à regarder
   *[other] backlog : {$count} tickets à regarder
}
sessions-light-backlog-unknown = backlog : inconnu tant que sa boucle ne le dit pas
sessions-light-critical = il tient le ticket critique : celui qui retient le plus de tickets ouverts
sessions-light-events = événements : {$count} pas encore vus — pings, réponses, décisions qui l'attendent
sessions-hub-mark = Sur le hub d'aiball, une autre machine : lu d'ici (son état, ses tickets), pas ouvert — une session s'attache depuis sa propre machine.

## Les onglets

sessions-tab-name = son nom
sessions-rename-failed = renommer le terminal
sessions-stop-failed = arrêter le terminal
sessions-rename-tip = le renommer (ou un double clic, F2)
sessions-close-shell = fermer : arrête ce terminal
sessions-close-tab = fermer l'onglet : son Claude continue
sessions-tab-shell-tip = un terminal sur l'hôte d'aiball, sans Claude ; un double clic (ou F2) le renomme
sessions-tab-tip = ctrl+pgup / ctrl+pgdn : l'onglet d'avant, d'après
sessions-new-tab-project = un terminal dans le dossier du projet
sessions-new-tab-home = un terminal dans le dossier personnel

## La fenêtre

sessions-pick = Choisis un terminal à gauche · ctrl+shift+espace les montre tous
sessions-on-hub = {$agent} — sur le hub
sessions-window-hub-project = {$name} — {$project} · {$agent} · sur le hub
sessions-window-hub = {$name} — {$agent} · sur le hub
sessions-bus-down = le bus d'aiball est coupé : tvty se reconnecte ; les listes peuvent être en retard entre-temps
sessions-bus-failing = l'abonnement à {$what} a échoué : réessayé, relu en entier ; les listes peuvent être en retard entre-temps
sessions-menu = Menu : à propos, aide, redémarrer, quitter
sessions-goto = Aller à un ticket : son numéro ou le lien #C. d'un commentaire, Entrée · ctrl+shift+g
sessions-themes = Les thèmes de couleur — le suivant
sessions-message-all = Un message à tous les agents en marche : envoyer, envoyer et retenir, libérer
sessions-settings = Réglages
sessions-user = sous quel nom tvty agit sur le board d'aiball
