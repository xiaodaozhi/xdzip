<a name="xdzip-coding-conventions-and-quality-engineering"></a>

# Conventions de code et ingénierie qualité

[English](../../design/quality-engineering.md) | [简体中文](../../zh-CN/design/quality-engineering.md) | [繁體中文](../../zh-TW/design/quality-engineering.md) | [日本語](../../ja-JP/design/quality-engineering.md) | [한국어](../../ko-KR/design/quality-engineering.md) | Français | [Deutsch](../../de-DE/design/quality-engineering.md) | [Español](../../es-ES/design/quality-engineering.md) | [Português (Brasil)](../../pt-BR/design/quality-engineering.md) | [Русский](../../ru-RU/design/quality-engineering.md)

<a name="xdzip-code-and-ownership"></a>

## Code et propriété

- **Rust.** Suivez la chaîne d’outils, les lints et le fichier de verrouillage de l’espace de travail. Vérifiez les longueurs, décalages et conversions d’entiers externes. Préservez les tailles inconnues et les erreurs typées. Documentez les préconditions d’unsafe et de FFI, la propriété, la durée de vie et le comportement en cas d’échec. Gardez les sections critiques courtes et effectuez les E/S d’archive en dehors de celles-ci.
- **C++ et Qt.** Utilisez C++17, RAII et des règles claires de propriété des QObject. Modifiez les modèles sur leur fil graphique. Passez entre fils des valeurs dont la propriété est transférée et associez les notifications en attente à un destinataire vivant. Gardez l’arrêt asynchrone si le destinataire a encore besoin de l’interface pour vider ses événements. Rust reste responsable de la politique sur les archives et les fichiers. [Passerelle Qt/Rust](qt-rust-bridge.md)
- **QML.** Exprimez l’état et l’intention utilisateur via les contrôleurs. Réutilisez les contrôles partagés et les ressources visuelles et textuelles générées. Laissez les E/S et l’interprétation des archives hors des délégués. Les changements de fenêtres suivent le registre et les règles natives et visuelles. [Interface Qt Quick](qt-quick-ui.md), [Démarche visuelle](../visual/design-workflow.md)

Documentez les invariants et l’état laissé après un échec. Pour les fichiers, indiquez par exemple ce que protège une autorisation de validation, quelles identités elle vérifie et ce qui peut être visible après une erreur. [Publication des fichiers](file-security.md)

<a name="xdzip-verification-layers"></a>

## Niveaux de vérification

| Changement | Vérifications pertinentes |
| --- | --- |
| Domaine, requêtes et services | Tests Rust sur identité, limites, doublons, ordre, réponses obsolètes, livraison bornée et annulation. |
| Opération de format | Jeu de test du projet, allers-retours indépendants quand ils s’appliquent, cas de rejet, corruption, profils hors périmètre et quotas. |
| Fichiers natifs et tâches | Identités et octets réellement écrits, courses entre annulation et validation, protection de la sortie existante et nettoyage sur la plateforme concernée. |
| Bureau, Qt et accessibilité | Tests des modèles et contrôleurs, contrôles QML et des composants partagés, interactions natives si pertinentes et preuves visuelles ou d’assistance réelles pour le comportement visé. |
| Performance et livraison | Charges liées au commit, mesures du rendu et des ressources, ou identité du build par canal, résultats d’installation et de mise à jour et justificatifs de dépendances. |

Le [guide de vérification](../../../developers/qa/README.md) fixe les contrôles nécessaires au changement. La conformité au protocole, les tests de fichiers ou hors écran, l’automatisation native et l’observation du bureau établissent des faits différents. Les [builds de développement](../engineering/developer-build.md) et la [matrice de traçabilité](requirements-traceability.md) renvoient aux tests existants et aux limites des livrables.

<a name="xdzip-existing-ci-and-additional-techniques"></a>

## CI et techniques complémentaires

La [CI habituelle](https://github.com/xiaodaozhi/xdzip/blob/main/.github/workflows/docs.yml) vérifie la documentation, les ressources générées, la disposition, le formatage et la structure des workflows. Les contrôles natifs plus lourds pour Rust, les formats et Qt utilisent des workflows Foundation et Qt distincts, déclenchés séparément. Les tâches de publication dédiées traitent les artefacts. La [politique CI](../../../developers/engineering/ci-policy.md) décrit les déclencheurs et les conditions. Les suites locales ou déclenchées dépendent du code et des plateformes touchés.

Utilisez fuzzing, analyseurs d’exécution, vérification de modèles et mesures de performance lorsqu’ils couvrent le changement. Choisissez des jeux de test bornés et redistribuables et conservez les résultats séparément de ce guide. Suivez la [politique de sécurité](../../../SECURITY.md) pour les rapports sensibles. Les vérifications envisagées et les exemples ne constituent ni une exécution planifiée ni un résultat d’acceptation.

<a name="xdzip-maintaining-the-design"></a>

## Maintenir le design

Si une implémentation modifie une identité, une commande, une capacité, une tâche ou une limite d’écriture, mettez à jour les documents de conception, les références techniques et leurs liens de validation. Gardez les versions linguistiques alignées. Les [décisions d’architecture](architecture-decisions.md) expliquent les choix et la [conception des performances](performance.md) établit les règles de mesure. Consignez les résultats et limites réels avec le changement examiné.
