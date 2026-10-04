# L'interface de tvty, en français. Mêmes fichiers et mêmes ids que
# l'anglais (assets/locales/en/), qui fait référence.
# La barre de l'agent, sous son terminal (src/shell/agentbar.rs).

## Qui mène la boucle

agentbar-afk-auto = auto
agentbar-afk-hold-10m = retenir 10 min
agentbar-afk-hold = retenir
agentbar-afk-boot = … démarrage
agentbar-afk-arming-tip = armé : le petit bonhomme montre le mode choisi avec F9, en vigueur 3 s après le dernier appui — ▶ ou ‖ dit celui en vigueur d'ici là
agentbar-afk-tip = qui mène la boucle : ▶ elle-même, ‖ retenue pour toi (ou pendant que tu tapes) ; le petit bonhomme est le mode AFK — gris : tu es absent, la boucle tourne seule ; les secondes d'une retenue de 10 min ; ∞ retenue. F9 le fait tourner (auto → 10 min → ∞), en vigueur 3 s après le dernier appui ; un clic choisit

## Ce que fait son Claude

agentbar-working = au travail
agentbar-starting = démarre
agentbar-idle = au repos
agentbar-offline = hors ligne
agentbar-boot-left = {" · "}encore {$seconds} s
agentbar-boot-tip = sa boucle démarre : Claude se charge, peut reprendre sa conversation ou compacter ; la boucle le réveille une fois le démarrage fini (30 s au moins, plus tant qu'une reprise ou une compaction s'affiche)
agentbar-state-tip = ce que fait son Claude, et depuis quand
agentbar-dialog = attend une réponse

## Redémarrer son Claude

