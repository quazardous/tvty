# L'interface de tvty, en français. Mêmes fichiers et mêmes ids que
# l'anglais (assets/locales/en/), qui fait référence.
# Raccourcis clavier (src/keymap.rs, src/options.rs, src/shell.rs). Les
# mots d'une commande vont par son nom.

## Ce que fait chaque commande

keys-notice-open = Aller à la notification la plus récente : le terminal de l'agent, son ticket
keys-slider-next = Carrousel : les groupes en piles, le plus récent d'abord ; lâcher Ctrl ouvre
keys-slider-back = Carrousel, à rebours
keys-tab-next = L'onglet d'après, dans le groupe affiché
keys-tab-back = L'onglet d'avant, dans le groupe affiché
keys-gallery-toggle = Galerie de tous les terminaux ; taper filtre, les flèches déplacent, Entrée ouvre
keys-panel-toggle = Replier ou déplier le panneau des tickets
keys-sidebar-toggle = Replier ou déplier la liste des projets
keys-afk-cycle = Le mode AFK de l'agent affiché : auto → retenir 10 min → retenir, en vigueur 3 s après le dernier appui (un terminal sans agent reçoit F9)
keys-sessions-filter = Filtrer les sessions : taper, Entrée ouvre, les flèches déplacent, Échap efface
keys-ticket-goto = Aller à un ticket : son numéro ou le lien #C. d'un commentaire, Entrée l'ouvre (le #… de la barre de titre)
keys-options-toggle = Réglages
keys-app-quit = Quitter tvty, comme en fermant sa fenêtre (les loops qui tournent : arrêtées ou gardées, au choix)
keys-help-menu = Aide : à propos de tvty, sa documentation, les nouveautés, un redémarrage
keys-window-fullscreen = La fenêtre en plein écran, ou retour
keys-font-bigger = Police des terminaux plus grande
keys-font-smaller = Police des terminaux plus petite
keys-font-reset = Police des terminaux à sa taille par défaut
keys-ticket-new = Un nouveau ticket (aussi + dans le panneau et la liste plein écran)
keys-project-megaphone = Le 📢 du projet affiché : sa consigne permanente et son focus de réveil
keys-list-full = La liste des tickets, en plein écran
keys-theme-next = Thème de couleurs suivant
keys-debug-inspector = L'inspecteur de GPUI : choisir un élément, voir son id et où il est fait (versions de debug)
keys-terminal-copy = Copier la sélection
keys-terminal-paste = Coller le presse-papiers
keys-terminal-tab = Tab, au programme (pas le focus à l'élément suivant)
keys-terminal-back-tab = Maj+Tab, au programme

## Les touches qui ne sont pas des commandes

keys-fixed-esc = Fermer la galerie, le carrousel, la liste des thèmes, les réglages, la liste plein écran
    .keys = Échap
keys-fixed-arrows = Dans le carrousel et la galerie : aller à la carte vue là
    .keys = Flèches
keys-fixed-select = Dans un terminal : sélectionner du texte · un mot · une ligne — copié dans la sélection primaire
    .keys = Glisser · double clic · triple clic
keys-fixed-middle-click = Dans un terminal : coller la sélection primaire
    .keys = Clic du milieu
keys-fixed-right-click = Dans un terminal : Copier, Coller — sur un lien, Ouvrir le lien, Copier le lien
    .keys = Clic droit
keys-fixed-ctrl-click = Dans un terminal, sur un lien : l'ouvrir
    .keys = Ctrl+clic
keys-fixed-wheel = Faire défiler l'historique
    .keys = Molette

## La page

keys-intro = Chaque raccourci est une commande, liée dans un contexte : Terminal quand un terminal a le focus, Fenêtre partout ailleurs. La liaison la plus profonde gagne ; une touche liée nulle part va au programme du terminal.
keys-how = Clique une touche pour la changer, + pour en ajouter une, × pour l'enlever. Gardé dans {$file}, que tu peux aussi modifier.
keys-reset-all = Tout réinitialiser
keys-not-applied = Pas appliqué : {$error}
keys-context-window = Fenêtre — partout dans tvty, pages plein écran comprises
keys-context-workspace = Espace de travail — les terminaux, la liste des sessions et le panneau à côté
keys-context-terminal = Terminal — quand un terminal a le focus
keys-press = Appuie sur une touche… (Échap abandonne)
keys-typing-refused = {$key} sert à taper : dans un terminal, elle reste au programme
keys-remove = enlever cette touche
keys-the-program = le programme
keys-masked = {$what} — masqué dans un terminal par {$by}
keys-no-key = aucune touche
keys-conflict = {$key} lance {$other} : la prendre pour {$name} ?
keys-replace = Remplacer
keys-cancel = Annuler
keys-freed-terminal = Rendue au programme du terminal
keys-freed-focus = Rendue à ce qui a le focus
