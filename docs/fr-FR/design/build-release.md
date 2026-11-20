<a name="xdzip-build-deployment-and-release-architecture"></a>

# Architecture de compilation, de déploiement et de publication

[English](../../design/build-release.md) | [简体中文](../../zh-CN/design/build-release.md) | [繁體中文](../../zh-TW/design/build-release.md) | [日本語](../../ja-JP/design/build-release.md) | [한국어](../../ko-KR/design/build-release.md) | Français | [Deutsch](../../de-DE/design/build-release.md) | [Español](../../es-ES/design/build-release.md) | [Português (Brasil)](../../pt-BR/design/build-release.md) | [Русский](../../ru-RU/design/build-release.md)

<a name="xdzip-build-and-package-inputs"></a>

## Entrées de compilation et de mise en paquet

CMake compile l’application de bureau Qt, les adaptateurs C++ et les auxiliaires natifs. Cargo compile le cœur d’exécution Rust et génère la passerelle CXX liée à l’application. La CLI autonome a sa propre commande de compilation et fonctionne sans Qt. `scripts/desktop.py` et `scripts/cli.py` coordonnent les étapes de compilation et de test. Le dépôt fixe Rust à la version 1.99.0 et le SDK Qt à la version 6.8.2. Comparez les résultats en conservant ensemble l’identité de la compilation, l’architecture cible, les fichiers de verrouillage et la provenance de la chaîne d’outils. [Chaîne d’outils](../engineering/toolchain.md), [Guide de compilation](../../engineering/developer-build.md)

Le paquet de bureau déploie les bibliothèques Qt, les modules QML, les ressources propres à la plateforme et les auxiliaires dont il a besoin. Le processus d’images privé utilise Qt pour décoder les images et afficher les SVG statiques. La distribution de la CLI comprend son processus d’archives. Les scripts et les contrats de plateforme définissent les profils de compilation, les listes de fichiers admis et les vérifications de déploiement. Pour connaître les dépendances installées et la taille totale du paquet, examinez le paquet complet. [Audit du déploiement et des dépendances](../engineering/deployment-audit.md), [Distribution de la CLI](../engineering/command-line.md)

<a name="xdzip-current-distribution-channels"></a>

## Canaux de distribution actuels

| Cible | Paquet et périmètre de confiance |
| --- | --- |
| macOS Apple Silicon | GitHub fournit un DMG et un ZIP contenant le paquet d’application. La version minimale est macOS 14. Le paquet utilise une signature d’intégrité ad hoc. Consultez les instructions de la plateforme pour l’installation et la mise à jour. |
| Windows 11 x86_64 | GitHub fournit un installateur par utilisateur sans signature et un ZIP portable. Le Microsoft Store gère sa propre identité MSIX, la signature et son canal de mise à jour. |
| Ubuntu 24.04 LTS x86_64 | GitHub fournit une archive du répertoire d’exécution et un DEB. Le périmètre pris en charge comprend X11 et glibc 2.39 ou une version ultérieure. |
| CLI autonome | La CLI a ses propres identités de paquet et de mise à jour. La CLI fournie avec l’application de bureau suit les règles d’intégration et de transfert de son paquet. |

La [politique de prise en charge](../engineering/support-policy.md) définit les systèmes et architectures couverts. Pour les fichiers publiés, consultez le [README du projet](../../../README.fr-FR.md) et les notes de la version correspondante. Le [guide d’installation](../../engineering/installers-associations.md) précise les limites des associations et de l’intégration.

<a name="xdzip-routine-checks-native-builds-and-release"></a>

## Contrôles courants, compilations natives et publications

Le [workflow CI courant](https://github.com/xiaodaozhi/xdzip/blob/main/.github/workflows/docs.yml) effectue des contrôles légers. Les tâches natives Foundation et Qt s’exécutent dans des workflows manuels distincts. Sur Apple Silicon, la CI ordinaire compile, teste et effectue des vérifications de démarrage sans déployer d’application téléchargeable. Le workflow dédié à la publication suit ses propres étapes autorisées de déploiement, de signature, de mise en paquet et de publication. Le [protocole de publication](../engineering/release-protocol.md) et la [politique CI](../../../developers/engineering/ci-policy.md) définissent les conditions et les preuves requises.

Les manifestes de publication, les sommes de contrôle, les inventaires, la nomenclature des composants logiciels (SBOM) et les documents de licence décrivent les artefacts livrés. Le workflow de publication ne génère ni ne téléverse d’archives ZIP de code source du projet. GitHub fournit ses propres instantanés de tags par un mécanisme distinct. Les obligations de licence dépendent des composants effectivement livrés, du [registre des licences](../engineering/licensing.md) et de la [politique de fourniture du code source](../../engineering/source-delivery.md).

<a name="xdzip-update-and-future-distribution"></a>

## Mises à jour et distributions futures

La sélection et la vérification des mises à jour sont indépendantes des tâches d’archives. Les installations admissibles suivent le contrat de mise à jour pour les métadonnées de confiance, les fichiers vérifiés, l’identité d’installation et le traitement des tâches actives avant le transfert. Windows et Linux utilisent l’auxiliaire applicable, macOS utilise Sparkle et le Microsoft Store met à jour son édition. Les versions portables et de développement ont leurs propres limites. [Mises à jour logicielles](../../engineering/software-update.md)

Toute extension à une autre architecture, version minimale de système, forme de paquet ou boutique exige des preuves distinctes de compilation et de validation. Les [décisions d’architecture](architecture-decisions.md) conservent l’historique des choix de distribution et les conditions de leur réexamen.
