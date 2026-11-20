<a name="xdzip-xdzip-design-guide"></a>

# Guide de conception de XDZip

[English](../../design/index.md) | [简体中文](../../zh-CN/design/index.md) | [繁體中文](../../zh-TW/design/index.md) | [日本語](../../ja-JP/design/index.md) | [한국어](../../ko-KR/design/index.md) | Français | [Deutsch](../../de-DE/design/index.md) | [Español](../../es-ES/design/index.md) | [Português (Brasil)](../../pt-BR/design/index.md) | [Русский](../../ru-RU/design/index.md)

Commencez par l’[architecture du système](system-architecture.md) pour voir comment le bureau Qt Quick et la CLI Rust indépendante accèdent au moteur d’archives. Les pages de conception décrivent les contrats actuels, les décisions et le périmètre des propositions. L’installation et les opérations courantes sont décrites dans le [manuel](../user-guide/README.md). Pour compiler ou modifier le projet, consultez [Compiler et contribuer](../contributing/README.md).

<a name="xdzip-find-a-design-topic"></a>

## Trouver un sujet de conception

| Sujet | À lire |
| --- | --- |
| Comportement du produit | Les [exigences du produit](product-requirements.md) définissent les opérations attendues et leur périmètre. La [traçabilité des exigences](requirements-traceability.md) recense les vérifications correspondantes. |
| Données des archives | Le [modèle de domaine](domain-data.md) décrit les sessions, versions des sources, entrées physiques, requêtes et paramètres d’opération. |
| Bureau | L’[interface Qt Quick](qt-quick-ui.md) couvre les vues, la propriété des contrôleurs et les interactions. La [passerelle Qt/Rust](qt-rust-bridge.md) décrit les commandes, identifiants, événements et cycles de vie réels. |
| Formats et extensions | Les [adaptateurs et l’essai d’extension](codec-extension-sdk.md) décrivent le registre intégré, le processus fourni et l’essai indépendant XDP1. |
| Sortie et tâches de fond | L’[autorisation et la publication des fichiers](file-security.md) couvre la préparation, la vérification et la validation finale. L’[ordonnancement](task-scheduling.md) explique l’admission, les invites, la progression et l’annulation. |
| Performance et qualité | La [conception des performances](performance.md) décrit les budgets, mesures et objectifs. L’[ingénierie qualité](quality-engineering.md) associe les modifications aux vérifications. |
| Livraison et décisions | La [construction et la publication](build-release.md) couvre plateformes, paquets et mises à jour. Les [décisions](architecture-decisions.md) exposent les motifs. L’[évolution de l’architecture](architecture-evolution.md) explique le passage des prototypes au produit. |
| Terminologie | Le [glossaire et les limites](glossary.md) définit le vocabulaire et renvoie aux limites actuelles de chaque sous-système. |

<a name="xdzip-interpreting-the-references"></a>

## Comprendre les références

Le comportement actuel est documenté avec des liens vers l’implémentation. Les options de conception et les outils expérimentaux précisent leur périmètre. La [référence technique](../reference/README.md) mène aux contrats. Le [README du projet](../../../README.fr-FR.md) et les notes de la version correspondante décrivent les distributions prises en charge et les fonctions livrées. Les mesures de temps et de ressources et les résultats natifs s’appliquent au build, à la plateforme et à la charge indiqués.
