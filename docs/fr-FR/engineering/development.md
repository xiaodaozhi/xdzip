<a name="xdzip-developer-preparation-guide"></a>

# Préparer l’environnement de développement

[English](../../engineering/development.md) | [简体中文](../../zh-CN/engineering/development.md) | [繁體中文（台灣）](../../zh-TW/engineering/development.md) | [日本語](../../ja-JP/engineering/development.md) | [한국어](../../ko-KR/engineering/development.md) | Français | [Deutsch](../../de-DE/engineering/development.md) | [Español](../../es-ES/engineering/development.md) | [Português (Brasil)](../../pt-BR/engineering/development.md) | [Русский](../../ru-RU/engineering/development.md)

Le [guide pour compiler et contribuer](../contributing/README.md) aide à choisir la construction et les vérifications adaptées au changement. L’[architecture](../design/system-architecture.md) et l’[organisation du dépôt](repository-layout.md) décrivent les frontières de Qt, de l’exécution Rust et des codecs.

<a name="xdzip-obtain-and-modify"></a>

## Récupérer et modifier le code

Partez du `main` courant et préservez les autres modifications du dossier de travail. La [politique de contribution](../../../CONTRIBUTING.md) définit le périmètre et la revue. Le [guide de livraison](../../../developers/engineering/repository-hygiene.md) couvre l’intégration autorisée. Un changement de contrat demande de mettre à jour ses versions linguistiques et les vérifications concernées. Les couleurs et la géométrie sont définies dans `specs/visual`.

<a name="xdzip-documentation-builds"></a>

## Construire et vérifier la documentation

Maintenez un README racine et une page correspondante dans docs pour chaque langue de l’interface. L’anglais par défaut utilise `README.md` et `docs/`. Les traductions utilisent `README.<locale>.md` et `docs/<locale>/`. Le manuel de maintenance utilise l’anglais américain et le chinois simplifié. Placez le choix de langue sous le titre. Affichez la langue courante en texte simple et liez les autres versions du même sujet. Depuis une page en chinois simplifié, liez la page de maintenance en chinois simplifié. Depuis les autres pages, liez la page de maintenance anglaise. Lors d’un ajout ou déplacement, mettez à jour index, liens entrants et ancres de section.

Les outils de documentation demandent Python 3.10+, lxml, Pandoc et Graphviz. Les polices CJK du système sont recommandées. Exécutez `python3 scripts/build-design-html.py` pour générer 16 documents de conception détaillés et une édition en un fichier sous build/docs-html. La source chinoise est dans `docs/zh-CN/design/`. L’archive téléchargeable est écrite sous dist/. Les autres guides techniques et de collaboration sont répertoriés dans docs/README.md. La sortie HTML est destinée à la documentation. L’interface du produit utilise Qt Quick.

Exécutez `python3 scripts/check-docs.py` pour contrôler la couverture des traductions, les liens réciproques de langue, la langue des liens de lecture, les ancres Markdown, le JSON et les ressources de conception. Le contrôle fonctionne hors ligne. Le générateur HTML vérifie la structure et les liens de ses pages. Exécutez `python3 scripts/test-check-docs.py` si vous modifiez le contrôleur.

<a name="xdzip-product-toolchain-preparation"></a>

## Préparer les outils du produit

Installez Xcode Command Line Tools et Qt sur macOS. Windows nécessite les outils MSVC, le SDK Windows et un Qt compatible. Linux nécessite le compilateur, un environnement glibc correspondant à la cible et les dépendances Qt Wayland/X11. Les fichiers d’outils à la racine et la CI fixent les versions. Consignez les sources et licences dans le [tableau des versions d’outils](toolchain.md). Suivez la [politique de prise en charge](support-policy.md) pour les systèmes et la maintenance. Pour examiner une provenance ou une licence, utilisez le ticket de revue du lancement concerné.

L’[application de développement Qt](../../../apps/desktop/README.md) fournit les commandes pour construire et lancer depuis un dossier propre, les options CMake, les architectures et cibles de déploiement macOS ainsi que les limites de l’essai hors écran. Revérifiez les chemins comportant des espaces ou des caractères chinois après une modification de la construction ou de l’installation. Ce projet ne requiert pas d’identité de signature payante pour le développement. Voir le [guide de construction](developer-build.md) pour la signature locale ad hoc du DMG et la limite de la CI qui exclut le déploiement de l’application Mac.

<a name="xdzip-independent-engineering-probes"></a>

## Essais techniques indépendants

La [sonde des outils](../../../tools/toolchain-probe/README.md) fournit la construction minimale et les commandes CTest d’une fenêtre Qt Quick autonome. Son README décrit les versions et le périmètre. Les résultats effectifs se trouvent dans le [ticket de validation initiale](https://github.com/xiaodaozhi/xdzip/issues/1).

La [sonde unifiée CMake/Cargo](../../../tools/bridge-probe/README.md) fournit une entrée de construction à la racine, des dépendances verrouillées et des contrôles de cycle de vie. Les dépendances sont détaillées dans le [choix de cxx](cxx-admission.md).
