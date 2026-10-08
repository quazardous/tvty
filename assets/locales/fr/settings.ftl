# L'interface de tvty, en français. Mêmes fichiers et mêmes ids que
# l'anglais (assets/locales/en/), qui fait référence.
# Réglages (src/settings.rs, src/options.rs, les pages de réglages de
# src/shell.rs). Les mots d'un réglage vont par sa clé.

## Les réglages

setting-appearance-language = Langue
    .about = La langue de l'interface. Ce que les gens écrivent (tickets, commentaires, noms) reste tel quel.
setting-appearance-terminal_font_size = Police des terminaux
    .about = Le texte des terminaux. Aussi Ctrl+Maj+= / Ctrl+Maj+− / Ctrl+Maj+0.
setting-appearance-window_font_size = Texte de la fenêtre
    .about = Tout ce qui entoure les terminaux — listes, tickets, menus.
setting-notifications-max = Affichées au plus
    .about = Dans le coin en haut à droite du terminal (en bas à gauche par-dessus un plein écran), la plus récente dans le coin ; les plus anciennes lui font place.
setting-notifications-seconds = Secondes affichées
    .about = Puis elle s'en va, sauf si le pointeur est dessus.
setting-notifications-own = Tes propres gestes
    .about = Un ticket fermé, une réponse postée, un plan accepté : dit une fois qu'aiball l'a. Un refus est toujours dit.
    .on = affichés
    .off = masqués
setting-notifications-copied = Copié
    .about = Un bref « Copié dans le presse-papiers » en bas quand du texte va au presse-papiers (pas pour une simple sélection).
    .on = affiché
    .off = masqué
setting-terminal-cursor = Curseur
    .about = Dans le terminal qui a les touches : clignotant, fixe, ou comme le programme le demande. Il reste allumé pendant que tu tapes et que le texte arrive.
setting-terminal-scrollback = Lignes d'historique
    .about = Ce que chaque terminal garde pour remonter. Plus coûte de la mémoire : environ 4 Ko par ligne à 160 colonnes, donc 5000 lignes font environ 19 Mo par terminal au plus. L'historique d'une session tmux est celui de tmux (son history-limit).
setting-terminal-paste_unfreezes = Coller
    .about = Une sélection fige l'écran du terminal ; un collage (Ctrl+V, Ctrl+Maj+V, le clic du milieu) le libère et remontre l'écran vivant. Désactivé, l'écran reste figé jusqu'à Échap ou une touche tapée.
    .on = libère l'écran
    .off = le garde figé
setting-terminal-paste_scrolls_down = Coller, en remontant
    .about = Un collage dans un terminal remonté dans son historique le ramène en bas, là où va le texte. Désactivé, la vue reste où elle est.
    .on = revient en bas
    .off = reste en place
setting-scroll-speed = Vitesse de la molette
    .about = Fois celle de tvty : à 1, un cran vaut environ trois lignes de tickets, cinq lignes de l'historique d'un terminal.