agentbar-restarting = redémarre…
agentbar-restarting-tip = son Claude redémarre, en reprenant sa conversation
agentbar-restart-pending = redémarrage prévu
agentbar-restart-pending-tip = un redémarrage est demandé : son Claude redémarre dès qu'il est inactif, en reprenant sa conversation ; un clic l'annule
agentbar-update = mise à jour
agentbar-update-tip = son Claude Code a installé une mise à jour : un clic demande de le redémarrer (tout de suite s'il est au repos, sinon dès qu'il l'est), en reprenant sa conversation
agentbar-restart-ask-busy = Redémarrer son Claude dès qu'il sera au repos ?
agentbar-restart-ask = Redémarrer son Claude maintenant ?
agentbar-restart = Redémarrer
agentbar-restart-resumed = sa conversation est reprise
agentbar-cancel = Annuler
agentbar-restart-done = le Claude de {$agent} redémarre dès qu'il est au repos, en reprenant sa conversation
agentbar-restart-failed = redémarrage du Claude de {$agent}

## Où tourne sa boucle, qui a les commandes

agentbar-place-host = hôte
agentbar-hands-copy = copie
agentbar-hands-controls = commandes
agentbar-moving = déplacement…
agentbar-runs-hosted = sa boucle tourne sur l'hôte de sessions d'aiball ;
agentbar-runs-mux = sa boucle tourne dans {$mux} (claude-loop) ;
agentbar-copy-tip = ce terminal est une copie : tu regardes, rien de ce que tu tapes n'arrive à son Claude, et la session garde sa taille.
agentbar-controls-tip = tu as les commandes, partagées avec tout autre client : la taille suit celui qui tape en dernier.
agentbar-others-attached = { $others ->
    [one] {$others} autre client attaché ({$typing} avec les commandes).
   *[other] {$others} autres clients attachés ({$typing} avec les commandes).
}
agentbar-proxy-alive = Le proxy de terminal devant Claude est en vie.
agentbar-place-click = Un clic : prendre ou laisser les commandes, déplacer la boucle.
agentbar-others-type = le terminal de claude-loop y tape aussi
agentbar-take-controls = Prendre les commandes
agentbar-leave-copy = Laisser pour une copie
agentbar-close-others = Fermer les autres ({$others})
agentbar-close-others-tip = { $others ->
    [one] l'autre client attaché à cette session la quitte (le terminal de claude-loop, un autre Terminal Velocity) ; son Claude et ce terminal continuent
   *[other] les {$others} autres clients attachés à cette session la quittent (le terminal de claude-loop, un autre Terminal Velocity) ; son Claude et ce terminal continuent
}
agentbar-move-into = Déplacer dans {$mux}
agentbar-move-to-host = Déplacer sur l'hôte
agentbar-move-tip-into = son Claude redémarre dans {$mux}, en reprenant sa conversation
agentbar-move-tip-to-host = son Claude redémarre sur l'hôte d'aiball, en reprenant sa conversation
agentbar-move-interrupts = {" — "}il travaille : le déplacement l'interrompt
agentbar-moved-into = {$agent} déplacé dans {$mux}, sa conversation reprise
agentbar-moved-to-host = {$agent} déplacé sur l'hôte d'aiball, sa conversation reprise
agentbar-move-failed-into = déplacement de {$agent} dans {$mux}
agentbar-move-failed-to-host = déplacement de {$agent} sur l'hôte d'aiball
agentbar-hold-failed = retenue de {$agent}
agentbar-closed-others = { $count ->
    [one] l'autre client de cette session est fermé
   *[other] les {$count} autres clients de cette session sont fermés
}
agentbar-closed-others-all = les autres clients de cette session sont fermés
agentbar-closed-others-asked = l'hôte de la session a été prié de fermer ses autres clients
agentbar-others-left = { $count ->
    [one] {$count} autre client encore attaché : {$mux} ne sait pas encore le distinguer de celui-ci
   *[other] {$count} autres clients encore attachés : {$mux} ne sait pas encore les distinguer de celui-ci
}
agentbar-close-others-failed = fermer les autres clients

## Remote Control, alertes, le prompt

agentbar-rc-on = Remote Control est actif : ce Claude peut être repris depuis claude.ai et l'app mobile
agentbar-rc-off = Remote Control est inactif. /rc dans la session l'active ; les boucles d'un dossier le prennent des réglages du projet
agentbar-trust = faire confiance à ce dossier ?
agentbar-not-logged-in = pas connecté
agentbar-api-unreachable = API injoignable
agentbar-link-down = lien de la boucle coupé
agentbar-aiball-unreachable = aiball injoignable
agentbar-prompt-input = le prompt de Claude est à l'écran, avec du texte pas encore envoyé
agentbar-prompt-empty = le prompt de Claude est à l'écran, vide
agentbar-typing = un humain a tapé dans son terminal il y a un instant : la boucle attend
agentbar-zen = zen
agentbar-zen-tip = mode zen : la boucle se tait

## Ses compteurs

agentbar-all = tous:{$count}
agentbar-all-tip = a : tous les tickets ouverts du projet
agentbar-backlog = backlog:{$count}
agentbar-backlog-tip = b : son backlog, les tickets qu'il doit regarder ; un clic les liste
agentbar-events = évts:{$count}
agentbar-events-tip = e : ses événements pas encore vus — pings, réponses, décisions qui l'attendent
agentbar-holds = tient:{$count}
agentbar-holds-tip = les tickets qu'il tient
agentbar-wake-tip = du travail attend la boucle : elle y réveille son Claude à la fin du compte à rebours
agentbar-pending-tip = du travail attend la boucle (événements ou backlog)

## Son backlog, sur la barre

agentbar-backlog-of = backlog de {$agent}
agentbar-reading = lecture…
agentbar-backlog-error = backlog : {$error}
agentbar-backlog-empty = rien dans son backlog
agentbar-tier-critical = critique
agentbar-tier-hot = chaud
agentbar-tier-yours = à lui
agentbar-tier-decision = sa décision en attente
agentbar-tier-waiting = attend les autres
agentbar-tier-blocked = bloqué
agentbar-tier-other = autre

agentbar-info-resuming = reprise
agentbar-info-compacting = compactage
agentbar-info-wait = en attente
agentbar-info-interrupted = interrompu
agentbar-info-user = un humain au clavier
agentbar-info-picker-session = choix de la session
agentbar-info-picker-mode = choix du mode
agentbar-info-error-rate-limit = limite de débit
agentbar-info-error-overloaded = API surchargée
agentbar-info-error-api = erreur d'API
agentbar-info-retry = nouvel essai {$attempt}
agentbar-restart-cancel-ask = Annuler le redémarrage en attente de son Claude ?
agentbar-restart-cancel = L'annuler
agentbar-restart-keep = Le garder
agentbar-restart-cancelled = le redémarrage du Claude de {$agent} est annulé ; la mise à jour reste proposée
agentbar-restart-none = aucun redémarrage du Claude de {$agent} n'était en attente
agentbar-restart-cancel-failed = annulation du redémarrage du Claude de {$agent}
agentbar-restart-too-old = sa boucle a démarré avant qu'aiball sache annuler un redémarrage : elle redémarrera
