# Journal des changements

[English](CHANGELOG.md) | [简体中文](CHANGELOG.zh-CN.md) | [繁體中文](CHANGELOG.zh-TW.md) | [日本語](CHANGELOG.ja-JP.md) | [한국어](CHANGELOG.ko-KR.md) | Français | [Deutsch](CHANGELOG.de-DE.md) | [Español](CHANGELOG.es-ES.md) | [Português (Brasil)](CHANGELOG.pt-BR.md) | [Русский](CHANGELOG.ru-RU.md)

<a id="unreleased"></a>

## Non publié

- Ajouter un SDK autonome expérimental en lecture seule avec un traitement CBOR borné, des entrées-sorties contrôlées, deux implémentations indépendantes du protocole et un contrôle de bac à sable qui refuse les exécutions dangereuses. Les extensions tierces restent désactivées dans le produit.

## 0.3.3

- Afficher les commentaires des archives et modifier ceux des archives ZIP, ZIP64 et ZIPX ordinaires accessibles en écriture. Afficher davantage de commentaires et d’attributs dans l’aperçu.
- Améliorer le traitement des noms et des chemins lors de la création, de la modification et de l’extraction, notamment dans les dossiers Windows partagés.
- Ajouter les archives récentes, l’extraction rapide et l’ouverture de l’emplacement de l’archive. Actualiser les icônes de l’application et des documents.

Consultez les [notes de version](release-notes/0.3.3.md) et la [version GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.3.3) pour les plateformes, les vérifications et les limites connues.

## 0.3.2

- Publier les paquets natifs de bureau et de CLI autonome pour les Mac Intel sous macOS 14 ou ultérieur, avec des mises à jour authentifiées pour les deux architectures Mac.
- Corriger la perte de certaines demandes de réduction de fenêtre lors des transitions en plein écran sur macOS.
- Aligner les étiquettes d’extension de format sur la page d’accueil et corriger la taille de la fenêtre d’extraction Windows ainsi que le style de ses listes déroulantes.
- Publier le Snap Ubuntu amd64 dans le canal stable.

Consultez les [notes de version](release-notes/0.3.2.md) et la [version GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.3.2) pour les plateformes, les vérifications et les limites connues.

## 0.3.1

- Placer le chinois simplifié et le chinois traditionnel en tête de la sélection manuelle de langue, sous les noms « 中文 (简体) » et « 中文 (繁体) ».
- Améliorer les catégories de formats et la pagination de la page d’accueil, et corriger la disposition de la barre d’outils et des fenêtres d’opération.
- Corriger le signalement prématuré de fin des requêtes paginées et conserver les informations d’origine des fichiers extraits sous Windows.
- Mettre à jour les dépendances de l’aperçu des images.

Consultez les [notes de version](release-notes/0.3.1.md) et la [version GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.3.1) pour les plateformes, les vérifications et les limites connues.

## 0.3.0

- Prendre en charge dix langues dans l’interface de bureau, les menus système et la documentation utilisateur. La langue système peut être choisie automatiquement ou remplacée manuellement.
- Étendre la modification à d’autres formats d’archives. Les entrées éligibles peuvent être ajoutées, remplacées, déplacées, renommées ou supprimées.
- Charger automatiquement les aperçus de fichiers et permettre leur actualisation à la demande. Améliorer les actions sur les archives, les associations de fichiers dans macOS et la transmission des fenêtres et des fichiers dans Ubuntu.

Consultez les [notes de version](release-notes/0.3.0.md) et la [publication GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.3.0) pour les plateformes prises en charge, le périmètre de vérification et les limitations connues.

## 0.2.1

- Étendre la consultation et l’extraction, notamment pour LHA/LZH, ARJ, ARC, DEB, ASAR, Zstandard et LZ4. Permettre la création d’archives LZH, d’images de données ISO9660/Joliet et de fichiers ZIP auto-extractibles pour Windows x64.
- Étendre l’aperçu aux images matricielles courantes et à un profil restreint de SVG statique. Utiliser la prise en charge système de HEIC/HEIF sous macOS et générer les miniatures dans un processus privé aux ressources limitées. La spécification des aperçus décrit les limites par format.
- Indexer jusqu’à un million d’enregistrements dans un ZIP/ZIP64 ordinaire à fichier unique grâce à un index sur disque construit en flux. Charger à la demande les résultats volumineux et conserver la sélection entre les pages.
- Ajouter à la fiche de propriétés l’aperçu sur demande de texte brut et d’images PNG/JPEG, dans des limites définies. Réutiliser la gestion des mots de passe, des contrôles d’intégrité et des annulations.

