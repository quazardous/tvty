# La config d'aiball, telle que Réglages > aiball la montre : par sa clé, stable
# (une clé que tvty ne connaît pas montre l'anglais d'aiball). Les points d'une
# clé deviennent des tirets.

aiball-group-autopoll = Autopoll
aiball-group-backlog = Backlog
aiball-group-claude-loop = Boucle Claude
aiball-group-defaults = Par défaut
aiball-group-earn = Gains
aiball-group-on-repetitive-denied = Refus répétés
aiball-group-rules = Règles
aiball-group-steps = Étapes
aiball-group-tickets = Tickets
aiball-group-updates = Mises à jour
aiball-group-wait-credit = Crédit d'attente

aiball-config-updates-check = Vérifier les mises à jour
    .about = true (default) = au démarrage du démon, et quand on le demande, il lit la dernière version d'aiball sur GitHub pour que la barre système, l'extension GNOME et `aiball version` puissent dire qu'une mise à jour est sortie. false = aucun appel sortant.
aiball-config-tickets-defaults-priority = Priorité par défaut
    .about = La priorité donnée à un nouveau ticket créé sans priorité explicite. Règle une valeur globale ; un projet peut la remplacer.
aiball-config-tickets-defaults-broadcast-new = Diffuser les nouveaux tickets
    .about = Activé, un nouveau ticket sans portée explicite est marqué diffusé (les abonnés du projet sont notifiés). Déclaré ; appliqué quand son consommateur arrivera.
aiball-config-tickets-rules-summary-max = Résumé max (caractères)
    .about = Le plus long summary_until qu'un agent peut écrire sur un commentaire. Plus long, il est refusé avec une explication et rien n'est posté ; il n'est jamais tronqué. Les humains en sont exemptés. 0 = pas de limite.
aiball-config-tickets-rules-require-then = then: ou handback exigé
    .about = Activé, le commentaire d'un agent sans then: doit fixer handback (true : il rend le ticket, false : il le garde), sinon il est refusé avec une explication et rien n'est posté. Les humains en sont exemptés.
aiball-config-tickets-rules-require-commits = Commits exigés
    .about = Activé, le commentaire d'un agent doit dire quels commits il livre (commits: ["<sha>"]) ou qu'il n'en livre aucun (commits: null ou "none"), sinon il est refusé avec une explication. Un client antérieur au champ est averti au lieu d'être refusé, jusqu'à ce qu'il se reconnecte. Les humains, la fermeture et la réouverture en sont exemptés.
aiball-config-tickets-steps-stale = Étape au point mort après
    .about = Une étape (then: continue) sans rien après elle pendant ce temps est signalée dans l'inbox : le travail qu'elle annonçait s'est tu. 0 = jamais signalée.
aiball-config-tickets-steps-hot = Minutes où une étape garde son ticket en tête du backlog
    .about = Après qu'un agent poste une étape (then: continue), son ticket mène le backlog de cet agent — juste après les événements, devant tous les autres tickets — pendant ce temps, compté depuis l'étape. Passé ce délai, le ticket se classe comme les autres. Cela ne change que l'ordre ; la marque « chaud » visible garde sa propre règle. 0 = une étape n'a aucune priorité.
aiball-config-tickets-steps-max-wait = Attente max d'une étape
    .about = Le plus qu'un agent peut mettre dans resume_on.timer sur une étape (then: continue). Une attente plus longue est refusée, avec cette limite dans la raison — au-delà, ce n'est plus une étape qui attend un job : rends le ticket, ou propose un plan.
aiball-config-tickets-wait-credit-enabled = Crédit d'attente
    .about = true (default) = le resume_on.timer d'une étape dépense le crédit d'attente d'un agent, gagné par preuve de travail. false = les attentes sont gratuites et sans plafond (toujours au plus tickets.step_after_max_minutes), rien n'est gagné, dépensé ni remboursé, et réponses et réveils ne parlent pas de crédit.
aiball-config-tickets-wait-credit-refund = Crédit : rembourser un retour anticipé
    .about = true (default) = un agent qui reparle sur un ticket avant la fin de l'attente de son étape récupère le reste de cette attente. false = une attente est dépensée en entier dès qu'elle est déclarée.
