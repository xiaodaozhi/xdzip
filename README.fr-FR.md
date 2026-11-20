<p align="center">
  <img src="design/assets/brand/xdzip-128.png" width="80" height="80" alt="Icône de l’application xdzip">
</p>

<h1 align="center">xdzip</h1>

<p align="center"><a href="README.md">English</a> | <a href="README.zh-CN.md">简体中文</a> | <a href="README.zh-TW.md">繁體中文</a> | <a href="README.ja-JP.md">日本語</a> | <a href="README.ko-KR.md">한국어</a> | Français | <a href="README.de-DE.md">Deutsch</a> | <a href="README.es-ES.md">Español</a> | <a href="README.pt-BR.md">Português (Brasil)</a> | <a href="README.ru-RU.md">Русский</a></p>

<p align="center"><strong>Compresser et décompresser, plus simplement</strong></p>

<p align="center">xdzip est un gestionnaire d’archives libre et multiplateforme. Les formats implémentés en Rust fonctionnent sans programme d’extraction externe. Parcourez les archives, recherchez des entrées, extrayez une sélection, ouvrez les archives chiffrées ou multivolumes et modifiez les entrées des formats compatibles.<br>xdzip prend en charge ZIP, 7z, RAR, TAR, CAB, ISO/UDF et d’autres formats sous Windows, macOS et Linux.</p>

<p align="center">
  <a href="https://github.com/xiaodaozhi/xdzip/actions/workflows/docs.yml"><img src="https://github.com/xiaodaozhi/xdzip/actions/workflows/docs.yml/badge.svg?branch=main" alt="Intégration continue"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-blue.svg" alt="Licence Apache 2.0"></a>
</p>

<p align="center">
  <a href="#xdzip-download-and-install">Télécharger</a> / <a href="#xdzip-using-xdzip">Utiliser xdzip</a> / <a href="#xdzip-format-support">Formats pris en charge</a> / <a href="#xdzip-build-from-source">Compiler à partir des sources</a> / <a href="docs/fr-FR/README.md">Documentation</a> / <a href="https://github.com/xiaodaozhi/xdzip/issues">Retours</a>
</p>

![Navigation dans une archive avec xdzip](docs/images/main-window.png)

<a name="xdzip-download-and-install"></a>

## Télécharger et installer

