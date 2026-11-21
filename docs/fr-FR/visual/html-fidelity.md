<a name="xdzip-html-visual-fidelity-review"></a>

# Revue de fidélité visuelle HTML

[English](../../visual/html-fidelity.md) | [简体中文](../../zh-CN/visual/html-fidelity.md) | [繁體中文](../../zh-TW/visual/html-fidelity.md) | [日本語](../../ja-JP/visual/html-fidelity.md) | [한국어](../../ko-KR/visual/html-fidelity.md) | Français | [Deutsch](../../de-DE/visual/html-fidelity.md) | [Español](../../es-ES/visual/html-fidelity.md) | [Português (Brasil)](../../pt-BR/visual/html-fidelity.md) | [Русский](../../ru-RU/visual/html-fidelity.md)

Les images haute fidélité constituent la référence visuelle ; le HTML est l’implémentation modifiable. Les dimensions au-dessus et au-dessous de la barre d’outils sont mesurées séparément sur la [fenêtre principale initiale figée](../../../design/hifi/screens/main-light.png), sans les déduire à rebours du CSS actuel. L’affichage courant utilise le [modèle clair à onglets en capsule](../../../design/hifi/screens/main-capsule-light.png) et le [modèle sombre](../../../design/hifi/screens/main-capsule-dark.png), exportés depuis la [source modifiable](../../../design/hifi/sources/main.html) ; ils ne remplacent pas une mesure indépendante. La barre de titre est acceptée selon la révision explicite de layout-scale.json. Les originaux ont des contours adoucis et des glyphes générés ; le nombre de tests ne prouve pas une identité parfaite au pixel sur toute la page.

<a name="xdzip-fixed-reproducible-conditions"></a>

## Conditions fixes reproductibles

- Le canevas source correspond au PNG à 1586×992 ; la fenêtre mesure 1548×940 à l’origine `(20,30)`. Les vérifications d’ancrage originales utilisent toujours une fenêtre de navigateur 1588×1000 avec un ratio de pixels d’appareil de 1. Recadrer le haut à `(20,30)-(1568,167)` et le contenu à `(20,167)-(1568,970)` sans étirer l’original.
- Les SHA-256 des originaux sont figés dans [header-reference.json](../../../design/html/header-reference.json) et [body-reference.json](../../../design/html/body-reference.json). Tout changement d’original exige une nouvelle mesure, pas un remplacement silencieux.
- La police hors ligne fournie [Xdzip Design Sans SC](../../../design/assets/fonts/README.md) est mesurée après `document.fonts.ready`. Les graisses réelles 400–750 sont fixées pour réduire les variations de largeur dues aux polices de repli des plateformes.
- La famille de police d’origine ne peut pas être déterminée à partir d’une image générée. Les contours des glyphes et l’anticrénelage système peuvent encore différer ; consigner ces différences séparément sans prétendre avoir restauré la police originale.

<a name="xdzip-display-size-and-source-pixels"></a>

## Taille d’affichage et pixels source

L’affichage normal utilise la [largeur de fenêtre de 1024 pixels logiques](README.md), en appliquant uniformément aux coordonnées source le rapport `1024 / 1548`. La fenêtre principale s’affiche à 1024 × environ 621,81 pixels CSS ; l’original utilise la même échelle. Ce rapport est une convention de conception explicite et non une déduction sur Retina ou les DPI à partir d’un PNG.

`display-spec.js` est l’unique source des dimensions de plan de travail et des échelles d’affichage. Le bureau utilise par défaut la taille normale et le mobile l’ajustement ; celui-ci est plafonné à la taille normale pour éviter un agrandissement automatique sur écran large. Le mode pixels d’origine utilise `scale=1`, conserve les dimensions en pixels d’origine et le défilement côté HTML comme côté image. Le ratio de pixels de l’appareil n’affecte que la densité de pixellisation, pas la largeur logique CSS.

