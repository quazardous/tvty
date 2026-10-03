# L'interface de tvty, en français. Mêmes fichiers et mêmes ids que
# l'anglais (assets/locales/en/), qui fait référence.
# La liste des tickets en plein écran, et ses actions groupées.

fulllist-title = Tickets — {$project}
fulllist-title-all = Tickets — tous les projets
fulllist-shown = { $count ->
    [one] {$count} affiché
   *[other] {$count} affichés
}
fulllist-new = + Nouveau ticket
fulllist-search = Chercher dans les titres…
fulllist-projects = Projets
fulllist-all-projects = Tous les projets
fulllist-bands = Bandes
fulllist-sort = Tri
fulllist-sort-activity = Dernière activité
fulllist-sort-turn = À qui le tour (bandes)
fulllist-sort-priority = Priorité
fulllist-sort-created = Création
fulllist-sort-number = Numéro
fulllist-filters = Filtres
fulllist-open = Ouverts
fulllist-all = Tous
fulllist-unread = Non lus
fulllist-reading-small = lecture…
fulllist-critical = Critique — ce qui retient le plus
fulllist-all-group = Tous
fulllist-reading = Lecture…
fulllist-none = Aucun ticket ici.

## Une ligne

fulllist-priority = priorité : {$priority}
fulllist-rejected = rejeté
fulllist-milestone = jalon {$title}
fulllist-assigned-to = assigné à {$who}
fulllist-claimed-by = pris par {$who}
fulllist-spoke-last = {$who} a parlé en dernier
fulllist-you-spoke-last = tu as parlé en dernier
fulllist-blocked = bloqué
fulllist-payload = payload
fulllist-by = par {$who} · {$date}
fulllist-hot = un agent a été actif dessus récemment

## Actions groupées

fulllist-selected = { $count ->
    [one] {$count} sélectionné
   *[other] {$count} sélectionnés
}
fulllist-select-all = Tout sélectionner
fulllist-clear = Effacer · Échap
fulllist-actions = Actions
fulllist-actions-about = Chacune agit sur les tickets choisis qui s'y prêtent : combien, à droite.
fulllist-working = En cours…
fulllist-count-of = {$count} sur {$of}
fulllist-confirm = { $count ->
    [one] {$action} {$count} ticket ?
   *[other] {$action} {$count} tickets ?
}
fulllist-cancel = Annuler
bulk-refused = , {$count} refusé(s) ({$first})
bulk-approve = Approuver
    .about = Laisse passer les tickets choisis qui attendent la modération
    .done = { $count ->
        [one] {$count} approuvé
       *[other] {$count} approuvés
    }
bulk-reject = Rejeter
    .about = Refuse les tickets choisis qui attendent la modération
    .done = { $count ->
        [one] {$count} rejeté
       *[other] {$count} rejetés
    }
bulk-close = Fermer
    .about = Ferme les tickets choisis qui sont ouverts (demande d'abord)
    .done = { $count ->
        [one] {$count} fermé
       *[other] {$count} fermés
    }
bulk-reopen = Rouvrir
    .about = Rouvre les tickets choisis qui sont fermés
    .done = { $count ->
        [one] {$count} rouvert
       *[other] {$count} rouverts
    }
bulk-mark-read = Marquer lu
    .about = Marque lus les tickets choisis qui ont du nouveau
    .done = { $count ->
        [one] {$count} marqué lu
       *[other] {$count} marqués lus
    }
bulk-mark-unread = Marquer non lu
    .about = Marque non lus les tickets choisis déjà lus
    .done = { $count ->
        [one] {$count} marqué non lu
       *[other] {$count} marqués non lus
    }
bulk-snooze = En sommeil 3 jours
    .about = Met de côté les tickets ouverts choisis pour 3 jours : ils reviennent alors
    .done = { $count ->
        [one] {$count} mis en sommeil
       *[other] {$count} mis en sommeil
    }
bulk-unsnooze = Réveiller
    .about = Ramène maintenant les tickets choisis qui sont en sommeil
    .done = { $count ->
        [one] {$count} réveillé
       *[other] {$count} réveillés
    }
bulk-step = Marquer comme étape
    .about = Marque le dernier mot des tickets ouverts choisis comme une étape : l'agent continue, rien à décider
    .done = { $count ->
        [one] {$count} marqué comme étape
       *[other] {$count} marqués comme étape
    }
bulk-link = Lier
    .about = Lie les tickets choisis : le plus récent est lié à chacun des autres
    .done = { $count ->
        [one] {$count} lié
       *[other] {$count} liés
    }
