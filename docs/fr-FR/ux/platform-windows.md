<a name="xdzip-platform-windows-and-title-bars"></a>

# Fenêtres de plateforme et barres de titre

[English](../../ux/platform-windows.md) | [简体中文](../../zh-CN/ux/platform-windows.md) | [繁體中文](../../zh-TW/ux/platform-windows.md) | [日本語](../../ja-JP/ux/platform-windows.md) | [한국어](../../ko-KR/ux/platform-windows.md) | Français | [Deutsch](../../de-DE/ux/platform-windows.md) | [Español](../../es-ES/ux/platform-windows.md) | [Português (Brasil)](../../pt-BR/ux/platform-windows.md) | [Русский](../../ru-RU/ux/platform-windows.md)

Les interfaces d’opération de xdzip utilisent des fenêtres autonomes de premier niveau, et non des feuilles rattachées à la fenêtre principale ou des panneaux superposés dessinés dans celle-ci. Une fenêtre autonome n’implique ni processus distinct ni entrée de barre des tâches pour chaque fenêtre auxiliaire. Les menus déroulants et les infobulles restent des Popups légers et ne sont pas des fenêtres d’opération.

<a name="xdzip-window-contract"></a>

## Contrat des fenêtres

| Fenêtre | Relation avec la fenêtre principale | Fermer / Échap | Réduire / agrandir |
| --- | --- | --- | --- |
| Formulaire d’extraction ou de création | `OperationWindow` non modale avec propriétaire transitoire si ouverte depuis la navigation | Fermer le brouillon ou la confirmation sans soumettre de nouvelle tâche | Désactivé / désactivé |
| Invite de conflit de tâche | Invite non modale appartenant à l’hôte de la fenêtre d’opération. La tâche attend une décision | Fermer l’invite. La rouvrir depuis la tâche ou y annuler explicitement la tâche | Désactivé / désactivé |
| Invite de mot de passe, de sortie ou de redémarrage | Modale pour l’application. Le sélecteur de fichiers natif est aussi modal pour l’application et associé à l’appelant transitoire | Le mot de passe, la sortie et le redémarrage ont chacun des actions explicites de fermeture ou de confirmation | Selon les capacités de la boîte de dialogue native |
| Progression et résultat de tâche | `TaskWindow` non modale. La navigation peut continuer si aucune autre fenêtre modale n’est active | Fermer les détails et les rouvrir depuis l’entrée de la tâche. Une opération autonome conserve un accès aux tâches actives | Réduire / désactivé |
| Paramètres et À propos | Non modale. Une entrée de la zone de notification peut l’ouvrir sans fenêtre de navigation propriétaire | Fermer la vue. Les paramètres déjà enregistrés restent actifs | Désactivé / désactivé |

L’annulation d’une tâche est une commande explicite indépendante de la fermeture de sa fenêtre de progression. Une fin en arrière-plan ne prend pas le focus au premier plan : mettre à jour les détails ouverts ou l’entrée de la tâche. La sortie de l’application suit toujours la politique de sortie des tâches ; la fermeture de la fenêtre principale ne peut pas directement entraîner celle des fenêtres auxiliaires.

<a name="xdzip-platform-appearance"></a>

## Apparence selon la plateforme

- macOS : utiliser une barre de titre décorée par le système avec les boutons rouge, jaune et vert en haut à gauche ; ne pas les remplacer par des points colorés personnalisés. Les fenêtres d’opération de taille fixe gardent le bouton rouge de fermeture et désactivent la réduction et le zoom indisponibles ; les fenêtres de progression peuvent être réduites. La disponibilité réelle dépend des capacités AppKit/Qt et fait l’objet d’une acceptation sur appareil réel. La fenêtre principale conserve le redimensionnement, la réduction et le plein écran normaux.
- Windows utilise une barre de titre principale étendue dans la zone cliente avec boutons QML, cadre natif redimensionnable et ombre DWM. Le déplacement, le redimensionnement par les bords et l’ancrage utilisent les fonctions de la plateforme. Les fenêtres d’opération autonomes utilisent les décorations natives. Voir les maquettes principales Windows et Ubuntu.
- Ubuntu X11 utilise des contrôles intégrés à droite et un contour arrondi à l’état normal. Les sessions composites avec façonnage de la zone d’entrée affichent aussi une ombre douce laissant passer les clics. Les fenêtres maximisées et en plein écran ont des angles droits et remplissent la zone disponible. Le système de fenêtres gère déplacement et redimensionnement. Les fenêtres d’opération autonomes gardent les décorations système. Les autres moteurs hors du chemin X11 personnalisé utilisent les décorations normales. Wayland est hors périmètre pris en charge. Voir la maquette principale Ubuntu.