Ne pas ajuster séparément l’original et le HTML à leurs conteneurs. Les ancrages d’en-tête et de corps restent mesurés en coordonnées de pixels source ; l’échelle d’affichage ne change ni l’espacement ni les tailles de police calibrés. La cohérence des tailles couvre séparément les limites réelles d’écran, la taille naturelle de l’original, le défilement mobile, les aperçus autonomes et les sorties hors ligne en fichier unique. Conserver les preuves selon la [méthode de vérification](../../../developers/qa/README.md).

<a name="xdzip-canvas-and-content-scrolling"></a>

## Canevas et défilement du contenu

Le HTML intégré à l’outil de revue est un plan de travail fixe avec `overflow:hidden` sur le document racine. Cela empêche les barres de défilement classiques de consommer les dimensions du canevas et de se déclencher mutuellement. La page de revue défile verticalement ; le canevas externe ne défile horizontalement que si la largeur est insuffisante. `scrollbar-gutter:stable` réserve la largeur de barre classique afin qu’une hauteur proche du seuil ne modifie pas sans cesse l’échelle d’ajustement ; les aperçus autonomes utilisent `auto`. Les listes, propriétés longues et fenêtres d’opération gardent leur défilement interne ; les canevas source ouverts directement hors intégration gardent le défilement de page. Cela ne modifie pas les dimensions du canevas, de la fenêtre ou des composants.

Playwright utilise `--hide-scrollbars` par défaut ; les anciennes matrices de captures et de tailles ne couvraient donc pas les barres classiques réelles. Les contrôles de taille et d’aperçu retirent maintenant explicitement cet argument et vérifient la réservation de barre racine, la visibilité des quatre bords de fenêtre, le défilement horizontal externe et l’accès au dernier bouton des propriétés longues. Un canevas trop petit de 1 px et un cas négatif supprimant les règles d’intégration reproduisent et détectent la concurrence pour l’espace des barres. Conserver les captures de reproduction et de correction dans les artefacts d’exécution selon la [méthode de vérification](../../../developers/qa/README.md) ; cela ne prouve pas l’apparence des barres sur tous les navigateurs ou systèmes.

<a name="xdzip-header-corrections"></a>

## Corrections de l’en-tête

| Zone | Avant | Correction selon l’original |
| --- | --- | --- |
| Onglet actif | Début x299, largeur 244 | x293, largeur 246 |
| Onglet inactif | x557, largeur 244 | x542, largeur 262 |
| Écart entre onglets | 14 px | 3 px ; incertitude d’environ 1 px permise pour les contours adoucis |
| Boutons plus et fermer | Dispositions verticales distinctes, centres séparés de 5 px | Même rangée d’onglets, avec décalage optique d’environ 1 px par rapport à l’original |
| Commandes de fermeture | Un bouton et un span nu | Même structure icône/nom/fermeture et emplacements explicites |
| Icône des paramètres | Graphique visible petit et décalé en haut à gauche | Centre `(1532,61)` ; examiner le dessin réel, pas seulement le conteneur SVG |
| Bord inférieur de l’onglet actif | Chevauchait le trait de la barre de titre | Rejoint la surface blanche de la barre d’outils, avec transition d’épaulement gauche définie séparément |
| Recherche et extraction | Limites horizontales et verticales décalées | Comparer respectivement à `(906,101,449,51)` et `(1368,101,183,51)` |
| Icônes à gauche de la barre d’outils | Disposition dictée par les tailles et écarts génériques de boutons | Positionner et vérifier selon les ancrages graphiques d’origine |

Les emplacements et la typographie de l’en-tête sont centralisés dans les sections barre de titre, onglets et barre d’outils de `hifi.css`, sans dépendre des dimensions génériques `.icon-button` ou d’une hauteur de ligne héritée. Les thèmes clair et sombre partagent la géométrie. Les décorations Windows/Linux restent des démonstrations dans le navigateur ; les fenêtres natives du produit font l’objet d’une acceptation distincte.

<a name="xdzip-content-below-the-toolbar"></a>

## Contenu sous la barre d’outils

