# xdzip : décisions d’architecture

[English](../../design/architecture-decisions.md) | [简体中文](../../zh-CN/design/architecture-decisions.md) | [繁體中文](../../zh-TW/design/architecture-decisions.md) | [日本語](../../ja-JP/design/architecture-decisions.md) | [한국어](../../ko-KR/design/architecture-decisions.md) | Français | [Deutsch](../../de-DE/design/architecture-decisions.md) | [Español](../../es-ES/design/architecture-decisions.md) | [Português (Brasil)](../../pt-BR/design/architecture-decisions.md) | [Русский](../../ru-RU/design/architecture-decisions.md)

Les premières décisions datent du 11 octobre 2026. Les suivantes indiquent leur propre date. Une décision acceptée est adoptée. Les notes sur l’implémentation actuelle relient chaque décision aux contrats maintenus et aux éléments attestant les capacités. Lorsqu’une décision est remplacée, conservez l’original et ajoutez une nouvelle décision.

## ADR-001 : Qt Quick comme seule interface du produit

**État : acceptée.**

**Implémentation actuelle.** Le bureau utilise Qt Quick/QML avec de minces adaptateurs C++. L’hôte SFX Windows à boîtes de dialogue système constitue l’exception limitée de l’ADR-013. [Interface de bureau](qt-quick-ui.md)

Contexte : le produit exige une interface de bureau moderne, une expérience cohérente et une faible consommation de ressources. Des prototypes Slint, Tauri et Qt Quick utilisant le même cœur ont été réalisés. En tenant compte des capacités de l’interface, de la cohérence et des mesures disponibles pour le rendu logiciel sous Linux, le projet a choisi Qt Quick pour le produit ; les autres plateformes doivent encore être vérifiées.

Décision : Qt Quick/QML et Qt Quick Controls 2 avec une mince adaptation C++. Slint et Tauri restent des essais historiques et ne sont pas inclus dans le produit.

Coûts : environnement d’exécution Qt, modules QML, construction multilangage et travail sur les licences. La saisie, l’accessibilité, le GPU et le déploiement sur Mac et Windows nécessitent d’autres tests. L’exécutable du prototype de 0,80 MiB ne constitue pas un installateur Qt complet.

Réexamen : Qt ne satisfait pas les besoins de fonctionnalité, de licence ou de maintenance de sécurité sur les plateformes cibles et ne respecte toujours pas les budgets essentiels après optimisation effective. Les variations ordinaires de performances ne déclenchent pas une nouvelle sélection de framework.

## ADR-002 : Rust détient toute la logique métier des archives

**État : acceptée.**

**Implémentation actuelle.** L’environnement Rust gère l’état des archives et les tâches. La sélection des fichiers exprime une intention du bureau ; la couche d’E/S de l’hôte autorise les accès au système de fichiers. [Architecture du système](../../design/system-architecture.md)

Décision : domaine, index, plans, tâches, quotas, règles de chemins et registre des formats résident dans Rust. Qt représente l’état, autorise la sélection de fichiers système et transmet les décisions de l’utilisateur. Les tests métier ne nécessitent pas Qt.

Autres solutions : connecter libarchive ou 7zip directement à C++/QML, ou répéter la logique de contrôle dans chaque interface. Elles couplent le cœur aux interfaces des plateformes et empêchent une vérification cohérente de la sécurité et de l’annulation. Rust ne supprime pas les risques des bibliothèques natives ; les bibliothèques concernées peuvent être isolées dans des processus de travail.

Réexamen : si une API de plateforme ne peut pas être implémentée de façon fiable en Rust, autoriser un petit portage C++ propre à cette plateforme sans transférer la responsabilité métier.

## ADR-003 : passerelle cxx générée à la construction

**État : acceptée.**

**Implémentation actuelle.** Le produit construit et utilise la passerelle typée avec des handles distincts pour les commandes et les événements. Le récepteur d’événements et le cycle de fermeture sont définis dans le [contrat de la passerelle](qt-rust-bridge.md).

Décision : utiliser dans le produit une passerelle cxx sûre au niveau des types, construite à partir du même commit. Ne pas maintenir à long terme une ABI C écrite à la main ni sérialiser du JSON sur les chemins critiques. La passerelle ne contient pas de logique métier Qt.

Autres solutions : conserver l’ABI C expérimentale, générer des QObject avec CXX-Qt ou relier l’interface et tout le cœur par IPC local. cxx préserve des limites explicites pour les modèles et les threads Qt sans ajouter, dans un premier temps, de framework de génération de QObject. Il faut en contrepartie maintenir la mince couche QObject et l’orchestration CMake/Cargo.

