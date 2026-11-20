<a name="xdzip-format-adapters-and-the-extension-trial"></a>

# Adaptateurs de format et essai d’extension

[English](../../design/codec-extension-sdk.md) | [简体中文](../../zh-CN/design/codec-extension-sdk.md) | [繁體中文](../../zh-TW/design/codec-extension-sdk.md) | [日本語](../../ja-JP/design/codec-extension-sdk.md) | [한국어](../../ko-KR/design/codec-extension-sdk.md) | Français | [Deutsch](../../de-DE/design/codec-extension-sdk.md) | [Español](../../es-ES/design/codec-extension-sdk.md) | [Português (Brasil)](../../pt-BR/design/codec-extension-sdk.md) | [Русский](../../ru-RU/design/codec-extension-sdk.md)

<a name="xdzip-three-execution-paths"></a>

## Trois chemins d’exécution

| Chemin | Implémentation et disponibilité |
| --- | --- |
| Adaptateurs intégrés dans l’hôte | Le bureau et la CLI utilisent le même registre de formats fixe et le même routage d’exécution. Les implémentations Rust de ZIP, TAR, filtres de flux et conteneurs pris en charge comme DEB et ASAR sont appelées dans le processus hôte. |
| Processus d’archive fourni | `xdzip-codec-native` dans l’hôte lance `xdzip-native-codec`. Sa couche protocolaire C++ appelle les backends Rust 7z, RAR, CAB et optiques via `xdzip-codec-bridge`. La création et la réécriture 7z admissibles empruntent le même chemin. La création ISO utilise son propre module d’écriture optique dans le processus. |
| Essai SDK externe | `xdzip-plugin-host` et `xdzip-plugin-protocol` constituent une expérience XDP1 distincte et explicite. Les versions produit du bureau et de la CLI ne découvrent, n’installent ni n’exécutent d’extensions tierces. |

Les deux premiers chemins sont des composants intégrés du produit. Le processus fourni est séparé et possède un protocole privé et une surveillance des ressources. L’essai externe dispose de sa propre validation de paquet, négociation de version et conditions de permission par plateforme. [Architecture système](../../design/system-architecture.md), [Contrat du processus d’archive](../engineering/native-codecs.md), [Contrat de l’essai SDK](../engineering/sdk-trial.md)

<a name="xdzip-product-routing-and-operation-capabilities"></a>

## Routage du produit et capacités des opérations

