<a name="xdzip-content-preview"></a>

# Aperçu du contenu

[English](../../engineering/content-preview.md) | [简体中文](../../zh-CN/engineering/content-preview.md) | [繁體中文](../../zh-TW/engineering/content-preview.md) | [日本語](../../ja-JP/engineering/content-preview.md) | [한국어](../../ko-KR/engineering/content-preview.md) | Français | [Deutsch](../../de-DE/engineering/content-preview.md) | [Español](../../es-ES/engineering/content-preview.md) | [Português (Brasil)](../../pt-BR/engineering/content-preview.md) | [Русский](../../ru-RU/engineering/content-preview.md)

Lorsque la carte de propriétés est visible, sélectionner un fichier pris en charge démarre un aperçu une fois la sélection stabilisée. L’icône d’actualisation apparaît en haut à droite lorsque le pointeur survole l’aperçu ou que le bouton reçoit le focus clavier ; elle recharge le fichier choisi. La carte signale les erreurs de lecture ou de décodage et affiche un message en cas de sélection multiple. La fenêtre principale et la visionneuse autonome de propriétés partagent ces comportements et la fenêtre de mot de passe existante. Les métadonnées restent disponibles séparément.

<a name="xdzip-capabilities-and-budgets"></a>

## Capacités et budgets

Seuls les fichiers réguliers lisibles, de taille connue et sans nom dangereux sont pris en charge. La liste autorisée d’extensions texte est txt, md, log, json, csv, tsv, toml, yaml, yml, ini, xml, rs, c, cpp, h et py. Seuls UTF-8 strict et UTF-16 avec BOM sont acceptés. L’entrée et l’UTF-8 converti sont chacun limités à 256 KiB ; les octets de contrôle binaires tels que NUL sont interdits. Le contenu est affiché en PlainText ; balises, URL, programmes et macros ne sont pas exécutés.

| Type d’image | Portée et comportement réel sur les trois plateformes |
| --- | --- |
| PNG, JPEG (jpg, jpeg, jfif, jpe) | Lire entièrement une image statique et appliquer les métadonnées de transformation disponibles pour son orientation |
| GIF, WebP | Image statique ou première image d’animation ; pas de lecture de l’animation |
| BMP, ICO/CUR | Image ordinaire ou image par défaut du décodeur ; CUR n’active pas de curseur système |
| ICNS | Images PNG ou JPEG2000 incorporées prises en charge par un décodeur système disponible, au plus 256 blocs ; types incorporés inconnus et anciennes icônes bitmap brut/RLE explicitement refusés |
| TGA | Images vraies couleurs non comprimées 16/24/32 bits, sans palette ni identifiant d’image, d’origine à gauche ; compléter un ancien pied absent pour la même disposition ; pas de RLE ou couleur indexée |
| TIFF (tif, tiff) | Première page seulement ; les pages ne sont pas parcourues automatiquement |
| SVG | Vecteur statique UTF-8 pixellisé après analyse restreinte ; voir le sous-ensemble ci-dessous |
| HEIC/HEIF | macOS uniquement, avec le plugin Qt officiel et ImageIO système ; explicitement refusé ailleurs |

AVIF, RAW, PSD, EXR, PDF, Office et les bibliothèques indépendantes de décodeurs tiers ne sont pas ajoutés. Extension, vérifications Rust des signatures de format et décodeurs Qt fixés déterminent ensemble l’admission. Les plugins supplémentaires installés sur le système ne deviennent pas des capacités du produit et d’autres décodeurs ne sont pas sondés automatiquement.

Le contenu d’image encodé est limité à 16 MiB. Les files d’événements Rust et la boîte Qt ont chacune une limite de 32 MiB ; le budget de commande reste 8 MiB. Largeur et hauteur d’origine sont chacune d’au plus 16 384 et le total d’au plus 33 554 432 pixels. Admission de la source et miniature de sortie sont comptabilisées séparément : largeur et hauteur finales sont chacune d’au plus 512, les proportions sont conservées et RGBA8 est limité à 1 MiB. Des photos de 12 MP peuvent être lues comme miniatures dans ces budgets. Certains décodeurs lisent l’original avant réduction ; la taille de miniature ne prouve donc pas une consommation interne limitée à 1 MiB. Les profils colorimétriques disponibles sont convertis en sRGB ; les profils inconnus conservent le résultat du décodeur. Une épreuve couleur professionnelle n’est pas promise.

Le fournisseur d’images asynchrone Qt ne reçoit que des jetons aléatoires, pas de chemins système ni d’URL réseau. Au plus 32 MiB de contenu encodé sont conservés globalement et au plus deux tâches de décodage s’exécutent en parallèle. Chaque tâche annulée encore en cours de récupération peut conserver au plus une entrée de 16 MiB. Chaque tâche lance le processus privé adjacent fixé `xdzip-image-preview`, sans lui transmettre chemins d’archive, mots de passe, handles de destination ou identifiants d’application. Sa sortie n’autorise qu’un en-tête de taille fixe et du RGBA8 dont largeur, hauteur et longueur correspondent strictement, sans sortie supplémentaire. Échec, plantage, trames invalides, délai dépassé et résultats tardifs ne peuvent pas être affichés.