Limites : cxx ne garantit pas la compatibilité binaire dynamique entre des versions arbitraires de Rust et de Qt et ne génère pas de protocole de format tiers.

Réexamen : la vérification de la construction de bureau ou du cycle de vie pris en charge échoue, ou la couche d’adaptation grossit fortement. Évaluer CXX-Qt ou une ABI C explicitement versionnée dans une ADR plutôt que changer silencieusement.

## ADR-004 : séparer le SDK source intégré du protocole de processus externe

**État : limite de conception acceptée. Le protocole externe est expérimental.**

**Implémentation actuelle.** Les formats du produit sont explicitement enregistrés et intégrés. Le processus de format fourni utilise son propre protocole. L’implémentation CBOR externe est un essai indépendant en lecture seule. [Adaptateurs de format et essai](codec-extension-sdk.md)

Décision : utiliser des interfaces source Rust pour les formats intégrés et un protocole de processus distinct et borné pour les extensions externes. L’hôte conserve l’autorité sur les E/S.

Motif : l’ABI des bibliothèques dynamiques Rust est instable et les extensions natives arbitraires dans le processus compromettent l’isolation des pannes et la gestion des ressources. Wasm n’est pas prioritaire comme environnement général de formats en raison du coût de migration, des threads, des accès par position et de la vérification des performances des codecs natifs existants ; son emploi futur reste possible.

Coûts : protocole, courtier, bac à sable et surcharge IPC. Ne figer le protocole qu’après l’interopérabilité de deux implémentations indépendantes.

## ADR-005 : l’hôte contrôle centralement les écritures réelles

**État : acceptée.**

**Implémentation actuelle.** L’environnement prépare les opérations et `xdzip-io` gère les autorisations, la sortie temporaire et la validation finale. Les adaptateurs de format utilisent des lecteurs et des récepteurs. [Autorisation des fichiers](file-security.md)

Décision : les moteurs n’écrivent que dans des `OutputSink` autorisés. L’hôte contrôle les chemins réels, les conflits, la sortie temporaire et la publication. Ne pas appeler directement une extraction générale de bibliothèque vers des répertoires arbitraires, sauf si ses adaptations ont été entièrement vérifiées comme limitées par l’hôte.

Motif : réimplémenter la protection des chemins pour chaque format crée facilement des incohérences ; les contrôles de chaînes ne suffisent pas contre les liens et les courses. Il faut en contrepartie implémenter et tester les pannes des couches de fichiers sécurisées sur trois plateformes.

Limites : l’hôte ne peut ni prouver la correction sémantique de la sortie d’une extension malveillante ni arrêter un processus malveillant contrôlant déjà le même utilisateur. La confiance et les permissions des extensions doivent rester explicites.

## ADR-006 : publier l’extraction fichier par fichier et les nouvelles archives en entier

**État : acceptée.**

**Implémentation actuelle.** L’extraction publie les fichiers individuellement. La création et la modification vérifient les archives temporaires avant remplacement ou publication. La création en volumes publie un ensemble vérifié partie par partie et tente un retour arrière limité en cas d’échec partiel. [Sémantique de publication](file-security.md)

Décision : chaque fichier extrait est préparé et vérifié avant publication ; une tâche en échec peut conserver une partie des résultats. Les nouvelles archives sont entièrement écrites avant publication. L’écrasement n’est pas activé par défaut.

Motif : la fusion dans un répertoire existant ne peut pas simplement garantir une transaction atomique sur tout le répertoire et toutes les plateformes. Prétendre annuler toute la tâche pourrait supprimer des modifications ultérieures de l’utilisateur. Il faut fournir des résultats et journaux de récupération détaillés et expliquer l’annulation.

Réexamen : un nouveau mode de publication d’un répertoire entièrement nouveau peut être implémenté séparément sous des conditions limitées, sans modifier la sémantique de fusion dans un répertoire existant.

## ADR-007 : acteurs et pools de travail bornés, événements poussés

**État : acceptée.**

**Implémentation actuelle.** Chaque connexion utilise des threads distincts pour les commandes et les tâches. Le bureau admet les tâches lourdes en série ; les files Rust et la boîte aux lettres Qt sont bornées. Un pool généralisé de jetons de ressources reste une option de conception. [Ordonnancement des tâches](task-scheduling.md), [passerelle Qt/Rust](qt-rust-bridge.md)

Décision : un ordonnanceur léger décide de l’état, des pools bornés exécutent le travail et Qt possède un thread distinct de réception des événements. Regrouper les mises à jour de progression, livrer les états terminaux et réserver une admission de contrôle indépendante à l’annulation et à la fermeture.