setting-mouse-focus = Focus
    .about = Ce qui donne le clavier au terminal ou à une boîte où écrire : un clic, ou le pointeur qui passe dessus (comme le focus d'un gestionnaire de fenêtres suit la souris). Par défaut, comme le système.
setting-appearance-theme = Fenêtre
    .about = Le thème de couleurs de la fenêtre. Ctrl+Maj+K les fait défiler.
setting-appearance-terminal_theme = Terminal
    .about = Celui des terminaux, ou celui de la fenêtre — un terminal sombre dans une fenêtre claire.
setting-appearance-terminal_opacity = Opacité des terminaux
    .about = Sous 100 %, le bureau se voit à travers le fond des terminaux, en direct. Les listes, les tickets et les couleurs qu'un programme fixe restent opaques.
setting-appearance-terminal_blur = Flou derrière
    .about = Derrière un terminal transparent, le bureau flouté. Seulement là où le compositeur le permet (KDE) ; ailleurs il reste net.
    .on = flouté
    .off = net
setting-updates-check = Vérifier les mises à jour
    .about = Au démarrage et une fois par jour, demande à GitHub si un Terminal Velocity plus récent est sorti, et le dit une fois (rien d'autre n'est envoyé). L'outil de mise à jour (Mises à jour… dans le menu) l'installe.
    .on = vérifier
    .off = jamais
setting-usage-display = Flèche d'usage
    .about = Dans la barre du haut, l'abonnement face au rythme qui l'épuiserait pile à la fin de la fenêtre : une flèche rouge vers le haut au-dessus, verte vers le bas en dessous, plus franche à mesure que l'écart grandit. Son chiffre : l'usage face à l'attendu (×1,3), l'écart en points du quota (+12 pts), ou quand le quota s'épuise à ce rythme. Un clic sur la flèche passe au suivant jusqu'au redémarrage de tvty. Affichée dès qu'une boucle lit l'usage dans Claude Code (abonnements Pro et Max).
setting-tips-show = Afficher les astuces
    .about = « Le savais-tu ? » : une courte astuce une fois après le démarrage, et la première fois qu'une page s'ouvre ; jamais pour ce que tu utilises déjà. Astuces… dans le menu les montre toutes.
    .on = afficher
    .off = jamais
setting-sessions-order = Ordre
    .about = Les projets dans la liste, le carrousel et la galerie : le dernier utilisé d'abord, comme ctrl+tab ; alphabétique ; ou le tien, en les glissant dans la liste. Aussi le ⇅ dans la tête de la liste.
setting-sessions-show_hub = Les sessions du hub
    .about = Quand cette machine joint aiball par un nœud proxy : lister aussi les sessions qui tournent sur le hub d'aiball, à part, sous « sur le hub ». Elles sont lues d'ici (état, tickets), jamais ouvertes : une session s'attache depuis sa propre machine. Les sessions des autres nœuds ne sont jamais listées.
    .on = listées
    .off = pas listées
setting-sessions-on_quit = En quittant
    .about = Quand tvty quitte avec des boucles Claude Code de cette machine en marche : demander, les arrêter (via aiball : elles restent redémarrables), ou les laisser tourner.
setting-sessions-on_start = Au démarrage
    .about = Les boucles que tvty a arrêtées en quittant : demander, les redémarrer comme elles étaient (en reprenant leur conversation, une boucle retenue l'est de nouveau), les redémarrer à neuf (démarrage, puis seules), ou les laisser arrêtées.
setting-tickets-panel_overlay = Place
    .about = À côté du terminal, qui est alors plus étroit — ou par-dessus, comme la liste des sessions à gauche : le terminal garde toute sa largeur et le panneau couvre son côté droit tant qu'il est ouvert.
    .on = par-dessus le terminal
    .off = à côté du terminal
setting-tickets-newest_first = Récents d'abord, en plein écran
    .about = Un ticket en plein écran montre d'abord le mot le plus récent de son fil (aussi ⇅ dans la tête du ticket).
    .on = récents d'abord
    .off = récents à la fin
setting-tickets-panel_newest_first = Récents d'abord, dans le panneau
    .about = Un ticket dans le panneau montre d'abord le mot le plus récent de son fil ; sinon à la fin, près de la réponse (aussi ⇅ là). Le panneau a son propre ordre : celui du plein écran n'est pas le sien.
    .on = récents d'abord
    .off = récents à la fin
setting-tickets-summary_open = Où ça en est, ouvert
    .about = Dans le panneau des tickets, le « Où ça en est » d'un ticket montre son texte tout de suite ; sinon il est replié sur son titre, à un clic. En plein écran il s'affiche toujours.
    .on = ouvert
    .off = replié
setting-tickets-ctrl_enter_opens = Ctrl+Entrée
    .about = Dans un nouveau ticket, Ctrl+Entrée le crée et revient là où tu étais (« Créer et sortir »), ou le crée et l'ouvre (« Créer le ticket »). Les boutons font chacun le leur, quoi que dise ce réglage.
    .on = le crée et l'ouvre
    .off = le crée et sort

## Les pages et les groupes, par leur nom

settings-title-project = Projet
settings-title-appearance = Apparence
settings-title-layout = Disposition
settings-title-ticket-list = Liste des tickets
settings-title-keyboard-shortcuts = Raccourcis clavier
settings-title-aiball = aiball
settings-title-about = À propos
settings-title-language = Langue
settings-title-sizes = Tailles
settings-title-notifications = Notifications
settings-title-mouse = Souris
settings-title-colours = Couleurs
settings-title-updates = Mises à jour
settings-title-tips = Astuces
settings-title-usage = Usage
settings-title-sessions = Sessions
settings-title-ticket-panel = Panneau des tickets
settings-title-thread = Fil
settings-title-new-ticket = Nouveau ticket
settings-title-sides = Côtés
settings-title-legend = Légende
settings-title-window = Fenêtre
settings-title-workspace = Espace de travail
settings-title-terminal = Terminal
settings-title-fixed-keys = Touches fixes
settings-title-folder = Dossier
settings-title-board = Board
settings-title-board-wide-only = Pour tout le board seulement
settings-title-per-project-only = Par projet seulement
settings-title-in-aiball-yaml = Dans .aiball.yaml

## La page

settings-heading = Réglages
settings-search-placeholder = Rechercher…
settings-modified-hint = @modified : ce que tu as changé
settings-search = Recherche : {$text}
settings-close = ✕  Échap
settings-nothing-found = Rien trouvé. Cherche un nom, un mot qu'il dit, une touche (ctrl+shift+b), une valeur — ou @modified.
settings-default = par défaut
settings-default-back = Défaut
settings-back-to = revenir à {$value}
settings-own-themes = Tes propres thèmes (au format de thème de gpui-component) vont dans ~/.config/tvty/themes/ : ils s'affichent ici la prochaine fois que cette page s'ouvre.

## Les choix

setting-language-auto = Automatique ({$lang})
settings-ask = Demander
settings-order-recent = Le plus récent d'abord
settings-cursor-blink = Clignotant
settings-cursor-steady = Fixe
settings-cursor-program = Comme le programme le demande
settings-usage-ratio = Usage face à l'attendu (×1,3)
settings-usage-points = Écart en points (+12 pts)
settings-usage-wall = Temps avant le mur (mur 1h40)
settings-order-alpha = Alphabétique
settings-order-yours = Le tien (glissé dans la liste)
settings-quit-stop = Les arrêter
settings-quit-keep = Les laisser tourner
settings-start-restart = Les redémarrer comme elles étaient
settings-start-fresh = Les redémarrer à neuf
settings-start-leave = Les laisser arrêtées
settings-focus-click = Clic
settings-focus-hover = Survol
settings-same-as-window = Comme la fenêtre
settings-dark = Sombres
settings-light = Clairs
settings-theme-dark = SOMBRES
settings-theme-light = CLAIRS
settings-theme-window = Fenêtre
settings-theme-terminal = Terminal
settings-theme-opacity = Opacité des terminaux

## Les côtés

settings-side-open = ouvert
settings-side-folded = replié
settings-side-said = {$state} · {$width} px
settings-side-list = Liste des projets
settings-side-list-about = Par-dessus le terminal, à gauche. Tire son bord pour la redimensionner ; sa poignée la replie.
settings-side-panel = Panneau des tickets
settings-side-panel-about = À droite du terminal. Tire son bord pour le redimensionner ; sa poignée le replie.
settings-widths = Largeurs
settings-widths-about = Retour aux valeurs par défaut : une liste de 290 px, un panneau d'un tiers de la fenêtre.
settings-reset = Réinitialiser

## À propos

settings-about-version = Version
settings-about-aiball-at = aiball à
settings-about-acting-as = Agit comme
settings-about-bus = Bus d'aiball
settings-about-bus-said = version {$version}, en tant que {$who}
settings-about-bus-said-kind = version {$version}, en tant que {$who} ({$kind})
settings-about-not-connected = pas connecté
settings-about-live = Board en direct
settings-about-not-subscribed = pas abonné
settings-about-subscriptions = { $count ->
    [one] {$count} abonnement sur le bus
   *[other] {$count} abonnements sur le bus
}
settings-about-theme = Thème
settings-about-terminal-theme = Thème des terminaux
settings-about-the-windows = celui de la fenêtre
settings-about-config = Réglages, raccourcis, thèmes
settings-about-state = Disposition et espace de travail
settings-about-fonts = Polices
settings-about-what = Un terminal natif pour travailler avec beaucoup d'agents de code IA à la fois : leurs terminaux regroupés par projet, les tickets de chaque projet à côté — l'agent demande, tu décides, il continue. Construit sur aiball, qui fait tourner les boucles des agents et leur board.
settings-about-github = GitHub ↗
settings-about-aiball = aiball sur GitHub ↗
settings-about-license = Licence MIT ↗
settings-about-footer = Terminal Velocity (tvty). Licence MIT. Thèmes fournis : voir themes/README.md.

## La config d'aiball

settings-aiball-failed = config d'aiball
settings-aiball-unreadable = la config d'aiball n'a pas pu être lue : {$error}. Choisis de nouveau la portée, en haut à gauche, pour réessayer.
settings-aiball-reading = Lecture de la config d'aiball…
settings-aiball-board = La config propre au board : ce que chaque projet reçoit sauf s'il dit autrement. Choisis un projet, en haut à gauche, pour la sienne.
settings-aiball-project = Ce que {$project} dit par-dessus la config du board ; ↺ rend une clé à la valeur du board.
settings-aiball-in-file = aiball lit ces clés dans le .aiball.yaml de chaque projet, pas dans sa config : change-les dans ce fichier.
settings-aiball-global-only = aiball déclare ces clés pour tout le board : une valeur pour tous les projets, réglée dans Global.
settings-aiball-project-only = aiball déclare ces clés par projet seulement : pas de valeur pour tout le board ; choisis un projet au-dessus pour les régler.
settings-aiball-open-global = Ouvrir Global
settings-aiball-protected = 🔒 protégée
settings-aiball-set-in-file = réglée dans .aiball.yaml
settings-aiball-board-wide = pour tout le board : réglée dans Global
settings-aiball-per-project = par projet
settings-aiball-from-board = {" "}Depuis la config du board.
settings-aiball-board-back = Board
settings-on = activé
settings-off = désactivé

## La légende de la liste des tickets

settings-legend-intro = La liste des tickets telle qu'aiball la calcule pour toi : la bande, à qui le tour et le glyphe d'état viennent d'aiball ; la bande latérale vient de tvty.
settings-legend-order = Ordre
settings-legend-band-moderate = le ticket, ou des commentaires dessus, attendent la modération
settings-legend-band-decide = un plan, une résolution, une fermeture sans correction ou une escalade attend ta décision
settings-legend-band-working = un agent le tient ou est sur une étape
settings-legend-band-open = ouvert, rien d'urgent
settings-legend-band = {$what} ; le plus récent d'abord
settings-legend-glyph = Glyphe d'état — en couleur quand il t'attend, atténué sinon
settings-legend-always-blue = {$what} — toujours bleu
settings-legend-always-amber = {$what} — toujours ambre
settings-legend-always-red = {$what} — toujours rouge
settings-legend-stripe = Bande latérale — à qui le tour
settings-legend-stripe-solid = une décision t'attend, et c'est le dernier message
settings-legend-stripe-dashed = une décision t'attend, mais la discussion a continué après
settings-legend-stripe-yours = ton tour : un agent t'a répondu
settings-legend-stripe-waiting = fine et pointillée : ton mot est le dernier, tu attends
settings-legend-stripe-none = pas de bande : la balle est chez l'agent
settings-legend-stripe-tape = ruban de chantier : le ticket attend la modération, rien n'avance tant que tu ne le laisses pas passer
settings-legend-rest = Le reste
settings-legend-unread = commentaires, en bleu : du nouveau pour toi, non lu (son titre en gras aussi)
settings-legend-others-spoke = clair, sa pointe à gauche : quelqu'un d'autre a parlé en dernier
settings-legend-you-spoke = discret, sa pointe à droite : tu as parlé en dernier
settings-legend-pending = commentaires en attente de modération
settings-legend-agent = agent
settings-legend-holder = l'agent qui le tient ; la flamme quand il a été actif récemment
settings-legend-critical = le ticket critique du projet : il retient 3 tickets ouverts
settings-legend-priority = priorité : urgente, haute, basse (normale n'affiche rien)
