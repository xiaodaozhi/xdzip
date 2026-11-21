# Arborescence B : conception, ressources et intégration

[English](../../ux/directory-tree.md) | [简体中文](../../zh-CN/ux/directory-tree.md) | [繁體中文](../../zh-TW/ux/directory-tree.md) | [日本語](../../ja-JP/ux/directory-tree.md) | [한국어](../../ko-KR/ux/directory-tree.md) | Français | [Deutsch](../../de-DE/ux/directory-tree.md) | [Español](../../es-ES/ux/directory-tree.md) | [Português (Brasil)](../../pt-BR/ux/directory-tree.md) | [Русский](../../ru-RU/ux/directory-tree.md)

État : **les interactions essentielles des répertoires sont intégrées aux exemples HTML et à la navigation Qt en lecture seule ; les ressources des états limites restent à intégrer**. Les interactions suivent le [contrat de navigation en lecture seule](../engineering/readonly-browser.md). La navigation ordinaire utilise les enfants directs, la recherche couvre récursivement la portée courante et l’autorisation de sortie reste distincte de la lecture seule.

Les trois planches actuelles présentent le [développement multiniveau](../../../design/hifi/screens/directory-tree-capsule.png), les [états des nœuds et des interactions](../../../design/hifi/screens/directory-tree-states.png) et les [limites de navigation et états vides](../../../design/hifi/screens/directory-tree-boundaries.png). La galerie partagée montre les planches complètes et la [bibliothèque de ressources](../../../design/assets/index.html#directory-tree) fournit les découpes individuelles. Le développement multiniveau est exporté depuis la source modifiable, avec une barre de titre synchronisée aux onglets en capsules actuels. Les listes en haut et au centre de la planche fournissent seulement le contexte et ne redéfinissent ni la barre de titre, ni la barre d’outils, ni le tableau des fichiers. L’[original multiniveau historique](../../../design/hifi/screens/directory-tree.png) reste inchangé comme source de 26 découpes et référence de mesure indépendante.

Les planches sont agrandies et les échelles des cartes diffèrent. Le texte, les rayons et les dimensions en pixels des images ne définissent pas directement la taille des contrôles ; les [paramètres des composants](../../../specs/visual/directory-tree.json) consignent explicitement les dimensions logiques implémentées et le manifeste des ressources les coordonnées physiques des découpes séparément. L’implémentation utilise une largeur logique de 1024 et une transformation commune du canevas source ; ne pas agrandir de nouveau l’ensemble des nouveaux composants.

Qt utilise directement les valeurs logiques ci-dessous. La conception principale HTML existante associe une fenêtre source de 1548 pixels à 1024 pixels logiques selon layout-scale.json : `source CSS = logical / (1024/1548)`. Ainsi, une hauteur de ligne de 32 correspond à 48.375 pixels CSS source, une police de 11.2 à 16.93125, un emplacement d’icône de 16 à 24.1875 et un retrait de 12.8 à 19.35. L’échelle de prévisualisation s’applique uniformément à toute la fenêtre après conversion. Le DPR détermine uniquement la densité d’échantillonnage et l’alignement des lignes ; il n’entre pas dans la conversion des dimensions logiques.

## Structure de l’arbre et décomptes

L’index complet fournit les répertoires explicites et implicites. Une feuille signifie **sans sous-répertoires**, pas sans fichiers. Si Glacier Lake contient un sous-répertoire, il possède sa propre flèche de développement, indépendamment de celle de son parent.

```text
Root                               ← current directory
▼ 01 Photos                     12
  ▼ Glacier Lake                 6
    ▸ Lakeside Trail             3  ← deeper directory, not expanded
      Snow Mountain View          3  ← files only, retains empty arrow slot
  ▸ Mountain Sunrise             4
    Other Photos                 2  ← no subdirectories
▸ 02 Documents                   8
  03 Maps                        4  ← no subdirectories
```

L’archive d’exemple contient 12 photos, 8 documents, 4 cartes et 6 fichiers à la racine ; la liste racine montre trois répertoires directs et six fichiers directs sans développer les descendants. Les descendants des branches repliées figurent aussi entièrement dans le fichier de paramètres afin de ne pas décorer les feuilles de fausses flèches. Le répertoire vide explicite d’une autre planche est un exemple d’état séparé, non ajouté à cet exemple de 30 entrées.

Les nombres des répertoires représentent tous les **enregistrements physiques de l’archive** descendants sous le préfixe, noms en double compris, mais excluent l’enregistrement du répertoire identique au préfixe. Ce ne sont pas des nombres de sous-répertoires et ils ne changent pas avec la recherche de nom. Les lignes de l’arbre n’affichent pas de nombre. Le résumé de l’archive conserve le nombre complet d’enregistrements physiques, tandis que la droite du titre de liste montre le nombre de lignes du répertoire courant ; leurs définitions diffèrent. Les listes ordinaires affichent le nom final, les résultats de recherche le chemin complet dans l’archive.

Les flèches dépendent uniquement des sous-répertoires connus : vers la droite si le parent est replié, vers le bas s’il est développé. Les nœuds ne contenant que des fichiers ou vides n’ont pas de flèche, mais leurs pairs gardent les mêmes emplacements de flèche et d’icône ainsi que le même début de texte. Pendant le chargement de l’index complet, ne pas deviner les flèches, nombres ou structures par nœud ni ajouter d’états de chargement asynchrone par nœud.

## Mise en page, polices et géométrie dynamique

| Élément | Taille logique / comportement normal |
| --- | --- |
| Largeur par défaut de la barre latérale | 193.16 ; la limite structurelle de la fenêtre principale est arrondie à 193 |
| Hauteur de ligne | Au moins 32 ; augmente selon la formule des paramètres pour les grandes polices |
| Retrait par niveau | 12.8 ; indépendant du type de nœud |
| Zone active / emplacement graphique de la flèche | Au moins 19.2×19.2 / 14.4×14.4 ; les feuilles gardent le même emplacement |
| Emplacement de l’icône de dossier | 16×16 ; le SVG partagé de 24 unités est redimensionné proportionnellement et remplace l’emplacement 18 de l’arbre actuel |
| Espace icône-texte / marge finale | 6.4 / 16 ; aucun nombre visible |
| Corps | 11.2 ; Qt arrondit à 11 et utilise la police de même source |
| Nombre d’enregistrements | Conservé dans les données et la sémantique accessible ; non affiché dans les lignes, sans colonne réservée |
| Répertoire courant | Fond bleu pâle, coins droits de rayon 6.4, bord gauche droit et indicateur de largeur 2.4 sur toute la hauteur |
| Focus clavier | Aucun contour ou barre de focus supplémentaire, aucune modification de l’espace occupé ; l’indicateur du répertoire courant reste présent |
| Lignes hiérarchiques | Largeur 1 ; hiérarchie discrète sur le fond |
| Marqueur du répertoire courant masqué | Point de 4.8 unités dans un emplacement transparent de 19.2 unités, centré sur la ligne du nom |

Les lignes de connexion sont dessinées dynamiquement à partir de la chaîne visible des ancêtres et des indicateurs de dernier élément frère. Les lignes verticales des parents traversent les sous-arbres développés jusqu’au centre du dernier élément frère ; la dernière connexion se termine en L sans se prolonger dans le groupe de premier niveau suivant. Le centre X de chaque ligne verticale est aligné sur la pointe X de la flèche descendante de son parent, y compris avec les grandes polices d’accessibilité. Les lignes internes se terminent indépendamment ; une image générique de ligne ne peut pas couvrir toutes les lignes. Aligner les segments sur le DPR cible ; texte, flèches et dossiers partagent le centre vertical de la ligne.

La zone hiérarchique défile horizontalement. Sa largeur utilise uniquement les noms réels, le retrait et la marge finale de 16 des nœuds visibles actuellement développés. Les noms restent complets sans superposition de nombres. Recalculer la largeur après réduction afin que les nœuds profonds masqués ne laissent pas de vide ; l’indicateur gauche reste fixé au bord de la zone d’affichage. Le survol d’une ligne n’affiche pas d’infobulle flottante ; les chemins complets sont disponibles dans les descriptions accessibles et les détails. Le défilement vertical reste dans la zone des répertoires ; les barres de défilement n’entourent pas la bordure de la fenêtre principale.

Tous les répertoires utilisent la même ressource de dossier. Les points de branche, icônes d’information et d’avertissement pleines et indicateurs de chargement segmentés existent en SVG clair/sombre et PNG 1×/2×/3× ; lignes, cadres de focus, fonds, squelettes, textes et invites sont des composants dynamiques. Les paramètres sombres réutilisent le thème global avec des couleurs supplémentaires pour les lignes et marqueurs ; **l’original complet sombre de l’arborescence reste à réaliser** et l’export des icônes sombres ne remplace pas la planche complète. Les conclusions existantes de rendu et d’acceptation sur bureau sont consignées séparément dans les issues et artefacts correspondants.

Consigner la cohérence des interactions HTML/Qt selon la [méthode de vérification](../../../developers/qa/README.md). Les planches et découpes conservent l’historique de la conception d’origine ; les nombres de lignes, le focus et les coins actuels suivent cette page et la spécification logique.

## Interactions et états indépendants

Le développement, le répertoire courant, le survol et le focus clavier sont stockés indépendamment et peuvent coexister. Le répertoire courant est également distinct de la sélection des cases de fichiers à droite.

La racine est une entrée navigable séparément, toujours développée, qui ne participe pas au développement ou à la réduction ordinaires des nœuds. Lorsqu’un clic droit, la touche Menu ou Maj+F10 ouvre son menu, « Replier les sous-répertoires » reste désactivé sans ligne explicative ni infobulle visible. La description accessible est « 根目录始终展开 » / « Root is always expanded » ; l’absence d’une ligne ordinaire d’arbre ne signifie pas qu’il n’y a pas de sous-répertoires. Le développement ou la réduction d’un parent ordinaire dépend de ses sous-répertoires réels ; les feuilles ordinaires conservent la description accessible « This node has no subdirectories ». Les raisons sont accessibles uniquement, sans indice visible. Ouvrir le menu ou cliquer sur des éléments désactivés ne doit modifier ni le répertoire, ni la sélection de fichiers, ni le développement de l’arbre.

| Action | Résultat |
| --- | --- |
| Cliquer sur la flèche | Développer ou réduire ce seul nœud ; la requête de répertoire, la sélection de fichiers et les détails restent inchangés |
| Cliquer sur le nom ou Entrée | Entrer avec l’ID de répertoire de la session courante ; après remplacement réussi de la requête, réinitialiser requête du répertoire courant, sélection de fichiers et détails selon le contrat existant |
| Haut / bas | Déplacer le focus entre les lignes de répertoires visibles sans changer automatiquement la liste de fichiers |
| Droite | Développer un parent réduit, puis aller au premier enfant s’il est déjà développé ; une feuille reste en place |
| Gauche | Réduire un parent développé, sinon aller au parent ; les nœuds de premier niveau restent en place |
| Réduire un ancêtre du répertoire courant | Conserver le répertoire courant et la liste ; le focus masqué revient à l’ancêtre visible le plus proche |
| Nouvelle session d’archive | Effacer anciens ID de nœud, développement, focus et points de branche ; ne pas réutiliser l’état d’une autre archive |

Par exemple, lorsque le chemin est `01 Photos / Glacier Lake`, réduire `01 Photos` laisse Glacier Lake affiché à droite. La ligne Photos montre seulement un point bleu final, sans recevoir le fond de répertoire courant de Glacier Lake ; la racine ne devient pas non plus le répertoire courant. Le point bleu figure sur **l’ancêtre replié visible le plus proche** ; un répertoire courant masqué possède un seul marqueur de branche. Développer de nouveau restaure le fond courant de Glacier Lake.

L’arbre constitue une seule entrée de navigation au clavier. Les nœuds conservent une sémantique accessible pour l’ID stable, le nom, le niveau, le développement, la position et le nombre d’enregistrements. Les feuilles ne se déclarent pas développables ; dossiers, lignes et points décoratifs ne sont pas annoncés de façon redondante. Les chemins courants masqués ont des descriptions accessibles et ne dépendent pas seulement du bleu. Les touches fléchées et clics sur les flèches ne doivent pas se propager à l’activation de la ligne.

## Couverture des états et récupération

| État | Arborescence | Liste de fichiers / récupération |
| --- | --- | --- |
| Lecture | Message de chargement, nombre inconnu ; le squelette est une ressource future ; navigation désactivée | Suivre l’état de chargement global ; remplacer l’arbre une fois après publication de l’index complet |
| Archive vide | Aucun nombre sur la ligne racine ; « Cette archive est vide » | Liste vide ; une autre archive peut toujours être ouverte |
| Fichiers à la racine uniquement | Aucun nombre sur la ligne racine ; « Aucun sous-répertoire » | Afficher normalement six fichiers ; ne pas transformer toute la page en état vide |
| Limite de l’index des répertoires dépassée | Ne pas publier un arbre partiel ; afficher « Le répertoire dépasse la limite d’affichage » | Racine, recherche et navigation dans les répertoires explicites admissibles continuent selon les capacités actuelles |
| Échec de lecture | Atténuer la navigation et afficher une erreur explicite | Rouvrir l’archive ; ne pas interpréter l’échec comme zéro ou un répertoire vide |
| Nœud avec sous-répertoires | Flèche indépendante à chaque niveau | Développer n’entre pas dans le répertoire |
| Feuille non vide / vide | Nombre conservé dans les données ; aucune n’affiche de nombre de ligne ni de flèche | Le nom reste activable ; un répertoire vide peut être parcouru |
| Nom long / imbrication profonde | Nom complet, chemin accessible, défilement horizontal et largeur selon les nœuds visibles | Le filtrage des chemins reste cohérent |
| Police 150 % (200 % est uniquement un ancien cas de contrainte) | Emplacements et lignes grandissent ; les lignes se recentrent | Noms, flèches et focus ne sont pas rognés |
| Répertoire courant masqué | L’ancêtre visible le plus proche montre un point de branche | La liste conserve le chemin courant ; le redéveloppement restaure la ligne courante |

Les limites de navigation et un échec global de lecture sont des états différents ; raisons, actions disponibles et sources de nombres ne peuvent partager un modèle « répertoire vide ». Les budgets existants de 32 768 nœuds, 64 niveaux et 4 MiB de noms de répertoires proviennent du contrat de lecture seule ; la conception ne les augmente ni ne les contourne.

## Livraison des ressources et intégration réelle

Les 26 découpes aux pixels d’origine se trouvent dans `design/assets/reference/tree-*.png`, chacune avec sa source, son SHA-256, `x,y,width,height` et son usage. `component_reference_coverage` pointe individuellement vers les vecteurs partagés ou composants dynamiques. Ne pas présenter les images statiques de lignes entières comme des contrôles interactifs ni supprimer le fond blanc des découpes pour fabriquer de la transparence.

| Partie | Ressource / dessin | Intégration actuelle |
| --- | --- | --- |
| Dossier, flèches droite/bas | `folder`, `chevron-right`, `chevron-down` des ressources partagées | Ressources communes existantes disponibles ; icônes et dimensions B utilisées par les lignes HTML récursives et les délégués Qt |
| Chargement, information, avertissement | Nouveaux `tree-spinner`, `tree-info`, `tree-warning` | Partie des huit SVG clair/sombre et 24 PNG transparents ; conserver les formes pleines et segmentées des planches au lieu d’icônes contour existantes |
| Branche courante masquée | Nouveau `tree-active-branch` | Six PNG transparents livrés pour deux thèmes ; Qt utilisent la ressource pour marquer l’ancêtre replié visible le plus proche |
| Lignes, fond courant, focus, squelette, indices | Dessin dynamique selon les dimensions et thèmes du fichier de paramètres | Style et logique spécifiés ; aucun second ensemble de contrôles bitmap livré |
| Planches officielles | `hifi/screens/directory-tree*.png` | Conserver les octets des originaux confirmés pour la comparaison de conception |

Qt intègrent les lignes de connexion, points de branche, emplacements de dossier de 16 unités et menus contextuels de répertoire. HTML utilise une hiérarchie d’exemple récursive complète ; Qt utilise l’index réel borné. Réduire l’ancêtre courant ne soumet pas de requête. Les squelettes de chargement et icônes dédiées d’information et d’erreur restent des ressources de conception ; la planche sombre complète reste à faire. L’intégration des icônes ne prouve pas l’acceptation des lecteurs d’écran ou du HiDPI sur chaque plateforme ; les conclusions manuelles existantes suivent [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5).

## Acceptation après intégration

1. Capturer la zone réelle des répertoires avec la même largeur logique 1024, configuration de police, DPR et données d’état, puis la comparer à la zone correspondante de la planche confirmée et à la spécification. Consigner le rapport d’échelle ; conserver séparément jugement visuel et assertions structurelles.
2. Aligner les débuts de flèche, d’icône et de nom à chaque niveau, sans colonne numérique de ligne. Icônes, texte, points et flèches partagent les centres des lignes. L’écart géométrique visé est d’au plus un pixel physique ; consigner la plateforme et la police de remplacement réelle en cas de différence.
3. Développer au moins trois niveaux, avec une branche plus profonde repliée et une feuille non vide. Valider la continuité et la terminaison des lignes jusqu’aux derniers éléments frères et à travers les sous-arbres développés.
4. Vérifier les états courant, survol et focus séparément et ensemble. Un clic sur la flèche ne soumet pas de requête de répertoire et n’efface pas la sélection de fichiers ; l’activation par Entrée suit la règle de réinitialisation après requête réussie.
5. Avec Glacier Lake comme répertoire courant, réduire Photos : le chemin de liste reste, la racine n’est pas sélectionnée et Photos ne montre qu’un point de branche. Redévelopper pour restaurer la ligne courante. La navigation clavier reste possible lorsque le focus est masqué.
6. Vérifier archives vides, fichiers racine, répertoires vides explicites, échec de lecture et limite de répertoires avec un index réel borné ; vérifier séparément nombres, désactivation et actions de récupération.
7. Avec noms longs, polices 100 %/125 %/150 % et l’ancien cas de contrainte 200 %, fenêtre minimale et profondeur maximale admise de l’index, les noms restent entièrement visibles, le défilement horizontal ne touche que la hiérarchie et les chemins complets sont accessibles. Le défilement de l’arbre ne déplace ni le résumé de l’archive ni les statistiques inférieures.
8. Effectuer l’acceptation du rendu réel clair/sombre, des résolutions normale/Retina/haute densité Windows, du clavier et du lecteur d’écran. Les contrôles d’image ou la génération de ressources ne remplacent pas ces vérifications à l’exécution.

Selon la [méthode de vérification](../../../developers/qa/README.md), contrôler séparément originaux, découpes, ressources et comportement réel de la souris, du clavier, des menus et de l’arbre. Indiquer explicitement l’accessibilité et les différences de plateforme non testées.