Consultez les [notes de version](release-notes/0.2.1.md) et la [publication GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.2.1) pour les plateformes prises en charge, le périmètre de vérification.

## 0.2.0

- Permettre aux versions de XDZip installées sous macOS d’utiliser la mise à jour native par connexion directe ou par un proxy HTTP CONNECT ou SOCKS5 personnalisé.
- Fournir des paquets CLI autonomes pour Windows, macOS et Linux. Ils proposent des commandes d’archivage, des résultats JSON, l’enregistrement explicite des commandes et des mises à jour vérifiées par des manifestes signés.
- Appliquer les correctifs officiels de QtNetwork pour CVE-2026-116337 et CVE-2026-105946. Conserver les correctifs complets, leur provenance et les instructions de remplacement.
- Déplacer les métadonnées de la CLI intégrée à macOS vers les ressources ordinaires de l’application, couvertes par la signature d’intégrité du paquet.

Consultez les [notes de version](release-notes/0.2.0.md) et la [publication GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.2.0) pour les plateformes prises en charge, le périmètre de vérification.

## 0.1.3

- Empêcher le Finder de considérer comme une application endommagée le dossier d’extraction par défaut portant le nom d’une archive terminée par `.app`.

Consultez les [notes de version](release-notes/0.1.3.md) et la [publication GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.1.3) pour les plateformes prises en charge, le périmètre de vérification.

## 0.1.2

- Conserver la liste de fichiers pendant la navigation entre dossiers, la recherche et le tri pour réduire le scintillement.
- Améliorer les explications et les étapes d’installation manuelle lorsque l’installation automatique est indisponible sous macOS.
- Corriger l’extraction sous macOS des liens symboliques ZIP sûrs, relatifs au dossier parent, ainsi que les mentions du moteur d’exécution Qt.

Consultez les [notes de version](release-notes/0.1.2.md) et la [publication GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.1.2) pour les plateformes prises en charge, le périmètre de vérification.

## 0.1.0

- Publier la première version stable, avec des installateurs natifs ou des paquets portables pour Windows, macOS et Linux et des mises à jour à l’initiative de l’utilisateur pour les installations éligibles.
- Améliorer le défilement des paramètres, l’accessibilité de l’état des mises à jour et la prise en charge des proxys système. Corriger le survol natif.
- Corriger la collecte des mentions tierces et normaliser les expressions de licence dans la nomenclature logicielle (SBOM) des sources et le graphe des dépendances du moteur d’exécution Rust candidat.

Consultez les [notes de version](release-notes/0.1.0.md) et la [publication GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.1.0) pour les plateformes prises en charge, les limites connues et les documents de publication.

## 0.1.0-alpha.9

- Retirer trois DLL du compilateur graphique du SDK Windows de l’installateur et du paquet portable. La mise à niveau supprime les anciens exemplaires livrés avec le produit.
- Améliorer l’accès aux paramètres et aux mises à jour. Les descriptions des types de fichiers Windows suivent la langue de l’application.
- Corriger la réduction de la fenêtre pendant les transitions plein écran de macOS et plusieurs associations de fichiers. Améliorer le démarrage et le chargement des polices.
- Corriger le nettoyage après fermeture d’une archive, la vérification de la destination d’extraction et les messages de tâche.
- Alléger la CI habituelle et lancer à la demande les compilations complètes et les vérifications de publication. Enrichir les documents de licence du code source et des composants tiers.

Les [notes de version](release-notes/0.1.0-alpha.9.md) décrivent les limites de plateforme et de licence de cette préversion expérimentale.

## 0.1.0-alpha.8