Autres solutions : appels synchrones depuis l’interface, un thread par fichier, Tokio ou `spawn_blocking` sans limite, ou interrogation continue. Le travail sur les archives est principalement synchrone et utilise CPU et E/S ; ne pas créer de tâches sans limite au seul motif qu’une opération est dite asynchrone. De futures E/S réseau réelles peuvent utiliser localement un environnement asynchrone.

Coûts : contre-pression, fermeture et allocations de ressources explicites. Avantages : faible charge au repos et interface que les tâches ne peuvent pas submerger.

## ADR-008 : formats limités pour la première version, selon les capacités

**État : acceptée.**

**Implémentation actuelle.** Les formats initiaux constituent un contexte historique. Le registre maintenu et l’admission à l’exécution définissent les capacités actuelles du code, y compris les profils de lecture et de création ajoutés et les restrictions propres à chaque opération. [Registre des formats](../engineering/format-registry.md)

Décision : la version v0.1 utilise ZIP Store/Deflate et TAR/GZIP ; chiffrement, volumes, RAR, 7z et algorithmes supplémentaires sont vérifiés séparément. L’interface ne déduit pas une prise en charge complète d’une extension.

Motif : le nom d’un format ne décrit pas toutes ses fonctionnalités réelles. La première table des formats est plus courte, mais honnête et vérifiable pour les utilisateurs.

## ADR-009 : séparer les petites données du produit du contenu utilisateur

**État : acceptée.**

**Implémentation actuelle.** Les paramètres, index et journaux ont des durées de vie distinctes. Les index sur disque sont privés et limités à la session. La réutilisation persistante des index et la reprise de tâches inachevées restent des options de conception. [Modèle de domaine](domain-data.md), [publication des fichiers](file-security.md)

Décision : les paramètres, index temporaires, journaux de transaction et manifestes d’extension ont des cycles de vie séparés. Pas de télémétrie par défaut, de mots de passe persistants ni de cache d’index d’archives entre les lancements.

Coûts : plusieurs schémas et règles de nettoyage. Réexaminer les index entre lancements exige une conception de l’identité et de l’invalidation des sources, de la confidentialité et de la capacité.

## ADR-010 : la distribution complète et les licences sont des critères d’architecture

**État : acceptée.**

**Implémentation actuelle.** Les binaires GitHub et les MSIX Microsoft Store suivent désormais des canaux de packaging documentés et distincts. Examiner les dépendances réellement déployées, l’intégrité des artefacts et les éléments de licence pour chaque canal. [Construction et publication](build-release.md)

Décision : déployer Qt dynamiquement par défaut et examiner les licences des modules réellement utilisés. La taille de l’installation comprend les environnements d’exécution. La signature, la SBOM et les obligations de remplacement ou de fourniture des sources entrent dans les vérifications de publication.

Autres solutions : comparer seulement les exécutables ou ajouter `LICENSE` juste avant la publication. Aucune ne prouve que le produit est léger ou distribuable légalement.

Coûts : maintenance continue du déploiement et des licences. Une construction statique de Qt ou une distribution en magasin exige une ADR et un examen des licences distincts ; une PR d’optimisation de taille ne peut pas les modifier accessoirement.

<a name="xdzip-adr-011-initial-support-scope-and-personal-project-maintenance-policy"></a>

## ADR-011 : périmètre initial et politique de maintenance d’un projet personnel

**État : acceptée le 19 octobre 2026, avec l’approbation explicite du propriétaire du projet.**

**Implémentation actuelle.** La [politique de prise en charge](../engineering/support-policy.md) et le [contrat de mise à jour](../../engineering/software-update.md) définissent les règles actuelles. La clause de mise à jour manuelle décrit v0.1.

Décision : prendre initialement en charge macOS 14 et ultérieur sur Apple Silicon arm64, les versions maintenues de Windows 11 x86_64 et Ubuntu 24.04 LTS x86_64 avec X11 et glibc 2.39 ou ultérieure. Donner la priorité à la seule dernière version publique, traiter les retours selon les moyens disponibles sans délai de réponse garanti, utiliser des mises à jour manuelles pour v0.1 et conserver le caractère expérimental des aperçus de développement. Voir la [politique de prise en charge](../engineering/support-policy.md).

Motif : les constructions natives, installateurs et dépendances d’exécution disponibles délimitent le périmètre. Les promesses non vérifiées d’anciens projets concernant les systèmes et architectures sont réduites aux capacités de maintenance d’un projet libre personnel. Intel Mac, Windows 10, Windows/Linux ARM64, d’autres distributions et Wayland ne sont pas promis. Une extension exige d’abord une implémentation et une vérification réelles.

