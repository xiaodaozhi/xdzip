<a name="xdzip-domain-model-and-archive-data"></a>

# Modèle de domaine et données d’archive

[English](../../design/domain-data.md) | [简体中文](../../zh-CN/design/domain-data.md) | [繁體中文](../../zh-TW/design/domain-data.md) | [日本語](../../ja-JP/design/domain-data.md) | [한국어](../../ko-KR/design/domain-data.md) | Français | [Deutsch](../../de-DE/design/domain-data.md) | [Español](../../es-ES/design/domain-data.md) | [Português (Brasil)](../../pt-BR/design/domain-data.md) | [Русский](../../ru-RU/design/domain-data.md)

Le runtime possède la session d’archive, l’index achevé et la dernière requête de cette session. Qt conserve une vue du résultat et la CLI peut lire les enregistrements physiques par sa commande de catalogue distincte. Les deux soumettent leurs opérations au même service Rust. [Architecture système](system-architecture.md), [Pont Qt/Rust](qt-rust-bridge.md)

<a name="xdzip-identity-and-source-version"></a>

## Identité et version de la source

| Identité ou donnée | Représentation et portée actuelles |
| --- | --- |
| `SessionId` | Identité runtime non nulle d’une archive ouverte. Chaque nouvelle ouverture réussie crée une session. Les chemins servent à l’entrée et au routage de l’interface. |
| `SourceVersion` | Le runtime enregistre identité, longueur et horodatages disponibles du fichier régulier de chaque partie admise. Il revérifie l’objet ouvert et le chemin correspondant aux frontières pertinentes de lecture et d’écriture. La détection des changements est faite au mieux. |
| `EntryId` | Identifie un enregistrement physique dans une session. Les enregistrements homonymes gardent des identifiants distincts. |
| Identité de navigation | Représente un répertoire indexé. Un répertoire virtuel synthétisé à partir des descendants peut apparaître dans l’arbre ou la liste sans identifiant d’entrée physique. |
| `QueryKey` | Combine session, identifiant de requête, génération et révision d’instantané. Le décalage désigne une position dans ce résultat. |
| Identité de demande, tâche et invite | Le service attribue les numéros de demande et de tâche dans une connexion. Une invite appartient à une tâche donnée. L’entrée de tâche desktop partagée utilise l’identité de session du contrôleur et le numéro de tâche. |

Les types du domaine imposent des identités d’archive et de requête non nulles. Dans le pont natif, les valeurs Rust typées sont représentées par les données CXX générées. QML reçoit des chaînes opaques pour les identifiants nécessitant les 64 bits. Un numéro de ligne n’est qu’une position dans une requête donnée. [Identités du domaine](../../../crates/xdzip-domain/src/identity.rs), [Implémentation de la source](../../../crates/xdzip-runtime/src/source.rs), [Sessions d’application](../engineering/application-sessions.md)

<a name="xdzip-physical-entries-and-output-names"></a>

## Entrées physiques et noms de sortie

`xdzip-codec-api::Entry` contient format, identité physique, représentations du nom, tailles, indicateurs de type de fichier, informations de modification et champs de méthode et de chiffrement propres au format. Les octets bruts du nom, son écriture UTF-8 décodée facultative, la décision d’encodage et le texte affiché échappé sont stockés séparément. Pour la sortie, le runtime valide le nom effectif selon la politique de chemins portables et la destination autorisée. Un nom non résolu peut rester visible tandis que l’extraction de cet enregistrement ou répertoire est refusée.

La taille d’origine a un indicateur distinct `size_known`, distinguant un fichier vide connu d’une taille inconnue. Certains conteneurs comprimés n’ont pas de taille comprimée par entrée. Les valeurs de méthode sont interprétées selon le format. Les dates conservent aussi les conventions de chaque format, par exemple l’heure DOS locale du ZIP ; une date absente reste inconnue. `payload_supported()` indique si l’enregistrement courant peut être décodé. La sortie dépend aussi de l’opération choisie, de l’état de la source, du nom et des règles du système de fichiers. [Champs d’entrée](../../../crates/xdzip-codec-api/src/lib.rs), [Noms de fichiers](../engineering/filename-encoding.md), [Autorisation des fichiers](file-security.md)

<a name="xdzip-index-query-and-page-lifecycle"></a>

## Cycle de vie des index, requêtes et pages

