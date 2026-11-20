<a name="xdzip-file-association-icons"></a>

# Icônes d’association de fichiers

[English](../../visual/file-association-icons.md) | [简体中文](../../zh-CN/visual/file-association-icons.md) | [繁體中文](../../zh-TW/visual/file-association-icons.md) | [日本語](../../ja-JP/visual/file-association-icons.md) | [한국어](../../ko-KR/visual/file-association-icons.md) | Français | [Deutsch](../../de-DE/visual/file-association-icons.md) | [Español](../../es-ES/visual/file-association-icons.md) | [Português (Brasil)](../../pt-BR/visual/file-association-icons.md) | [Русский](../../ru-RU/visual/file-association-icons.md)

Les icônes système réutilisent le document à coin replié de xdzip, le badge à courbure continue, les quatre bandes entrelacées et la marque ouverte en U. L’entrée de conception est la [page de revue haute fidélité](../../../design/hifi/sources/file-association-icons.html), les paramètres proviennent de [file-association-icons.json](../../../specs/visual/file-association-icons.json), et les sources et conteneurs de plateforme se trouvent dans [file-associations](../../../design/assets/file-associations). Les utilisateurs choisissent toujours leurs applications par défaut.

<a name="xdzip-shape-and-identification"></a>

## Forme et identification

La géométrie du papier et de la marque réutilise [brand_assets.py](../../../scripts/brand_assets.py). Les formats partagent papier, pli, position du badge et centre optique ; des étiquettes identifient les formats et des couleurs facilitent le repérage. TAR.GZ, TGZ et GZ forment une famille bleu-vert. Le repli générique utilise un gris-bleu neutre et des lignes de document sans étiquette de format.

Les étiquettes suivent strictement les glyphes ZIP originaux : hauteur 61, trait 14, extrémités carrées, jointures arrondies, haut au Y=817 du maître et couleur #39465D. ZIP réutilise directement le tracé complet du texte du maître original ; les autres lettres respectent les mêmes mesures et règles de trait. Aucun texte SVG, police système, icône tierce ou bitmap intégré n’est utilisé. Largeur maximale 504. Conserver l’étiquette TAR.GZ complète sans étirer les glyphes ; ne réduire uniformément et centrer verticalement que si elle dépasse le budget de largeur. La couleur n’est pas le seul moyen d’identification.

Le maître mesure 1024 unités. Les PNG/ICO génériques gardent le viewBox `32 32 960 960`, avec papier d’environ 90% de hauteur ; les PNG/ICNS macOS gardent `-20 -20 1064 1064`, avec papier d’environ 81,2%. Cela poursuit l’étalonnage optique du projet et n’est pas présenté comme exigence système. Les icônes ont un canevas transparent, sans papier propre au thème, fond de bureau ni masque arrondi supplémentaire.

À 16 et 24 pixels, le badge est agrandi et le papier simplifié ; à 32 pixels le badge est légèrement plus petit. Ces trois tailles omettent étiquettes et ombres des grandes images tout en conservant les proportions de la marque. À ces tailles, les icônes seules ne distinguent pas sûrement tous les formats, notamment la famille GZIP de même couleur ; les colonnes de nom et type permettent une identification précise. Les maîtres étiquetés sont utilisés dès 48 pixels ; les longues étiquettes sont plus lisibles à partir de 64 pixels. Rendre chaque taille directement depuis les vecteurs plutôt que réduire successivement un PNG 1024.

Les icônes de 24 unités de l’interface et la densité de la fenêtre principale et des menus continuent de suivre [layout-scale.json](../../../specs/visual/layout-scale.json). Ces ressources n’ajoutent aucune fenêtre QML et ne changent aucune dimension logique de la fenêtre principale ou des menus. Les dimensions PNG de plateforme sont des dimensions d’échantillonnage et ne doivent pas entrer dans la géométrie logique des composants QML.

<a name="xdzip-format-and-platform-mapping"></a>

## Correspondance des formats et plateformes

| ID d’icône | Étiquette | Extension / usage | Couleur |
| --- | --- | --- | --- |
| `zip` | ZIP | `.zip` | Bleu de marque |
| `7z` | 7Z | `.7z` | Indigo |
| `rar` | RAR | `.rar` | Violet |
| `tar` | TAR | `.tar` | Ambre |
| `tar-gz` | TAR.GZ | Suffixe composé `.tar.gz` confirmé | Bleu-vert |
| `tgz` | TGZ | `.tgz` | Bleu-vert |
| `gz` | GZ | Candidat système `.gz` | Bleu-vert |
| `cab` | CAB | `.cab` | Bleu acier |
| `iso` | ISO | `.iso` | Bleu lac |
| `udf` | UDF | `.udf` | Bleu cyan |
| `archive` | Aucune | Repli pour archives reconnues sans icône dédiée | Gris-bleu neutre |

Ce tableau décrit une intention de conception, pas un registre uniforme entre plateformes. Quand c’est possible, la correspondance privilégie `.tar.gz` complet avant `.gz` ; un système ne reconnaissant que l’extension finale peut afficher GZ pour les deux. Ne pas promettre que toutes les plateformes distinguent automatiquement les suffixes composés. L’icône GZ désigne une extension commune au GZIP autonome et à la couche externe TAR.GZ ; la lecture dépend toujours de la validation du contenu et des contrats de format.

