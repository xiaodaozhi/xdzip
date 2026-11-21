<a name="xdzip-high-fidelity-states-and-implementation-contract"></a>

# États haute fidélité et contrat d’implémentation

[English](../../ux/hifi-state-spec.md) | [简体中文](../../zh-CN/ux/hifi-state-spec.md) | [繁體中文](../../zh-TW/ux/hifi-state-spec.md) | [日本語](../../ja-JP/ux/hifi-state-spec.md) | [한국어](../../ko-KR/ux/hifi-state-spec.md) | Français | [Deutsch](../../de-DE/ux/hifi-state-spec.md) | [Español](../../es-ES/ux/hifi-state-spec.md) | [Português (Brasil)](../../pt-BR/ux/hifi-state-spec.md) | [Русский](../../ru-RU/ux/hifi-state-spec.md)

Ce guide décrit le comportement, le focus et les actions de fermeture des fenêtres de création, d’extraction et de tâche, avec des liens vers les [maquettes des états d’opération](../../../design/hifi/README.md) et la [bibliothèque de ressources partagées](../../../design/assets/README.md). Les identifiants du tableau désignent des plans de travail de conception et ne modifient pas le protocole des tâches.

<a name="xdzip-screens-and-task-semantics"></a>

## Écrans et sémantique des tâches

| État / plan de travail | Condition d’entrée | Focus et action par défaut | Fermeture et reprise |
| --- | --- | --- | --- |
| C01 / 01 | Création, aucune tâche en cours | Premier bouton d’entrée ou nom de fichier ; valider les entrées, le nom, la destination et les capacités avant création | Abandonner le brouillon de cette fenêtre ; aucune écriture |
| E02 / 02 | Un instantané de tâche contient une invite de conflit active | Ignorer, Renommer et Écraser sont des décisions explicites. Renommer est l’action principale. Appliquer aux conflits suivants est initialement décoché | Fermer masque l’invite et laisse la tâche en attente. La rouvrir depuis la tâche ou utiliser son action Annuler pour arrêter |
| T01 / 03, 08 | Plan accepté et événement d’exécution reçu | Retour à la navigation et annulation sont des boutons ordinaires ; aucune pause inventée | Masquer uniquement la vue ; l’entrée de tâche permet de la rouvrir ; la fin en arrière-plan ne vole pas le focus |
| T01 / contrôle d’intégrité | Même composant que la progression, archive en lecture seule et étapes de vérification | Retour à la navigation, annulation du contrôle et résultat réel de la tâche ; la démonstration HTML et l’aperçu du composant Qt restent identifiés séparément | Fermer n’annule pas ; les contrôles réels rejoignent la file d’arrière-plan depuis la barre d’outils |
| R01 / 04 | La tâche a des éléments publiés et inachevés | Réessayer sélectionne uniquement les éléments inachevés et revient à la confirmation du plan | Fermer la vue ne supprime pas les fichiers déjà validés |
| S01 / 05 | Ouvrir les paramètres et sélectionner l’un des six onglets | Toutes les modifications sont accessibles au clavier ; elles prennent effet immédiatement après réussite et les adresses doivent être validées | Fermer conserve les paramètres appliqués ; un échec conserve les valeurs précédentes |
| R02 / 06 haut | Réception de l’unique état terminal de réussite | L’extraction peut ouvrir le dossier cible sur choix explicite ; la création ouvre les résultats pris en charge uniquement dans l’application, jamais automatiquement un dossier système. Zéro échec utilise un état neutre | Fermer les détails sans répéter la notification de fin |
| T02 / 06 milieu | Annulation acceptée, nettoyage inachevé | L’annulation répétée est désactivée ; le retour à la navigation reste possible | La vue peut être masquée ; « Annulé » ne peut pas être affiché prématurément |
| R03 / 06 bas | Réception de l’état terminal d’annulation | Conserver les éléments publiés ; si une nouvelle tentative est permise, replanifier le périmètre inachevé | Fermer n’annule pas les éléments publiés |
| C02 / 07 | La maquette illustre une archive de sortie existante | Un conflit réel de création suit les règles de nom sûr et de publication de la tâche active | Fermer l’invite laisse la tâche en attente. Renommer, Remplacer ou Annuler explicitement suit le résultat de la tâche |

