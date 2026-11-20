<a name="xdzip-architecture-evolution-and-current-boundaries"></a>

# Évolution de l’architecture et périmètre actuel

[English](../../design/architecture-evolution.md) | [简体中文](../../zh-CN/design/architecture-evolution.md) | [繁體中文](../../zh-TW/design/architecture-evolution.md) | [日本語](../../ja-JP/design/architecture-evolution.md) | [한국어](../../ko-KR/design/architecture-evolution.md) | Français | [Deutsch](../../de-DE/design/architecture-evolution.md) | [Español](../../es-ES/design/architecture-evolution.md) | [Português (Brasil)](../../pt-BR/design/architecture-evolution.md) | [Русский](../../ru-RU/design/architecture-evolution.md)

<a name="xdzip-from-framework-experiments-to-the-product"></a>

## Des essais de frameworks au produit

Les premiers essais de frameworks utilisaient une liste d’archives fictive et des tâches simulées. Le produit a adopté Qt Quick et une passerelle CXX typée, puis a mis en œuvre les opérations réelles sur les archives, les tâches et les fichiers dans Rust. Le JSON, l’ABI C et les mesures de durée du prototype se rapportent à ces essais. Les [décisions d’architecture](architecture-decisions.md) expliquent les choix et les conditions de leur réexamen.

Le produit utilise des modèles de lignes bornés et vérifie l’identité des requêtes pour empêcher qu’une ancienne réponse remplace la vue courante. Il ajoute des identifiants d’entrées physiques, des capacités de format explicites, des autorisations de fichiers, une sortie temporaire et une publication vérifiée. La CLI indépendante emploie le même cœur sans Qt. Voir l’[architecture du système](system-architecture.md) et le [modèle de domaine](domain-data.md).

<a name="xdzip-current-experimental-and-proposed-work"></a>

## Fonctions actuelles, expérimentations et propositions

| Domaine | État actuel |
| --- | --- |
| Exécution et indexation | L’hôte les met en œuvre. Un ZIP/ZIP64 ordinaire en un seul fichier peut passer à un index privé sur disque. Les limites des tâches et des métadonnées restent distinctes. [Conception des performances](performance.md) |
| Isolement des formats | Le routage intégré choisit l’implémentation Rust dans le processus ou le processus de format fourni avec le produit. Chaque opération dépend du registre, du profil et de l’admission à l’exécution. [Adaptateurs de format](codec-extension-sdk.md) |
| Extensions externes | L’essai indépendant de XDP1 en lecture seule utilise un protocole borné et des contrôles propres aux plateformes. Il s’exécute dans un hôte séparé. La découverte, l’installation et l’activation d’extensions dans le produit restent des propositions. [Essai du SDK](../engineering/sdk-trial.md) |
| État persistant et concurrence | L’ordonnanceur admet une opération de bureau lourde à la fois. Les index persistants entre lancements, la reprise des tâches et un ordonnancement partagé généralisé restent des options de conception. [Ordonnancement](task-scheduling.md), [Publication des fichiers](file-security.md) |

Le [tableau des formats](../../../README.fr-FR.md#xdzip-format-support) indique les capacités du code actuel. Pour une version publiée, consultez ses notes et ses artefacts. L’implémentation native, les outils expérimentaux et les mesures ont chacun leur propre périmètre.