aiball-config-tickets-wait-credit-earn-commit-max-age = Crédit : âge max d'un commit qui rapporte
    .about = Un commit cité dont la date est plus ancienne que ceci ne rapporte rien : le crédit récompense le travail frais.
aiball-config-tickets-wait-credit-earn-commits-per-comment = Crédit : commits comptés par commentaire
    .about = Le plus de commits qu'un commentaire peut citer pour du crédit ; ceux au-delà ne rapportent rien, et la réponse le dit.
aiball-config-tickets-wait-credit-start = Crédit d'attente de départ
    .about = Chaque agent commence chaque projet avec ce crédit d'attente, pour qu'un nouvel agent puisse attendre un premier build. Le crédit se dépense par les minuteurs d'étape (resume_on.timer) et se gagne par preuve de travail.
aiball-config-tickets-wait-credit-max = Crédit d'attente max d'un agent
    .about = Un solde ne dépasse jamais ceci : ce qui le ferait dépasser n'est pas crédité, et un solde déjà au-dessus est ramené. 0 = pas de plafond.
aiball-config-tickets-wait-credit-floor = Attente garantie, même sans crédit
    .about = À court de crédit, l'attente d'une étape est plafonnée au solde mais jamais sous ceci, pour qu'un agent sans crédit ne revienne pas en boucle. Elle ne fait jamais passer le solde sous zéro. Une étape qui demande 0 (continuer tout de suite) est toujours accordée.
aiball-config-tickets-wait-credit-earn-resolved = Crédit gagné : ticket résolu, avec commit
    .about = Gagné une fois par ticket par l'agent dont la résolution a été acceptée, s'il a cité un commit sur ce ticket (commits: [...]) avant sa fermeture.
aiball-config-tickets-wait-credit-earn-resolved-no-commit = Crédit gagné : ticket résolu, sans commit
    .about = Gagné une fois par ticket par l'agent dont la résolution a été acceptée sans avoir cité de commit sur ce ticket : une résolution sans code vaut moins.
aiball-config-tickets-wait-credit-earn-wontfix = Crédit gagné : fermé sans correction
    .about = Gagné une fois par ticket par l'agent dont la fermeture sans correction (wontfix) a été acceptée.
aiball-config-tickets-wait-credit-earn-lines-per-minute = Lignes changées par minute de crédit
    .about = Un commit qu'un agent cite dans une réponse (commits: [...]) rapporte une minute par tant de lignes changées, lues dans le checkout de l'agent. Une fois par commit.
aiball-config-tickets-wait-credit-earn-commit-max = Crédit max par commit
    .about = Le plafond de ce que rapporte un seul commit, quelle que soit la taille de son diff.
aiball-config-tickets-wait-credit-earn-commit-min = Crédit min par commit
    .about = Ce que rapporte un commit cité avec au moins une ligne changée, si petit soit son diff : un court correctif aussi est du travail. 0 = seul le taux par ligne compte.
aiball-config-tickets-backlog-claim-protect = Minutes où le claim d'un agent actif est protégé
    .about = Combien de temps un claim tient face au claim d'un autre agent, compté depuis la dernière action de son détenteur sur le ticket — y travailler garde la protection en vie. Le claim d'un autre agent dans ce délai est refusé ; au-delà, le ticket peut être repris, et le fil le note. Une assignation l'emporte toujours sur un claim. 0 = pas de protection.
aiball-config-tickets-backlog-rest = Repos d'un ticket après un réveil de backlog
    .about = Après qu'un réveil de backlog a nommé un ticket, combien de temps il reste hors des prochains réveils de backlog de l'agent tant que personne d'autre ne bouge dessus. Un ticket bloqué se repose `blocked_multiplier` fois plus longtemps, un ticket dont la dernière action est une étape seulement `after_step`. Une boucle lancée avec CL_BACKLOG_COOLDOWN_SEC applique cette valeur à la place. 0 = pas de repos.
aiball-config-tickets-backlog-depth = Profondeur du backlog qui réveille
    .about = Le palier de backlog le plus profond pour lequel un réveil peut nommer un agent, quand il n'a aucun événement à lire. Les tickets critiques, chauds et actionnables le réveillent toujours. followup : aussi un ticket où quelqu'un a répondu mais que la décision en attente de l'agent retient. waiting : aussi un ticket où l'agent a parlé en dernier et où rien n'a bougé depuis. blocked : aussi un ticket retenu par une dépendance ouverte (il revient de toute façon quand ce qui le bloque est fermé). Un ticket sous ce réglage reste dans le backlog, affiché ; il ne réveille simplement personne.