« Utiliser le nouveau nom » ne fait que proposer un nom ; le démarrage vérifie encore l’existence de la destination. La vérification du nom ne constitue pas l’autorisation finale d’écriture. Une création avec remplacement doit générer et vérifier entièrement une archive temporaire avant de revérifier les conditions de destination pour la publication. Une annulation reçue après le point de validation linéarisé affiche la fin selon le résultat du cœur, sans transformer une réussite en annulation.

<a name="xdzip-creation-capabilities-and-field-errors"></a>

## Capacités de création et erreurs des champs

- Les quatre niveaux ZIP correspondent à Store/Deflate 1/6/9, avec 2 normal par défaut. TAR et TAR.GZ peuvent être créés ; changer de format synchronise les extensions et efface le mot de passe et le découpage en volumes. TAR désactive le niveau ; les niveaux TAR.GZ 0/1/2/3 correspondent à GZIP 0/1/6/9. Le changement de format synchronise l’extension sans écraser silencieusement une cible.
- Sans entrée, avec un nom vide, une destination invalide ou une autorisation manquante, Créer est désactivé et la raison apparaît près du champ. Le texte d’erreur est accessible à la lecture d’écran, au lieu de modifier seulement la couleur de bordure.
- Le cœur traite les entrées en double et le développement des dossiers ; l’interface affiche seulement des résumés de sélection. Les icônes d’image de la création indiquent un type, sans prévisualiser l’image.
- ZIP et 7z présentent leurs réglages de mot de passe et de volumes pris en charge. 7z peut aussi chiffrer les noms de fichiers. Le registre des formats indique les capacités d’écriture réelles. [Création ZIP](../engineering/zip-creation.md), [registre des formats](../engineering/format-registry.md)

<a name="xdzip-task-data-and-exceptions"></a>

## Données de tâche et exceptions

L’exemple A contient trois fichiers : 8,4 + 6,2 + 12,8 = 27,4 MB. Avec deux réussites, 14,6 MB sont enregistrés et le PDF restant fait 12,8 MB ; les nombres de réussites, d’éléments ignorés et inachevés ne doivent pas être confondus. L’exemple B concerne une archive entière de 30 fichiers et 77,5 MB ; 46,5 MB correspondent à 60%, et non au même moment que l’exemple A.

Les pourcentages proviennent du total connu en octets et de la mesure réelle. Un total inconnu utilise une progression indéterminée sans inventer de temps restant. Réestimer la vitesse après un changement de phase ; les valeurs illustrées de 6,2 MB/s et environ 5 secondes ne sont qu’un exemple de phase stable. Les nombres de fichiers distinguent ceux terminés de ceux en cours, sans utiliser le numéro de l’entrée courante comme nombre de réussites.

Disque plein : arrêter et conserver les éléments publiés ; replanifier après libération d’espace ou changement de destination. Source modifiée : réessayer consiste à rouvrir l’archive ; les anciens index ne peuvent pas être réutilisés. Simple annulation : les éléments inachevés utilisent une couleur ambre ou neutre, sans signaler de corruption. Les codes d’erreur figurent dans les détails ; les écrans ordinaires ne montrent ni pile d’appels ni termes internes du pont.

<a name="xdzip-immediately-applied-settings"></a>

## Paramètres appliqués immédiatement

Le thème, la taille de police, la densité, la langue, le mode d’ouverture et les options de confidentialité sont enregistrés et appliqués immédiatement après réussite. Effacer l’historique récent est immédiat et ne supprime pas les fichiers d’origine. En cas d’échec, les anciens paramètres sont conservés et un message apparaît. À propos et les mises à jour figurent dans la page À propos des paramètres ; les associations de fichiers exigent une confirmation du système et reflètent les résultats réels. La page réseau enregistre les adresses validées du proxy et du miroir. Voir le [contrat des paramètres](settings-localization.md).

Les budgets de sortie par défaut de l’hôte existant sont de 8 GiB par fichier et 20 GiB par tâche ; la fenêtre de paramètres actuelle ne permet pas de modifier les quotas. Si des limites sont ajoutées ultérieurement, vérifier les entiers positifs et les dépassements, s’assurer que la limite par fichier ne dépasse pas celle de la tâche et appliquer uniquement aux nouveaux plans. Ne pas changer l’autorisation immuable des tâches actives ni contourner les contrôles d’espace.