Le corps comprend la carte d’archive, l’arborescence, la barre de chemin, les en-têtes et lignes du tableau, la sélection, l’aperçu et la barre d’état inférieure. Relever limites de colonnes, hauteurs de lignes, marges internes, centres d’icônes et encre visible et les comparer à la [référence indépendante](../../../design/html/body-reference.json). Les tolérances de géométrie stricte et d’encre d’icône isolée sont de 2 px ; les contours du texte ne sont consignés que pour revue manuelle. La taille optique du dossier est une révision explicite : le nouveau contour est uniformément agrandi d’environ 25,7 % dans les ressources partagées. Cinq anciens exemples d’encre restent pour comparaison et sont contrôlés par rapport aux nouvelles limites relevées indépendamment ; les autres ancrages d’origine restent inchangés.

La revue du corps utilise des recadrages de contenu de même taille et conserve des comparaisons régionales pour la barre latérale, la liste, l’aperçu et la barre d’état. Captures avant/après, commits et mesures appartiennent uniquement aux artefacts de l’issue ou d’Actions correspondants, pas à la spécification comme résultats d’une exécution particulière. La revue mobile se fait par région et ne doit pas modifier la disposition de bureau pour masquer les écarts.

L’original sélectionne un élément tout en laissant Tout sélectionner vide, et indique une économie de 26 % avec une barre remplie à environ 56 % ; ces états statiques sont contradictoires. Le HTML conserve respectivement une sélection partielle `indeterminate` et un remplissage réel de 26 %, exceptions sémantiques documentées. Les contrôles propres à l’aperçu couvrent le débordement normal et le défilement du contenu long ; les contrôles d’interaction existants couvrent sélection, navigation, tri, recherche et actions d’exemple. La géométrie ne remplace pas la vérification fonctionnelle.

Enregistrer séparément la revue du corps, les comparaisons régionales de même taille et les mesures selon la [méthode de vérification](../../../developers/qa/README.md). Conserver les explications sur les polices originales inconnues, contours adoucis générés, éclairage et différences d’anticrénelage ; ces preuves HTML ne démontrent pas le calibrage correspondant du produit Qt.

<a name="xdzip-checks-and-delivery"></a>

## Contrôles et livraison

Exécuter :

```sh
python scripts/build-html-design.py
python scripts/check-html-header.py
python scripts/check-html-body.py
python scripts/check-html-preview.py
python scripts/check-html-references.py
python scripts/check-html-scale.py
python scripts/check-html-design.py
python scripts/build-html-header-review.py
python scripts/build-html-body-review.py
```

Les contrôles d’en-tête utilisent des ancrages originaux indépendants : écart maximal de 1 px des limites/centres DOM et de 2 px des limites d’encre réelle des icônes dans les régions isolées. L’encre blanche de la flèche déroulante a une tolérance de 1 px ; l’épaulement droit de l’onglet échantillonne la surface blanche et la courbe d’origine avec une tolérance de 10/255 sur un canal pour l’adoucissement. Les contours de glyphes sont seulement mesurés et consignés. Superpositions et images de différence incluent les écarts d’anticrénelage, d’éclairage et de glyphes ; ne pas transformer la différence totale de pixels en une similarité inventée.

Le fichier d’en-tête hors ligne est `dist/xdzip-header-review.html` et celui du corps `dist/xdzip-body-review.html`. Tous deux permettent de choisir original, avant, après, superposition transparente et différences de pixels ; la revue du corps ajoute le choix de région mobile. L’interface en fichier unique est `dist/xdzip-hifi-design.html`.

Exécuter ces contrôles et revues hors ligne localement selon les besoins ; ils ne font pas partie par défaut de la CI du produit de bureau. Examiner les échecs d’ancrage et les superpositions même si la géométrie passe. Les contrôles HTML de la fenêtre principale ne remplacent pas le calibrage des autres pages ou du produit Qt, ni l’acceptation native du bureau.

La densité normale de la fenêtre principale et des menus suit la [norme d’interface](design-standard.md). Les rapports de taille des polices de menus et listes Qt doivent être vérifiés ensemble. Lors de la revue HTML locale, vérifier le zoom réel selon la [méthode de vérification](../../../developers/qa/README.md) ; la correspondance avec les anciennes coordonnées de l’image source ne la remplace pas.
