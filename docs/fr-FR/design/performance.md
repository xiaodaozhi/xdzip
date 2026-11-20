<a name="xdzip-performance-design-and-measurement"></a>

# Conception et mesure des performances

[English](../../design/performance.md) | [简体中文](../../zh-CN/design/performance.md) | [繁體中文](../../zh-TW/design/performance.md) | [日本語](../../ja-JP/design/performance.md) | [한국어](../../ko-KR/design/performance.md) | Français | [Deutsch](../../de-DE/design/performance.md) | [Español](../../es-ES/design/performance.md) | [Português (Brasil)](../../pt-BR/design/performance.md) | [Русский](../../ru-RU/design/performance.md)

XDZip limite séparément mémoire et travail pour l’indexation, le transport, les tâches et l’aperçu d’images. Les limites de code ci-dessous sont des règles d’implémentation. Durées, mémoire et débit dépendent de la compilation et de la charge. Les objectifs de référence plus bas sont des objectifs d’ingénierie.

<a name="xdzip-limits-and-mechanisms-in-the-current-code"></a>

## Limites et mécanismes du code actuel

| Domaine | Comportement et coût actuels |
| --- | --- |
| ZIP/ZIP64 ordinaire en un fichier | L’analyseur en flux admet jusqu’à un million d’enregistrements physiques, sous réserve des budgets de noms, répertoires et lecture. Au-delà de 64 MiB de métadonnées actives ou si une nouvelle entrée dépasse 100 000 entrées résidentes, les enregistrements passent dans un stockage disque privé. Le fichier de métadonnées est limité à 512 MiB, avec une table distincte de positions. La navigation analyse les métadonnées sans décoder chaque contenu. |
| Requêtes en mémoire et sur disque | Le runtime construit une fois l’ordre immuable du résultat courant. Sur disque, il écrit des séquences de tri jusqu’à 8 MiB puis les fusionne en ordre de jetons. Le stockage des noms et du tri est borné. Le cache contient la requête courante, pas une vue entièrement rendue ni un historique indéfini. |
| Navigation et transfert Qt | L’arbre de répertoires a un budget de 32 768 nœuds, 64 niveaux et 4 MiB de noms. En cas de dépassement, le runtime peut garder navigation à la racine et recherche. Le pont envoie jusqu’à 128 lignes et 64 KiB de noms par page. Les grandes vues gardent jusqu’à 1 024 lignes résidentes et un nombre logique couvrant tout le résultat. |
| Commandes, tâches et écritures | Les threads de métadonnées et de tâches ont des files bornées distinctes. Une seule tâche desktop lourde est admise à la fois. Travail de décodage, octets de sortie, validation des fichiers et ressources des processus sont comptabilisés séparément. Les formats solides ou en flux peuvent lire ou décoder des données précédentes avant l’élément choisi. |
| Aperçu et processus natifs | Le texte et les images encodées ont des limites d’octets distinctes. Dimensions, nombre de pixels, exécution du processus et taille des miniatures sont bornés indépendamment. Les processus de format ont leurs propres limites de protocole et de processus. |

[Implémentation de l’index disque](../../../crates/xdzip-runtime/src/disk_index.rs), [Index des grandes archives](../engineering/large-index.md), [Limites du pont Qt/Rust](qt-rust-bridge.md), [Admission des tâches](task-scheduling.md), [Budgets de sortie](file-security.md), [Budgets d’aperçu](../engineering/content-preview.md)

Le coût mémoire comprend Qt, polices, rendu et modèles, métadonnées d’archive, table de positions et tampons de requêtes, état des décodeurs, processus enfants et tampons d’E/S. Le seuil de migration de 64 MiB concerne les métadonnées. La mémoire de l’arbre de processus comprend aussi tout ce qui est hors de cet index. Une admission de métadonnées plus élevée n’augmente pas les quotas d’extraction et n’active pas la vérification ou modification complète des archives sur disque. Écritures d’index privé et préparation de l’extraction ont des coûts disque distincts.

<a name="xdzip-measuring-the-current-implementation"></a>

## Mesurer l’implémentation actuelle

| Entrée | Mesure ou vérification |
| --- | --- |
| `scripts/probe-large-index.py` et `xdzip-m2-probe` | Génèrent un ZIP64 du projet, exécutent l’analyseur réel, le tri et les requêtes paginées, et rapportent séparément phases et stockage. Inclure doublons, noms longs et structure de répertoires et indiquer si le chemin disque est atteint. |
| `scripts/measure-desktop.py` et `xdzip-process-metrics` | Mesurent l’arbre de processus desktop, le démarrage et la navigation. Avec l’option d’extraction, exécutent une extraction réelle et vérifient indépendamment les octets de sortie. Consigner moteur de rendu, identité de compilation et charge exacte. |
| `scripts/run-performance-suite.py` | Recueille des observations Release sur démarrage, repos stable et navigation et compare l’extraction à une sonde cœur fixée sur des jeux du projet. Le rapport conserve les identités de compilation et de jeu. La comparaison est observationnelle ; l’approbation sur machine de référence reste distincte. |
| `scripts/check-runtime-resources.py` | Vérifie le reçu des ressources et polices du runtime issu de la sonde. Il établit leur identité et couverture pour cette sonde sans mesurer la latence interactive. |