- Ajouter à la fenêtre À propos la recherche de mises à jour à l’initiative de l’utilisateur. Les installations éligibles peuvent être mises à niveau après vérification de la signature, de la version et de la plateforme. La version publique précédente, alpha.6, nécessite une mise à niveau manuelle.
- Utiliser les polices du système et leurs solutions de repli pour les noms d’archives, de fichiers et les chemins. Les libellés fixes conservent la police du produit. Afficher des messages distincts pour les dossiers vides, les archives vides et les recherches sans résultat.
- Retirer la bibliothèque OpenGL logicielle des paquets de développement Windows. Conserver Direct3D 11 comme choix par défaut et WARP pour le rendu logiciel.
- Corriger le double comptage des tâches actives lorsqu’un instantané arrive avant l’acceptation de la tâche. Conserver l’acceptation, l’annulation et le nettoyage à la fermeture.
- Détecter uniformément les anciens encodages de noms de fichiers pour la consultation, la navigation, l’extraction, l’ouverture temporaire, l’exportation et la modification. Privilégier les déclarations Unicode, conserver la validation des noms bruts et écrire les noms réécrits en UTF-8. Les noms courts et les encodages mixtes peuvent rester ambigus.
- Retirer les anciens critères de publication et modèles de rapport. Harmoniser les règles de contribution, la synthèse de l’état de préparation et les générateurs de documents. Suivre la provenance et les licences dans la liste de contrôle de publication.
- Utiliser des notes relues propres à chaque version et vérifier la version et le numéro de build. Interrompre l’opération si le brouillon diffère du fichier relu afin qu’une relance ne remplace pas les limites d’acceptation consignées.
- Mettre à jour la documentation sur la publication dédiée à macOS, la lecture des volumes fractionnés et le journal des changements.

## 0.1.0-alpha.6

- Effacer l’état de survol résiduel après avoir masqué ou réduit une fenêtre, notamment le bref fond rouge du bouton de fermeture lors d’une restauration depuis la zone de notification Windows. Mutualiser ce nettoyage entre les fenêtres de consultation et d’opération.
- Consulter et extraire les volumes ZIP numérotés et PKZIP. Les lectures entre volumes, les décalages ZIP64, les mots de passe et l’intégrité passent par le même traitement d’extraction. Chaque volume est protégé contre la modification de la source et l’écrasement.
- Ouvrir une entrée ordinaire via le système à partir d’une copie temporaire privée. Masquer la progression une fois la demande acceptée par le système et conserver les échecs pour permettre une nouvelle tentative. La copie est conservée selon une politique de sept jours et ses modifications restent hors de l’archive.
- Corriger l’enregistrement du mode d’ouverture des archives. Ajouter le défilement des onglets qui débordent, une liste des onglets et le réordonnancement par glisser-déposer. Employer une graisse normale pour les onglets inactifs.
- Prendre en charge les noms longs GNU dans les TAR de sources Git. Fixer les messages d’erreur en haut de l’accueil. Fermer une fenêtre retire ses onglets et conserve les tâches d’arrière-plan. Adapter le libellé du compteur de tâches dans la zone de notification.

## 0.1.0-alpha.1

