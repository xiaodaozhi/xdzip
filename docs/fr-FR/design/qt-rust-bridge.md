<a name="xdzip-qt-rust-bridge-and-application-service"></a>

# Pont Qt/Rust et service applicatif

[English](../../design/qt-rust-bridge.md) | [简体中文](../../zh-CN/design/qt-rust-bridge.md) | [繁體中文](../../zh-TW/design/qt-rust-bridge.md) | [日本語](../../ja-JP/design/qt-rust-bridge.md) | [한국어](../../ko-KR/design/qt-rust-bridge.md) | Français | [Deutsch](../../de-DE/design/qt-rust-bridge.md) | [Español](../../es-ES/design/qt-rust-bridge.md) | [Português (Brasil)](../../pt-BR/design/qt-rust-bridge.md) | [Русский](../../ru-RU/design/qt-rust-bridge.md)

Le bureau Qt appelle le runtime Rust par le pont `cxx` lié au produit. Chaque `BrowserController` possède une `RuntimeConnection` et une instance de service distincte. Le pont transporte des commandes typées et des résultats dont il transfère la propriété dans le processus de bureau. La CLI utilise directement le même service Rust. [Architecture du système](system-architecture.md), [Sessions de bureau](../../engineering/application-sessions.md)

<a name="xdzip-handles-threads-and-ownership"></a>

## Handles, threads et propriété

`new_runtime()` démarre les threads Rust de commandes et de tâches. Le `Runtime` temporaire remet un `CommandClient` et un `EventReceiver`. Le thread d’interface utilise le handle de commande. Un thread récepteur Qt dédié possède le handle d’événement et attend dans `next_event()`. Le thread de commandes Rust possède l’archive courante et son index. Le thread de tâches distinct possède le travail préparé pendant son exécution.

CXX transporte des types valeur et des conteneurs Rust propriétaires. Le code Qt récepteur conserve ou convertit ces valeurs tant qu’il en possède la propriété. Aucun handle d’archive ou de fichier natif ne passe à Qt. Pour transmettre à QML des identifiants ou des nombres exacts d’octets nécessitant les 64 bits, le contrôleur utilise des chaînes. Les en-têtes générés et la bibliothèque statique sont compilés avec l’hôte. Cette interface appartient à la compilation du produit. Le processus de travail fourni utilise un [protocole de processus](../engineering/native-codecs.md) distinct.

[Déclarations et conversions du pont](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-qt-bridge/src/lib.rs), [Connexion Qt](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/RuntimeConnection.cpp), [Service Rust](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/service.rs)

<a name="xdzip-request-query-and-task-identities"></a>

## Identités des requêtes, consultations et tâches

| Identité | Rôle |
| --- | --- |
| ID de requête | Le service Rust attribue un ID à l’admission, propre à la connexion. `Submission` renvoie `Accepted`, `Busy`, `Closed`, `Invalid` ou `Exhausted`. Les refus renvoient l’ID zéro. |
| ID de session | Identifie l’instantané de l’archive ouverte. Les commandes sur cette archive doivent lui correspondre. Chaque nouvelle ouverture reçoit un nouvel ID de session. |
| Identité et offset de consultation | Une page est liée à la session, l’ID de consultation, la génération de l’interface, la révision et l’offset. Le contrôleur les vérifie avant d’accepter les lignes. |
| ID d’entrée physique ou de navigation | Une entrée physique garde son identité lors du tri et de la pagination. Les répertoires synthétisés ont leurs propres identités. Un numéro de ligne, un nom affiché ou une plage de consultation n’autorise jamais l’accès à un chemin du système de fichiers. |
| ID de tâche et d’invite | Un ID de tâche appartient à sa connexion. Une invite identifie une réponse de conflit ou de mot de passe en attente dans cette tâche. L’entrée de tâche applicative connaît aussi son contrôleur propriétaire. |
| Séquence d’événement | Le récepteur attribue une séquence croissante par connexion lorsqu’il prend les événements. Qt écarte les événements dont la séquence n’est pas plus récente que la dernière consommée. Les mises à jour de tâche partagent cette séquence et utilisent l’ID de requête initial pour la corrélation. |

