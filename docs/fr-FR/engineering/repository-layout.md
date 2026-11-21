<a name="xdzip-repository-layout-and-dependencies"></a>

# Organisation du dépôt et dépendances

[English](../../engineering/repository-layout.md) | [简体中文](../../zh-CN/engineering/repository-layout.md) | [繁體中文](../../zh-TW/engineering/repository-layout.md) | [日本語](../../ja-JP/engineering/repository-layout.md) | [한국어](../../ko-KR/engineering/repository-layout.md) | Français | [Deutsch](../../de-DE/engineering/repository-layout.md) | [Español](../../es-ES/engineering/repository-layout.md) | [Português (Brasil)](../../pt-BR/engineering/repository-layout.md) | [Русский](../../ru-RU/engineering/repository-layout.md)

Les fichiers Cargo.toml et CMakeLists.txt à la racine définissent les modules actuels du produit. Le tableau décrit les répertoires existants. Les divisions prévues dans les plans d’architecture restent à réaliser.

| Répertoire | Responsabilités et dépendances |
| --- | --- |
| apps/desktop | Interface Qt/QML, contrôleurs C++ minces, modèles, paramètres et interactions avec le système. Accès au cœur par la passerelle. |
| crates/xdzip-cli | Arguments, terminal, JSON, installation gérée et hôte de mise à jour sans Qt. Dépend directement de l’environnement d’exécution, des E/S sécurisées et du cœur de mise à jour. Le traitement des formats est délégué. |
| crates/xdzip-domain | Types purs et invariants. Aucune dépendance à Qt ou à des codecs concrets. |
| crates/xdzip-codec-api | Contrats de capacités, d’entrée empruntée, de sortie encodée et d’annulation. Dépend du domaine. |
| crates/xdzip-compression | Algorithmes et enveloppes communs Deflate/Deflate64/GZIP, LZMA/LZMA2, BZip2, PPMd7, MSZIP, LZX et Quantum. Aucune dépendance de production tierce ni autorité sur les chemins ou la publication. Voir le [contrat](../../engineering/independent-rust-codecs.md). |
| crates/xdzip-crypto | Hachages SHA/HMAC et BLAKE indépendants du format, à mémoire fixe et avec effacement de l’état sensible. Réutilise les primitives admises. L’interprétation des mots de passe et les décisions de publication relèvent d’autres modules. |
| crates/xdzip-codec-zip, xdzip-codec-tar | Implémentations ZIP et TAR/GZIP. Aucun chemin de sortie arbitraire. |
| crates/xdzip-codec-lha, xdzip-codec-arj, xdzip-codec-arc, xdzip-codec-bh | Profils indépendants de structure et de membres pour [LHA](lha-containers.md)/[PMA](pma-streams.md), [ARJ](arj-streams.md), [ARC](arc-streams.md) et [BH](bh-containers.md), sans autorité sur le système de fichiers. Le registre des formats détermine l’accès dans le produit. |
| crates/xdzip-codec-7z | Conteneur 7z interne, graphes de méthodes, filtres, composition AES, création en flux et réécriture bornée, sans maîtrise des chemins. |
| crates/xdzip-codec-cab | Conteneur CAB interne en lecture seule. Utilise les volumes autorisés par l’hôte et les algorithmes communs. Conserve les noms bruts, les déclarations UTF-8 et les enregistrements ordinaires de noms en double. La découverte des volumes et la publication restent du ressort de l’hôte. Voir le [contrat](../../engineering/independent-rust-codecs.md#cab只读规格) et les [sources](cab-format-sources.md). |
| crates/xdzip-codec-optical | Lecteurs ISO9660/Joliet/Rock Ridge/UDF et écriture ISO9660/Joliet pour de nouvelles images de données. Utilise des flux sans maîtrise des chemins. |
| crates/xdzip-codec-bridge | Passerelle CXX mince entre le processus isolé et les codecs Rust 7z, RAR, CAB et optiques. Flux autorisés, limites de ressources et annulation. |
| crates/xdzip-codec-native, tools/native-codec | Processus isolé pour les modules Rust 7z/RAR/optiques/CAB. La couche C++ mince fournit uniquement le protocole et les services système de calendrier et de codage. Le récepteur natif vérifie séparément les sorties déjà comptabilisées et le travail auxiliaire. Voir le [contrat d’E/S](native-codecs.md). |
| crates/xdzip-extract-plan | Vérification préalable des métadonnées et rapports bornés, sans autorisation de destination ni droits d’écriture. |
| crates/xdzip-io | Arbres d’entrée, autorisation et identité natives, préparation sécurisée et validation des écritures. |
| crates/xdzip-runtime | Sessions, index bornés en mémoire ou sur disque, requêtes, sélection, aperçus bornés, tâches de création et d’extraction et ordonnancement. |
| crates/xdzip-qt-bridge | Passerelle de types cxx, deux handles et gestion du cycle de vie. Dépend de l’environnement d’exécution. |
| crates/xdzip-plugin-protocol | CBOR et tramage bornés, avec la seule bibliothèque standard et sans autorité sur les chemins, les processus ou Qt. |
| crates/xdzip-plugin-host, examples/sdk | Essai de SDK indépendant en lecture seule, E/S contrôlées, conditions de confinement natif et pair Python indépendant. Extérieur à la découverte des formats et aux dépendances d’exécution du produit. |
| tools | Sondes indépendantes pour la passerelle, les formats, les E/S et le nombre de processus. L’interface graphique a ses propres points d’entrée. |
| scripts, .github | Génération, contrôles, outils de déploiement, CI et modèles de collaboration. |
| release-notes | Notes relisibles propres à chaque version et métadonnées de version et de compilation. Les rapports d’exécution et les paquets sont conservés ailleurs. |
| design/hifi, design/html, design/assets | Maquettes haute fidélité, conceptions interactives et ressources d’entrée pour l’exécution. |
| docs | Conception durable en Markdown, contrats techniques, méthodes de vérification réutilisables et images nécessaires. Les rapports d’exécution sont conservés ailleurs. |
| specs | Inventaires lisibles par machine pour les aspects visuels, la terminologie et les dépendances. Ils alimentent les ressources du produit, les générateurs et les contrôles. |
| third_party | Sources tierces et documents de licence conservés avec le code. |

Les futurs modules indépendants xdzip-index, xdzip-platform et les outils complets de conformité des formats externes restent à concevoir ou à réaliser. Actuellement, l’indexation est dans l’environnement d’exécution, et les opérations de fichiers propres aux plateformes sont dans io et les adaptateurs du bureau. Le module plugin-host actuel sert à un [essai de SDK en lecture seule](sdk-trial.md). Les tests sont principalement maintenus avec les modules Rust, apps/desktop/tests et les contrôles de scripts. Les modules vides sans implémentation ne sont pas créés.

Le produit achemine toujours CAB vers Rust. Les modules Cab officiels et leurs variantes LZX/Quantum restent hors de la compilation. Les appels utilisent le même processus isolé, sans exécutable supplémentaire, option de test ou repli sur échec. codec-api définit la comptabilisation anticipée d’une trame complète. Les conteneurs conservent un crédit par trame, tandis que les récepteurs natifs et l’environnement d’exécution vérifient indépendamment leur crédit et leurs limites de sortie. Voir le [contrat interne](../../engineering/independent-rust-codecs.md#cab只读规格).

Les règles de dépendance figurent dans l’[architecture système](../design/system-architecture.md). Évitez les dépendances inverses du domaine vers l’environnement d’exécution, les notifications directes des codecs à QML et le choix des textes d’interface par les couches de plateforme. Placez les liaisons générées, échantillons, rapports et résultats de compilation dans build/, target/ ou dist/. Les [instructions d’installation et d’association](../../engineering/installers-associations.md#兼容基线的长期输入) décrivent les entrées à conserver pour la compatibilité historique.

`tools/image-preview/` est un processus interne QtGui/QtSvg pour les images statiques. Il reçoit du contenu borné et vérifié et renvoie des données RGBA bornées. Il ne détient aucun chemin d’archive ni de destination. Voir le [contrat des aperçus](content-preview.md) pour la prise en charge des plateformes et le comportement en cas d’échec.
