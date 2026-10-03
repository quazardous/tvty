# L'interface de tvty, en français. Mêmes fichiers et mêmes ids que
# l'anglais (assets/locales/en/), qui fait référence.
# Les espaces de travail, et la liste qui choisit des sessions.

## La liste des espaces

workspaces-new = + espace
workspaces-new-tip = garde les groupes qui tournent maintenant, choisis dans une liste, sous un nom
workspaces-none = Aucun espace pour l'instant : un espace garde des groupes, et comment tourne chacune de leurs sessions.
workspaces-default-name = Espace
workspaces-its-name = son nom
workspaces-act-open = Ouvrir
workspaces-act-open-tip = démarre ce qui est arrêté et met chaque session comme gardée — demandé d'abord
workspaces-act-shut = Fermer
workspaces-act-shut-tip = arrête ses sessions — demandé d'abord, lesquelles ; un groupe qu'un autre espace a aussi est laissé
workspaces-act-save = enregistrer
workspaces-act-save-tip = le garde de nouveau tel que tout est maintenant, ou ajoute un groupe — choisi dans une liste
workspaces-again-tip = le groupe affiché, gardé de nouveau tel qu'il tourne maintenant
workspaces-add-tip = ajoute le groupe affiché, ses sessions telles qu'elles tournent
workspaces-act-rename = renommer
workspaces-act-rename-tip = un autre nom ; Entrée confirme
workspaces-act-delete = supprimer
workspaces-act-delete-sure = supprimer : sûr ?
workspaces-act-delete-tip = oublie l'espace ; ses sessions ne sont pas touchées
workspaces-no-project = sans projet
workspaces-drop-tip = retire ce groupe de l'espace (ses sessions ne sont pas touchées)
workspaces-now-stopped = arrêtée
workspaces-now-runs = tourne
workspaces-now-held = retenue
workspaces-kept-own = gardée toute seule ; maintenant {$now}
workspaces-kept-held = gardée retenue ; maintenant {$now}
workspaces-open-sets = {" — "}Ouvrir la remet comme gardée

## Ce qu'une session deviendra

workspaces-start-own = arrêtée : à démarrer, toute seule
workspaces-start-held = arrêtée : à démarrer, retenue
workspaces-let-go = retenue : à libérer
workspaces-to-hold = tourne seule : à retenir
workspaces-as-kept = comme gardée

## La liste qui choisit

workspaces-quit-title = Arrêter aussi les sessions Claude Code ?
workspaces-quit-summary = Activé : arrêtée quand tvty quitte (elles restent redémarrables). Désactivé : elle continue.
workspaces-new-title = Un nouvel espace
workspaces-new-summary = Activé : gardée dedans, chacune telle qu'elle tourne (seule, ou retenue).
workspaces-save-title = Garder {$name} tel que tout est maintenant
workspaces-save-summary = Activé : dedans, chacune telle qu'elle tourne. Désactivé : retirée.
workspaces-shut-title = Fermer {$name} : arrêter ses sessions ?
workspaces-shut-summary = Activé : arrêtée. Désactivé : elle continue. Un groupe qu'un autre espace a aussi est laissé désactivé.
workspaces-open-title = Ouvrir {$name}
workspaces-open-summary = Activé : fait. Désactivé : laissé tel quel.
workspaces-restart-title = Redémarrer les sessions arrêtées quand tvty a quitté ?
workspaces-restart-summary = Activé : redémarrée, en reprenant sa conversation. Comme elles étaient : une session retenue l'est de nouveau ; à neuf : chacune démarre, puis tourne seule.
workspaces-host = hôte
workspaces-held-for-good = {" · "}retenue jusqu'à nouvel ordre
workspaces-held-while = {" · "}retenue un moment
workspaces-ran-on = {$place} · a continué : son arrêt n'a pas pris
workspaces-not-running = ne tourne pas : gardée telle qu'elle était
workspaces-new-in-group = nouvelle dans ce groupe
workspaces-not-in-yet = pas encore dedans
workspaces-also-in = aussi dans {$others}
workspaces-some = {$ticked} sur {$of}
workspaces-whole-group = tout le groupe
workspaces-name = Nom
workspaces-cancel = Annuler
workspaces-keep = Garder
workspaces-quit-keep = Quitter, tout laisser tourner
workspaces-quit-stop = Quitter, arrêter les {$count} activées
workspaces-shut-keep = Fermer, tout laisser tourner
workspaces-shut-stop = Fermer, arrêter les {$count} activées
workspaces-open-do = Ouvrir, faire les {$count} activées
workspaces-not-now = Pas maintenant
workspaces-restart-fresh = Redémarrer les {$count} à neuf
workspaces-restart-as-were = Redémarrer les {$count} comme elles étaient
workspaces-remember-quit = Retenir ce choix (toutes les arrêter, ou toutes les garder)
workspaces-remember-restart = Retenir ce choix (à chaque fois, toutes)
workspaces-remember-tip = Réglages > Disposition > Sessions le change

## Ce qui a été fait

workspaces-none-runs = {$name} : aucune de ses sessions ne tourne
workspaces-all-as-kept = {$name} : chaque session est comme gardée
workspaces-kept = { $count ->
    [one] espace {$name} gardé : {$count} groupe
   *[other] espace {$name} gardé : {$count} groupes
}
workspaces-shut-run-on = {$name} fermé : ses sessions continuent
workspaces-shut-stopped = { $count ->
    [one] {$name} fermé : {$count} session arrêtée
   *[other] {$name} fermé : {$count} sessions arrêtées
}
workspaces-shut = fermer
workspaces-no-loop = { $count ->
    [one] {$count} session sur aucune boucle de cette machine : pas arrêtée
   *[other] {$count} sessions sur aucune boucle de cette machine : pas arrêtées
}
workspaces-nothing-to-start = {$agent} : ni boucle ni dossier connus pour le démarrer
workspaces-astray = {$agent} : {$cwd} est le dossier de {$other}, pas démarré
workspaces-mode-failed = {$agent} : son mode : {$error}
workspaces-opened = { $count ->
    [one] {$name} ouvert : {$count} session réglée
   *[other] {$name} ouvert : {$count} sessions réglées
}
workspaces-opening = ouverture de {$name}
workspaces-added = {$project} est dans {$name}
