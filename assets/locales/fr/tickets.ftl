# L'interface de tvty, en français. Mêmes fichiers et mêmes ids que
# l'anglais (assets/locales/en/), qui fait référence.
# Le panneau des tickets et ce qu'il dit d'un ticket (src/panel.rs,
# src/thread.rs, src/rowstate.rs).

## Le panneau

tickets-title = Tickets
tickets-no-project = Pas de projet aiball sur ce terminal.
tickets-none-open = Aucun ticket ouvert.
tickets-loading = Chargement…
tickets-back = ← Tickets
tickets-fold-panel = Replier le panneau des tickets
tickets-pinned = Épinglé : le panneau reste à côté du terminal. Non épinglé : il passe par-dessus
tickets-no-session = aucune session ouverte
tickets-megaphone = La consigne permanente du projet et le focus de réveil
tickets-new = + Nouveau
tickets-new-tip = Un nouveau ticket
tickets-full-list = La liste des tickets, en plein écran
tickets-sunk = enfoncé dans le backlog de {$agent} jusqu'à {$until} (dans {$left}) : sa boucle ne le remontera pas avant, sauf si le fil bouge

## Les bandes de la liste

tickets-band-moderate = À modérer
tickets-band-decide = Attend ta décision
tickets-band-working = Des agents dessus
tickets-band-open = Ouverts
tickets-band-closed = Fermés

## Les marques d'une ligne

tickets-pending-comments = { $count ->
    [one] {$count} commentaire en attente de modération
   *[other] {$count} commentaires en attente de modération
}
tickets-unread = non lu : du nouveau pour toi
tickets-you-spoke-last = tu as parlé en dernier
tickets-someone-spoke-last = quelqu'un d'autre a parlé en dernier
tickets-comments-tip = { $count ->
    [one] {$count} commentaire — {$why}
   *[other] {$count} commentaires — {$why}
}
tickets-held-by = tenu par {$holder}
tickets-held-hot = tenu par {$holder}, actif dessus récemment
tickets-critical = { $holds ->
    [one] le ticket critique du projet : il retient {$holds} ticket ouvert
   *[other] le ticket critique du projet : il retient {$holds} tickets ouverts
}
tickets-critical-quiet = , silencieux depuis {$quiet}
tickets-step-resumes = étape — l'agent reprend à {$at}
tickets-critical-said = retient {$holds}
tickets-critical-said-quiet = retient {$holds} · silencieux {$quiet}
tickets-stage-rejected = rejeté
tickets-stage-closed-resolved = fermé, résolu
tickets-stage-closed = fermé
tickets-stage-resolved = résolu
tickets-stage-blocked = bloqué
tickets-stage-snoozed = en sommeil
tickets-stage-pending = en attente
tickets-stage-open = ouvert
tickets-to-moderate = à modérer

## Ce que veut dire un glyphe

tickets-glyph-escalation = un agent escalade : il a besoin que tu agisses
tickets-glyph-plan = un plan est proposé
tickets-glyph-resolution = une résolution est proposée
tickets-glyph-wontfix = une fermeture sans correction est proposée
tickets-glyph-stalled = l'étape d'un agent s'est tue
tickets-glyph-step = un agent est sur une étape (then: continue)
tickets-glyph-rejected = le dernier plan ou la dernière résolution a été rejeté
tickets-glyph-closed-resolved = fermé, résolu
tickets-glyph-closed = fermé sans résolution

## Les décisions

tickets-kind-plan = plan
tickets-kind-resolution = résolution
tickets-kind-wontfix = fermeture sans correction
tickets-kind-escalation = escalade
tickets-proposes-plan = {$who} propose un plan
tickets-proposes-resolution = {$who} propose une résolution
tickets-proposes-wontfix = {$who} propose de fermer sans correction
tickets-proposes-escalation = {$who} escalade
tickets-decision-pending = {$noun} · en attente
tickets-decision-accepted = { $kind ->
    [plan] {$noun} accepté
   *[other] {$noun} acceptée
}
tickets-decision-rejected = { $kind ->
    [plan] {$noun} rejeté
   *[other] {$noun} rejetée
}
tickets-decision-superseded = { $kind ->
    [plan] {$noun} · remplacé
   *[other] {$noun} · remplacée
}
tickets-accept-close = Accepter → fermer
tickets-accept-wontfix = Accepter → fermer, sans correction
tickets-accept-escalation = Fait → accepter
tickets-accept-plan = Accepter → go
tickets-reject = Rejeter
tickets-approve = Approuver
tickets-decided-once-approved = décidé une fois le ticket approuvé
tickets-reject-say-why = Pour rejeter, dis d'abord pourquoi ci-dessous.
tickets-waits-moderation = Ce ticket attend la modération

## Où en est un ticket (la phrase sous son titre)