[Sonde d’index](../../../scripts/probe-large-index.py), [Mesure desktop](../../../scripts/measure-desktop.py), [Suite de performances](../../../scripts/run-performance-suite.py), [Sonde de ressources](../../../scripts/check-runtime-resources.py), [Collecte des preuves](../../../developers/engineering/candidate-verification.md#performance-and-resource-evidence)

Pour comparer, consigner commit et configuration, système et architecture, moteur de rendu, matériel, nombre d’exécutions, jeu d’essai, conditions du périphérique et du cache. Séparer démarrages à froid et répétés. Quand la mesure le permet, rapporter séparément première interaction, analyse d’archive, ordre de requête, transfert de page et rendu visible. Distinguer retour d’annulation et arrêt réel avec nettoyage. Séparer octets d’entrée, décodés et validés dans les débits.

L’outil d’arbre de processus distingue PSS Linux, empreinte physique macOS et engagement privé Windows, et indique aussi RSS ou ensemble de travail si disponibles. Comparer la même métrique dans des conditions comparables. Rapporter séparément tailles de téléchargement et d’installation des paquets réels. Les mesures hors écran et du prototype ne décrivent que leurs propres conditions.

<a name="xdzip-workloads-for-comparative-tests"></a>

## Charges pour les comparaisons

| Charge | Mesures |
| --- | --- |
| B01/B08 Fenêtre vide, repos et changement de thème | Temps de démarrage et de trame, CPU au repos, mémoire conservée et moteur de rendu. |
| B02 100 000 et un million d’enregistrements ZIP | Analyse des métadonnées, migration, tri, récupération des pages et coûts mémoire et disque séparés. |
| B03/B04 Un grand fichier ordinaire ou beaucoup de petits | Débit, annulation, travail sur destination, synchronisation et nettoyage. Les opérations de fichiers peuvent dominer pour les petits fichiers. |
| B05/B06 Entrées très comprimées ou profondes dans un flux | Gestion des quotas et décodage supplémentaire avant l’entrée choisie. |
| B07 Navigation pendant des travaux lourds en attente | Latence de recherche et des pages pendant qu’une opération lourde s’exécute et qu’une autre attend. |
| B09 Processus inclus et paquet complet | Utilisation de l’arbre de processus, coût du protocole, tailles téléchargée et installée. |

Ce sont des scénarios comparatifs. Choisir la charge correspondant au changement et utiliser des jeux du projet dont tailles et empreintes sont consignées.

<a name="xdzip-reference-targets-and-planned-research"></a>

## Objectifs de référence et recherches prévues

La conception existante utilise comme référence un Apple Silicon avec 16 GiB de mémoire, SSD local, écran 60 Hz, police système, GPU réel et compilation Release. Les objectifs sont des candidats pour des mesures comparables. Ils n’affirment rien sur la dernière livraison ou une autre plateforme.

| Scénario | Objectif de conception |
| --- | --- |
| Démarrage répété d’une fenêtre vide | P95 au plus 500 ms jusqu’à une trame interactive, sur au moins 30 exécutions |
| Ouvrir et interroger 100 000 enregistrements ZIP en cache | P95 au plus 1 s pour préparer les métadonnées et 80 ms entre requête indexée et affichage |
| Saut de page à chaud et ordonnancement GUI | P95 au plus 50 ms pour un saut visible et 8 ms pour un blocage continu du thread GUI |
| Mémoire | Objectif pour l’arbre de processus : au plus 150 MiB à vide et 256 MiB en naviguant dans 100 000 entrées sans aperçu ni dictionnaire de décodage |
| CPU au repos | Moyenne au plus 0,5% d’un cœur logique après stabilisation |
| Annulation | P95 au plus 50 ms pour le retour et 250 ms pour l’arrêt coopératif local en flux, mesurés séparément |
| Extraction et distribution | Comparer GUI et cœur à charge et E/S identiques, avec un objectif de perte de débit au plus 10%. Les objectifs Mac monoarchitecture sont au plus 100 MiB téléchargés et 250 MiB installés. |

Extraction parallèle par bloc, réutilisation persistante d’index et ordonnanceur partagé de jetons CPU restent des options de conception. Profiler le chemin actuel de tâches sérielles et les chemins mémoire et disque avant de changer concurrence ou stockage. Garder une vérification équivalente des octets, l’annulation et la publication sûre dans toute comparaison. [Évolution de l’architecture](architecture-evolution.md), [Publication des fichiers](file-security.md)