Limite : une distribution personnelle libre sur GitHub n’exige ni compte développeur payant, ni certificat de signature, ni Developer ID, ni notarisation. macOS conserve la signature ad hoc nécessaire à l’intégrité. Cette politique ne remplace pas la vérification des origines de licence et des paquets et ne suppose pas la compatibilité avec toutes les futures versions des systèmes.

## ADR-012 : migrer chaque format vers des algorithmes Rust indépendants

**État : acceptée le 25 octobre 2026, avec l’approbation explicite du propriétaire pour l’implémentation. Remplace le choix à long terme de moteurs de format tiers ; ne signifie pas que la migration est terminée.**

**Implémentation actuelle.** Le routage du produit utilise des implémentations de format Rust. Les lectures 7z, RAR, CAB et de supports optiques concernées utilisent le processus de travail fourni. La matrice actuelle comprend un moteur d’écriture ISO dans le processus. Vérifier chaque opération dans le [registre des formats](../engineering/format-registry.md) et la [conception des adaptateurs](codec-extension-sdk.md).

Décision : utiliser des implémentations Rust indépendantes pour l’analyse des formats et les algorithmes de compression et décompression ; les bibliothèques générales de cryptographie, d’encodage de caractères et de fondations système sont autorisées. Définir d’abord les comportements à partir des spécifications officielles, puis implémenter indépendamment ; ne pas traduire ligne par ligne ni copier en renommant le code de référence. Les développeurs ayant vu ce code ne revendiquent pas un développement strict en salle blanche. Les sources insuffisantes et les restrictions de licence exigent un examen séparé, notamment l’absence de développement d’un encodeur RAR.

Le plan prévoyait d’accepter puis de remplacer séparément ZIP, TAR, TAR.GZ, 7z, RAR, CAB, ISO et UDF selon les capacités d’alors. Conserver l’implémentation actuelle pour les formats non acceptés ; ne pas effectuer de repli silencieux ni étendre le périmètre en lecture seule de RAR, CAB, ISO ou UDF. Réutiliser les algorithmes de compression communs. Les noms, identités, E/S autorisées, ressources, annulation et publications restent sous le contrôle de l’hôte Rust ; l’isolation des processus demeure après la migration vers Rust.

Coûts : maintenir indépendamment la compatibilité, la sécurité, les performances, le fuzzing et les algorithmes dans la durée. Des références tierces peuvent servir aux tests, mais les formats migrés doivent retirer les moteurs correspondants des dépendances et fichiers livrés. Voir le [contrat des moteurs internes](../../engineering/independent-rust-codecs.md) pour les sources et interfaces ; la progression et les validations effectives sont suivies dans [#45](https://github.com/xiaodaozhi/xdzip/issues/45).

<a name="xdzip-adr-013-standalone-windows-sfx-host"></a>

## ADR-013 : hôte SFX Windows autonome

**État : acceptée le 16 novembre 2026 pour le premier profil SFX autorisé par le propriétaire.**

**Implémentation actuelle.** Le [contrat SFX](../engineering/sfx-creation.md) définit le profil Windows et le point d’entrée autonome Rust utilisant les boîtes de dialogue système.

Contexte : une archive autoextractible Windows x64 générée doit fonctionner sans installation de xdzip ni de Qt. Fournir une deuxième application Qt complète dans chaque archive ajouterait un environnement de déploiement sans rapport avec ce besoin.

Décision : conserver Qt Quick pour le bureau et réutiliser sa fenêtre de création. L’exécutable SFX autonome du projet utilise uniquement les boîtes de dialogue Windows de dossier et de tâche, avec polices, DPI, dimensions et accessibilité contrôlés par le système. ZIP, vérification, autorisation de sortie, conflits, progression, annulation et nettoyage réutilisent le cœur Rust existant. Il s’agit d’une exception limitée pour l’hôte de distribution à l’ADR-001, et non d’un autre framework de bureau ou d’une nouvelle bibliothèque de contrôles personnalisés.

Coûts : maintenir et tester automatiquement un exécutable Windows supplémentaire, une enveloppe versionnée, une liaison CRT statique, un audit des dépendances sans Qt et l’hôte de dialogues natifs. Celui-ci n’exécute jamais le contenu extrait. La signature de l’exécutable généré est distincte de celle d’un programme fixe.

Réexamen : toute demande d’installateur, d’exécution automatique du contenu, de plateforme exécutable supplémentaire ou d’interface SFX personnalisée exige son propre périmètre et un examen d’architecture.
