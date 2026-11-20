<a name="xdzip-v01-interaction-prototype-and-feature-mapping"></a>

# Prototype d’interaction v0.1 et correspondance des fonctions

[English](../../ux/prototype-spec.md) | [简体中文](../../zh-CN/ux/prototype-spec.md) | [繁體中文](../../zh-TW/ux/prototype-spec.md) | [日本語](../../ja-JP/ux/prototype-spec.md) | [한국어](../../ko-KR/ux/prototype-spec.md) | Français | [Deutsch](../../de-DE/ux/prototype-spec.md) | [Español](../../es-ES/ux/prototype-spec.md) | [Português (Brasil)](../../pt-BR/ux/prototype-spec.md) | [Русский](../../ru-RU/ux/prototype-spec.md)

Statut : brouillon interactif de validation de conception. Les sources sont dans [design/html](../../../design/html/README.md), avec [index.html](../../../design/html/index.html) comme entrée. Le brouillon Figma reste une exploration historique ; les fonctions et états actuels continuent d’être affinés dans le dépôt.

<a name="xdzip-covered-flows"></a>

## Parcours couverts

| État | Entrée et sortie | Exigences / acceptation |
| --- | --- | --- |
| B01 Navigation | Ouvrir une archive exemple ; double-clic ou Entrée dans un dossier, retour par le fil d’Ariane | P01/P02/P08, AC-01/02/08 |
| B02 Recherche et sélection | Case à cocher, sélection simple, multiple avec ⌘/Ctrl, plage avec Shift et tout sélectionner ; recherche dans le dossier courant et ses descendants, ou toute l’archive à la racine | P02/P03, AC-02/03 |
| E01 Confirmer l’extraction | Bouton principal ou menu de portée ; destination, dossier de même nom, portée et politique de conflit modifiables | P03/P07, AC-03/07 |
| E02 Conflit | Entrée lors d’une collision avec la politique de demande ; continuer après ignorer, renommer automatiquement ou écraser | P03/P07, AC-03/07 |
| T01 Tâche | Démarrer → préparer → traiter → vérifier → résultat ; revenir à la navigation ou demander l’arrêt | P06/P15, AC-06/14 |
| R01 Résultat | Succès, échec partiel, annulation, changement de source ; réessayer les éléments inachevés après nouvelle confirmation | P03/P04/P15, AC-03/04/14 |
| C01 Créer | Ajouter/retirer des entrées, format/niveau/volumes/mot de passe et emplacement d’enregistrement ; confirmer d’abord les destinations existantes | P04/P05, AC-04/05 |
| S01 Paramètres | Thème, taille du texte, densité, langue et historique récent ; fermer revient à la navigation | P08/P11, AC-08/11 |
| X01 Exceptions | Archive vide, aucun résultat, chargement, corruption, fonctions non prises en charge, permissions et échec de tâche | P05/P07/P15, AC-05/07/14 |

Les « annotations de conception » dans l’en-tête du prototype donnent ces identifiants et les règles d’état actuelles. Les résultats de permission et d’espace dépendent des scénarios ; une exécution réussie du prototype ne valide pas l’acceptation technique P09/P10/P12.

<a name="xdzip-interaction-decisions"></a>

## Décisions d’interaction

<a name="xdzip-selection-and-search"></a>

### Sélection et recherche

- Chaque sélection correspond à un ID d’entrée stable, pas à un numéro de ligne triée. Changer de consultation ou de dossier efface la sélection pour éviter les sélections cachées mal comprises.
- Tout sélectionner dans la liste courante et extraire toute l’archive sont distincts. Cocher un dossier développe ses fichiers descendants en supprimant les doublons.
- Ne pas envoyer de requête pendant la composition IME ; actualiser après compositionend. Les tests d’événements simulés du navigateur ne remplacent pas l’acceptation IME native sur macOS/Windows/Linux.
- Entrée ou double-clic sur un fichier ordinaire extrait une copie temporaire et l’ouvre avec l’association système ; sur un dossier, il y entre. L’entrée des propriétés reste disponible. Le HTML ne démontre que la progression et le masquage automatique ; il n’écrit aucun fichier et n’appelle pas le système. Voir le [contrat d’aperçu](../engineering/content-preview.md) pour l’aperçu automatique Qt.