<a name="xdzip-window-layout-and-asset-mapping"></a>

## Correspondance des fenêtres, de la disposition et des ressources

Toutes les fenêtres d’opération suivent la [spécification des fenêtres par plateforme](platform-windows.md). Les barres de titre natives macOS n’utilisent pas de boutons tricolores découpés. La couche plateforme vérifie la position et les états désactivés des boutons de titre Windows/Linux, les menus système et le déplacement. Les commandes à droite sous Linux dans les images comparatives ne constituent pas une promesse codée en dur pour chaque environnement de bureau.

Les dimensions actuelles suivent [layout-scale.json](../../../specs/visual/layout-scale.json), le [registre des fenêtres](../../../specs/visual/window-registry.json) et la [norme de conception](../visual/design-standard.md), en remplacement des premières suggestions trop grandes. Création, extraction, conflit, progression et résultats partagent une largeur de 491 ; les paramètres utilisent 612 comme différence liée à leur fonction et enregistrée. Titre : 20, corps : 13, boutons de pied : 13 et texte secondaire : 12. La hauteur suit le contenu et la limite de chaque fenêtre ; le corps défile. Les fenêtres d’opération avec pied le gardent fixe et accessible ; les paramètres n’ont pas de pied. Les paramètres proposent des tailles de police 100%/125%/150% ; ne pas déduire les dimensions des composants d’une mise à l’échelle globale ni des pixels physiques d’une capture.

| Utilisation | Ressources ajoutées | Parties partagées / dynamiques |
| --- | --- | --- |
| Titre de création, ajout d’entrées | archive-add, image-file, remove | folder, file-pdf, new |
| Conflit, échec partiel | document-warning, status-warning, info | file-pdf ; cartes de comparaison et texte dessinés par QML |
| Progression en cours | document-download, image-file, info | Barres de progression et nombres dessinés dynamiquement |
| Réussite, élément ignoré, annulation, arrêt | status-success, status-skipped, status-stopped, spinner, clock | La commande Qt fait tourner spinner |
| Paramètres | settings-tile, theme-system | Les aperçus de thème, interrupteurs et champs numériques sont des composants ; settings est une petite icône |

Les ressources résident sous `design/assets/`. `reference/` conserve les découpes aux pixels d’origine pour comparer l’apparence ; `svg/` et `png/` sont des ressources d’exécution transparentes et ne doivent pas être appelées calques d’origine. Tous les SVG utilisent une toile de 24×24 ; les PNG sont exportés en 24×24, 48×48 et 72×72, avec la même taille logique d’affichage. Les listes HTML utilisent un emplacement de 48×48 dans l’espace source ; Qt utilise environ 31,75×39,69 pixels logiques. Les icônes PDF, texte, code et Markdown partagent un centre et une hauteur visible de 22 pixels tout en conservant des largeurs de contour distinctes ; les dossiers sont centrés dans la même toile. Les SVG sombres sont des adaptations de ressources ; le comportement sombre complet exige encore des vérifications réelles de Qt et du contraste.

<a name="xdzip-directory-tree-addition"></a>

## Ajout de l’arborescence

La navigation B01 ajoute la [spécification B de l’arborescence](directory-tree.md), avec trois plans de travail formels et 26 découpes partagées. Les flèches des nœuds dépendent des sous-dossiers ; développement, dossier courant et focus clavier sont indépendants. Chargement, archives vides, fichiers à la racine, limites de navigation, échecs et chemins courants masqués sont spécifiés séparément ; le contrat de domaine existant en lecture seule ne change pas. Qt intègrent la navigation principale, les lignes de connexion et les indicateurs de branche courante masquée ; les squelettes et icônes dédiées aux limites restent des ressources de conception. Voir la spécification de l’arborescence pour le périmètre d’intégration.

<a name="xdzip-changes-and-revalidation"></a>

## Modifications et nouvelle validation

Les principaux états ont été utilisés en Qt. Les modifications ultérieures vérifient toujours les maquettes et paramètres révisés, la correspondance des commandes et fonctions, les composants partagés et les contrats de fenêtre, et effectuent la vérification des plateformes affectées. Les maquettes ou contrôles statiques seuls ne prouvent pas le fonctionnement natif ni l’accessibilité. Enregistrer les conclusions réelles dans l’issue correspondante ; cette liste ne remet pas à zéro l’acceptation historique.