- Introduire l’epoch Debian 1 afin que les paquets alpha soient reconnus comme des mises à niveau des anciens paquets dev. Conserver l’identité des fichiers de test historiques.
- Préserver les états normal, maximisé et plein écran pendant les dialogues, la restauration depuis la zone de notification et l’ouverture de fichiers externes.
- Partager un processus et une icône de notification par session utilisateur. Transférer les demandes d’activation et d’ouverture groupée pendant le démarrage, rejeter les demandes expirées et récupérer l’état résiduel après un plantage.
- Proposer des onglets ou des fenêtres séparées, avec contrôleurs d’archives et états de consultation indépendants. Conserver les tâches et résultats après la fermeture d’une page, les acheminer par identifiant de session et de tâche et sérialiser l’admission des opérations lourdes.
- Séparer le numéro de préversion du numéro de build. Préparer six types de paquets multiplateformes, les manifestes immuables, les sommes de contrôle, les documents de sources et de licences et les procédures de publication. Les mises à jour automatiques restaient à venir et la licence de publication était suivie séparément.
- Conserver toutes les entrées ordinaires explicitement choisies, y compris une cible existante ou un ancien volume. Préparer la sortie dans un espace privé et résoudre les conflits à l’écriture finale. Préserver une cible également sélectionnée en entrée et proposer un autre nom ou l’annulation.
- Mettre à jour les entrées du menu des tâches sur place par identifiant pour éviter le scintillement lors de la progression. Préserver le survol, la mise en évidence et le focus clavier, et ne modifier que les entrées qui rejoignent ou quittent l’historique.
- Limiter les DMG de développement macOS aux binaires Qt arm64, au style Qt Quick Basic et aux dépendances partagées requises. Consigner l’architecture, le style et la taille et vérifier le paquet final. Garder l’application et le SDK d’origine et utiliser une signature d’intégrité ad hoc locale.
- Déployer sur Windows uniquement le style Qt Quick Basic épinglé. Retirer les modules de débogage QML et les métadonnées de types de développement, tout en gardant le style natif et les solutions graphiques de repli. Corriger le chargement des traductions système Qt et livrer la même version du runtime VC++ dans les paquets portables et dans les installateurs par utilisateur. Les audits refusent l’usage du runtime de l’hôte.
- Uniformiser le nom de l’exécutable principal en xdzip sur Windows, macOS et Linux. Harmoniser les lanceurs, les ressources, la CI, la reconstruction depuis les sources et la documentation. Conserver la compatibilité des tests de mise à niveau avec les anciennes versions.
- Ajouter l’extraction ZIP AES-256 à volume unique pour AE-1/AE-2 et Store/Deflate. Prévoir une demande de mot de passe dans la tâche, une nouvelle tentative et l’annulation. Écrire les fichiers après authentification. Mettre à jour la conception, les libellés, la couverture du protocole et les vérifications d’interopérabilité.
- Ajouter un nettoyage ciblé des entrées « Ouvrir avec » des copies de développement macOS. Supprimer les nouvelles inscriptions après vérification ou échec et conserver l’application principale, les fichiers et les valeurs système par défaut. Ajouter des tests de limites et de nettoyage après échec.
- Terminer les travaux de développement et d’acceptation avec l’accord du propriétaire. Consigner dix contrôles automatisés, huit contrôles manuels et la politique de prise en charge du candidat figé. Suivre la provenance et les licences dans la liste de publication et les tickets. La publication officielle restait en attente.
- Ajouter la barre de menus native macOS et les menus de la zone de notification sur les trois plateformes. Partager les commandes, le focus d’édition et l’état des tâches. L’application reste par défaut dans la zone de notification après la fermeture de la dernière fenêtre. Les paramètres permettent de l’empêcher. Quitter et redémarrer conservent la confirmation des tâches et le nettoyage. Ajouter des libellés bilingues, des petits pictogrammes modèles, des maquettes et des tests de cycle de vie.
- Unifier la gestion des versions dans VERSION.json, les métadonnées natives et la fenêtre À propos. Ajouter des libellés bilingues et la copie de l’identification du build. Continuer à recueillir les mesures de performances du processus et les preuves de licence en distinguant les contrôles de bureau, de machine vierge et de licence.
- Aligner les dimensions de la fenêtre principale Qt, les volets, les listes, les textes, les icônes, les onglets et les aperçus sur les mesures du HTML et des maquettes figés. Partager les sources de polices et ajouter des mesures CI indépendantes ainsi que des captures sur de vrais ZIP. Permettre de fermer les onglets et de tout sélectionner dans la page courante. Conserver les états désactivés et les dates inconnues.
- Terminer les menus partagés, les indications, les états des boutons et le fil d’Ariane. Le HTML reçoit des menus contextuels, déroulants et imbriqués, des onglets de démonstration, une grille, le tri, la sélection, le tactile et des tâches de test fictives. Qt reçoit les menus de sélection, copie, affichage et dossiers en lecture seule. Vérifier 22 nouvelles interactions, 20 parcours existants et cinq groupes Qt locaux. Garder les actions indisponibles désactivées et les démonstrations signalées.
- Ajouter en HTML et Qt les lignes de l’arbre des dossiers, les compteurs fixes, les indicateurs de parents repliés et les emplacements communs d’icônes. Préserver la maquette principale antérieure. Suivre séparément les indications de limites, les écrans d’attente et l’accessibilité manuelle.
- Livrer trois planches de l’arbre des dossiers, 26 découpes à l’échelle d’origine, des indicateurs clairs et sombres, les spécifications de mise en page et d’interaction, un HTML autonome et des ressources. L’intégration HTML et Qt restait à faire. Conserver les maquettes et les photos d’origine.
- Déplacer les maquettes haute fidélité dans design/hifi avec les vues des fenêtres principales et d’opération, les anciennes propositions et les empreintes. Harmoniser les sources des ressources et les liens. La maquette sombre correspondante restait à faire.
- Ajouter un arbre de dossiers borné avec dossiers implicites, tri sur tous les résultats et en-têtes de colonne utilisables au clavier. Paginer après le tri et garder les ID des doublons. Ajouter les limites de nœuds, de profondeur et d’octets, les messages de repli, les tests d’interaction et 54 contrôles visuels.
- Ajouter le socle d’exécution avec deux handles cxx, des commandes et événements bornés, des contrôles d’annulation et d’arrêt indépendants, une file Qt et des barrières d’arrêt. Vérifier la CI produit sur trois plateformes, dix cycles de vie par plateforme et le lancement et l’arrêt natifs après déploiement sur macOS et Windows. Fournir une version de développement Apple Silicon et un guide de vérification ponctuelle.
- Réessayer de manière asynchrone et bornée lors d’une brève contention du verrou d’admission. Désactiver les commandes déconnectées, signaler les arguments invalides et protéger la destruction pendant le traitement d’un événement.
- Ajouter un contrôle préalable d’extraction borné, limité aux métadonnées. Développer les dossiers sélectionnés, vérifier les chemins, conflits et tailles, et afficher le résultat dans une fenêtre native. L’autorisation d’écrire reste du ressort de l’extraction.
- Améliorer l’explorateur à trois volets avec des thèmes clairs et sombres, le détail des entrées, la portée par dossier, la recherche et la sélection multiple par ID physique. Ajouter des tests de souris et de filtrage.
- Ajouter la consultation réelle de ZIP en lecture seule à l’aide d’un adaptateur Rust indépendant. Borner l’analyse, prendre en charge les doublons et noms chinois, paginer par 128 entrées et gérer l’annulation et la fermeture asynchrones. Documenter l’adoption de rawzip et les captures Qt.
- Ajouter aux tâches Qt des cartes de chemins, des indicateurs sur trois colonnes, une progression en dégradé et un pied de fenêtre partagé, conformément aux maquettes. Conserver la réorganisation à grande taille de texte et les libellés de démonstration. Capturer les états clair, sombre et à 200 %.
- Définir les invariants d’identité et de pagination du domaine, les aperçus autonomes de fenêtres de tâches Qt, les indications partagées et les paramètres de thème générés. Ajouter les contrôles de disposition, de fermeture et de CI hors écran.
- Ajouter un outil de diagnostic autonome du pont Rust/cxx, une compilation unifiée CMake/Cargo, des dépendances épinglées et une CI native. Vérifier sous Linux la propriété des données, les erreurs, UTF-8 et l’arrêt.
- Ajouter un diagnostic autonome de la chaîne Qt Quick. Documenter la fenêtre Linux hors écran et la compilation depuis des chemins contenant des caractères chinois et des espaces. Les vérifications des trois plateformes et de cxx restaient à faire.
- Ajouter huit planches haute fidélité et les ressources pour la création, les conflits, la progression, les résultats, les paramètres et les comparaisons de plateformes. Définir les parcours d’enregistrement, d’annulation et de récupération.
- Livrer 24 découpes à l’échelle d’origine, 27 types d’icônes vectorielles claires et sombres et des PNG transparents à plusieurs densités, avec des notes de source et de précision.
- Aligner les icônes de type de fichier, la géométrie des listes et la colonne des types. Définir les fenêtres autonomes, leurs barres de titre et leur fermeture sur les trois plateformes.
- Ajouter un prototype d’interaction HTML hors ligne pour la consultation, la sélection, l’extraction, la création, les tâches et les échecs. Le relier aux exigences et aux vérifications.
- Adopter Apache-2.0, ajouter la licence complète et mettre à jour les documents de contribution et de publication.

<a id="pre-development-baseline-2026-10-11"></a>

### Référence avant le développement (2026-10-11)

- Consigner la conception logicielle, la traçabilité des exigences et les décisions d’architecture.
- Choisir Qt Quick avec une fine couche C++ et Rust.
- Définir la référence visuelle et les paramètres des thèmes pour les vues claire, sombre et d’extraction.
- Ajouter les documents de contribution, gouvernance, sécurité, ingénierie, acceptation, risques et lots de travail.
- Le code du produit et les publications binaires restaient à venir.