<a name="xdzip-extraction-and-conflicts"></a>

### Extraction et conflits

- Afficher le chemin final en direct ; n’ajouter qu’une fois le dossier de même nom. Un chemin exemple vide ou manifestement inaccessible en écriture bloque le démarrage ; relancer la vérification préalable après changement de destination.
- Demander par défaut en cas de conflit ; ignorer, renommer automatiquement et écraser sont disponibles. Le texte adjacent explique les conséquences de l’écrasement.
- Le focus initial du panneau de conflit est sur Ignorer ; écraser n’est pas l’action Entrée par défaut. L’application aux conflits suivants se limite à cette tâche ; les données d’exemple ne créent qu’un conflit.
- Les renommages figurent dans les résumés et détails de fichiers. Les éléments ignorés ne sont pas des échecs ; les fichiers préexistants de l’exemple ne comptent pas comme écritures achevées.

<a name="xdzip-tasks-and-results"></a>

### Tâches et résultats

- Revenir à la navigation n’annule pas la tâche ; la barre d’état peut la rouvrir.
- Demander l’annulation → arrêt en cours, nouvelle annulation désactivée → résumé d’annulation. L’extraction conserve les éléments publiés ; la création d’archive ne publie rien d’intermédiaire.
- Un disque plein arrête la tâche et conserve les réussites. Réessayer ne sélectionne que les éléments échoués ou inachevés et revient à la confirmation du plan.
- Une source modifiée exige de rouvrir l’archive, sans réutiliser silencieusement l’ancien index.
- Une tâche de création compte une archive finale ; l’annulation affiche zéro archive créée et préserve clairement l’ancienne.

<a name="xdzip-archive-creation-and-capabilities"></a>

### Création d’archive et capacités

Le prototype illustre les niveaux ZIP, l’empaquetage TAR et les options de compression GZIP. Il vérifie les champs de base des entrées, du nom et de la destination et change l’extension selon le format. Les choix actuels du produit utilisent le [registre intégré des formats](../engineering/format-registry.md), y compris les profils de création [ISO](../engineering/iso-creation.md) et [SFX Windows](../engineering/sfx-creation.md). Voir l’[interaction de mot de passe](password-extraction.md) et le [contrat des volumes](../engineering/archive-opening.md) pour les options admises.

<a name="xdzip-visuals-and-adaptation"></a>

### Visuel et adaptation

Conserver les thèmes gris chaud, bleu cobalt et graphite. La hauteur de ligne standard est 40 et compacte 32. L’accent du texte sombre est indépendant du remplissage des boutons pour garder les petits caractères lisibles. Aux faibles largeurs de bureau, les détails deviennent une superposition refermable. La revue mobile replie la barre latérale et les colonnes secondaires ; les panneaux d’opération s’adaptent aux écrans étroits et défilent.

Les interfaces d’opération sont des fenêtres autonomes selon la [spécification des fenêtres de plateforme](platform-windows.md). Le HTML les simule avec dialog : les fenêtres de décision contraignent le focus ; progression, résultats et paramètres ne sont pas modaux ; fermer tente de restaurer le focus. Fermer un conflit annule la tâche sans ignorer silencieusement ; fermer la progression ne fait que masquer les détails. Qt utilise de vraies fenêtres autonomes de premier niveau.

<a name="xdzip-theme-selection"></a>

### Choix du thème

La page des paramètres organise trois boutons radio natifs, système, clair et sombre, avec `fieldset#setting-theme` et la légende « Appearance theme ». Chaque carte utilise la ressource partagée `theme-system`, `theme-light` ou `theme-dark` et associe son libellé et son entrée. Les contrôles radio réels participent au clic sur la carte, au clavier et au focus. La sélection utilise le marqueur radio partagé et synchronise bordure et aperçu, sans remplacer le contrôle par une capture non interactive.

Ceci décrit la validation de conception HTML. Les paramètres actuels sont en mémoire ; le thème système n’est résolu qu’à sa sélection. Les changements du thème système en cours d’exécution, la persistance entre lancements ainsi que l’enregistrement immédiat et la gestion des échecs dans les paramètres Qt du produit exigent une vérification distincte.

