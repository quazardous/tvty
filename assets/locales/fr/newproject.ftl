# L'interface de tvty, en français. Mêmes fichiers et mêmes ids que
# l'anglais (assets/locales/en/), qui fait référence.
# L'assistant de nouveau projet. Ce qu'aiball dit avoir fait s'affiche tel
# qu'aiball le dit.

newproject-title = Nouveau projet
newproject-step-folder = Dossier
newproject-step-identity = Qui y travaille
newproject-step-done = Mise en place
newproject-step-next = Et ensuite

## Le dossier

newproject-which-folder = Quel dossier devient le projet ?
newproject-folder-placeholder = le dossier du projet, par ex. ~/dev/app
newproject-choose-folder = Choisir le dossier du projet
newproject-browse = Parcourir…
newproject-folder-empty = Le dossier où l'agent travaillera : la racine du projet.
newproject-folder-missing = Ce dossier n'existe pas.
newproject-folder-file = C'est un fichier, pas un dossier.
newproject-folder-checking = Question à aiball…
newproject-folder-project = Déjà un projet aiball : {$name} ({$file}). Suivant montre ce qu'il dit, pour le remettre en place.
newproject-folder-configured = Déjà configuré pour aiball ({$file}). Suivant montre ce qu'il dit, pour le remettre en place.
newproject-folder-git = Un dépôt git : prêt.
newproject-folder-ready = Prêt (pas un dépôt git).
newproject-open-running = Ouvrir {$label}, en marche
newproject-open = Ouvrir {$name}
newproject-next = Suivant →
newproject-back = ← Retour

## Qui y travaille

newproject-project-placeholder = projet
newproject-agent-placeholder = agent
newproject-field-project = projet
newproject-field-agent = agent
newproject-bad-project = Le nom du projet : lettres, chiffres, -, _ et .
newproject-bad-agent = Le nom de l'agent : lettres, chiffres, -, _ et .
newproject-filled-from = Rempli depuis {$file} : change ce que tu veux, ce qui est ici est ce qui sera mis en place.
newproject-crew = un agent d'équipe
newproject-crew-about = À côté du responsable du projet, sur les tickets qu'on lui donne ; sinon : le responsable.
newproject-host = ses boucles sur l'hôte d'aiball
newproject-host-about = Où tournent les boucles démarrées dans ce dossier ; sinon : dans {$mux}.
newproject-rc = Remote Control
newproject-rc-about = Son Claude se rejoint depuis claude.ai et l'app Claude.
newproject-private = un projet privé
newproject-private-about = aiball lui sert son kit privé (pas de tickets publics, pas d'abonnés).
newproject-noclaim = pas de claim
newproject-noclaim-about = L'agent ne travaille que sur les tickets qui lui sont assignés, n'en prend jamais dans le pool.
newproject-in = Dans {$folder}, aiball :
newproject-will-file = {$will} {$file}
newproject-will-created = crée
newproject-will-added = ajoute son entrée à
newproject-will-rewritten = réécrit son entrée dans
newproject-will-patched = met à jour
newproject-will-overwrote = écrase
newproject-will-kept = garde
newproject-joins = {$name} est déjà sur le board : ce dossier le rejoint (un autre dossier, ou un agent d'équipe).
newproject-then-sets = puis règle {$what} dans .aiball.yaml
newproject-files = .mcp.json : le serveur MCP d'aiball pour Claude Code · .aiball.yaml : projet, agent, rôle, où tournent ses boucles.
newproject-set-up = Mettre en place
newproject-setting-up = Mise en place…
newproject-unsaved = mis en place, mais où tournent ses boucles et son Remote Control n'ont pas été enregistrés : {$error}
newproject-set-in = réglé {$what} dans {$file}

## Mise en place

newproject-done = {$name} est en place. aiball a dit :
newproject-failed = aiball n'a pas pu le mettre en place :
newproject-nothing-said = (rien dit)

## Et ensuite

newproject-next-start = Démarrer l'agent
newproject-next-start-host = « Démarrer sa première session » démarre le Claude Code de {$agent} sur l'hôte d'aiball, dans le dossier du projet ; son terminal s'ouvre ici, ses tickets à côté.
newproject-next-start-mux = « Démarrer sa première session » démarre le Claude Code de {$agent} dans {$mux}, dans le dossier du projet ; son terminal s'ouvre ici, ses tickets à côté.
newproject-next-mcp = Accepter le serveur MCP d'aiball
newproject-next-mcp-about = À son premier démarrage dans ce dossier, Claude Code demande s'il doit utiliser le serveur MCP que déclare .mcp.json (aiball) : accepte-le. Sans lui, l'agent ne peut ni lire le board ni répondre à ses tickets. Refusé par erreur ? /mcp dans Claude Code l'active.
newproject-next-skill = Installer le skill d'aiball
newproject-next-skill-about = Claude Code n'a pas encore le skill d'aiball sur cette machine : `aiball init skill`, une fois, l'installe — l'agent connaît alors les bons gestes du board.
newproject-next-work = Lui donner du travail
newproject-next-work-about = « + Nouveau » dans le panneau des tickets crée un ticket sur {$name} ; l'agent le prend à son prochain réveil, et ses plans et ses questions reviennent en notifications.
newproject-close = Fermer
newproject-start-first = Démarrer sa première session

newproject-step-mcp-created = {$path} : créé, avec l'entrée d'aiball
newproject-step-mcp-added-others = {$path} : entrée d'aiball ajoutée, les autres serveurs gardés
newproject-step-mcp-added = {$path} : entrée d'aiball ajoutée
newproject-step-mcp-rewritten = {$path} : entrée d'aiball réécrite dans sa forme actuelle
newproject-step-mcp-kept = {$path} : entrée d'aiball déjà là, gardée (force l'écrase)
newproject-step-file-created = {$path} : créé ({$set})
newproject-step-file-overwrote = {$path} : écrasé ({$set})
newproject-step-file-kept = {$path} : déjà là, gardé (force l'écrase)
newproject-step-consumer = {$path} : identité réglée ({$set})
newproject-step-type-kept = {$path} : type de projet déjà {$value}
newproject-step-type = {$path} : type de projet {$value}
newproject-step-type-was = {$path} : type de projet {$value} (avant : {$previous})
newproject-step-deny = {$path} : outils de code refusés ({$set})
