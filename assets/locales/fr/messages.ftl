# L'interface de tvty, en français. Mêmes fichiers et mêmes ids que
# l'anglais (assets/locales/en/), qui fait référence.
# Les notifications et les petits dialogues.

## Ce qui a pingé

messages-proposes-plan = propose un plan
messages-proposes-close = propose de fermer
messages-proposes-wontfix = propose de fermer sans correction
messages-escalates = escalade
messages-new-ticket = un nouveau ticket
messages-new-comment = un nouveau commentaire
messages-new-ticket-moderate = un nouveau ticket à modérer
messages-missed = { $more ->
    [0] {$what} · pendant que tvty était fermé
    [one] {$what} · et {$more} autre pendant que tvty était fermé
   *[other] {$what} · et {$more} autres pendant que tvty était fermé
}
messages-dismiss = fermer

## Les mises à jour

messages-update-out = Terminal Velocity {$latest} est sorti (tu as {$running}) : {$how}
messages-update-how = Mises à jour… dans le menu l'installe
messages-update-dev = celui-ci est une version de développement : mets à jour son checkout (Mises à jour… dans le menu dit comment)
messages-aiball-old = aiball {$version} est plus ancien que ce dont Terminal Velocity a besoin ({$needs}) : Mises à jour… dans le menu le met à jour

## Une session terminée

messages-detached = — détachée par un autre client
messages-ended = — la session est terminée
messages-attach-again = Rattacher
messages-starting = Démarrage…
messages-restart = Redémarrer
messages-close = Fermer
messages-copies-failed = faire des autres terminaux des copies

## Quitter, redémarrer

messages-stopping = Arrêt des sessions Claude Code, puis fermeture…
messages-quit-stop-failed = arrêter en quittant tvty
messages-ran-on = {$agents} a continué : l'arrêt n'a pas pris
messages-restarted-as-were = { $count ->
    [one] {$count} session redémarrée comme elle était, en reprenant sa conversation
   *[other] {$count} sessions redémarrées comme elles étaient, en reprenant leur conversation
}
messages-restarted-fresh = { $count ->
    [one] {$count} session redémarrée à neuf, en reprenant sa conversation
   *[other] {$count} sessions redémarrées à neuf, en reprenant leur conversation
}
messages-restart-failed = redémarrage des sessions arrêtées
messages-restart-tvty-failed = redémarrer tvty

## Quelle conversation reprendre

messages-its-agent = son agent
messages-ago = il y a {$time}
messages-some-time-ago = il y a un moment
messages-last-one = La dernière, {$when} :
messages-says-nothing = (elle ne dit encore rien)
messages-quoted = « {$said} »
messages-older = { $count ->
    [one] {$count} plus ancienne aussi : /resume dans Claude Code choisit parmi elles.
   *[other] {$count} plus anciennes aussi : /resume dans Claude Code choisit parmi elles.
}
messages-new-conversation = Nouvelle conversation
messages-resume-last = Reprendre la dernière conversation
messages-start = Démarrer {$who}
messages-resume-question = Claude Code a déjà des conversations dans {$cwd}, aucune d'une boucle : laquelle {$who} reprend-il ?

## La carte des astuces

messages-tip-title = Le savais-tu ?
messages-tips-browsing = Astuces · {$at} / {$of}
messages-not-now = Pas maintenant
messages-previous = ‹ Précédente
messages-next = Suivante ›
messages-show-all-again = Toutes les remontrer
messages-got-it = Compris
messages-next-tip = Astuce suivante
messages-turn-off = Désactiver les astuces
messages-tips-off = Astuces désactivées : Réglages > Disposition > Astuces les réactive
messages-tips-again = Chaque astuce se montrera de nouveau, une à la fois