<a name="xdzip-verification-and-remaining-boundaries"></a>

## Vérification et limites restantes

Le script de vérification du navigateur est scripts/check-html-design.py. Il vérifie clics réels, statistiques, conditions désactivées, liaison des formats, récupération, débordement mobile et fonctionnement hors ligne en fichier unique. Les rapports accompagnent le paquet du prototype exporté ; les vérifications de navigateur utilisent uniquement Chromium.

La livraison actuelle de la fenêtre principale comprend deux sessions d’archives exemples indépendantes, liste/grille, tri par champ, arbre récursif, fil d’Ariane hiérarchique, menus contextuels/déroulants/sous-menus et états des boutons. Voir le [contrat des composants et interactions](../../ux/main-window-components.md) et la [méthode de vérification](../../../developers/qa/README.md). `scripts/check-html-controls.py` les vérifie par de vraies entrées navigateur ; la [page de revue des composants](../../../design/html/components.html) utilise la même implémentation que l’interface principale.

Le HTML ne vérifie que des états d’exemple ; il ne prouve ni ordonnancement réel, ni autorisation, courses sur les chemins, CRC ou accessibilité de plateforme. Qt intègre création/extraction réelles, glisser-déposer, sélecteurs système, paramètres persistants et sessions indépendantes dans des onglets facultatifs ou plusieurs fenêtres. Les portées réelles des formats, de l’indexation d’un million d’entrées dans un ZIP ordinaire à fichier unique et de l’aperçu limité figurent respectivement dans les [moteurs de format](../engineering/native-codecs.md), la [grande indexation](../engineering/large-index.md) et l’[aperçu](../engineering/content-preview.md) ; l’acceptation est consignée dans les issues correspondantes. Les paramètres HTML ne conservent l’état de démonstration que dans la page, pas entre les lancements, et ne prouvent ni suivi natif du thème système ni acceptation d’accessibilité.

Ces limites ne changent pas les capacités visées par la conception logicielle. « Terminé » dans le prototype décrit seulement un état de tâche de démonstration et ne prouve pas la livraison d’une compression ou extraction réelle.

<a name="xdzip-current-directory-queries-and-card-revisions"></a>

## Consultations du dossier courant et révisions des cartes

L’entrée racine est « Root » et ne liste que les fichiers et dossiers directs. La recherche trouve les descendants de la portée courante et affiche les chemins complets dans l’archive. Les listes et grilles ordinaires n’affichent que le nom final ; copie et navigation conservent les chemins complets. La droite du fil d’Ariane affiche le nombre de lignes de la liste courante. Le titre de l’arbre est « Archive », centré verticalement avec le fil d’Ariane ; le survol de l’arbre n’affiche pas d’infobulle flottante.

Le survol de liste couvre toute la ligne. Les grilles ont environ 16 pixels logiques de marge horizontale ; une case de 8 pixels en haut à gauche apparaît au survol et reste visible lorsqu’elle est cochée. Les menus ouverts par un bouton sont à 4 unités de celui-ci et ont une ombre douce. Le bouton de réouverture de l’aperçu mesure 28 de côté. Les paramètres figurent dans layout-scale.json, directory-tree.json et controls.json ; la vérification réelle suit la [méthode générale](../../../developers/qa/README.md).

<a name="xdzip-settings-window"></a>

## Fenêtre des paramètres

Les paramètres gardent les titres à icône, cartes d’aperçu de thème, densité, historique récent et explication de confidentialité d’origine, et ajoutent taille du texte et langue de l’interface. Les aperçus de thème et de police peuvent être annulés ; Enregistrer et Restaurer les valeurs par défaut utilisent des brouillons. Le HTML ne conserve l’état de démonstration que dans la page courante. Dans la démonstration HTML, le choix de langue montre une configuration prenant effet au prochain démarrage de Qt ; ni la page de revue ni le contenu de l’archive exemple ne prouvent la couverture de traduction du produit. Pour le changement de langue, les ressources incluses et la persistance Qt actuels, voir [paramètres et localisation](settings-localization.md).
