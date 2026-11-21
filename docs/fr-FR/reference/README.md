<a name="xdzip-product-design-and-technical-reference"></a>

# Conception du produit et référence technique

[English](../../reference/README.md) | [简体中文](../../zh-CN/reference/README.md) | [繁體中文](../../zh-TW/reference/README.md) | [日本語](../../ja-JP/reference/README.md) | [한국어](../../ko-KR/reference/README.md) | Français | [Deutsch](../../de-DE/reference/README.md) | [Español](../../es-ES/reference/README.md) | [Português (Brasil)](../../pt-BR/reference/README.md) | [Русский](../../ru-RU/reference/README.md)

Consultez d’abord l’[architecture du système](../design/system-architecture.md) pour les chemins d’exécution du bureau et de la CLI. Les sections ci-dessous mènent au contrat qui définit chaque comportement. Le [tableau des formats](../../../README.fr-FR.md#xdzip-format-support) présente les opérations prises en charge. Pour les étapes d’utilisation ou la préparation du développement, consultez le [manuel](../user-guide/README.md), les [exemples de CLI](../integration/README.md) et le [guide de contribution](../contributing/README.md).

<a name="xdzip-architecture-and-interfaces"></a>

## Architecture et interfaces

- [Guide de conception](../design/index.md)
- [Exigences et périmètre des opérations](../design/product-requirements.md)
- [Modèle de domaine, identités d’entrées et versions des sources](../design/domain-data.md)
- [Passerelle de commandes et d’événements Qt/Rust](../design/qt-rust-bridge.md)
- [Ordonnancement, progression, annulation et arrêt](../design/task-scheduling.md)
- [Budgets de performance et vérification](../design/performance.md)
- [Définitions et limites d’implémentation](../design/glossary.md)
- [Traçabilité des exigences et des vérifications](../design/requirements-traceability.md)
- [Décisions d’architecture](../design/architecture-decisions.md)
- [Évolution des prototypes vers le bureau et Rust](../design/architecture-evolution.md)
- [Organisation du dépôt et dépendances](../engineering/repository-layout.md)
- [Contrats d’interface et contrôles](../engineering/contracts.md)
- [Exécution du produit et cycle de vie de Qt](../engineering/product-runtime.md)
- [Sessions et arrêt de l’application](../engineering/application-sessions.md)
- [Choix de la passerelle cxx](../engineering/cxx-admission.md)
- [Services de calendrier et d’encodage de l’hôte](../engineering/host-platform-services.md)

<a name="xdzip-reading-and-writing-archives"></a>

## Lecture et écriture des archives

- [Autorisations, préparation et publication sûre](../design/file-security.md)
- [Navigation en lecture seule](../engineering/readonly-browser.md)
- [Navigation ZIP en lecture seule](../engineering/zip-readonly-browsing.md)
- [Indexation des grandes archives et navigation à la demande](../engineering/large-index.md)
- [Lecture multivolume et ouverture temporaire](../engineering/archive-opening.md)
- [Encodage adaptatif des noms de fichiers](../engineering/filename-encoding.md)
- [Aperçu du contenu](../engineering/content-preview.md)
- [Vérification préalable des métadonnées à l’extraction](../engineering/extraction-preflight.md)
- [Extraction ZIP sûre](../engineering/zip-safe-extraction.md)
- [Vérification de l’intégrité des archives](../engineering/archive-checking.md)
- [Ouverture d’entrées, glisser-déposer et modification](../engineering/archive-transfers.md)
- [Mots de passe et ouverture des répertoires chiffrés](../engineering/password-extraction.md)
- [Contrat de création ZIP](../engineering/zip-creation.md)
- [Création ISO9660 et Joliet](../engineering/iso-creation.md)
- [Archives ZIP auto-extractibles intégrées pour Windows](../engineering/sfx-creation.md)

<a name="xdzip-format-support-and-implementation"></a>

## Formats pris en charge et implémentation

- [Capacités intégrées, profils de création et portée des contrôles](../engineering/format-registry.md)
- [Adaptateurs de format et conception de l’essai SDK externe](../design/codec-extension-sdk.md)
- [Essai du SDK externe en lecture seule et périmètre d’exécution](../engineering/sdk-trial.md)
- [Contrat du processus de format isolé](../engineering/native-codecs.md)
- [Formats et algorithmes Rust internes](../../engineering/independent-rust-codecs.md)
- [Décompte du travail algorithmique auxiliaire](../engineering/auxiliary-decoding-work.md)
- [Choix de la dépendance pour les métadonnées ZIP](../engineering/zip-metadata-admission.md)
- [Choix des dépendances d’extraction et d’écriture ZIP](../engineering/zip-extraction-admission.md)
- [Sources du format 7z et implémentation indépendante](../engineering/sevenzip-format-sources.md)
- [Bases de l’implémentation RAR et limites de compatibilité](../../engineering/rar-format-sources.md)
- [Sources du format CAB et bases d’implémentation](../engineering/cab-format-sources.md)
- [Contrat d’implémentation TAR/GZIP](../engineering/tar-gzip.md)
- [Flux à fichier unique et paquets dérivés de ZIP](../engineering/single-file-streams.md)
- [Contrat des flux LZ4](../engineering/lz4-streams.md)
- [Contrat des flux Zstandard](../engineering/zstandard-streams.md)
- [Contrat Unix compress Z](../engineering/compress-streams.md)
- [Contrat des flux Brotli ordinaires](../engineering/brotli-streams.md)
- [Structure et algorithmes LHA](../engineering/lha-containers.md)
- [Conteneurs PMA et méthodes PMarc](../engineering/pma-streams.md)
- [Flux compressés et conteneurs ARJ](../engineering/arj-streams.md)
- [Conteneurs ARC et profils de flux compressés](../engineering/arc-streams.md)
- [Conteneurs BH et profil BlakHole pris en charge](../engineering/bh-containers.md)
- [Sources et conditions de licence ACE, ALZ et EGG](../engineering/ace-alz-egg-format-sources.md)
- [Enveloppes AES Crypt en lecture seule](../engineering/aescrypt-envelopes.md)
- [Contrat des conteneurs DEB](../engineering/deb-containers.md)
- [Contrat des conteneurs ASAR](../engineering/asar-containers.md)
- [Enveloppes texte UU, UUE et XXE](../engineering/text-envelopes.md)
- [Sources et licences WIM, MSI et installateurs](../engineering/wim-msi-installer-format-sources.md)
- [Sources et licences des images disque et autres formats](../engineering/disk-image-archive-format-sources.md)
- [Inventaire des formats Bandizip et faisabilité xdzip](../engineering/bandizip-format-assessment.md)

<a name="xdzip-desktop-interaction-and-visual-design"></a>

## Interactions et design du bureau

- [Vues Qt Quick et responsabilités des contrôleurs](../design/qt-quick-ui.md)
- [Spécifications visuelles du bureau](../../visual/README.md)
- [Norme de conception de l’interface](../../visual/design-standard.md)
- [Conception et correction de l’interface](../visual/design-workflow.md)
- [Icônes d’association de fichiers](../visual/file-association-icons.md)
- [Vérification de la fidélité visuelle HTML](../visual/html-fidelity.md)
- [Interaction et libellés complémentaires](../ux/interaction-spec.md)
- [Fonctionnement, composants et interactions de la fenêtre principale](../ux/main-window-components.md)
- [Conception et intégration de l’arborescence](../ux/directory-tree.md)
- [Page d’accueil de la fenêtre principale](../ux/welcome.md)
- [Chargement sur la page d’accueil et fin de création](../ux/welcome-opening.md)
- [Saisie du mot de passe pour l’extraction](../ux/password-extraction.md)
- [Fenêtres de tâches et vérification d’intégrité](../ux/task-windows.md)
- [Fenêtre des paramètres et langue d’interface](../ux/settings-localization.md)
- [Terminologie de l’interface en chinois simplifié](../ux/chinese-terminology.md)
- [Menus système et zone de notification](../ux/desktop-integration.md)
- [Fenêtres et barres de titre par plateforme](../ux/platform-windows.md)
- [États détaillés et contrat d’implémentation](../ux/hifi-state-spec.md)
- [Prototype d’interaction v0.1 et correspondance des fonctions](../ux/prototype-spec.md)

<a name="xdzip-platform-integration-releases-and-trust"></a>

## Intégration système, publication et confiance

- [Installateurs et associations de fichiers](../engineering/installers-associations.md)
- [Prise en charge et maintenance des systèmes](../engineering/support-policy.md)
- [Actions du gestionnaire de fichiers et hôte de tâches autonome](../engineering/shell-actions.md)
- [Mises à jour lancées par l’utilisateur](../engineering/software-update.md)
- [Cycle de vie des données et diagnostic](../engineering/privacy.md)
- [Version du produit et identité du build](../engineering/versioning.md)
- [Protocole GitHub Release v1](../engineering/release-protocol.md)
- [DMG de développement local macOS](../engineering/macos-deployment.md)
- [Répertoire d’exécution Linux autonome et vérification](../engineering/linux-deployment.md)
- [Vérification des dépendances de déploiement et SBOM](../engineering/deployment-audit.md)
- [Attributions et licences des composants utilisés](../engineering/runtime-attribution.md)
- [Provenance et notices de l’exécution native](../engineering/native-runtime-materials.md)
- [Correspondance du SDK Qt et de ses sources](../engineering/qt-sdk-correspondence.md)
- [Licences des outils et composants Windows](../engineering/windows-licensing.md)
- [Choix de licence et ressources tierces](../engineering/licensing.md)
- [Admission des dépendances et moteurs de format](../../engineering/dependencies.md)
- [Sources correspondantes et matériaux de reconstruction](../engineering/source-delivery.md)
- [Reconstruction des données ICU 73.2 depuis CLDR](../engineering/icu-cldr-rebuild.md)

<a name="xdzip-build-and-cli-reference"></a>

## Référence de construction et de la CLI

- [Construction, déploiement et publication](../design/build-release.md)
- [Conventions de code et niveaux de vérification](../design/quality-engineering.md)
- [Préparation de l’environnement de développement](../engineering/development.md)
- [Builds de développement et vérification par plateforme](../../engineering/developer-build.md)
- [Versions des outils et plateformes de référence](../engineering/toolchain.md)
- [Commandes CLI, JSON, installation et reçus de mise à jour](../engineering/command-line.md)