tickets-you = toi
tickets-an-agent = un agent
tickets-closed-resolved-by = Fermé, résolu par {$who}
tickets-closed-resolved = Fermé, résolu
tickets-closed-unresolved = Fermé sans résolution
tickets-yours-moderation = À toi : ce ticket attend la modération
tickets-your-proposal-plan = Ton plan attend une décision
tickets-your-proposal-resolution = Ta résolution attend une décision
tickets-your-proposal-wontfix = Ta fermeture sans correction attend une décision
tickets-your-proposal-escalation = Ton escalade attend une décision
tickets-yours-escalates = À toi : {$who} escalade — agis, puis accepte
tickets-yours-decide-plan = À toi : accepte ou rejette le plan de {$who}
tickets-yours-decide-resolution = À toi : accepte ou rejette la résolution de {$who}
tickets-yours-decide-wontfix = À toi : accepte ou rejette la fermeture sans correction de {$who}
tickets-step-quiet = l'étape de {$agent} s'est tue
tickets-on-step = {$agent} est sur une étape
tickets-step-resumes-in = {" · "}reprend dans {$span}
tickets-step-waits-on = {" · "}attend #{$ticket}
tickets-yours-answer = À toi : réponds à {$who}
tickets-holds-you-spoke = {$holder} le tient : tu as parlé en dernier
tickets-yours-nobody = À toi : personne d'autre dessus
tickets-their-turn-of = Au tour de {$holder} : tu as parlé en dernier
tickets-their-turn = À eux : tu as parlé en dernier
tickets-span-under-minute = moins d'une minute
tickets-span-minutes = {$n} min
tickets-span-hours = {$n} h

## Le fil

tickets-images = { $count ->
    [one] 🖼 image
   *[other] 🖼 {$count} images
}

tickets-where-it-stands = Où ça en est · {$who}
tickets-edit = ✎ modifier
tickets-edit-tip = Modifier le titre et le corps
tickets-newest-first = ⇅ récents d'abord
tickets-newest-last = ⇅ récents à la fin
tickets-close-full = ✕  Échap
tickets-more = ⤢ plus
tickets-comments-spoke = { $count ->
    [one] {$count} commentaire · {$who} a parlé en dernier
   *[other] {$count} commentaires · {$who} a parlé en dernier
}
tickets-unfold-all = tout déplier
tickets-fold-before-summary = replier avant le résumé
tickets-fold = replier
tickets-unfold = déplier
tickets-snoozed-until = en sommeil jusqu'au {$until}
tickets-tokens = {$count} tok
tickets-priority-tip = la priorité : un clic la change
tickets-priority-label = Priorité
tickets-rel-depends = dépend de
tickets-rel-blocks = bloque
tickets-rel-relates = lié à
tickets-rel-duplicates = doublon de
tickets-rel-duplicated = a pour doublon
tickets-rel-parent = parent de
tickets-rel-child = enfant de
tickets-answer = Répondre
tickets-in-reply = ✓ dans la réponse
tickets-step = étape
tickets-resumes = reprend
tickets-resumes-on = sur
tickets-resumes-or-on = ou sur
tickets-resumes-at = l'agent reprend à {$at}
tickets-resumes-when = l'agent reprend quand #{$ticket} bouge (une réponse, une décision, une fermeture)
tickets-or-when = ou quand #{$ticket} bouge (une réponse, une décision, une fermeture)

## Les gestes d'un commentaire

tickets-comment-actions = les gestes du commentaire
tickets-edit-comment = Modifier
tickets-delete = Supprimer
tickets-really-delete = Vraiment supprimer ?
tickets-classify-as = en {$noun}
tickets-no-decision = sans décision
tickets-not-a-step = pas une étape
tickets-a-step = une étape
tickets-resurface = Refaire surface
tickets-cancel = Annuler
tickets-save = Enregistrer

## Écrire

tickets-reply-placeholder = Répondre… (ctrl+entrée envoie)
tickets-editing = Modification de #{$id}
tickets-edit-keys = ctrl+entrée enregistre · échap remet comme avant
tickets-title-first = D'abord un titre.
tickets-nothing-to-preview = Rien à prévisualiser pour l'instant.
tickets-sending-answers = { $count ->
    [one] La réponse répond à {$count} question.
   *[other] La réponse répond à {$count} questions.
}
tickets-close = Fermer
tickets-reopen = Rouvrir
tickets-wake = Réveiller
tickets-snooze = Mettre en sommeil ▾
tickets-snooze-for = En sommeil pour
tickets-snooze-hour = 1 heure
tickets-snooze-day = un jour
tickets-snooze-week = une semaine
tickets-assign = Assigner ▾
tickets-assigned-menu = → {$who} ▾
tickets-assign-to = Assigner à
tickets-no-agent = le projet n'a pas d'agent
tickets-claim-assigns = {$agent} le tient par un claim : un clic le lui assigne
tickets-unassign-tip = désassigner : le reprend à {$holder} ; personne ne le tient, il attend qui le prendra
tickets-without-notifying = sans notifier
tickets-reply = Répondre

## La colonne des champs

