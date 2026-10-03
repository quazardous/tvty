# L'interface de tvty, en français. Mêmes fichiers et mêmes ids que
# l'anglais (assets/locales/en/), qui fait référence.
# Le 📢 d'un projet et ses options.

## Le 📢

megaphone-no-project = Une consigne permanente et un focus de réveil appartiennent à un projet : affiche-en un d'abord
megaphone-standing = Consigne permanente — {$project}
megaphone-standing-hint = Placée en tête de chaque réveil de ses agents, événement comme backlog. Laisse-en une avant de t'absenter ; efface-la à ton retour.
megaphone-prompt-placeholder = par ex. du débogage léger d'abord, pas de gros changements
megaphone-focus = Focus de réveil
megaphone-focus-hint = Seuls ces tickets réveillent les agents du projet, backlog et événements. 123, 456 ne garde que ceux-là ; !789 garde tout sauf lui. 123+ ajoute ses enfants, 123++ tous ses descendants, +123 / ++123 ses parents, 123~ ses tickets liés. Les événements hors focus restent non lus jusqu'à ce que tu l'effaces.
megaphone-tickets-placeholder = par ex. 2518, 2523++   ou   !2180
megaphone-until-placeholder = jusqu'au (facultatif) : 2026-09-30 18:00
megaphone-not-a-date = jusqu'au : {$typed} n'est pas une date (2026-09-30 18:00)
megaphone-not-a-time = jusqu'au : {$typed} n'est pas une heure valable ici
megaphone-clear = Effacer
megaphone-save = Enregistrer
megaphone-said-nothing = {$project} : plus rien ne guide ses agents
megaphone-said-prompt = {$project} : ses agents lisent « {$prompt} » à chaque réveil
megaphone-said-focus = {$project} : seul son focus réveille ses agents
megaphone-tip-standing = Consigne permanente : {$prompt}
megaphone-tip-focus = Focus de réveil : {$focus}

## Un message à tous les agents

megaphone-message = Message à tous les agents
megaphone-message-hint = Tapé maintenant dans chaque session d'agent en marche, quoi qu'elle fasse. Envoyer et retenir retient aussi chaque boucle (pas AFK ∞) : aucun réveil ne lance de nouveau travail tant que tu ne les libères pas. Laissé vide, le texte affiché est envoyé.
megaphone-no-loop = Aucune boucle d'agent ne tourne.
megaphone-running = {$count} en marche : {$names}
megaphone-release = Libérer
megaphone-send = Envoyer
megaphone-send-hold = Envoyer et retenir
megaphone-typed-into = tapé dans {$names}
megaphone-queued-for = mis en file pour {$names}
megaphone-held = retenu {$names}
megaphone-released = libéré {$names}
megaphone-hold-not-applied = retenue pas appliquée

## Les options d'un projet

projectopts-global = Global
projectopts-a-project = un projet…
projectopts-failed = réglages du projet
projectopts-choose = Choisis un projet au-dessus.
projectopts-no-folder = aiball ne connaît aucun dossier de ce projet sur cette machine : aucun de ses agents n'y travaille.
projectopts-asking = Question à aiball…
projectopts-could-not-say = aiball n'a pas pu le dire : {$error}
projectopts-no-file = {$folder} n'a pas de .aiball.yaml, ni aucun dossier au-dessus : les valeurs par défaut d'aiball s'appliquent. Nouveau projet… le met en place.
projectopts-written-in = Écrit dans {$file}.
projectopts-also-serves = Il sert aussi {$others} : un changement ici vaut pour eux aussi.
projectopts-on-named = activé : {$name}
projectopts-none-set = {$project} ne règle rien de la config du board : il a les valeurs du board.
projectopts-all-keys = Toutes ses clés