aiball-config-tickets-backlog-blocked-multiplier = Repos multiplié des tickets bloqués
    .about = Combien de fois plus longtemps un réveil de backlog garde un ticket BLOQUÉ (retenu par un depends_on ouvert) hors du pool de réveil, par rapport aux autres tickets. Il doit remonter pour ne pas être oublié, mais rien ne bouge dessus entre deux réveils. 1 = même repos que les autres.
aiball-config-tickets-backlog-after-step = Repos après une étape
    .about = Combien de temps un réveil de backlog garde un ticket hors du pool de réveil quand sa dernière action est une étape (then: continue), au lieu du repos entier. Court exprès : l'étape dit qu'il y a du travail maintenant, et la pause laisse seulement la file tourner. 0 = jamais enfoncé.
aiball-config-claude-loop-on-repetitive-denied-threshold = Refus avant que la boucle réponde
    .about = Combien d'appels d'outils le système de permissions de Claude Code doit refuser à un agent dans la dernière heure avant que sa boucle envoie le prompt ci-dessous (ou la sortie de la commande). Rien n'est envoyé tant que les deux sont vides.
aiball-config-claude-loop-on-repetitive-denied-max-per-hour = Prompts par heure, au plus
    .about = Au plus ce nombre de prompts par heure pour des refus répétés, pour qu'un cycle refus, prompt, refus s'arrête. 0 = jamais envoyé.
aiball-config-claude-loop-on-repetitive-denied-prompt = Prompt sur refus répétés
    .about = Tapé tel quel dans le prompt de Claude, comme un réveil et seulement quand un réveil le pourrait (pas de retenue AFK, zen, frappe, limite d'usage, démarrage ni Claude occupé). Vide = rien (par défaut).
aiball-config-claude-loop-on-repetitive-denied-command = Commande qui écrit le prompt
    .about = Une commande shell lancée sur la machine de la boucle, dans son dossier, pendant 10 s au plus. Elle reçoit les refus en JSON sur stdin (agent, project, cwd, tool, reason, last_hour, recent) ; ce qu'elle affiche est le prompt. Elle l'emporte sur le prompt ci-dessus. Vide = aucune.
aiball-config-autopoll-volatile = Autopoll : rappels uniques
    .about = true = notifier seulement quand un ping strictement plus récent arrive (pas de rappels selon le temps). false (default) = un rappel persistant revient après throttle_seconds.
aiball-config-autopoll-throttle = Autopoll : cadence des rappels
    .about = Cadence des rappels en secondes, ignorée quand volatile=true. 0 = à chaque Stop (envahissant). Les nouveaux pings et nouveaux tickets ouverts passent outre.
aiball-config-autopoll-recent-tickets = Autopoll : titres récents inclus
    .about = Jusqu'à N titres de tickets récents non lus dans la raison du hook, pour que l'agent sache ce qui l'attend avant de vider la file. 0 = le nombre seulement.
aiball-config-autopoll-backlog = Autopoll : le backlog déclenche
    .about = true (default) = les tickets ouverts dans la portée déclenchent des notifications même sans ping non lu. false = contexte seulement (affichés dans la raison, jamais déclencheurs).
aiball-config-autopoll-tone = Autopoll : ton
    .about = hint = poli, facile à ignorer. directive (default) = nomme l'action explicitement. imperative = dernier recours si l'agent persiste à demander la permission.
aiball-config-assign-window-sec = Durée d'assignation
    .about = Combien de temps une assignation ou un claim reste actif avant que le ticket retourne au pool commun.
aiball-config-hot-window-sec = Durée « chaud »
    .about = À quel point un mouvement doit être récent pour garder un ticket chaud dans l'inbox.
aiball-config-upstream-transport = Upstream : transport
    .about = Comment partent les appels upstream (GitHub / GitLab) : gh (les identifiants du CLI), http, ou auto (gh quand il marche). Le transport propre d'un couplage l'emporte.
aiball-config-upstream-sync = Upstream : suivi
    .about = pull (default) = un dépôt couplé est suivi et ses changements annoncés ; off = lié seulement. Le sync propre d'un couplage l'emporte.