<a name="xdzip-commands-in-the-current-bridge"></a>

## Commandes du pont actuel

| Commande ou méthode | Rôle et résultat ordinaire |
| --- | --- |
| `Open`, `OpenWithPassword` | Ouvrir et analyser la source choisie. Une réponse réussie comprend la première `Page` et des données de navigation bornées. Un répertoire chiffré peut exiger un mot de passe avant l’ouverture d’une session. |
| `Browse` | Démarrer une consultation ou la poursuivre avec l’identité et l’offset correspondants. Chaque commande renvoie au plus une `Page`, limitée à 128 lignes et 64 KiB de noms. |
| `FreezeSelection`, `Preflight` | Résoudre les sélecteurs courants en ID physiques ou de navigation dont `Selected` détient la propriété, ou résumer les constatations de métadonnées dans `Preflight`. L’instantané et la portée choisie sont revérifiés pour l’opération réelle. |
| `Preview` | Résoudre une entrée physique. En cas de succès, renvoyer les octets `Preview` bornés et transférés en propriété, avec l’identité de l’entrée et de la session. |
| `CheckTarget`, `InspectInputs` | Renvoyer le nombre de restes dans `TargetChecked` après ouverture et nettoyage d’un espace de sondage par le runtime. La vérification de création utilise la session zéro. `Inputs` rapporte les métadonnées superficielles dans l’ordre d’entrée, sans parcourir récursivement les dossiers. Aucune des réponses n’autorise la validation. |
| `Extract`, `Create`, `Check`, `OpenEntry` | Valider l’intention et mettre une tâche en file. `TaskAccepted` contient l’ID attribué. L’autorisation de destination et la validation sont assurées par le runtime et la couche d’E/S. |
| `Update`, `Move`, `Export`, `PublishExport`, `ExportManifest` | Les modifications d’archive et publications de données entrent dans la file des tâches lourdes. Après admission d’`Update` ou `Move`, le service abandonne l’instantané courant pour empêcher l’utilisation ultérieure des anciens ID d’entrée. `ExportManifest` renvoie les métadonnées relatives de transfert comme réponse ordinaire. |
| `Snapshot`, `CloseSession` | Renvoyer l’ID de session de l’archive courante ou zéro. Fermer la session identifiée abandonne son instantané de navigation, mais les entrées de tâches conservées indépendamment peuvent rester actives. |
| `cancel`, `cancel_task`, `decide_task`, `respond_password`, `release_export`, `shutdown` | Méthodes de contrôle hors de la file ordinaire. L’annulation d’une requête possède son propre ID et son `Finished` ; les contrôles de tâche agissent sur la tâche existante. |

Le service Rust expose d’autres opérations propres à la CLI comme `Catalog` et `SelectMembers`. Le pont valide la taille des entrées et les valeurs d’énumération avant soumission. Par exemple, les chemins de création sont limités à 4 KiB chacun et 4 MiB au total. Le runtime valide la session et la sémantique de l’opération ; la couche d’E/S ouvre les fichiers et obtient les capacités de sortie. [Contrat du service runtime](../engineering/product-runtime.md), [Contrat CLI](../engineering/command-line.md), [Autorisation des fichiers](file-security.md)

<a name="xdzip-event-delivery-and-backpressure"></a>

## Livraison des événements et saturation

Une requête ordinaire admise produit au plus un résultat de données suivi d’un `Finished` en livraison normale. Une erreur est représentée par un `Finished` portant son code. Le contrôleur attend la réussite avant de publier la page ou l’aperçu correspondant. Un résultat remplacé est écarté tandis que sa requête se termine quand même. Le parcours d’index volumineux conserve jusqu’à 1 024 lignes dans le cache Qt et demande les pages manquantes. Il fige une sélection sur plusieurs pages avant de présenter la confirmation d’extraction. [Indexation des grandes archives](../engineering/large-index.md), [Aperçu du contenu](../engineering/content-preview.md)

