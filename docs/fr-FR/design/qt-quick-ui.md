<a name="xdzip-qt-quick-desktop-interface"></a>

# Interface de bureau Qt Quick

[English](../../design/qt-quick-ui.md) | [简体中文](../../zh-CN/design/qt-quick-ui.md) | [繁體中文](../../zh-TW/design/qt-quick-ui.md) | [日本語](../../ja-JP/design/qt-quick-ui.md) | [한국어](../../ko-KR/design/qt-quick-ui.md) | Français | [Deutsch](../../de-DE/design/qt-quick-ui.md) | [Español](../../es-ES/design/qt-quick-ui.md) | [Português (Brasil)](../../pt-BR/design/qt-quick-ui.md) | [Русский](../../ru-RU/design/qt-quick-ui.md)

Le bureau utilise Qt Quick pour la présentation et les interactions. Les contrôleurs C++ transmettent les intentions et maintiennent l’état des vues. Le cœur Rust interprète les archives et détient la politique des tâches et les autorisations sur les fichiers. L’[architecture du système](system-architecture.md) et le [contrat de la passerelle](qt-rust-bridge.md) décrivent les appels ci-dessous.

<a name="xdzip-windows-sessions-and-models"></a>

## Fenêtres, sessions et modèles

| Composant | Responsabilité actuelle |
| --- | --- |
| `Browser.qml` et `SessionPage.qml` | Accueil, onglets d’archives, chemin et recherche, navigation dans les dossiers, liste ou grille, propriétés et commandes. La page présente l’état actuel du contrôleur et appelle les commandes. |
| `ApplicationSessions` | Achemine les demandes d’ouverture vers des sessions indépendantes et coordonne les onglets ou fenêtres, l’accès commun aux tâches et la fermeture de l’application. Le mode d’ouverture s’applique aux prochaines demandes. Fermer une page libère la vue, et le contrôleur d’une tâche peut rester actif. |
| `BrowserController` et `ArchiveTableModel` | Maintiennent la sélection, l’état de la liste et l’identité du résultat. Les petits résultats sont intégrés comme modèle complet. Les grands ensembles ont un nombre logique de lignes, avec 1 024 lignes résidentes au maximum. Les pages manquantes sont chargées à la demande. Les lignes non chargées ne peuvent pas soumettre d’actions sur les éléments. |
| `TaskListModel` et fenêtres d’opération | Présentent des instantanés bornés, la progression, les résultats et la décision en attente. L’identité composite du bureau achemine chaque commande de tâche vers le contrôleur qui la possède. |
| Paramètres et adaptateurs de plateforme | Gèrent l’apparence, la langue, la taille du texte et le comportement de l’application. Les adaptateurs natifs gèrent la zone de notification, l’activation, les dialogues de fichiers, les associations et le transfert au système selon la plateforme. |

[Entrée du bureau](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/browser_main.cpp), [Page de session](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/ui/SessionPage.qml), [Contrôleur et modèles](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/BrowserController.h), [Sessions d’application](../engineering/application-sessions.md), [Paramètres et localisation](../ux/settings-localization.md)

<a name="xdzip-browsing-and-operation-flow"></a>

## Navigation et exécution des opérations

L’ouverture publie un instantané complet des métadonnées. Toute nouvelle recherche, tout tri ou changement de dossier porte sa propre identité de résultat. Une page obsolète ne peut pas remplacer le modèle courant. Pendant une requête, la vue affiche le dernier résultat intégré tant que le contrôleur le considère valide. Le transport du cœur envoie au plus 128 lignes et 64 KiB de noms par lot. [Données du domaine](domain-data.md), [Indexation des grandes archives](../engineering/large-index.md)

La commande d’ouverture d’un dossier permet de le parcourir. Double-cliquer sur un fichier ordinaire admissible, ou utiliser la commande d’ouverture équivalente, lance une tâche d’ouverture temporaire. Les propriétés offrent une action Aperçu distincte. Les invites de conflit et de mot de passe gardent l’identité de l’opération courante. L’accès aux tâches présente les résultats et la progression même après la fermeture de la page de navigation. [Navigation](../user-guide/browse.md), [Ouverture temporaire](../engineering/archive-opening.md), [Aperçu du contenu](../engineering/content-preview.md), [Fenêtres de tâches](../ux/task-windows.md)

Un dépôt ouvre une archive ou fournit des entrées à la création selon la cible. Les déplacements internes et les glissements vers l’extérieur admissibles préparent une tâche et passent la main à la plateforme selon ses règles. Celle-ci détermine les capacités de transfert, dont le contenu asynchrone et les écritures à destination. [Transferts d’archives](../engineering/archive-transfers.md)

<a name="xdzip-shared-controls-and-native-boundaries"></a>

## Composants partagés et limites natives

Les écrans d’opération réutilisent `OperationButton`, `OperationFooterButton`, `OperationFrame`, `OperationSelect` et `OperationSegmentedGroup`. Les composants communs définissent dimensions et interactions. Les choix discrets acheminent les actions à la souris, au clavier et via l’accessibilité vers le même modèle d’état. Les textes et données de fichiers suivent les règles de police et de traduction de la [norme de conception](../visual/design-standard.md). Les noms contenus dans les archives restent des données et s’affichent en toute sécurité.

Les fenêtres d’opération autonomes suivent le registre des fenêtres et les règles de modalité et de fermeture propres à chacune. macOS utilise les contrôles natifs de la barre de titre. Windows et Linux utilisent leurs adaptateurs système. Le [contrat des fenêtres de plateforme](../ux/platform-windows.md) définit la validation des fenêtres réelles et le [contrat d’intégration au bureau](../ux/desktop-integration.md) décrit la zone de notification et l’activation.

<a name="xdzip-design-and-verification-scope"></a>

## Périmètre de conception et de vérification

Le code et les tests décrivent le comportement du produit. Les modifications du focus natif, des menus, de l’IME, des actions d’accessibilité, de la mise à l’échelle de l’affichage ou du gestionnaire de fichiers nécessitent des preuves par plateforme. Les nouvelles configurations de bureau et les nouvelles fenêtres d’opération demandent leur propre implémentation et validation. [Processus visuel](../visual/design-workflow.md), [Ingénierie de la qualité](quality-engineering.md)
