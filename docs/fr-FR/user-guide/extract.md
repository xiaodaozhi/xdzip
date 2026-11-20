# Extraire tous les fichiers ou seulement la sélection

[English](../../user-guide/extract.md) | [简体中文](../../zh-CN/user-guide/extract.md) | [繁體中文](../../zh-TW/user-guide/extract.md) | [日本語](../../ja-JP/user-guide/extract.md) | [한국어](../../ko-KR/user-guide/extract.md) | Français | [Deutsch](../../de-DE/user-guide/extract.md) | [Español](../../es-ES/user-guide/extract.md) | [Português (Brasil)](../../pt-BR/user-guide/extract.md) | [Русский](../../ru-RU/user-guide/extract.md)

<a name="xdzip-from-the-archive-window"></a>

## Depuis la fenêtre de l’archive

1. Ouvrez l’archive. Le bouton principal de la barre d’outils extrait la sélection si elle existe, ou toute l’archive si aucun élément n’est sélectionné. La flèche à côté du bouton permet de choisir **Tout extraire** ou **Extraire la sélection**.
2. Vérifiez les éléments à extraire et choisissez une destination accessible en écriture. Vous pouvez activer la création d’un dossier portant le nom de l’archive si cette option est disponible.
3. Choisissez comment traiter les conflits. **Demander** met l’opération en pause dans l’attente d’une décision. **Ignorer** conserve le fichier existant. **Renommer** enregistre un fichier distinct. **Remplacer** écrase le fichier existant si les conditions le permettent, après vérification.
4. Lancez l’extraction et consultez le résultat. En cas d’échec ou d’annulation, les fichiers extraits auparavant peuvent rester sur place. Consultez les détails pour savoir ce qu’il reste à traiter avant de réessayer. [Vérifications, tâches et résultats](checks-and-tasks.md)

Le menu propose également **Vérifier les métadonnées avant extraction**. Cette vérification porte sur une sélection comprenant au plus 128 entrées réelles de l’archive. Elle signale par exemple les noms dangereux, les méthodes non prises en charge et les limites déclarées. Pour obtenir ce rapport, sélectionnez explicitement des entrées. Un dossier implicite affiché pour la navigation peut quand même être extrait si le format le permet. La vérification préalable ne lit pas le contenu et ne donne aucune autorisation d’écriture. [Vérification préalable et limites](../engineering/extraction-preflight.md)

Les grandes archives ZIP indexées sur disque permettent d’extraire une sélection de fichiers ordinaires dans les limites de la tâche. Si l’archive entière dépasse la limite d’extraction, réduisez la sélection. Si la source est modifiée ou remplacée, rouvrez-la et refaites la sélection. [Limites des grandes archives](../engineering/large-index.md)

<a name="xdzip-from-the-file-manager"></a>

## Depuis le gestionnaire de fichiers

Sous Windows 11, choisissez **Afficher plus d’options** pour accéder au menu classique. L’édition du Microsoft Store peut aussi afficher les actions dans le menu moderne. Sous Ubuntu, utilisez l’extension Nautilus. Sous macOS, activez l’extension Finder dans les réglages système ou utilisez le service Finder enregistré. Si le menu est indisponible, ouvrez l’archive dans xdzip et lancez l’extraction depuis sa fenêtre.

Depuis le gestionnaire de fichiers, vous pouvez extraire chaque archive dans son dossier actuel, créer un dossier distinct pour chacune, laisser xdzip déterminer le dossier ou choisir vous-même une destination. Le mode automatique évite d’ajouter un dossier parent uniquement si le résultat validé contient déjà exactement un dossier de premier niveau. Le traitement par lot affiche un résultat pour chaque archive et se poursuit après un échec individuel. L’annulation interrompt les opérations suivantes et demande l’arrêt de celle en cours. [Actions et réglages du gestionnaire de fichiers](../engineering/shell-actions.md)