Sous Windows 11 x64, recherchez **XDZip** dans le Microsoft Store. Pour un téléchargement direct ou l’outil en ligne de commande autonome, choisissez votre système sur la page de la [dernière version stable sur GitHub](https://github.com/xiaodaozhi/xdzip/releases/latest).

| Système | Périmètre pris en charge | Téléchargement GitHub |
| --- | --- | --- |
| Windows | Versions x86_64 de Windows 11 encore maintenues | Programme d’installation par utilisateur `setup.exe` ou version portable `.zip` |
| macOS | macOS 14 ou ultérieur, Apple Silicon | Image `.dmg` ou application au format `.app.zip` |
| Linux | Ubuntu 24.04 LTS, x86_64, X11, glibc 2.39 ou ultérieure | Paquet `.deb` ou répertoire d’exécution au format `.tar.gz` |

Les noms des fichiers pour le bureau commencent par `xdzip-<version>-`. Ceux de la [CLI](#xdzip-command-line) autonome commencent par `xdzip-cli-<version>-`. Utilisez `SHA256SUMS` sur la page de la version pour vérifier un téléchargement direct. Le programme d’installation Windows de GitHub n’est pas signé. Le fichier macOS utilise une signature ad hoc d’intégrité et n’est pas notarisé. Le [guide d’installation](docs/fr-FR/user-guide/install.md) indique comment lancer chaque version et définir les associations de fichiers.

Le Microsoft Store gère les mises à jour de son édition et la publie indépendamment de GitHub. Pour les téléchargements GitHub, consultez les mises à jour dans **À propos de xdzip**. L’installation depuis l’application est disponible pour les installations Windows par utilisateur admissibles, les apps placées dans Applications sur macOS et les installations Ubuntu effectuées avec le paquet DEB. Pour les copies portables, effectuez la mise à niveau manuellement. Consultez les [paramètres et mises à jour](docs/fr-FR/user-guide/settings-and-updates.md) et la [politique de support](docs/fr-FR/engineering/support-policy.md).

**Note sur les versions.** Les fonctions décrites ici correspondent au code source actuel. Pour un paquet publié, consultez les notes de la version correspondante. Les compilations à partir des sources sont des versions de développement expérimentales.

<a name="xdzip-using-xdzip"></a>

## Utiliser xdzip

Ouvrez une archive pour parcourir ses dossiers, rechercher et trier ses entrées et choisir les fichiers à extraire. Les dossiers restent en tête de liste dans les deux sens de tri. L’interface propose des vues en liste et en grille, les propriétés des fichiers, le choix de la langue dans les paramètres, les thèmes clair et sombre ainsi que le réglage de la taille du texte et de la densité. Par défaut, xdzip utilise la langue du système si elle est disponible et l’anglais américain dans les autres cas. Un ZIP ou ZIP64 ordinaire stocké dans un seul fichier peut indexer jusqu’à un million d’entrées pour la navigation. Les autres formats ont généralement des limites plus basses. L’extraction, la vérification de l’archive entière et la modification ont des limites distinctes. Consultez le [manuel](docs/fr-FR/user-guide/README.md) et la page sur [l’indexation des grandes archives](docs/fr-FR/engineering/large-index.md).

- **Aperçu du contenu.** Lorsque la fiche des propriétés est visible, sélectionnez un seul fichier pour afficher automatiquement un aperçu des textes bruts, images matricielles ou SVG statiques pris en charge. macOS prend aussi en charge HEIC et HEIF. L’aperçu apparaît après la lecture complète et les contrôles prévus par le format. Vous pouvez annuler l’opération. Consultez les [formats et limites de l’aperçu](docs/fr-FR/engineering/content-preview.md), notamment les budgets de décodage.
- **Extraire et ouvrir.** Choisissez une destination et la règle en cas de conflit de noms, suivez la progression et demandez l’annulation si nécessaire. Un double-clic sur un fichier ordinaire extrait une copie dans un dossier temporaire privé avant de l’ouvrir avec le système. Les modifications apportées à la copie ne sont pas répercutées dans l’archive. Au bout de sept jours, une ouverture ultérieure nettoie les copies mises en cache.
- **Créer et modifier.** Créez des archives ZIP, TAR, TAR.GZ, GZIP, XZ, TAR.XZ, LZ4, TAR.LZ4, Zstandard, TAR.ZST, LZH ou 7z et des images de données ISO9660/Joliet. Windows x64 peut aussi créer des ZIP auto-extractibles. Ajoutez, remplacez, déplacez, renommez ou supprimez des entrées dans les archives ordinaires admissibles stockées dans un seul fichier. Vous pouvez aussi faire glisser des fichiers hors de la liste, ou les déposer dans la liste et confirmer leur ajout. Le tableau ci-dessous détaille les formats et les limites.
- **Vérifier.** Testez toute une archive en arrière-plan sans écrire de fichiers. Les contrôles dépendent du format. TAR, ISO, UDF et Compound File (MSI), par exemple, permettent de contrôler la structure et la lisibilité sans somme de contrôle du contenu par fichier. Consultez la [vérification de l’intégrité des archives](docs/fr-FR/engineering/archive-checking.md).
- **Archives multiples.** Ouvrez les archives dans de nouveaux onglets par défaut ou configurez l’ouverture dans de nouvelles fenêtres. Les fenêtres d’une même session de connexion partagent le processus de l’application et l’icône de la zone de notification. Les tâches d’arrière-plan restent accessibles depuis le point d’accès aux tâches après la fermeture des pages de navigation.
- **Actions du gestionnaire de fichiers.** Sous Windows 11, utilisez **Afficher plus d’options** pour accéder au menu classique d’une édition installée. L’édition Microsoft Store peut aussi figurer dans le menu moderne. Sous Ubuntu, utilisez Fichiers (Nautilus) avec l’extension installée. Sous macOS, activez l’extension Finder dans les réglages système ou utilisez le service Finder enregistré. Consultez [l’extraction depuis le gestionnaire de fichiers](docs/fr-FR/user-guide/extract.md#xdzip-from-the-file-manager).

Les fichiers `.DS_Store` et les dossiers `__MACOSX` et `_MACOSX` sont exclus lors de la création d’archives et de l’ajout de fichiers. L’extraction vérifie les chemins et les autorisations d’écriture, traite les conflits de noms et écrit d’abord les fichiers de manière temporaire. Les formats compatibles contrôlent aussi le contenu. Les liens et objets spéciaux sont traités selon le format et le système. Consultez le [contrat d’extraction sûre](docs/engineering/zip-safe-extraction.md).

<a name="xdzip-format-support"></a>

## Formats pris en charge

Le tableau indique si xdzip peut ouvrir, créer ou modifier une archive. « Modifier » couvre l’ajout, le remplacement, le déplacement, le changement de nom et la suppression d’entrées dans une archive existante. La lecture et la vérification dépendent du contenu réel, de la méthode de compression et du type d’enregistrement. L’ouverture selon l’extension lance la détection. Consultez le [registre des formats intégrés](docs/fr-FR/engineering/format-registry.md) et le [guide de vérification de l’intégrité](docs/fr-FR/user-guide/checks-and-tasks.md).

| Format | Parcourir, extraire et vérifier | Créer | Modifier |
| --- | --- | --- | --- |
| ZIP et fichiers candidats ZIPX/XPI/APK/IPA/CBZ | Store, Deflate, Deflate64, BZip2, LZMA, XZ. Profils AES-256 et multivolumes admis | ZIP avec Store, Deflate, AES-256 et volumes numérotés | Archives ZIP/ZIPX admissibles utilisant les méthodes indiquées |
| TAR, TAR.GZ (dont `.tgz`) | Fichiers ordinaires et dossiers | Pris en charge. TAR archive les fichiers sans compression | Archives ordinaires en un seul fichier admissibles |
| GZIP (`.gz`) | Un fichier logique, y compris les membres GZIP concaténés. Un TAR dans GZIP expose son arborescence TAR | Un fichier ordinaire, Deflate 0/1/6/9 | Non pris en charge |
| BZip2, LZMA Alone, Lzip v1 | Un fichier ordinaire, dans les limites du flux et de la mémoire | Non pris en charge | Non pris en charge |
| TAR.BZ2, TAR.LZMA, TAR.LZ | Fichiers ordinaires et dossiers | Non pris en charge | Archives admissibles |
| XZ, TAR.XZ (dont `.txz`) | Contrôles déclarés, LZMA2, Delta/BCJ et flux concaténés | LZMA2, niveaux 1/6/9. XZ avec un fichier ordinaire | TAR.XZ |
| LZ4, TAR.LZ4 | Trames standard, blocs indépendants ou liés, concaténation et contrôles déclarés | LZ4, niveaux 1/6/9. LZ4 ordinaire avec un fichier ordinaire | TAR.LZ4 |
| Zstandard, TAR.ZST (dont `.tzst`) | Trames standard, fenêtres admises, Huffman/FSE, décalages répétés et contrôles déclarés | Zstandard, niveaux 1/6/9. Zstandard ordinaire avec un fichier ordinaire | TAR.ZST |
| 7z | Mots de passe, noms de fichiers chiffrés, archives solides et volumes numérotés | Chiffrement, archives solides et volumes numérotés | Archives admissibles non chiffrées et à volume unique |
| DEB | Membres ar courants, vues TAR du contrôle et des données, profils GZIP/XZ/Zstandard/BZip2/LZMA admis | Non pris en charge | Non pris en charge |
| LHA/LZH | En-têtes de niveaux 0–3, Store, LH1–LH7, LZS/LZ5 et CRC déclarés | LH5, niveaux 1/6/9. En-têtes de niveau 2, fichiers ordinaires et dossiers | Archives admissibles de niveau 2 dotées de CRC par entrée |
| [PMA](docs/fr-FR/engineering/pma-streams.md) | Niveau 0, PM0/PM1/PM2, structure CP/M exacte et CRC déclarés | Non pris en charge | Non pris en charge |
| [ARJ](docs/fr-FR/engineering/arj-streams.md) | Store et méthodes 1–4, en-têtes validés et CRC-32 complet du membre | Non pris en charge | Non pris en charge |
| [ARC](docs/fr-FR/engineering/arc-streams.md) | Ancienne et nouvelle variantes de Store, méthodes 3–9, structure exacte et CRC-16 complet du membre | Non pris en charge | Non pris en charge |
| [BH](docs/fr-FR/engineering/bh-containers.md) | Membres ordinaires `BH 05 07`, méthode 8 Deflate brut, tailles exactes et CRC-32 | Non pris en charge | Non pris en charge |
| [ALZ](docs/fr-FR/engineering/ace-alz-egg-format-sources.md) | Membres ordinaires à volume unique, Store ou Deflate brut, tailles exactes et CRC-32 | Non pris en charge | Non pris en charge |
| [EGG](docs/fr-FR/engineering/ace-alz-egg-format-sources.md) | Membres à volume unique non chiffrés et non solides, Store ou Deflate brut, tailles exactes et CRC-32 | Non pris en charge | Non pris en charge |
| [PEA](docs/fr-FR/engineering/disk-image-archive-format-sources.md) | PEA 1.x à volume unique, noms UTF-8, fichiers et dossiers PCOMPRESS0 stockés sans chiffrement. Aucun contrôle d’objet, de flux ou de volume | Non pris en charge | Non pris en charge |
| [Compound File (MSI)](docs/fr-FR/engineering/wim-msi-installer-format-sources.md#xdzip-compound-file-and-msi) | Parcours des stockages et extraction des flux bruts. Contrôle de la structure et de la lisibilité | Non pris en charge | Non pris en charge |
| [NSIS (`.exe`)](docs/fr-FR/engineering/wim-msi-installer-format-sources.md#xdzip-nsis-and-other-sfx) | Métadonnées statiques `firstheader` uniquement. Extraction et vérification de toute l’archive indisponibles | Non pris en charge | Non pris en charge |
| ASAR | Fichiers et dossiers stockés, plages de données partagées et SHA-256 déclarés pour les fichiers et les blocs | Non pris en charge | Non pris en charge |
| Z, TAR.Z (dont `.taz`) | Unix compress LZW, dictionnaires de 9 à 16 bits et profils admis de CLEAR et d’alignement | Non pris en charge | TAR.Z |
| Brotli, TAR.BR | Flux RFC 7932 ordinaires, fenêtres de 10 à 24 bits et toutes les transformations du dictionnaire statique | Non pris en charge | TAR.BR |
| AES Crypt (`.aes`) | Versions 0–3, mot de passe à l’ouverture, HMAC de tout le texte chiffré et règles de longueur propres à chaque version | Non pris en charge | Non pris en charge |
| UU/UUE, XXE | Lignes de longueur déclarée traditionnelles et enveloppes begin-base64 admises, y compris plusieurs membres | Non pris en charge | Non pris en charge |
| RAR | RAR historique, RAR5 et extensions du format RAR 7, y compris mots de passe, archives solides et multivolumes | Non pris en charge | Non pris en charge |
| CAB | Store, MSZip, Quantum, LZX et volumes dans le même dossier | Non pris en charge | Non pris en charge |
| ISO | ISO9660, Joliet et Rock Ridge | Image de données ISO9660 + Joliet | Non pris en charge |
| UDF | UDF | Non pris en charge | Non pris en charge |
| ZIP SFX | Archives ZIP auto-extractibles validées | Windows x64, Store/Deflate, `.sfx.exe` | Non pris en charge |

- Dans un ZIP AES-256, seul le contenu est chiffré. Les noms restent visibles. AES-128/192 et ZipCrypto traditionnel ne sont pas pris en charge. La lecture accepte les volumes `.zip.001`, PKZIP `.z01` et les conventions associées. La création utilise des volumes numérotés. Consultez [l’extraction avec mot de passe](docs/fr-FR/engineering/password-extraction.md) et [l’ouverture des archives multivolumes](docs/fr-FR/engineering/archive-opening.md).
- Les flux BZip2, LZMA Alone et Lzip v1 autonomes sont en lecture seule. LZMA ne fournit pas de somme de contrôle du contenu. Le contrôle porte sur la structure, les tailles et la lisibilité. Voir [contrat des flux à un seul fichier](docs/fr-FR/engineering/single-file-streams.md) pour les suffixes, les entrées, la mémoire et les conditions d’admission.
- XZ exécute chaque contrôle déclaré par le flux. Les flux déclarant None ne permettent de contrôler que la structure et la lisibilité. ZIPX lit les méthodes ZIP du tableau. Les variantes PPMd, JPEG et WavPack restent exclues. La modification conserve les données compressées et la méthode des entrées ZIP/ZIPX gardées. Les nouvelles entrées utilisent Deflate.
- La modification d’un 7z reconstruit l’archive. La mémoire de travail pour les données conservées et ajoutées est limitée à 512 MiB. Les nouvelles archives lisent les données en continu et ne sont pas soumises à cette limite de reconstruction. La modification des ZIP et 7z est indisponible si l’archive est chiffrée ou multivolume. Voir la [modification des archives](docs/engineering/archive-transfers.md).
- RAR, CAB et UDF sont en lecture seule. La [création d’images ISO](docs/fr-FR/engineering/iso-creation.md) accepte les fichiers ordinaires et les dossiers sur un seul volume. Les fichiers font moins de 4 GiB et chaque composant d’un nom Joliet est limité à 64 caractères UCS-2. Les images amorçables, l’écriture UDF ou Rock Ridge et la modification d’images ne sont pas disponibles. Lire une image ne la monte ni ne l’exécute. Consultez le [contrat RAR](docs/engineering/rar-format-sources.md) et le [contrat des moteurs de format](docs/fr-FR/engineering/native-codecs.md).
- La [création de SFX](docs/fr-FR/engineering/sfx-creation.md) produit un extracteur ZIP autonome pour Windows x64, sans dépendance à Qt ni à une installation de xdzip. Il permet de choisir la destination, de régler les conflits et d’annuler. Il n’exécute jamais les fichiers extraits. Mots de passe, archives multivolumes et commandes de lancement automatique sont indisponibles. Les `.exe` ordinaires restent exclus des associations de fichiers.
- L’exportation DEB présente les vues `members/`, `control/` et `data/`. Tout extraire écrit les membres bruts et le contenu décompressé. Aucun paquet n’est installé et aucun script n’est exécuté. La partie contrôle accepte l’absence de compression, GZIP, XZ et Zstandard. La partie données accepte en plus BZip2 et LZMA. La vérification confirme la lisibilité du conteneur et les contrôles déclarés, sans authentifier le paquet. Consultez le [contrat DEB](docs/fr-FR/engineering/deb-containers.md).
- La vérification de Z et TAR.Z porte sur la structure et la lisibilité, sans somme de contrôle. La création de Z autonome et la réparation de flux anciens endommagés ne sont pas proposées. Voir [contrat Unix compress](docs/fr-FR/engineering/compress-streams.md).
- UU/UUE et XXE sont contrôlés pour leur structure et leur lisibilité, sans somme de contrôle. Les chemins et modes des en-têtes n’accordent aucun droit d’écriture ni d’exécution. Les noms de fichiers encodés à la manière de GNU et la réparation multivolume sont hors du [profil des enveloppes de texte admis](docs/fr-FR/engineering/text-envelopes.md).
- Ouvrir ou extraire un ASAR en lecture seule ne lance pas son application. Les données externes `unpacked` et les liens déclarés sont visibles et restent incompatibles. Aucun accès aux dossiers voisins ni rétablissement du mode exécutable n’est autorisé. Les empreintes facultatives détectent les altérations. Sans empreinte, seuls la structure et la lisibilité sont contrôlées. Consultez le [contrat ASAR](docs/fr-FR/engineering/asar-containers.md).
- Un fichier NSIS n’est reconnu que si sa surcouche PE contient un enregistrement `firstheader` valide et de taille bornée. xdzip affiche cette ligne de métadonnées statiques. Le programme ne lance jamais l’amorce PE, le script d’installation, les modules ni les données incorporées. Il ne décompresse pas l’en-tête de l’installeur et ne permet pas d’en extraire les fichiers. L’entrée `.exe` est absente du glisser-déposer et des associations du gestionnaire de fichiers. La prise en charge se limite à NSIS.
- La création LHA/LZH utilise LH5, des en-têtes de niveau 2 et des noms UTF-8. Les anciens outils Windows peuvent afficher les noms dans leur page de codes. Les archives admissibles de niveau 2 peuvent être modifiées. Les anciens noms sont décodés avant l’interprétation des séparateurs. Les CRC présents sont vérifiés au décodage du contenu. Pour les anciennes entrées sans CRC, le contrôle porte sur la structure et la lisibilité. Mots de passe, volumes et réparation sont exclus. PMA a son propre profil en lecture seule. Voir [contrat LHA](docs/engineering/lha-containers.md).
- ARJ est en lecture seule. Les membres binaires et les anciens membres texte restituent les octets couverts par le CRC, sans conversion des sauts de ligne du système. Les mots de passe, volumes, profils de récupération ou de sécurité, liens et objets spéciaux ne sont pas admis. Les méthodes inconnues restent visibles et empêchent la réussite de la vérification de toute l’archive.
- ARC est en lecture seule. Les fichiers ordinaires sont traités séquentiellement et la fin est vérifiée avec exactitude. Le décodage avec mot de passe, les volumes, SFX et les extensions Spark ne sont pas admis. Les méthodes inconnues restent visibles sous leur identifiant, sans alias ZIP, et empêchent la vérification de réussir. L’ancien chiffrement XOR ne possède pas d’indicateur d’en-tête. Le décodage exige toujours un flux valide, une taille exacte et le CRC, sans solution de repli par mot de passe ou réparation.
- La lecture BH accepte le profil Deflate brut de méthode 8 pour `BH 05 07`. Les méthodes inconnues, le chiffrement, les dossiers, les volumes, les données de fin et les préfixes SFX sont exclus.

- Les grands ZIP utilisant l’index sur disque ne permettent pas actuellement de tester ni de modifier l’archive entière. Il est possible d’extraire une sélection de fichiers ordinaires dans les limites de la tâche. Consultez [l’indexation des grandes archives](docs/fr-FR/engineering/large-index.md).
- Les TAR contenant des fichiers creux et les extensions tierces ne sont pas pris en charge. Les GZIP ordinaires et les candidats XPI/APK/IPA/CBZ dérivés de ZIP suivent le [contrat des flux à un seul fichier](docs/fr-FR/engineering/single-file-streams.md). Les encodages historiques des noms sont détectés lorsque c’est possible. Les noms courts et les encodages mélangés peuvent rester ambigus. Consultez [l’encodage des noms de fichiers](docs/fr-FR/engineering/filename-encoding.md).

- Le mot de passe d’ouverture d’AES Crypt est demandé avant d’afficher les métadonnées. Tous les MAC du texte chiffré doivent être valides. Les versions 0–2 n’authentifient pas le reste de longueur final et les extensions ne confèrent aucun droit. Consultez le [contrat AES Crypt](docs/fr-FR/engineering/aescrypt-envelopes.md).
- La détection de Brotli et TAR.BR sans signature exige une validation complète du contenu. Le format ne contient pas de somme de contrôle. Les modifications de TAR.BR écrivent des méta-blocs non compressés standard, ce qui peut agrandir l’archive. Les dictionnaires partagés et les extensions à grande fenêtre sont hors du périmètre de [profil Brotli admis](docs/fr-FR/engineering/brotli-streams.md).

<a name="xdzip-command-line"></a>

## Ligne de commande

La CLI autonome fonctionne sans Qt ni session graphique. Elle peut lister, extraire, créer, enrichir et tester des archives, et produire du JSON. Décompressez le fichier `xdzip-cli-<version>-` et exécutez `./bin/xdzip --help` sous macOS ou Linux, ou `.\bin\xdzip.exe --help` dans Windows PowerShell. Conservez le processus de travail fourni à côté de l’exécutable. Le [guide de démarrage de la CLI](docs/fr-FR/integration/README.md) présente les premières commandes et l’installation dans PATH. Vous pouvez également compiler à partir des sources.

```sh
git clone https://github.com/xiaodaozhi/xdzip.git
cd xdzip
python3 scripts/cli.py build
python3 scripts/cli.py test
build/cli/bin/xdzip --help
build/cli/bin/xdzip list archive.zip --json
build/cli/bin/xdzip extract archive.zip -C output
```

Sous Windows, utilisez `python` et ajoutez `.exe` au nom du programme. Sous Linux, l’exécution exige OpenSSL 3 et les certificats du système. La compilation exige aussi `libssl-dev` et `pkg-config`. Le [guide de la CLI](docs/engineering/command-line.md) décrit les mots de passe, le mode non interactif, les codes de sortie, l’installation de la commande et les limites de mise à jour.

<a name="xdzip-build-from-source"></a>

## Compiler à partir des sources

L’interface de bureau utilise Qt Quick/QML et communique avec le cœur Rust par une fine couche C++. Les formats, les requêtes et les tâches sont indépendants de Qt. La lecture des 7z, RAR, CAB, ISO et UDF et l’écriture des 7z utilisent des processus isolés. La création d’images ISO s’effectue dans le processus Rust principal. Le SFX Windows autonome utilise des boîtes de dialogue système sans Qt.

La documentation se trouve dans `docs/`. Les paramètres visuels, la terminologie et les listes de dépendances utilisés par les générateurs et les validations se trouvent dans `specs/`. Les maquettes haute fidélité se trouvent dans `design/hifi/`.

Installez Python 3.10+, Git, CMake 3.24+, Ninja, rustup ainsi que le compilateur et le SDK de votre plateforme. Récupérez ensuite les sources et compilez le programme. Si vous avez déjà cloné le dépôt pour la CLI, commencez par `prepare`.

```sh
git clone https://github.com/xiaodaozhi/xdzip.git
cd xdzip
python3 scripts/desktop.py prepare --install-rust --install-qt
python3 scripts/desktop.py doctor
python3 scripts/desktop.py build
python3 scripts/desktop.py test
```

Sous Windows, utilisez un environnement de développement MSVC x64. Sous macOS, utilisez un terminal natif Apple Silicon et les outils en ligne de commande de Xcode. La chaîne d’outils Qt 6.8.2 par défaut se trouve dans `build/toolchain/`. Passez `--qt-prefix` si Qt est déjà installé. Le [guide de compilation](docs/engineering/developer-build.md) décrit les dépendances, le lancement, la préparation des paquets et les vérifications. La CLI et l’application de bureau se compilent par des points d’entrée distincts. La compilation de bureau fournit également une CLI privée dont le point d’entrée doit être activé explicitement.

Les paquets de développement sont des versions expérimentales. Les fichiers de développement Windows et Linux sont également disponibles après certaines exécutions manuelles réussies du [workflow des composants Qt](https://github.com/xiaodaozhi/xdzip/actions/workflows/qt-components.yml). La CI quotidienne ne produit pas automatiquement ces paquets.

<a name="xdzip-documentation-and-contributing"></a>

## Documentation et contributions

| Sujet | Guide de départ |
| --- | --- |
| Application de bureau et scripts | [Manuel](docs/fr-FR/user-guide/README.md), [intégration en ligne de commande](docs/fr-FR/integration/README.md) |
| Conception et implémentation du produit | [Référence technique](docs/fr-FR/reference/README.md) |
| Confidentialité et traitement des données | [Politique de confidentialité](PRIVACY.md), [version chinoise](PRIVACY.zh-CN.md) |
| Compiler et contribuer | [Guide des contributions](docs/fr-FR/contributing/README.md) |
| Modifications et travaux prévus | [Historique des modifications](CHANGELOG.md), [tickets GitHub](https://github.com/xiaodaozhi/xdzip/issues) |
| Contributions | [Règles de contribution](CONTRIBUTING.md), [code de conduite](CODE_OF_CONDUCT.md) |
| Politiques du projet | [Gouvernance](GOVERNANCE.md), [sécurité](SECURITY.md) |

Les retours et les contributions sont les bienvenus. Pour signaler un bogue, indiquez la version ou le commit, le système et l’architecture, les étapes de reproduction et le résultat observé. Créez vos propres exemples et évitez d’inclure des informations sensibles. Signalez les problèmes de sécurité en privé via [Report a vulnerability](https://github.com/xiaodaozhi/xdzip/security/advisories/new).

<a name="xdzip-license"></a>

## Licence

Le contenu original du projet est sous [licence Apache 2.0](LICENSE). Consultez aussi [COPYRIGHT](COPYRIGHT). Qt, les codecs, les polices et les autres dépendances sont soumis à leurs propres licences. La [documentation sur les licences](docs/fr-FR/engineering/licensing.md) recense les sources et les mentions. Les versions officielles ne fournissent pas d’archive séparée du code source correspondant. Consultez la [mise à disposition du code source](docs/engineering/source-delivery.md). Les obligations applicables en matière de code source, de réédition des liens ou de mentions restent en vigueur. Chaque version candidate doit faire l’objet de sa propre vérification. Voir les anciens tickets [#4](https://github.com/xiaodaozhi/xdzip/issues/4) et [#53](https://github.com/xiaodaozhi/xdzip/issues/53).
