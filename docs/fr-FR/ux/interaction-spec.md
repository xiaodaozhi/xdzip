<a name="xdzip-additional-interaction-and-copy-specification"></a>

# Interactions et libellés complémentaires

[English](../../ux/interaction-spec.md) | [简体中文](../../zh-CN/ux/interaction-spec.md) | [繁體中文](../../zh-TW/ux/interaction-spec.md) | [日本語](../../ja-JP/ux/interaction-spec.md) | [한국어](../../ko-KR/ux/interaction-spec.md) | Français | [Deutsch](../../de-DE/ux/interaction-spec.md) | [Español](../../es-ES/ux/interaction-spec.md) | [Português (Brasil)](../../pt-BR/ux/interaction-spec.md) | [Русский](../../ru-RU/ux/interaction-spec.md)

La fenêtre principale et la confirmation d’extraction sont complétées par la [spécification des états](hifi-state-spec.md) pour la création, les conflits, les tâches, les résultats, les paramètres et les comparaisons de plateformes. Le [contrat de la page d’accueil](welcome-opening.md) décrit l’accueil et la progression de l’ouverture. Les maquettes et références de disposition suivent le [registre des fenêtres](../../../specs/visual/window-registry.json). Les contrats techniques et d’interaction précisent ce qui est implémenté et ce qui a été vérifié sur le bureau natif.

| Situation | Présentation et action | Suite possible |
| --- | --- | --- |
| Premier lancement | Ouvrir ou créer une archive. Accepter le dépôt de fichiers locaux. | Éviter le balayage de tous les disques et l’association automatique de tous les formats. |
| Lecture du répertoire | Nombre d’entrées lues et progression indéterminée. Total inconnu. | Autoriser la fermeture de la session. Indiquer que les résultats partiels sont incomplets. |
| Archive vide | « Cette archive est vide ». Conserver les propriétés du fichier. | Ouvrir une autre archive. Désactiver l’extraction et en expliquer la raison. |
| Recherche sans résultat | Afficher le terme et le périmètre. | Effacer la recherche. Signaler l’absence de résultats sans présenter l’archive comme vide. |
| Fonction indisponible | Indiquer le format, l’algorithme ou le chiffrement en cause. | Conserver les métadonnées lisibles et interrompre toute sortie partielle non sûre. |
| Création | Entrées, format, destination, niveau et vérification préalable. | Afficher l’erreur près du champ concerné. Préserver les fichiers existants par défaut. |
| Conflit de noms | Afficher les deux éléments, Ignorer, Renommer et Remplacer, et le périmètre des décisions suivantes. | Fermer l’invite laisse la tâche en attente. La tâche permet de rouvrir l’invite ou d’annuler. |
| Fin de tâche | Comptes de réussite, d’éléments ignorés, d’échecs et de résultats écrits. | Après extraction, ouvrir le dossier uniquement sur demande. La création peut ouvrir dans l’application une archive unique compatible ou le premier volume. Une nouvelle tentative exige un nouveau plan. |
| Échec partiel | Conserver les fichiers écrits et expliquer les échecs. | Laisser les fichiers utilisateur déjà écrits en place. |
| Quitter l’application | Nombre de tâches actives, attendre leur fin ou annuler et quitter. | Le cœur confirme le nettoyage. L’interface reste réactive pendant l’arrêt. |
| Paramètres | Thème système, clair ou sombre, densité, taille, langue, historique et zone de notification. | Appliquer dès l’enregistrement réussi, conserver les anciennes valeurs en cas d’échec. Les quotas ne sont pas configurables dans cette version. |
| Extensions de format | Le produit utilise les formats intégrés. Un [essai de SDK externe](../engineering/sdk-trial.md) vérifie séparément l’exécution en lecture seule. | Le [registre de formats](../engineering/format-registry.md) indique les opérations disponibles. |

Le bouton d’extraction suit la sélection. Son menu propose les deux périmètres autorisés par l’état actuel. La recherche couvre le dossier courant et ses sous-dossiers. Sélectionner un seul fichier compatible affiche automatiquement un [aperçu de texte ou d’image](../engineering/content-preview.md) dans le panneau visible des propriétés. Lorsque la place manque pour le panneau, il peut s’ouvrir dans une fenêtre séparée. Les grandes tailles de texte agrandissent le contenu et permettent de défiler jusqu’aux commandes de confirmation.

Le message d’erreur explique ce qui s’est passé et l’action possible. Les codes restent dans les détails. Les utilisateurs n’ont pas besoin de piles d’appels ni du jargon IPC. Affichez les noms de fichiers dangereux comme du texte et les messages des modules sans interprétation HTML. Les raccourcis, la saisie IME, le focus et les lecteurs d’écran suivent la [conception Qt Quick](../design/qt-quick-ui.md).