Le runtime ouvre la source et termine son analyse bornée avant de publier l’archive et la première page. L’analyse peut être annulée. Une fin endommagée, un mot de passe manquant ou une source modifiée ne publie aucun index partiel. La navigation dispose d’un budget distinct. Si l’arbre de répertoires le dépasse, la navigation à la racine et la recherche physique peuvent rester disponibles. Un ZIP/ZIP64 ordinaire en un seul fichier peut contenir jusqu’à un million d’enregistrements par le chemin disque privé. Les autres formats et chemins fractionnés gardent leurs propres limites.

`Archive::query_page` reçoit répertoire, texte, mode de tri, décalage et identité de requête attendue. Une nouvelle requête commence à zéro avec une nouvelle génération. Le runtime construit un ordre complet de résultats et conserve une requête courante immuable par archive. Les pages résolvent les identités de lignes selon cet ordre. Recherches et tris préservent les identités initiales des enregistrements. Chaque tri place les répertoires avant les fichiers, départage les égalités de façon déterministe par identité et place les tailles inconnues après les connues dans chaque groupe.

Le type de domaine générique `PageRequest` valide jusqu’à 256 lignes. Le transport du runtime desktop demande jusqu’à 128 lignes avec un budget de noms de 64 KiB. Qt valide intégralement les petits résultats et garde au plus 1 024 lignes résidentes pour les grands. Une réponse dont session, requête, génération, révision ou décalage ne concorde pas ne peut remplacer les données courantes. [Requêtes](../../../crates/xdzip-runtime/src/query.rs), [Listes et tri](../../../crates/xdzip-runtime/src/listing.rs), [Index des grandes archives](../engineering/large-index.md)

<a name="xdzip-selections-and-operation-input"></a>

## Sélections et entrées d’opération

Le modèle Qt stocke par identifiant les sélections de résultats chargés et, pour les grands résultats, des plages bornées dans le résultat courant. Tout sélectionner couvre ce résultat. Une tâche peut distinguer l’archive entière d’un ensemble explicitement choisi de résultats de recherche. Avant la confirmation d’extraction, Rust résout les sélecteurs entre pages selon la clé complète de requête avec `FreezeSelection`. Le développement des répertoires résout les descendants réels et déduplique les identifiants physiques. Dépasser la limite de l’opération renvoie une erreur. La CLI résout les chemins de membres et identifiants physiques avec `SelectMembers`.

Extraction, vérification et modification préparent des travaux propres à chaque opération avec la source et les métadonnées conservées nécessaires. La création reçoit des options puis analyse et valide un `InputTree` distinct. Le précontrôle des métadonnées peut décrire des problèmes potentiels sans donner d’autorité de sortie à une tâche. Le runtime revérifie les entrées et obtient l’autorisation pertinente avant d’écrire. Une fois la sélection détenue par le travail, les changements de l’interface ne modifient plus son périmètre. [Modèle desktop](../../../apps/desktop/BrowserController.cpp), [Commandes runtime](../../../crates/xdzip-runtime/src/service.rs), [Ordonnancement](task-scheduling.md)

<a name="xdzip-results-and-data-lifetime"></a>

## Résultats et durée de vie des données

| Donnée | Durée de vie |
| --- | --- |
| Archive courante, requête et index disque | Appartiennent à la session du service. Remplacer la requête ou fermer la session libère les données privées associées, sous réserve des ressources conservées indépendamment par les travaux. L’index de métadonnées disque est reconstruit à la réouverture de l’archive. |
| Instantanés de tâches | Le runtime publie des instantanés bornés indiquant état réel, nombres d’octets et d’éléments, état de nettoyage et liste bornée de détails. Le desktop conserve les résultats via le contrôleur propriétaire pendant la session de l’application. |
| Préférences utilisateur | Gérées séparément des index et résultats de tâches. Rétention et confidentialité sont documentées dans les contrats de réglages et de confidentialité. |
| Espaces de travail et journaux | La préparation privée des sorties et les enregistrements relatifs de publication facilitent nettoyage et inspection. Un redémarrage ne rejoue pas une écriture d’archive inachevée. |

[Champs d’instantané](../../../crates/xdzip-runtime/src/extraction/mod.rs), [Publication de fichiers](file-security.md), [Réglages et localisation](../ux/settings-localization.md), [Cycle des données et confidentialité](../engineering/privacy.md)

<a name="xdzip-design-options-outside-the-current-implementation"></a>

## Options de conception non implémentées

La publication incrémentale d’index consultables, un plan réutilisable avec sa propre révision et son jeton de démarrage, et un historique persistant de tâches paginé sont des options de conception. Les commandes actuelles préparent et mettent en file leurs propres travaux bornés, conservent une seule requête courante et renvoient des instantanés de tâches bornés. Une récupération persistante exigerait aussi une conception distincte de validation de la source et de la sortie.