tickets-group-state = État
tickets-group-fields = Champs
tickets-group-people = Personnes
tickets-group-links = Liens
tickets-group-project = Projet
tickets-group-tokens = Tokens
tickets-group-payload = Payload
tickets-lifecycle = statut
tickets-lifecycle-closed-resolved = fermé, résolu
tickets-lifecycle-closed = fermé
tickets-lifecycle-moderation = attend la modération
tickets-lifecycle-open = ouvert
tickets-snoozed = en sommeil
tickets-until = jusqu'au {$date}
tickets-field-intent = intention
tickets-field-priority = priorité
tickets-field-level = niveau
tickets-field-scope = portée
tickets-field-tags = tags
tickets-field-milestone = jalon
tickets-field-reporter = auteur
tickets-field-claimed = pris par
tickets-field-assigned = assigné à
tickets-field-followers = abonnés
tickets-field-project = projet
tickets-filed = ouvert le {$date}
tickets-lapsed = {$who} (expiré)
tickets-claim-until = {$who}, jusqu'au {$date}
tickets-muted = en sourdine : pas notifié de ce ticket, même par son rôle — ✕ le remet à ce que dit son rôle
tickets-followers-tip = Qui suit ce ticket par son propre choix (ou l'a mis en sourdine). Les propriétaires du projet sont notifiés par leur rôle : ils ne sont pas listés ici.
tickets-sub-ticket = sous-ticket
tickets-new-sub = + un nouveau
tickets-sub-ticket-of = parent
tickets-sub-tickets = sous-tickets
tickets-remove-relation = retirer ce lien
tickets-relate = lier
tickets-relate-to = à…
tickets-relate-add = + un ticket
tickets-move-to = Déplacer vers {$project}
tickets-in-out = entrée · sortie
tickets-cache = cache
tickets-cache-said = {$written} écrits · {$read} lus
tickets-payload = ce ticket porte un payload (voir l'interface web)
tickets-pick-project = un projet
tickets-pick-project-search = un projet…
tickets-pick-tag = ajouter un tag
tickets-pick-tag-search = un tag…
tickets-pick-milestone = aucun
tickets-pick-milestone-search = un jalon…
tickets-pick-reporter = l'auteur
tickets-pick-agent-or-you = un agent, ou toi…
tickets-pick-follower = en ajouter un
tickets-pick-intent-search = une intention…
tickets-pick-priority-search = une priorité…
tickets-pick-level-search = un niveau…
tickets-pick-scope-search = une portée…
tickets-pick-nobody = personne
tickets-pick-agent-search = un agent…

## Ce qu'un geste a fait (dit dans une notification)

tickets-did-reply = réponse postée
tickets-did-with-reply = {$what}, avec une réponse
tickets-did-on = {$title} — {$what}
tickets-did-comment-edited = commentaire modifié
tickets-did-tagged = tag {$name} ajouté
tickets-did-tag-removed = tag {$name} retiré
tickets-did-milestone-set = jalon fixé
tickets-did-milestone-removed = jalon retiré
tickets-did-reporter = auteur : {$name}
tickets-did-assigned = assigné à {$name}
tickets-did-unassigned = désassigné
tickets-did-follows = {$name} le suit
tickets-did-unfollows = {$name} ne le suit plus
tickets-did-intent = intention {$value}
tickets-did-priority = priorité {$value}
tickets-did-level = niveau {$value}
tickets-did-scope = portée {$value}
tickets-did-edited = titre et corps modifiés
tickets-did-related = lié à #{$target} ({$kind})
tickets-did-unrelated = lien vers #{$target} retiré
tickets-did-moved = déplacé vers {$project}
tickets-did-snoozed = mis en sommeil
tickets-did-woken = réveillé
tickets-did-accepted = décision acceptée
tickets-did-rejected = décision rejetée
tickets-did-approved = approuvé
tickets-did-rejected-moderation = rejeté en modération
tickets-did-closed = fermé
tickets-did-reopened = rouvert
tickets-did-deleted = commentaire supprimé
tickets-did-classified = commentaire classé en {$noun}
tickets-did-plain = commentaire sans décision
tickets-did-step-marked = étape marquée
tickets-did-step-unmarked = étape retirée
tickets-did-voted = vote enregistré
tickets-did-resurfaced = commentaire refait surface

## Les événements du fil

tickets-event-closed = a fermé le ticket
tickets-event-reopened = a rouvert le ticket
tickets-event-resolved = l'a marqué résolu
tickets-event-blocked = l'a signalé à décider
tickets-event-taken-over = a repris le claim
tickets-event-sub-added = a ajouté un sous-ticket{$source}
tickets-event-referenced = l'a référencé depuis{$source}
tickets-event-dependency-closed = une dépendance s'est fermée{$source}
tickets-event-dependency-rejected = une dépendance a été rejetée{$source}
tickets-event-related-closed = un ticket lié s'est fermé{$source}
tickets-event-linked = lié{$source}
