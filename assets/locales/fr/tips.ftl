# L'interface de tvty, en français. Mêmes fichiers et mêmes ids que
# l'anglais (assets/locales/en/), qui fait référence.
# Les astuces (tips/<id>.md : leur anglais est celui du fichier, un test le
# vérifie). {$key} est la touche de la commande de l'astuce, {$key-font-reset}
# celle de font.reset : telles quelles, jamais traduites. **gras** et `code` restent.

tip-slider = **{$key}** montre tes projets en piles, le plus récent d'abord. Lâche Ctrl pour ouvrir celui choisi.
tip-gallery = **{$key}** montre tous les terminaux en direct, côte à côte. Tape pour filtrer, les flèches pour te déplacer, Entrée pour ouvrir.
tip-gallery-filter = Chaque vignette est en direct : regarde tous tes agents travailler à la fois, et ouvre celui qui a besoin de toi.
tip-notice = **{$key}** saute à la notification la plus récente : le terminal de l'agent, son ticket à côté.
tip-goto = **{$key}** va à un ticket : tape son numéro, ou colle le lien `#C.` d'un commentaire.
tip-full-list = **{$key}** montre tous les tickets en plein écran, le critique en haut.
tip-bulk = **Ctrl+clic** choisit des tickets, **Maj+clic** une plage : puis ferme-les, mets-les en sommeil ou marque-les tous d'un coup.
tip-afk = **{$key}** fait tourner le mode AFK de l'agent affiché : sa boucle toute seule ▶, retenue 10 minutes ‖, retenue jusqu'à ce que tu la libères ■.
tip-remote-control = **RC** s'allume dans la barre de l'agent quand son Claude est en Remote Control : suis-le et réponds-lui depuis claude.ai. `/rc` l'active.
tip-move-loop = La pastille **… · commandes** de la barre de l'agent (hôte, tmux ou psmux) : un clic prend ou laisse les commandes, ou déplace la boucle de l'autre côté.
tip-critical = Un **!** rouge marque le ticket critique : l'ouvert qui en retient le plus d'autres. Le faire avancer libère le plus de travail.
tip-references = Chaque **#N** d'un ticket est un lien, même vers un ticket d'un autre projet : un clic l'ouvre.
tip-comment-link = Clique sur la marque **#C.** d'un commentaire pour copier son lien, puis colle-le dans n'importe quel ticket ou dans la boîte « Aller à un ticket ».
tip-reply = **Ctrl+Entrée** envoie ta réponse ; **sans notifier** la poste sans prévenir personne.
tip-preview = **Écrire / Aperçu** : vois ton Markdown rendu, images comprises, avant de le poster.
tip-escape = **Échap** quitte d'abord le champ où tu tapes ; un second **Échap** ferme la page.
tip-sections = Tire le titre d'une section pour la redimensionner ; un double clic sur un titre répartit de nouveau la place.
tip-fullscreen = **{$key}** met la fenêtre en plein écran, et l'en sort.
tip-filter = **{$key}** filtre les sessions : tape, Entrée ouvre, Échap efface.
tip-new-project = **Nouveau projet…** dans le menu (l'icône de l'app, en haut à gauche) prépare un dossier pour aiball et démarre son agent.
tip-restart = **Redémarrer tvty**, dans le menu, ne redémarre que la fenêtre : les sessions de tes agents continuent de tourner.
tip-shortcuts = Chaque raccourci peut changer : ici dans **Raccourcis clavier**, ou dans `keymap.toml` dans le dossier de config de tvty.
tip-opacity = **Opacité des terminaux** sous 100 % laisse ton bureau transparaître à travers les terminaux.
tip-aiball-board = **aiball**, dans le menu, ouvre le board d'aiball dans ton navigateur : les tickets de tous les projets.
tip-font = **{$key}** grossit la police des terminaux, **{$key-font-smaller}** la rapetisse, **{$key-font-reset}** la remet comme avant.
tip-new-ticket = **{$key}** crée un nouveau ticket de n'importe où, dans le projet où tu as créé le dernier.
tip-room = **{$key}** replie le panneau des tickets, **{$key-sidebar-toggle}** la liste des projets : toute la place au terminal.
tip-themes = **{$key}** passe d'un thème de couleurs au suivant ; le **◐** de la barre de titre les liste tous.
tip-wizard = L'assistant montre ce qu'il va écrire avant d'écrire quoi que ce soit : rien ne change avant que tu le mettes en place.
tip-file-and-exit = **Créer et sortir** crée le ticket sans l'ouvrir : tu es de retour au terminal, ou à la liste complète.
tip-panel-pin = L'**épingle** dans l'en-tête du panneau des tickets : épinglé, le panneau reste à côté du terminal ; non épinglé, il passe par-dessus et le terminal garde sa largeur.
tip-followers = Les **abonnés** sont prévenus de chaque mouvement de ce ticket : ✕ en retire un, la liste déroulante en ajoute un. Un abonné en sourdine est grisé.
tip-thread-order = **⇅** retourne le fil : récents d'abord ou à la fin, dans le panneau et en plein écran chacun à sa façon.
tip-held-selection = Sélectionner du texte dans un terminal **le fige** pendant que tu copies : la sortie de l'agent attend, rien ne défile sous la souris.
tip-workspaces = Un **espace** garde un ensemble de projets et la façon dont tourne chacun de leurs agents, toute seule ▶ ou retenue ■. **+ espace** garde sous un nom ce qui tourne maintenant.
tip-workspace-open = **Ouvrir** un espace démarre ce qu'il garde et libère ce qu'il fait tourner tout seul ; **Fermer** arrête ses sessions, en laissant celles que d'autres espaces partagent.