`TaskAccepted` et `Finished` appartiennent à la livraison ordinaire des requêtes. `TaskUpdate` utilise des emplacements d’attente distincts par tâche et transporte l’ID de requête initial, l’ID de tâche et l’ID de session source. Une mise à jour peut arriver avant `TaskAccepted` ou après `Finished`. Le contrôleur corrèle une tâche précoce à la requête initiale et dirige les mises à jour suivantes vers cette tâche. Une mise à jour ultérieure ne termine jamais une seconde fois la requête ordinaire. Les emplacements conservent le dernier instantané de chaque tâche. La progression peut être regroupée ; l’instantané terminal persiste jusqu’à sa prise par le récepteur. Les invites figurent dans les instantanés tant que la tâche attend leur réponse. [État des tâches et annulation](task-scheduling.md)

- **Rust.** Par connexion, l’admission ordinaire permet 128 requêtes en cours et un budget de 8 MiB pour les octets des commandes en file. Le budget des données des événements ordinaires est de 32 MiB. Les requêtes acceptées réservent de la place pour leur résultat et leur fin. Les accusés d’annulation de requête ont leur propre file bornée ; l’arrêt n’a pas besoin de place dans celle-ci.
- **Qt.** La boîte contient au plus 256 événements, dont au plus 128 événements de données et 32 MiB de données comptabilisées. Si elle est pleine, le récepteur attend que l’interface la vide. Une seule chaîne de notification en file traite jusqu’à 32 événements ou 4 ms par tour. La notification atteint le thread d’interface et les événements conservent leurs données en propriété pendant l’attente.

<a name="xdzip-prompts-failures-and-shutdown"></a>

## Invites, défaillances et arrêt

- Les décisions de conflit utilisent `decide_task`. Les mots de passe de tâche utilisent `respond_password` et un ID d’invite distinct. Les deux vérifient l’invite courante sans réponse. La réponse de mot de passe entre dans un stockage Rust effacé à la libération et n’apparaît jamais dans un résultat ni un événement de progression. `OpenWithPassword`, `Preview` et la création chiffrée portent des secrets propres à l’opération que le pont transfère dans les types de mot de passe Rust correspondants. [Gestion des mots de passe](../engineering/password-extraction.md)
- Des codes d’erreur stables décrivent l’admission et les résultats des requêtes et tâches. Exemples : `stale_session`, `stale_query`, `source_changed`, `unsafe_target`, `target_changed`, `disk_full` et `password_required`. Le contrôleur traduit les codes pour l’affichage. Une modification de la source invalide l’instantané de navigation concerné. `RuntimeFault` et les déconnexions inattendues invalident le travail d’interface en attente avec `runtime_unavailable` ; une rupture peut empêcher l’arrivée des événements terminaux d’origine.
- `shutdown` ferme l’admission et annule le travail accepté, y compris les tâches attendant une réponse. Le récepteur Qt continue de vider les événements pendant la sortie des threads Rust. Le récepteur Rust attend leur fin avant de produire `ShutdownReady`, puis le pont produit `End`. Qt signale une déconnexion propre après vidage de la boîte. Si la connexion est détruite prématurément, elle ferme la boîte et réveille le récepteur sans attendre sa fin sur le thread d’interface. [Arrêt des tâches](task-scheduling.md), [Cycle de vie du runtime](../engineering/product-runtime.md)

Pour modifier cette frontière, la [sonde du pont](https://github.com/xiaodaozhi/xdzip/blob/main/tools/bridge-probe/README.md), les tests du service Rust et les [tests du contrôleur de bureau](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/tests/browser_test.cpp) couvrent des couches différentes. Le [guide de compilation](../../engineering/developer-build.md) indique les cibles concernées.
