# Compilations de développement et vérification par plateforme

[English](../../engineering/developer-build.md) | [简体中文](../../zh-CN/engineering/developer-build.md) | [繁體中文](../../zh-TW/engineering/developer-build.md) | [日本語](../../ja-JP/engineering/developer-build.md) | Français | [Deutsch](../../de-DE/engineering/developer-build.md) | [Español](../../es-ES/engineering/developer-build.md) | [Português (Brasil)](../../pt-BR/engineering/developer-build.md)

Cette page est l'entrée unifiée pour préparer, compiler, tester, empaqueter et vérifier les environnements des trois plateformes. L'orchestration commune est implémentée par [scripts/desktop.py](../../../scripts/desktop.py). Les documents de plateforme complètent les structures de déploiement, le comportement des outils et les limites système sans maintenir des commandes courantes séparées.

La CLI autonome utilise `python3 scripts/cli.py build`, `test` et `package` sans Qt, X11 ni session graphique. La compilation Linux exige aussi `libssl-dev pkg-config` ; l'exécution utilise OpenSSL et les certificats système. Les compilations du bureau créent automatiquement un répertoire CLI privé. Voir le [contrat CLI](command-line.md) pour les commandes, l'installation et les limites des mises à jour de confiance.

Les compilations de développement restent expérimentales et ne sont pas des installateurs Release officiels. Voir le [README](../../../README.fr-FR.md) racine pour les capacités et limites et la [politique de support](support-policy.md) pour le périmètre pris en charge. La revue historique de `v0.1.0` figure dans [#4](https://github.com/xiaodaozhi/xdzip/issues/4) et [#53](https://github.com/xiaodaozhi/xdzip/issues/53) ; chaque future version exige de nouveaux enregistrements pour ses candidats précis.

<a id="environment-and-entry-points"></a>

## Environnement et points d'entrée

Installez d'abord Python 3.10 ou ultérieur, Git, CMake 3.24 ou ultérieur, Ninja, rustup et le compilateur/SDK de la plateforme. La version Rust provient de rust-toolchain.toml à la racine ; Qt est épinglé à 6.8.2. Voir la [politique des outils](toolchain.md) pour les règles de dépendance et de mise à jour. Exécutez les commandes à la racine du dépôt. Sous Windows, remplacez `python3` dans les exemples par un `python` disponible et utilisez un environnement de développement MSVC x64.

## Répertoires locaux des outils

Lors du chargement de l'environnement PowerShell Visual Studio, conservez les valeurs Unicode de `USERPROFILE`, `LOCALAPPDATA` et `APPDATA` et restaurez-les si le chargeur modifie leur encodage. Qt utilise ces emplacements pour les chemins de cache et de configuration par défaut ; des valeurs corrompues peuvent provoquer des erreurs de système de fichiers même si le compilateur fonctionne.

`prepare` compile et vérifie dans le SDK existant un QtNetwork corrigé pour CVE-2026-116337 et CVE-2026-105946 ; la bibliothèque reconstruite ne remplace pas les autres modules d'exécution. Perl, make (nmake sous Windows) et l'outil en ligne de commande OpenSSL sont aussi nécessaires. Ubuntu exige `zlib1g-dev libpcre2-dev libicu-dev libglib2.0-dev libzstd-dev libkrb5-dev`. Sources, déclarations générées et caches de compilation restent sous `build/`, avec réutilisation par défaut de `build/toolchain/qt-network-security/<platform>`. Si les digests des entrées et de la bibliothèque réelle diffèrent, compilation et empaquetage échouent immédiatement ; relancez `prepare`. Les [notes de maintenance du correctif](../../../patches/qt/README.md) décrivent provenance, ABI, fonctionnement TLS et récupération.

Les dépendances locales de l'entrée unifiée utilisent uniquement `build/toolchain/`, ignoré par Git et non commité :

| Contenu | Répertoire fixe | Notes |
| --- | --- | --- |
| Qt SDK 6.8.2 | `build/toolchain/Qt/6.8.2/<platform-subdirectory>` | `<platform-subdirectory>` est `macos`, `msvc2022_64` ou `gcc_64` ; qmllint, qmake, configuration CMake et runtimes Qt en proviennent |
| Environnement d'outils Python | `build/toolchain/desktop-tools` | Environnement virtuel fixe pour aqtinstall et les scripts d'empaquetage et de vérification |
| Préparation des téléchargements Qt | `build/toolchain/qt-install-tool` | Téléchargements et entrées temporaires de l'installateur Qt uniquement |

`scripts/desktop.py prepare --install-qt`, `doctor`, `build` et `test` partagent ces emplacements par défaut. Ne créez pas de copies `.cache/Qt`, `.cache/desktop-tools` ou similaires pour chaque pipeline de développement. Ne fournissez explicitement `--qt-prefix`, `--qt-root` et `--tools-dir` que pour isoler des versions Qt, plateformes ou compilations expérimentales différentes. La CI peut les diriger explicitement vers des répertoires temporaires du runner ; ses chemins de repli ne changent pas les valeurs locales. CMake, Ninja, rustup, Rust et le compilateur C++ de plateforme font partie des outils de l'hôte et sont installés ou appelés selon les exigences de contrôle préalable, sans être copiés par le répertoire Qt.

Après un cycle de développement et le nettoyage des branches et worktrees, supprimez les répertoires et fichiers temporaires créés par ce cycle sous `build/`. Conservez les outils partagés réutilisables comme `build/toolchain/` ; retirez les compilations, diagnostics et préparations de téléchargement propres au cycle. Voir la [gestion du contenu et de l'historique du dépôt](../../../developers/engineering/repository-hygiene.md#研发周期工件收尾) pour les règles et la méthode manuelle.

| Plateforme | Environnement système à préparer d'abord | Sous-répertoire Qt SDK |
| --- | --- | --- |
| macOS | Terminal natif Apple Silicon, Xcode Command Line Tools et SDK macOS valide | macos |
| Windows | Outils C++ MSVC 2022 x64 et Windows SDK, Developer PowerShell ou ligne de commande x64 | msvc2022_64 |
| Ubuntu 24.04 | Compilateur C++ x86_64, paquets de développement OpenGL/EGL/xkbcommon et bibliothèques d'exécution X11 | gcc_64 |

Sous macOS, `xcode-select --install` prépare les outils en ligne de commande. Les paquets de développement de base Ubuntu incluent `libgl1-mesa-dev libegl1-mesa-dev libxkbcommon-dev libx11-dev libxext-dev libnautilus-extension-dev pkg-config` ; l'empaquetage exige aussi patchelf, binutils (readelf et strip) et les outils de paquets Debian, et la vérification d'exécution isolée nécessite Docker accessible. Installer Visual Studio sous Windows sans entrer dans un environnement de développement peut laisser cl, link, rc ou dumpbin indisponibles ; le contrôle préalable identifie les outils réellement manquants. L'entrée commune n'installe pas automatiquement les compilateurs système, SDK ou paquets nécessitant des droits administrateur.

Les développeurs Windows peuvent préparer leurs outils selon la [voie gratuite de licence Community](windows-licensing.md), en lisant et acceptant les conditions de cette version. Build Tools ou VC_redist seuls ne prouvent pas l'éligibilité. L'utilisation en entreprise et la redistribution associée sur les runners standard hébergés par GitHub disposent d'une autorisation publique distincte ; les mainteneurs n'ont pas à acheter Enterprise pour les builds hébergés ni installer localement un IDE Community, et cette autorisation ne s'applique pas aux machines auto-hébergées. Les outils 2019 existants n'ont pas à être désinstallés pour la revue de licence ; la compatibilité des alternatives locales avec le SDK de publication 2022 actuel doit être vérifiée séparément.

Toutes les commandes partagent ces options :

| Option | Valeur par défaut et rôle |
| --- | --- |
| `--qt-prefix` | D'abord `XDZIP_QT_PREFIX`, sinon `build/toolchain/Qt/6.8.2/<platform-subdirectory>` ; pointe vers le préfixe SDK, pas bin |
| `--build-dir` | `build/desktop`, répertoire CMake de compilation et test |
| `--output-dir` | `build/package`, répertoire des artefacts livrables et résultats de vérification |
| `--tools-dir` | `build/toolchain/desktop-tools`, outils Python isolés et matériaux de préparation |
| `--configuration` | `Release` ; une autre configuration de développement peut être sélectionnée explicitement |

build-dir et output-dir doivent être des sous-répertoires indépendants dans build/ du dépôt, conformément aux frontières de sortie de l'empaquetage et de la vérification. Conservez les mêmes options pour les étapes d'une exécution. `--qt-prefix` explicite prévaut sur la variable d'environnement ; une modification globale de CMAKE_PREFIX_PATH est inutile. L'entrée prépare les environnements de recherche Qt et des outils pour les sous-processus. Les lancements normaux du bureau ne doivent pas hériter des variables de tests hors écran. Les sorties générées vont uniquement dans build/, target/, dist/ ou caches ignorés et ne sont pas commitées.

## Exécution par étapes

Les étapes peuvent être exécutées indépendamment et ne relancent pas implicitement la préparation, compilation ou les tests précédents. Des prérequis manquants produisent des erreurs ; exécutez l'étape correspondante puis réessayez. Chaque étape vérifie son environnement avant de travailler ; `doctor` présente les problèmes sans compiler ni empaqueter.

Les compilations Windows protégées coordonnent les processus dans leur console existante. Des consoles distinctes peuvent fonctionner en parallèle ; les compilations partageant une même console existante sont séquentielles. Pendant une compilation, aucun autre code ne doit détacher ou rattacher des processus protégés à cette console. Une console temporaire créée par un processus qui n'en possède pas est réservée à ce processus et ses enfants ; les processus externes ne doivent pas s'y attacher. Les anciens scripts utilisent un verrou de repli commun pour les consoles sans handle de fenêtre et ne peuvent pas se coordonner avec les nouveaux ; ne démarrez pas une compilation de nouvelle version dans la même console sans fenêtre avant la fin de l'ancienne. Si les listes de processus ne peuvent être déterminées, arrêtez sans modifier la page de codes.

Pour une compilation non interactive, un wrapper de console indépendante peut être utilisé explicitement avec stdin, stdout et stderr entièrement redirigés. Exemple PowerShell :

```powershell
New-Item -ItemType Directory -Force build\isolated-logs | Out-Null
cmd /c "python scripts\windows_detached_console.py --timeout 7200 -- python scripts\desktop.py build <NUL >build\isolated-logs\desktop.log 2>&1"
$LASTEXITCODE
```

Le wrapper crée une console masquée indépendante par commande et termine les enfants au délai ou à la sortie du superviseur ; l'absence de signaux de fin provoque un échec. Les compilations parallèles exigent toujours des valeurs `--build-dir` distinctes ; le wrapper ne verrouille pas les répertoires de compilation. Les contrôles existants de dépendances desktop-build et `windows_ninja_safety.py` continuent, mais encapsuler d'autres commandes ne les ajoute pas automatiquement. Cette entrée ne concerne que les commandes non interactives ; invites, saisie de mot de passe et interactions avec pseudoconsole ne sont pas prises en charge.

`prepare` vérifie l'inventaire actuel des sources de formats produit. ZIP, TAR/GZIP, 7z, RAR, CAB et les conteneurs optiques utilisent Rust indépendant ; la liste officielle de sources produit est vide et les moteurs officiels UnRAR/7-Zip ne sont ni téléchargés ni compilés. Les références de test indépendantes sont préparées depuis leurs propres sources épinglées et séparées des dépendances d'exécution du produit. Les avis de source originaux des correspondances RAR limitées restent dans les paquets et matériaux de source correspondante ; voir le [contrat de licence](licensing.md). `build` n'utilise que les entrées préparées et ne télécharge pas de versions flottantes. Pour des tests Rust uniquement, compilez d'abord `xdzip-native-codec`, pointez `XDZIP_NATIVE_CODEC` sur ce processus, puis exécutez les tests du workspace. Le flux du moteur natif exécute aussi `cargo test -p xdzip-runtime --locked native_backend_tests -- --ignored` ; `test` unifié et Foundation CI l'exigent tous deux. Un état ignored ne vaut pas contrôle de format réussi. Le [contrat du moteur natif](native-codecs.md) décrit sources, autorisation des flux et limites de licence.

| Commande | Rôle et prérequis |
| --- | --- |
| `doctor --for-stage build` | Vérifie PATH, Rust épinglé, Qt, compilateur, SDK et versions ; vérifie l'environnement de compilation par défaut |
| `prepare` | Prépare outils et dépendances partagés ; le téléchargement ou l'installation de Qt, Rust et matériaux de paquet exige les options explicites ci-dessous |
| `build` | Configure et compile le produit réel, les tests et diagnostics ; exige un environnement préparé et accepte des changements non commités |
| `test` | Exécute par défaut les tests du workspace Rust, qmllint et CTest ; exige une compilation terminée de même configuration et accepte les changements non commités |
| `package` | Déploie le runtime, collecte les matériaux de licence et provenance, audite le déploiement et produit les artefacts sélectionnés ; exige des sources propres, les reçus de build et test du même commit et les outils de paquet |
| `verify` | Vérifie un runtime ou paquet existant sans reconstruire, installer le produit ou changer les associations par défaut ; exige l'artefact et l'environnement de vérification de plateforme |

Le contrôle préalable distingue outils requis manquants, versions incompatibles et outils inutiles à l'étape actuelle ; l'absence de Docker ou Inno Setup ne bloque pas les compilations ordinaires. Le contrôle d'empaquetage vérifie réellement l'appel Rust/Cargo et les métadonnées de dépendances verrouillées afin que les problèmes de PATH ou cache ne soient pas découverts seulement pendant la collecte de licences. Les commandes échouées et les limites de vérification restent dans la sortie locale ou CI ; une réalisation partielle n'est pas déclarée réussie.

Préparez d'abord les outils Qt et Rust épinglés :

```sh
python3 scripts/desktop.py prepare --install-rust --install-qt --qt-root build/toolchain/Qt
python3 scripts/desktop.py doctor --for-stage build
python3 scripts/desktop.py build
```

`--install-rust` utilise rustup installé pour préparer l'outil épinglé par le dépôt ; il n'installe pas rustup. `--install-qt` télécharge explicitement le SDK Qt épinglé dans `build/toolchain/Qt` par défaut. Un SDK existant n'a pas besoin d'être retéléchargé ; transmettez son chemin à chaque étape :

```sh
python3 scripts/desktop.py doctor --qt-prefix build/toolchain/Qt/6.8.2/gcc_64
python3 scripts/desktop.py build --qt-prefix build/toolchain/Qt/6.8.2/gcc_64
```

Après compilation, préparez explicitement les entrées de test publiques du lot complet `native_backend_tests`. Les fixtures CAB, ISO/UDF et RAR de base sont conservées dans les sources du projet sous `tests/fixtures/cab-iso/` et `tests/fixtures/native-archives/`. Les tests 7z créent leurs propres archives avec le processus Rust nouvellement compilé. Ce lot d'exécution ne nécessite ni outils officiels 7-Zip/UnRAR ni images montées ; les autres références d'interopérabilité Foundation suivent le [contrat du moteur natif](native-codecs.md).

Pour la première acquisition des entrées publiques épinglées RAR PPM, anciens mots de passe et autres, exécutez explicitement :

```sh
python3 scripts/prepare-rar-rust-fixtures.py --output build/native-test-inputs
```

Avec un cache épinglé existant vérifié par taille et SHA-256, utilisez explicitement le mode hors ligne, en remplaçant le chemin et en conservant la structure du sous-répertoire `junrar/` du manifeste :

```sh
python3 scripts/prepare-rar-rust-fixtures.py --output build/native-test-inputs --archives /path/to/fixed-rar-cache --offline
```

Sources et avis épinglés figurent dans `scripts/rar-rust-fixtures.json` et son inventaire public de compatibilité. La sortie `sources.json.environment` utilise des chemins relatifs à la racine des sources. Les matériaux actuels de source correspondante ne contiennent pas ces sept fichiers publics de cache ; `--offline` de Cargo ne les fournit pas. Le préparateur n'exporte pas de variables d'environnement shell. Avant le `test` complet, démarrez un processus enfant avec la transmission explicite suivante :

```sh
python3 -c "
import json, os, pathlib, subprocess, sys
root = pathlib.Path.cwd()
worker = pathlib.Path(sys.argv[1]).resolve(strict=True)
prepared = json.loads((root/'build/native-test-inputs/sources.json').read_text(encoding='utf-8'))
environment = os.environ.copy()
environment.update({name: str(root/value) for name, value in prepared['environment'].items()})
environment.update(XDZIP_NATIVE_CODEC=str(worker), XDZIP_NATIVE_CODEC_WORKER=str(worker), XDZIP_CAB_BACKEND='rust')
environment['XDZIP_RAR_FIXTURE_PASSWORD'] = 'password'
subprocess.run([sys.executable, 'scripts/desktop.py', 'test', *sys.argv[2:]], env=environment, check=True)
" build/desktop/tools/native-codec/xdzip-native-codec
```

Sous Windows, ajoutez `.exe` au chemin du processus. Avec un SDK, build-dir ou tools-dir personnalisé, ajoutez les mêmes options après ce chemin, par exemple `--qt-prefix /path/to/Qt/6.8.2/gcc_64`, et adaptez le chemin au répertoire réel. `XDZIP_CAB_BACKEND=rust` sélectionne le contrôle Rust CAB fixé ; `password` est une valeur publique existante de fixture transmise seulement à l'enfant de test. Cette transmission conserve les tests par défaut du workspace et le lot ignored complet ; elle ne télécharge pas les entrées et ne compile pas implicitement le processus.

`test` n'active hors écran/logiciel que pour les enfants et ne contamine pas les commandes bureau suivantes. Pour les changements courants, reconstruisez et vérifiez selon l'impact. La vérification du seul noyau Rust peut lancer `cargo test --locked` pour le crate affecté sans Qt. `test --skip-rust` exécute qmllint et Qt CTest, mais omet le workspace Rust et les tests complets du moteur natif ; les reçus marquent Rust `not-run`. Lorsqu'une conclusion Rust est nécessaire, exécutez-la séparément et consignez les résultats réels.

Les tests du navigateur peuvent être lancés directement ou en groupes via l'entrée existante de la suite. Chaque processus crée une racine temporaire courte et unique sous `build/test-tmp/` du projet. À la sortie, après destruction des objets de test et avant destruction de l'application Qt, il restaure TEMP, TMP et TMPDIR et nettoie la racine. Les fichiers de configuration par défaut sont aussi isolés. Unix vérifie la longueur en octets de la racine temporaire et du nom IPC existant par rapport à la capacité réelle de sun_path. Un chemin trop long fait échouer explicitement la précondition de fixture ; raccourcissez le chemin du checkout plutôt que sortir du projet ou modifier l'IPC produit. Les processus auxiliaires d'un groupe héritent de la racine temporaire du parent pour préserver l'identité du répertoire IPC d'instance unique, utilisent des sondes préalables privées et se terminent avant le nettoyage du parent. Les groupes ne partagent pas de racine. Les enfants de redémarrage de langue et de mode d'ouverture auxquels une configuration de test est explicitement transmise continuent de la partager au lieu d'en utiliser une nouvelle vide. Avant l'exécution, vérifiez réellement création de fichiers enfants, verrous, préparation, commits atomiques et relecture de configuration. Un échec signale `Fixture precondition` et un code non nul ; ne sautez pas les tests et n'affaiblissez pas les assertions de masquage des fenêtres. L'existence du répertoire et QTemporaryDir::isValid seuls ne prouvent pas qu'il est accessible en écriture ; les permissions système ne sont pas modifiées pour faire passer les fixtures.

Les chemins de configuration de test utilisent la sauvegarde/restauration limitée de `tests/test_environment`. Windows conserve les valeurs en caractères larges et distingue les entrées absentes des entrées explicitement vides dans le bloc d'environnement système ; Unix conserve les octets d'origine. Le setter CRT MSVC supprime les valeurs vides ; le bloc natif est donc restauré séparément. Vérifiez la présence des valeurs vides avec processValue de l'auxiliaire ou QProcessEnvironment, sans promettre la distinction vide/nul du CRT Qt. Cet auxiliaire est réservé au thread principal des tests séquentiels : définissez-le avant la création des moteurs/processus concernés et restaurez-le après destruction ou jonction. Il ne sert pas au code produit ni aux modifications concurrentes de l'environnement. Les caractères ASCII, chinois et non représentables en ANSI exigent la vérification des fichiers réels au chemin d'origine, des enfants de restauration de langue et de la conservation des fenêtres et brouillons en cas d'échec. Les résultats automatisés restent distincts des opérations de configuration effectuées par une personne.

Le produit réel porte le nom `xdzip` sur toutes les plateformes. Entrées de compilation par défaut :

| Plateforme | Point d'entrée |
| --- | --- |
| macOS | `build/desktop/apps/desktop/xdzip.app/Contents/MacOS/xdzip` |
| Windows | `build/desktop/apps/desktop/xdzip.exe` |
| Linux | `build/desktop/apps/desktop/xdzip` |

L'exemple de composant `xdzip-ui-preview` ne sert pas à l'acceptation sur fichiers réels. Adaptez les chemins si build-dir change. L'identité de compilation réelle enregistre version, canal, commit et état modifié. Voir l'[application Qt de développement](../../../apps/desktop/README.md) pour les modules et le débogage CMake de bas niveau.

## Génération des polices d'interface

`prepare` installe FontTools/Brotli épinglés dans l'environnement partagé et met en cache les polices d'interface upstream vérifiées par hash. Chaque compilation bureau collecte automatiquement les catalogues de langues prises en charge et le texte fixe QML/C++, puis génère les sous-ensembles de polices sous `<build-directory>/generated/fonts/`. Ajouter des caractères ne nécessite pas de commiter un binaire de police. Après préparation, les compilations fonctionnent hors ligne ; un cache source manquant ou corrompu provoque un échec explicite. Les compilations CMake directes doivent sélectionner cet environnement via `Python3_EXECUTABLE`.

Pour la prévisualisation HTML seule, l'export hors ligne, l'import manuel de sources et les contrôles de police, voir le [guide des polices partagées](../../../design/assets/fonts/README.en.md). L'empaquetage valide le manifeste de polices de la compilation produit correspondante sans le régénérer. Le texte fixe d'interface et la sélection de police système pour les données utilisateur conservent leurs rôles respectifs.

## Empaquetage et vérification

`test` et `package` contrôlent reçus de compilation, empreintes des sources, configuration, identité du SDK et fichiers produit réels. Recompilez d'abord après toute modification des sources ou de la configuration. Commitez les sources livrables avant l'empaquetage, puis compilez et testez ce commit propre. package ne doit pas contourner les contrôles en ignorant des changements, modifiant l'identité des artefacts ou réutilisant d'anciens rapports. Matériaux de licence, métadonnées Rust et version CLI réelle doivent correspondre à ces sources. Les outils de plateforme vérifient les restrictions d'utilisation et d'écrasement des répertoires de paquet existants. Préférez un nouvel output-dir pour les candidats reconstruits afin d'éviter de mélanger des fichiers anciens.

Sous Windows/Linux, préparez les outils de paquet puis exécutez les étapes unifiées :

```sh
python3 scripts/desktop.py prepare --packaging
python3 scripts/desktop.py doctor --for-stage package
python3 scripts/desktop.py build
python3 scripts/desktop.py test
python3 scripts/desktop.py package --kind all
python3 scripts/desktop.py verify --kind all
```

`prepare --packaging` prépare les dépendances Python isolées de validation, les textes de licence épinglés, les caches de schémas et les matériaux de plateforme ; Windows inclut le CRT et Inno Setup épinglés. Les développeurs installent toujours d'abord les outils système. `package --kind runtime` ne prépare que le répertoire d'exécution déployé et les matériaux d'audit. On peut ensuite exécuter `package --kind installer` avec le même output-dir pour créer un installateur depuis le runtime vérifié ; Linux crée aussi un tar.gz conservant les permissions d'exécution. `--kind all`, par défaut, effectue les deux en une commande. L'empaquetage Windows utilise Release ou RelWithDebInfo sans déployer les bibliothèques Qt Debug. Windows fournit un répertoire déployé et un installateur par utilisateur ; Linux fournit un répertoire d'exécution, tar.gz et .deb. La vérification Windows exige l'exécutable de test du navigateur de même configuration et les entrées Qt Test/hors écran. La vérification Linux exige m2-flat.zip généré par les tests de compilation. Les fichiers de test n'entrent pas dans le répertoire produit. verify `--kind runtime` vérifie le runtime ; `--kind all` par défaut vérifie les livrables sélectionnés. La vérification ordinaire n'installe pas, ne met pas à jour ou ne revient pas en arrière et ne modifie pas les associations de fichiers utilisateur.

Un DMG macOS exige une compilation principale choisie séparément et autorisée à s'enregistrer comme candidate de gestion de documents, afin d'éviter que les copies ordinaires de développement ou test apparaissent dans « Ouvrir avec » :

```sh
python3 scripts/desktop.py prepare --packaging
python3 scripts/desktop.py build --build-dir build/desktop-dmg --macos-handler-rank Alternate
python3 scripts/desktop.py test --build-dir build/desktop-dmg
python3 scripts/desktop.py doctor --for-stage package --build-dir build/desktop-dmg
python3 scripts/desktop.py package --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
python3 scripts/desktop.py verify --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
```

Le rang du gestionnaire des builds macOS ordinaires est `None` par défaut et ne change pas les choix utilisateur ; choisissez `Alternate` uniquement pour une application principale durable ou une entrée DMG. Le paquet macOS local ne prend en charge que `all` et produit un DMG de développement arm64 avec la signature ad-hoc d'intégrité nécessaire. verify vérifie Cocoa/Metal après déplacement et une extraction réelle. La CI ordinaire ne doit pas exécuter le paquet Mac ni déployer, signer, compresser ou envoyer de paquet d'application ; voir le [protocole de publication](release-protocol.md) pour l'exception du Release dédié. Le [guide de déploiement Mac](macos-deployment.md) décrit les limites complètes et la maintenance des enregistrements de copies.

Voir [déploiement Linux](linux-deployment.md), [DMG Mac](macos-deployment.md) et [installation et associations](installers-associations.md) pour les structures et outils sous-jacents. Les installateurs Windows/Linux ne sont pas formellement signés ; macOS n'a ni signature Developer ID ni notarisation. Diagnostiquez les problèmes d'installation ou d'exécution avec le produit réel et la vérification de plateforme pertinente ; l'exécution sous le SDK ne démontre pas la vérification d'un paquet autonome.

## Relation avec la CI

Avant commit ou push, exécutez les compilations ou vérifications locales adaptées au contenu proposé. Les changements documentaires exécutent `python3 scripts/check-docs.py` et vérifient les sorties générées affectées. Les changements Rust vérifient le format et compilent ou testent au moins les crates affectés. Les tests de moteurs de format isolés préparent le processus réel et les fixtures nécessaires comme ci-dessus. Les changements C++, de pont et QML compilent le produit affecté avec `python3 scripts/desktop.py build`, puis exécutent les tests pertinents. Chaque changement ne nécessite pas toute l'interopérabilité des formats ou dix répétitions complètes de la suite du navigateur.

À l'ajout ou la modification d'une fenêtre QML, exécutez `python3 scripts/check-window-contracts.py --self-test`, `python3 scripts/check-operation-components.py` et les régressions de disposition affectées en deux langues et plusieurs tailles. Après compilation, `python3 scripts/desktop.py test --skip-rust` exécute tous les contrôles de composants Qt et CTest. Autre possibilité : après configuration des chemins SDK Qt et de l'environnement hors écran, exécutez les cas pertinents via des entrées existantes telles que `ctest --test-dir build/desktop -R qt_task_components --output-on-failure`. Les changements de boutons radio/cases à cocher exécutent aussi les régressions QAccessibleActionInterface adaptées, telles que `accessibleRadioActionsCommitSelection` et `accessibleCheckboxActionsCommitIntent`.

La [CI courante](../../../.github/workflows/docs.yml) n'exécute que des contrôles statiques légers et ne remplace pas les compilations locales produit. [Foundation](../../../.github/workflows/foundation.yml) et le [workflow Qt](../../../.github/workflows/qt-components.yml) sont lancés manuellement à la demande. Qt utilise le même desktop.py pour compilation, tests Qt, empaquetage et vérification d'exécution Windows/Linux. [ci-desktop.ps1](../../../scripts/ci-desktop.ps1) conserve les contrôles à la demande de répétition, visuels, d'interopérabilité et de compatibilité d'installation ; il n'est pas obligatoire pour la vérification locale courante.

Les cycles historiques du produit complet, d'installation réelle avec mise à jour ou retour arrière et d'associations ne sont exécutés que sur des hôtes CI Windows/Linux jetables ou des conteneurs isolés ; l'entrée unifiée ne les rend pas ordinaires par défaut dans verify local. Le job macOS du workflow Qt à la demande utilise l'audit de déploiement en mode SDK et des contrôles Cocoa de démarrage/ouverture de fichiers, en conservant seulement journaux et JSON. Voir la [politique CI](../../../developers/engineering/ci-policy.md) pour tous les contrôles et limites de preuve.

## Limites des fichiers et tâches

L'ouverture permet seulement la navigation ; elle n'extrait pas automatiquement. Les fichiers ordinaires homonymes permettent demander, ignorer, renommer ou remplacer explicitement. Ne publiez un fichier entier qu'après vérification et synchronisation de son contenu. Une annulation avant validation ou une erreur d'écriture conserve l'ancien fichier ; les entrées extraites déjà validées restent présentes. Fermer une fenêtre de tâche masque seulement les détails. La sortie utilise la décision sur les tâches actives et la barrière d'arrêt asynchrone.

ZIP peut restaurer des liens symboliques relatifs et bits exécutables sous conditions POSIX restreintes, mais pas les bits de privilège, attributs étendus ou dates originales de modification. Cela ne démontre ni l'installation d'un paquet d'application ni la vérification de sa signature. Source et destination doivent respecter le [contrat d'extraction sécurisée](zip-safe-extraction.md). La préparation après plantage est seulement signalée et conservée pour inspection ; un contenu inconnu n'est ni récupéré ni supprimé automatiquement.

## Microsoft Store MSIX

Pour une soumission Store, suivez la [procédure des mainteneurs](../../../developers/market/windows/build.md).

<a id="windows-msix-实验构建"></a>

## Compilation expérimentale MSIX Windows

MSIX réutilise la compilation bureau Windows Release, les tests et le répertoire d'exécution audité du même commit. Terminez `prepare --packaging`, `build` et `test` comme ci-dessus, puis exécutez avec les mêmes options :

```powershell
python scripts/desktop.py package --kind runtime
python scripts/desktop.py package --kind msix --msix-name xdzip.Development --msix-publisher 'CN=xdzip development' --msix-publisher-display-name xdzip
python scripts/desktop.py verify --kind msix
```

L'identité d'exemple est réservée au développement local. Le nom de paquet, l'éditeur et son nom affiché d'une identité Store officielle doivent correspondre à Partner Center. La sortie `build/package/xdzip-windows-x64.msix` est un paquet expérimental non signé, pas automatiquement distribuable en installation latérale ni certifié Store. Une installation latérale normale exige une signature approuvée par l'appareil correspondant à l'éditeur. Le mode restreint de développement non signé de Windows 11 exige en outre un OID d'éditeur précis et, pour du contenu exécutable, généralement des droits administrateur ; l'identité d'exemple ne satisfait pas ce mode. Les scripts ne créent pas de certificats, ne signent pas, n'installent pas et ne soumettent pas à Store. Le minimum technique est Windows 10 22H2 x64 et n'élargit pas les engagements de plateformes prises en charge. La version `1.0.<product build>.0` est distincte de la version affichée ; la politique de version Store doit être revue avant l'épuisement des numéros de build.

La compilation exige des sources propres, des reçus build/test correspondants, un répertoire d'exécution entièrement audité et MakeAppx et MakePRI du Windows SDK. Les scripts revérifient le runtime et ses marqueurs de capacités de mise à jour, association et menu, génèrent manifeste et ressources d'icônes dans une préparation séparée, comparent chaque fichier empaqueté et enregistrent les hashes. Ils n'écrasent ni paquets ni préparation existants. Le manifeste déclare les types d'extension existants du registre des formats, une entrée bureau à confiance complète et des classes de menu propres au canal.

Les icônes de l'application incluent des variantes d'échelle et tailles cibles exactes sous formes par défaut, sans plaque sur fond sombre et sans plaque sur fond clair. MakePRI les indexe dans `resources.pri`, permettant à Windows de choisir la taille prévue sans plaque système. Les associations déclarent leur propre logo de format et réutilisent les images PNG 16–256 pixels des ICO Windows existants sans modifier dessin, taille visible, marges ou position. La politique est dans [msix-icons.json](../../../specs/visual/msix-icons.json). Ce sont des ressources du paquet installé ; les logos de fiche Partner Center sont distincts. `verify --kind msix` confronte les nouveaux reçus d'icônes au paquet réel, notamment dimensions, variantes complètes, images canoniques de format et présence de l'index. Les changements ne parviennent aux utilisateurs installés qu'après compilation et livraison d'un paquet mis à jour.

L'exemple utilise `--msix-channel sideload` par défaut, qui n'enregistre que le menu classique Windows 11. Pour une soumission Store, utilisez `package --kind msix --msix-channel store` avec l'identité exacte Partner Center. Les menus moderne et classique sont alors déclarés ; l'entrée moderne reste masquée jusqu'à ce que le paquet possède réellement une signature système Store. Les marqueurs de canal sont ajoutés lors de l'empaquetage sans recompilation du runtime audité. Gardez les sorties dans des répertoires distincts. Voir la [politique des canaux de menu](shell-actions.md#limite-du-paquet-windows). Licences, installation réelle, associations, désinstallation, signature Store et acceptation sur le système cible restent distinctes de la réussite de l'empaquetage.

Le manifeste MSIX déclare les dix langues du registre partagé correspondant aux catalogues intégrés ; les reçus enregistrent et vérifient la liste. Ce flux MakeAppx ne convertit pas automatiquement les ressources de traduction Qt en déclarations de langues du paquet. « Langues prises en charge par le package » dans Partner Center reflète le paquet téléchargé ; après correction des sources, il faut reconstruire et remplacer le paquet du brouillon pour l'actualiser. Les « langues supplémentaires de la fiche Store » sont configurées séparément et ne remplacent pas les déclarations du manifeste. Voir la [documentation Microsoft des langues du manifeste](https://learn.microsoft.com/windows/apps/design/globalizing/manage-language-and-region).

## Preuves et nouvelle vérification

Sélectionnez les contrôles affectés selon la [vérification générale](../../../developers/qa/README.md) et la [vérification bureau à la demande](../../../developers/qa/private-test-manual.md) ; le travail courant n'exige pas de formulaire d'acceptation manuelle. Les reçus de build et test sont `desktop-build.json` et `desktop-test.json` sous build-dir. Les reçus de préparation, paquet et vérification sont `<stage>.json` sous output-dir et lient les preuves sous-jacentes. Les reçus automatisés enregistrent les étapes et sources réelles. Le rendu hors écran/logiciel, les contrôles Cocoa natifs et les contrôles manuels effectués sont consignés séparément et ne se remplacent pas. L'acceptation existante demeure dans [#5](https://github.com/xiaodaozhi/xdzip/issues/5) et n'est pas répétée pour la seule régénération d'un inventaire de candidats.

La livraison des artefacts Windows/Linux inclut les sources du même commit et les avis de licence ; Actions conserve les artefacts 14 jours. Après expiration, reconstruisez avec les identités et entrées épinglées. deployment-audit.json et les inventaires CycloneDX par source/fichier aident à vérifier la provenance mais peuvent être incomplets et ne prouvent pas l'approbation des licences. Voir [livraison des sources](source-delivery.md) et le [contrat de vérification automatisée](../../../developers/engineering/candidate-verification.md) pour la reconstruction des sources et le remplacement de Qt.

Les builds, captures, mesures, rapports et paquets restent uniquement des artefacts locaux ignorés ou de CI. Ne présentez pas des formulaires générés en attente, contrôles hors écran ou SBOM partiels comme des validations d'environnement propre et de toutes les conditions de publication.

## Framework de mise à jour macOS

Les builds natifs utilisent `prepare-sparkle.py` pour préparer `build/toolchain/Sparkle/2.10.0-xdzip-proxy.2`. La bibliothèque principale du framework est compilée depuis la source upstream épinglée et le correctif proxy ; assistants d'installation, ressources et outils de signature conservent la provenance du SDK officiel. Voir le [contrat de dépendances](dependencies.md#macos-更新提供者). Les reçus de build valides peuvent être réutilisés ; un ancien SDK officiel ne remplace pas le framework dérivé avec interfaces proxy. La CI peut sélectionner explicitement un répertoire SDK temporaire.

`python3 scripts/prepare-sparkle.py --output build/toolchain/Sparkle/2.10.0-xdzip-proxy.2` utilise Clang natif de Xcode Command Line Tools ou Xcode existant et ne nécessite pas de SDK Qt distinct. Les archives originales SDK et source sont mises en cache dans `build/source-cache/` ; le mode hors ligne peut recevoir séparément `--archive` et `--source-archive`. La compilation n'exécute pas les scripts des sources téléchargées, ne modifie pas les proxies système, n'accède pas aux clés de signature de production et ne collecte pas d'archives de livraison des sources du projet.

`check-macos-proxy.py` vérifie par défaut uniquement le téléchargeur propre en boucle locale. `--system-conflicts` est réservé aux runners Apple Silicon hébergés jetables dans un workflow dédié. Un auxiliaire restreint prépare, restaure et vérifie les paramètres proxy système et hosts des domaines de test fixés. N'ajoutez pas cette option localement.