L’hôte impose un délai total de cinq secondes depuis le lancement et vérifie la validité de la requête toutes les 50 ms. Fermeture, annulation ou changement d’entrée termine et récupère le processus. Celui-ci limite les allocations QImage à 128 MiB. Windows utilise un Job avec 512 MiB de mémoire engagée du processus et trois secondes de CPU utilisateur. Unix utilise trois secondes CPU, désactive les dumps core et limite les descripteurs. Linux limite l’espace d’adressage à 512 MiB et la taille des fichiers réguliers à zéro. macOS ajoute une marge d’espace d’adressage de 512 MiB aux réservations virtuelles dyld existantes ; ce n’est pas une limite stricte RSS. ImageIO système peut utiliser des ressources runtime adossées à des fichiers : chaque fichier régulier est donc limité à 16 MiB et tout dépassement termine le décodage. L’application n’exporte aucune copie d’entrée ; ressources système et sortie d’entrée sont gérées séparément. Il s’agit d’une isolation de panne et de ressources pour un décodeur privé, pas d’un bac à sable de permissions noyau pour des programmes tiers arbitraires.

Chaque lecture de contenu d’archive est limitée à 256 MiB de lectures cumulées de source, 64 MiB de travail de décodage et d’omission, et 256 MiB de travail algorithmique auxiliaire. Dictionnaires de format, processus et graphes de méthodes restent soumis aux budgets distincts du [contrat des moteurs de format](native-codecs.md). Une archive solide ou un flux comprimé séquentiel peut refuser l’aperçu d’un petit fichier si le décodage précédent dépasse le budget de travail.

<a name="xdzip-static-svg-subset"></a>

## Sous-ensemble SVG statique

SVG utilise le même chemin de rendu Qt Svg sur les trois plateformes desktop. Avant le rendu, un analyseur XML en flux reconstruit graphiques statiques, texte, styles, définitions internes, dégradés, découpes, masques et filtres restreints pris en charge. Limites : 8 192 éléments, 64 niveaux, 64 attributs par élément, 1 MiB par attribut et 16 MiB de sortie totale. Les métadonnées ne participent pas au rendu. Les éléments de style n’acceptent que du texte ; réunir segments XML et CDATA avant de vérifier le CSS complet afin que la segmentation ne contourne pas les restrictions de références externes.

Refuser DTD, entités, instructions de traitement, scripts, événements, animation, foreignObject, éléments image, fichiers de police, styles externes et références à des ressources externes. href et CSS url n’acceptent que des identités de fragment simples du document courant. xml:base, src, imports CSS et formes échappées sont interdits. Les bitmaps incorporés et SVGZ sont hors du sous-ensemble. Les éléments de dessin inconnus ou contenus interdits font échouer tout l’aperçu plutôt que d’être ignorés silencieusement et présentés comme complets. Cela prend en charge le SVG statique courant sans promettre toutes les capacités SVG ou navigateur.

<a name="xdzip-integrity-passwords-and-lifecycle"></a>

## Intégrité, mots de passe et cycle de vie

L’aperçu réutilise l’identité physique figée, la version source, l’emplacement des données et les décodeurs existants. Le contenu doit être entièrement décodé jusqu’à EOF et réussir les contrôles d’intégrité et d’authentification disponibles du format avant que texte ou images encodées arrivent à l’aperçu. Les formats tels que TAR sans somme de contrôle de contenu par fichier ne conservent que leurs garanties existantes de structure et de lisibilité. Les grands index sur disque lisent les données sélectionnées de façon bornée sans charger toutes les métadonnées dans Qt.

Les mots de passe ne sont liés qu’à la requête courante et utilisent le conteneur existant effacé à zéro ; erreurs, ambiguïté d’authentification et annulation suivent les catégories existantes, avec au plus dix tentatives. Fermer la fenêtre de mot de passe n’annule que l’aperçu et conserve le répertoire utilisable. Ouverture, navigation et autres commandes de lecture au premier plan priment sur une lecture d’aperçu en attente. Fermer l’aperçu, changer d’entrée, masquer une page d’archive ou fermer une session efface le contenu ; requêtes invalidées et résultats de décodage ne peuvent restaurer l’ancien contenu. Échec et annulation écartent les données partielles non vérifiées, n’écrivent aucune copie d’entrée et ne lancent aucune application système. L’ouverture temporaire par double-clic suit son [parcours distinct](archive-opening.md).

<a name="xdzip-toolchain-and-acceptance"></a>

## Outils et acceptation

`desktop.py prepare --install-qt` ajoute qtimageformats de Qt 6.8.2 fixé aux SDK nouveaux ou existants ; doctor vérifie Svg et les plugins nécessaires avant compilation. Sources officielles, empreintes, avis et correspondance SDK figurent dans la configuration commune. libwebp/libtiff inclus par Qt et bibliothèques de formats ajoutées séparément sont gérés indépendamment. `check-image-preview.py` vérifie le processus réel, les plugins et le décodage minimal dans les tests par défaut, la vérification de déploiement Windows/Linux et l’empaquetage Mac autorisé. Mac inclut explicitement offscreen et efface les surcharges de recherche SDK avant l’autocontrôle ; les répertoires de plugins du SDK de développement ne remplacent pas les tests du paquet runtime réel.

Les régressions utilisent des images propres au projet couvrant chaque type, premières images d’animation, premières pages TIFF, transparence, photos 12 MP, HEIC macOS et refus sur les autres plateformes, définitions internes SVG et refus de ressources externes, entrée endommagée, dépassement de pixels, trames invalides, délais et annulation. Les chemins d’archives réels continuent de couvrir mots de passe, intégrité, identité source et nettoyage après invalidation de l’interface. La disposition suit la [norme visuelle](../visual/design-standard.md#xdzip-content-preview) ; réutiliser les preuves de disposition inchangées selon la [vérification](../../../developers/qa/README.md) et observer l’affichage réel des nouvelles images.