Chaque fenêtre d’opération possède son titre, sa bordure, son ombre et sa barre de titre déplaçable ; la fenêtre principale derrière elle n’est ni assombrie ni floutée comme une page web. À la première ouverture, elle est centrée dans la zone disponible de l’écran du propriétaire. Les positions restaurées doivent être ramenées dans les écrans visibles. Les barres d’outils et le contenu ne doivent pas recouvrir la barre de titre.

<a name="xdzip-qt-quick-implementation-constraints"></a>

## Contraintes d’implémentation Qt Quick

`OperationWindow` utilise une `Window` autonome avec `transientParent` facultatif et `Qt.NonModal` par défaut. `PasswordWindow` et `OperationConfirmationWindow` utilisent explicitement `Qt.ApplicationModal`. Le sélecteur natif prend la fenêtre réellement appelante comme propriétaire transitoire et ne restaure son activation que si elle reste visible. Voir le [registre des fenêtres](../../../specs/visual/window-registry.json) et les [fenêtres de tâches](task-windows.md).

Les barres de titre personnalisées Windows/Linux nécessitent de véritables fonctions de déplacement, redimensionnement et menu système de la plateforme, et non une simulation du déplacement par modification manuelle des positions QML. `WindowController` gère le cycle de vie, la propriété, les demandes de fermeture et la réouverture des vues de tâches. Le cœur possède les tâches ; détruire une vue ne peut pas détruire une tâche active. Les mises à jour d’état entre threads utilisent uniquement les événements de pont définis.

<a name="xdzip-prototype-evidence-and-limits"></a>

## Éléments du prototype et limites

Le prototype HTML simule l’apparence des fenêtres autonomes, le déplacement de la barre de titre, les interactions bloquantes ou non et la fermeture. Le sélecteur « Platform » en haut compare macOS, Windows et Linux ; ce n’est pas un paramètre du produit. Le `dialog` du navigateur reste dans une seule page et ne fournit ni véritables fenêtres natives, boutons système, réduction système, barre des tâches ni comportement multiécran. La réduction d’une fenêtre de tâche est simulée en masquant les détails.

Les commandes de plateforme de la fenêtre principale du prototype illustrent l’apparence. Le retour de la confirmation d’écrasement vers la création réinitialise encore le formulaire d’exemple ; l’implémentation du produit doit préserver le brouillon. Ces limites ne constituent pas une preuve d’acceptation native achevée.

<a name="xdzip-real-device-acceptance-criteria"></a>

## Critères d’acceptation sur appareil réel

1. Chaque fenêtre d’opération est une fenêtre de plateforme autonome correctement associée à sa fenêtre principale, sans feuille ni superposition de type page web.
2. L’ordre des boutons colorés ou à droite, leur état désactivé et les demandes de fermeture correspondent aux capacités de la plateforme et de la fenêtre ; titres et boutons système ne se chevauchent pas aux différentes tailles de police et échelles.
3. Déplacement, limites d’écran, plusieurs écrans, échelles 125%/150%/200% et thèmes clair/sombre fonctionnent correctement.
4. Fermer les détails laisse la tâche active et son entrée permet de les rouvrir. Fermer une fenêtre de conflit laisse la tâche en attente jusqu’à la réouverture de l’invite ou l’annulation explicite.
5. Modalité du propriétaire, restauration du focus, navigation au clavier, titres pour lecteurs d’écran et raccourcis système fonctionnent ; les fenêtres d’archives sans rapport ne sont pas inutilement bloquées.
6. Vérifier séparément le menu système et l’ancrage Windows et Linux X11 ; Wayland est hors du périmètre initial. Les conclusions Qt sur appareil réel existantes figurent dans [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5). Cette liste sert à revérifier ultérieurement les points affectés et ne réinitialise pas l’acceptation achevée.