Chiffrement et fractionnement sont des états d’archive qui ne sont pas déduits des icônes d’extension ; ne pas dessiner de cadenas ou de marque de succès. Les volumes ZIP, 7z et RAR reconnus peuvent réutiliser leur famille dans l’application sans enregistrer les extensions génériques `.001`, `.rNN`, `.zNN` ou `.img`. Les icônes ISO/UDF n’impliquent ni montage ni exécution. Une extension correspondante ne prouve pas la validité du contenu.

L’installateur Windows ordinaire utilise des ProgID et ICO par extension. MSIX déclare des candidats « Ouvrir avec » avec des PNG issus des images ICO correspondantes. Les deux suivent la correspondance d’associations générée, qui peut partager une illustration entre suffixes. macOS utilise un ICNS par type de document, dont TAR comprimé et GZIP autonome. Linux utilise les icônes MIME système. Le choix par défaut et les caches d’icônes influent sur l’affichage réel. Voir [installation et associations](../../engineering/installers-associations.md).

MSIX déclare un `uap:Logo` distinct pour chaque association de format. Les PNG à taille cible réutilisent octet pour octet les images ICO Windows canoniques, avec simplification des petites tailles, étiquettes des grandes tailles, limites visibles et centrage. Les logos d’application sont des ressources distinctes avec formes par défaut et sans plaque pour les deux thèmes. [msix-icons.json](../../../specs/visual/msix-icons.json) définit tailles et variantes ; MakePRI les indexe avant l’empaquetage MakeAppx. Aucun fond système ni repli sur le logo d’application ne fait partie de l’illustration prévue du document.

L’entrée d’application Windows dans les applications par défaut et « Ouvrir avec » utilise l’[ICO de marque](../../../design/assets/brand/xdzip.ico), intégré à l’exécutable principal et au lanceur d’association. Les icônes de documents par format ne remplacent pas l’identité de l’application. Le système devrait afficher une seule entrée facultative xdzip avec l’icône de marque ; l’ancienne commande partagée assure la compatibilité avec les choix existants. Cela réutilise les ressources existantes sans modifier la géométrie logique QML, de la fenêtre ou des menus. Les icônes natives d’application, de document et le nombre de candidats sont acceptés séparément ; une boîte d’application générique de l’OS ne doit pas être classée comme problème d’icône de document.

<a name="xdzip-generation-and-delivery"></a>

## Génération et livraison

Exécuter depuis la racine du dépôt :

```sh
python scripts/build-association-icons.py
python scripts/build-design-assets.py --metadata-only
python scripts/build-association-icons.py --check --check-runtime
# Exporter les artefacts locaux avec Pillow, Playwright et Chromium existants.
python scripts/build-association-icons.py --export build/file-association-icons
python scripts/build-association-icons.py --check --export build/file-association-icons
# Après copie des exports ICO/ICNS souhaités dans le dossier de ressources canonique :
python scripts/build-design-assets.py --metadata-only --refresh-association-icons
```

`XDZIP_CHROMIUM` peut désigner un chemin Chromium/Edge existant. Maîtres SVG, sources petites tailles, spécification et page de revue sont versionnés ; les sources sont inscrites dans le [manifest.json](../../../design/assets/manifest.json) principal. L’entrée de génération est [build-association-icons.py](../../../scripts/build-association-icons.py), avec compositions dans [association_icon_assets.py](../../../scripts/association_icon_assets.py).

Les exports contiennent des PNG génériques et macOS de 16 à 1024 pixels par format, ainsi que des ICO et ICNS. Les ICO et ICNS du produit, recensés par le registre des formats, résident à côté des sources vectorielles dans `file-associations/` ; les conteneurs ZIP gardent leurs octets d’origine avant migration et les autres proviennent des exports correspondants. Les compilations ordinaires lisent directement ces conteneurs sans navigateur. PNG temporaires, exports complets, planches de contact et reçus restent dans `build/` ou `dist/`.

<a name="xdzip-verification-and-acceptance"></a>

## Vérification et acceptation

Les contrôles de génération vérifient que les SVG correspondent à la spécification et n’ont aucune dépendance de police ou d’image externe. Les contrôles de plateforme couvrent canevas et RGBA, coins transparents, hauteur et centrage visibles, empreintes vectorielles et concordance des pixels décodés de chaque représentation ICO/ICNS. Les limites du corps sont mesurées à alpha ≥ 128, avec la tolérance de marque de 1,25 pixel physique.

La revue visuelle couvre fonds clair, sombre et sélectionné, tailles de 16 à 256 pixels et rendu réel en DPR normal et élevé. La page de revue utilise les pixels CSS ; les captures du navigateur ne doivent pas être présentées comme acceptation native Explorer, Finder ou Linux. L’intégration ultérieure doit vérifier noms réels, suffixes composés, caches d’icônes, avant et après choix par défaut, et affichage après installation ou désinstallation. L’état d’étape et le périmètre réel sont enregistrés dans les issues GitHub plutôt que d’accumuler les rapports ici.
