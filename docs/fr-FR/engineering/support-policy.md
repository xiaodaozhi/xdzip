<a name="xdzip-system-support-and-maintenance-policy"></a>
# Politique de support et de maintenance du système

[English](../../engineering/support-policy.md) | [简体中文](../../zh-CN/engineering/support-policy.md) | [繁體中文（台灣）](../../zh-TW/engineering/support-policy.md) | [日本語](../../ja-JP/engineering/support-policy.md) | [한국어](../../ko-KR/engineering/support-policy.md) | Français | [Deutsch](../../de-DE/engineering/support-policy.md) | [Español](../../es-ES/engineering/support-policy.md) | [Português (Brasil)](../../pt-BR/engineering/support-policy.md) | [Русский](../../ru-RU/engineering/support-policy.md)

Le responsable du projet a approuvé le périmètre initial le 2026-10-19. La version `v0.1.0` est sortie publiquement le 2026-11-06. Consultez la [page des versions](https://github.com/xiaodaozhi/xdzip/releases) pour trouver les versions actuelles et leurs fichiers. Cette politique définit le périmètre du support. Les contrôles de version minimale restent en vigueur. Les résultats de CI et les validations utilisateur doivent être lus dans le contexte de l’environnement testé. La vérification manuelle a été validée par l’utilisateur.

<a name="xdzip-initial-support-scope"></a>
## Périmètre initial

| Plateforme | Périmètre initial | Vérifications et limites |
| --- | --- | --- |
| macOS | macOS 14 ou ultérieur, Apple Silicon arm64 | Cible de déploiement 14.0, CI native et DMG de développement local. Aucun paquet Intel ou universel. Les versions futures ne sont pas garanties. |
| Windows | Versions x86_64 de Windows 11 encore maintenues par Microsoft | Compilation MSVC, installation par utilisateur et scénarios réels d’installation, de mise à niveau, de retour arrière et de désinstallation en CI. Les tests sur Windows Server ne couvrent pas toutes les versions de Windows 11. Aucun paquet ARM64. |
| Linux | Ubuntu 24.04 LTS x86_64, X11, glibc 2.39 ou ultérieure | Vérification du paquet .deb actuel et de l’environnement d’exécution isolé sans SDK Qt. Les autres distributions, les anciennes versions de glibc, Wayland et ARM64 ne font pas partie du périmètre. |

La version minimale de macOS suit la cible de déploiement actuelle. La version 13 évoquée dans d’anciens documents n’a pas été vérifiée. Le support standard de Windows 10 a pris fin le 2025-10-14. Windows 10 est hors du périmètre initial, et LTSC et ESU n’y entrent pas automatiquement. Consultez l’[annonce de fin de support de Microsoft](https://learn.microsoft.com/en-us/lifecycle/announcements/windows-10-end-of-support). La [liste des plateformes prises en charge par Qt 6.8](https://doc.qt.io/qt-6.8/supported-platforms.html) décrit le périmètre de cette dépendance. xdzip effectue ses propres vérifications.

<a name="xdzip-maintenance"></a>
## Maintenance

Ce projet personnel est maintenu dans la mesure des moyens disponibles. Il ne propose ni service payant, ni délai de réponse garanti, ni engagement de sécurité au-delà du travail effectué. La dernière version stable publique est prioritaire. Les versions de développement peuvent introduire des changements de compatibilité. Les notes de version indiquent les problèmes connus et les possibilités de retour arrière. En cas de problème de sécurité, les téléchargements concernés peuvent être retirés. Depuis la page À propos de xdzip, l’utilisateur peut rechercher des mises à jour et lancer leur installation. L’application ne recherche pas de mises à jour en arrière-plan et ne les installe pas silencieusement. Le code actuel fournit aussi une commande CLI de mise à jour. Le [contrat de la ligne de commande](command-line.md) indique quelles installations sont admissibles et précise les différences avec les fichiers déjà publiés.

Les versions candidates utilisent Qt 6.8.2 et les versions de Rust et de Cargo déjà verrouillées afin de rendre les vérifications reproductibles. Cette politique ne garantit ni l’absence de vulnérabilités dans les anciens correctifs, ni la gratuité permanente des mises à jour LTS. Avant chaque publication, la personne chargée de la maintenance consulte les avis de sécurité officiels de Qt et les mises à jour libres disponibles. L’évaluation porte en priorité sur les versions publiques maintenues. Une faille non corrigée d’une dépendance, si elle affecte xdzip, suspend la publication concernée. Une mise à niveau de Qt exige de revérifier le code source, les licences des modules, les interfaces privées Windows, le remplacement dynamique, la compilation sur les trois plateformes ainsi que le fonctionnement réel des menus et fenêtres. Un contrat de maintenance commerciale reste facultatif.

<a name="xdzip-approved-scope-and-changes"></a>
## Périmètre approuvé et évolutions

Le périmètre approuvé comprend les plateformes initiales, la maintenance de la seule dernière version publique selon les moyens disponibles, l’absence de délai de réponse fixe et l’identification des versions de développement comme expérimentales. Les résultats de `v0.1.0` figurent dans la publication et les tickets de validation correspondants. Les performances de toutes les versions des systèmes ne sont pas garanties par ces résultats. Toute nouvelle architecture, plateforme ou voie de distribution doit être mise en œuvre et vérifiée avant d’actualiser cette politique. Voir [ADR-011](../design/architecture-decisions.md#xdzip-adr-011-initial-support-scope-and-personal-project-maintenance-policy).

La signature officielle, la notarisation et les comptes payants ne sont plus exigés. Pour signaler une vulnérabilité, consultez la [politique de sécurité](../../../SECURITY.md).
