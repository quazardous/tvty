# L'interface de tvty, en français. Mêmes fichiers et mêmes ids que
# l'anglais (assets/locales/en/), qui fait référence.
# Le reste : le menu et les marques du terminal, la visionneuse d'images,
# quelques notifications et erreurs.

## Notifications

misc-aiball-started = aiball ne tournait pas : démarré
misc-aiball-silent = aiball ne répond pas à {$at}
misc-service-failed = le service d'aiball n'a pas démarré : {$error}
misc-no-service = {$why} ; il n'y a pas de service aiball à démarrer
misc-filed = créé — {$title}
misc-filed-moderate = {$title} — un nouveau ticket à modérer
misc-filed-other = {$title} — un nouveau ticket
misc-something-new = du nouveau
misc-held = {$said} : sa boucle est retenue jusqu'à ce que tu la libères
misc-go-to = aller à
misc-go-to-what = aller à {$what}
misc-not-a-ticket = {$typed} n'est pas un ticket : un numéro, ou le lien #C. d'un commentaire
misc-copied = Copié dans le presse-papiers
misc-new-terminal = nouveau terminal
misc-no-folder = aucun dossier connu pour {$project}
misc-not-a-directory = {$folder} n'est pas un dossier
misc-not-idle = son Claude travaille : une fois au repos
misc-stop-not-received = {$agent} : aucune de ses boucles n'a reçu l'arrêt (elle ne tourne pas, ou pas là où aiball peut l'atteindre)

## Le terminal

misc-open-link = Ouvrir le lien
misc-copy-link = Copier le lien
misc-copy = Copier
misc-paste = Coller
misc-frozen = figé · sélection
misc-frozen-tip = L'écran est figé tant que du texte est sélectionné ; la session continue. Un clic ici, Échap ou une touche le libère.
misc-size-taken = taille prise par un autre client · cliquer pour la reprendre
misc-session-ended = La session est terminée.
misc-hub-session = Cette session tourne sur le hub d'aiball, une autre machine : elle ne peut pas s'ouvrir depuis celle-ci. Ses tickets sont dans le panneau.
misc-all-terminals = Tous les terminaux
misc-gallery-hint = taper filtre · flèches déplacent · entrée ouvre · échap ferme

## Le modèle de l'agent, ses limites, ses refus

misc-price = {$input} / {$output} par M tokens
misc-model-out = {$name} est sorti
misc-prices-from = prix de {$catalog}
misc-limit = limite d'usage atteinte
misc-limit-resets = limite d'usage atteinte · remise à zéro {$resets}
misc-denials = { $count ->
    [one] {$count} appel d'outil refusé par Claude Code dans la dernière heure{$ago}{$why} — un agent refusé s'arrête là
   *[other] {$count} appels d'outil refusés par Claude Code dans la dernière heure{$ago}{$why} — un agent refusé s'arrête là
}
misc-denials-last = , le dernier il y a {$ago}

## Les images

misc-image-too-large = image trop grande pour s'afficher ici
misc-image-unavailable = image indisponible
misc-fitted = {" "}(ajustée)
misc-viewer-hint = molette ou + − zoom · 1 taille réelle · 0 ajuster · glisser pour déplacer · Échap

## The usage arrow in the top bar

usage-ratio = ×{$ratio}
usage-points = {$points} pts
usage-wall = mur {$left}
usage-no-wall = pas de mur
usage-window-five_hour = Fenêtre de 5 h
usage-window-seven_day = Semaine
usage-tip-window = {$window} : {$used} % utilisés, {$expected} % à rythme régulier · remise à zéro {$resets} · {$end}
usage-tip-wall = à ce rythme, le quota s'épuise dans {$left}
usage-tip-lasts = à ce rythme, il tient jusqu'à la remise à zéro
usage-tip-click = Un clic : l'écart dit autrement.