La [source du registre de formats](https://github.com/xiaodaozhi/xdzip/blob/main/specs/engineering/archive-formats.json) déclare extensions, types de conteneurs et filtres, opérations prises en charge, méthodes et options de création. Le générateur synchronise le registre Rust, les listes du bureau et les artefacts de conception. À la lecture, le runtime examine les octets de la source et valide le format avant de choisir un backend. À la création, le nom de sortie demandé sélectionne un profil que le runtime vérifie avec les options. La liste d’extensions sert à la découverte et aux indications d’interface. La détection et l’analyse réussies établissent le format ouvert.

Lecture, aperçu, extraction, vérification de l’archive entière, création et réécriture ont des contraintes distinctes. Une entrée listée peut être visible sans méthode de contenu prise en charge. Mots de passe, volumes et enregistrements spéciaux influent sur l’admission. Le runtime traite aussi les distinctions propres aux formats, par exemple les contrôles structurels ISO/UDF et les profils de réécriture non pris en charge. [Routage du runtime](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/formats.rs), [Contrat du registre](../engineering/format-registry.md), [Vérification d’archives](../engineering/archive-checking.md)

<a name="xdzip-internal-rust-adapter-interfaces"></a>

## Interfaces internes des adaptateurs Rust

| Interface | Responsabilité |
| --- | --- |
| `ReadSource` et `MetadataCodec` | Lire ou positionner une source autorisée par l’hôte, y compris les métadonnées de volumes applicables, et produire des enregistrements de format bornés. |
| `PayloadCodec`, `PayloadReader` et `DecodedSink` | Résoudre le contenu et décoder un enregistrement physique admissible vers un récepteur de l’hôte. Distinguer octets ordinaires, travail de décodage ignoré et travail auxiliaire lorsque le backend les signale. |
| `CreationCodec`, `EncodeInput` et `EncodeOutput` | Encoder à partir de métadonnées d’entrée figées et de flux indexés. Écrire vers un récepteur de l’hôte, avec écritures aléatoires bornées si le module d’écriture l’exige. |
| `RewriteCodec` | Réécrire une archive prise en charge à partir des entrées d’origine conservées, modifications et ajouts autorisés vers une nouvelle archive temporaire. |

L’hôte maîtrise les chemins, la revalidation de la source, l’annulation, les mots de passe, les quotas et toute publication finale. Un codec reçoit les capacités nécessaires via les interfaces de l’opération. Le processus d’archive demande des plages de source autorisées et fournit des enregistrements ou blocs de sortie bornés. L’hôte valide l’état du protocole et les quotas d’octets avant d’agir. Une panne du processus ou une réponse invalide ne peut pas autoriser la publication. Cette frontière isole pannes et ressources. Les restrictions de permissions noyau pour du code externe arbitraire sont une exigence distincte de l’essai. [Contrats de métadonnées](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/lib.rs), [Contrats de contenu](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/payload.rs), [Contrats d’encodage et réécriture](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/encode.rs), [Autorisation des fichiers](file-security.md)

Ces traits Rust sont des interfaces source internes. Un adaptateur produit est intégré à l’espace de travail et au routage explicite du runtime. Les opérations autorisées nécessitent des tests distincts de fichiers valides, fonctions non prises en charge, limites de ressources, corruption et comportement de sortie. Utilisez les [contrats des codecs Rust indépendants](../../engineering/independent-rust-codecs.md) et le [guide de développement](../contributing/README.md).

<a name="xdzip-implemented-external-sdk-trial"></a>

## Essai SDK externe implémenté

L’essai accepte un manifeste de paquet natif explicitement sélectionné et une source choisie par l’hôte. Il permet d’analyser un échantillon XDP1 propre au projet et de lire une entrée ordinaire. L’admission du paquet valide le manifeste version 1, la cible exécutable et son hachage, les chemins internes au paquet, le profil fixe de permissions et les limites. L’hôte exécute une copie privée vérifiée. Le hachage déclaré établit la cohérence du paquet ; l’authentification de l’éditeur nécessiterait une politique de confiance distincte.

Le protocole d’essai version 1 utilise du CBOR borné à préfixe de longueur et vérifie identité et ordre des opérations. L’hôte autorise les lectures de source et un récepteur de sortie au moyen de jetons opaques. Le pair reçoit des plages d’octets et un crédit de sortie d’un bloc. L’hôte mesure et vérifie le contenu, contrôle la fin du protocole et l’état de la source, puis ne publie sans écrasement qu’après la sortie normale du pair. L’exemple n’expose pas au pair les mots de passe d’archive, volumes fractionnés, création, mises à jour, liens ou chemins de l’hôte.

L’essai possède des conditions natives explicites de permissions sur macOS et Linux, avec exigences par plateforme et refus en cas d’échec définis dans son contrat. Windows refuse l’exécution. Les tests natifs d’interopérabilité et de refus exercent ces limites indépendamment du bureau. Consultez le [contrat d’exécution et de protocole de l’essai](../engineering/sdk-trial.md) pour le schéma, les budgets et les entrées de test réels. [Hôte de l’essai](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-plugin-host/src/lib.rs), [Implémentation du cadrage](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-plugin-protocol/src/lib.rs)

<a name="xdzip-design-options-outside-the-current-implementation"></a>

## Options de conception hors implémentation actuelle

Un gestionnaire d’extensions destiné au produit, la confiance de l’éditeur et la politique de mise à niveau, la compatibilité au-delà du protocole d’essai et les capacités externes d’écriture ou de mot de passe restent à concevoir. L’admission au produit exigerait la vérification des restrictions de permissions, de l’interopérabilité et du comportement en cas de panne pour chaque plateforme et capacité prises en charge. Le processus fourni et l’expérience XDP1 n’activent pas cette fonction. [Décisions d’architecture](architecture-decisions.md)
