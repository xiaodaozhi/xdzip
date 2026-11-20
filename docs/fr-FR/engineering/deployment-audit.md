<a name="xdzip-deployment-dependency-verification-and-sbom"></a>

# Vérification des dépendances déployées et SBOM

[English](../../engineering/deployment-audit.md) | [简体中文](../../zh-CN/engineering/deployment-audit.md) | [繁體中文](../../zh-TW/engineering/deployment-audit.md) | [日本語](../../ja-JP/engineering/deployment-audit.md) | [한국어](../../ko-KR/engineering/deployment-audit.md) | Français | [Deutsch](../../de-DE/engineering/deployment-audit.md) | [Español](../../es-ES/engineering/deployment-audit.md) | [Português (Brasil)](../../pt-BR/engineering/deployment-audit.md) | [Русский](../../ru-RU/engineering/deployment-audit.md)

La vérification du déploiement génère par défaut des éléments de revue de développement explicitement marqués `incomplete` ; elle ne démontre pas automatiquement qu’une publication est prête. Le Release officiel collecte séparément les éléments du même candidat et les fournit avec le paquet après la revue du processus de publication. Le format utilise le [schéma officiel CycloneDX 1.6](https://github.com/CycloneDX/specification/blob/55343ba19dee1785acf1ce9191540d5fd7b590db/schema/bom-1.6.schema.json). Le commit amont et les SHA-256 de trois fichiers de schéma sont épinglés dans la [configuration](../../../scripts/sbom-schemas.json). La validation ne télécharge et n’analyse que ces ressources, mises en cache sous build/ ; elle ne télécharge pas de schémas depuis des URL arbitraires du SBOM.

| Artefact | Source et objectif | Ce qu’il ne peut pas établir |
| --- | --- | --- |
| source-build.cdx.json | Graphe Cargo de l’espace de travail résolu pour l’hôte avec outils de développement et compilation ; modules Qt déclarés ; hachages des sources de polices et de marque | Qu’un paquet Rust est nécessairement lié statiquement au produit ou que tous les plugins Qt et tiers intégrés sont identifiés |
| deployment-audit.json | SHA-256 des fichiers sous la racine réelle indiquée, imports binaires natifs, dépendances SDK, système ou non résolues, et identité réelle du build CLI | Tous les chargements dynamiques de QML ou plugins par nom, une installation propre, l’ABI système, les signatures ou la conformité des licences |
| artifact-files.cdx.json | Artefacts réels et fichiers SDK externes observés, relations connues d’après les imports natifs, liés à l’inventaire source et à la version, au commit et à l’état dirty de l’exécutable réel | Identité complète des composants déployés, attribution des liaisons statiques, licences des bibliothèques intégrées à Qt ou SBOM produit formel achevé |
| Enregistrements de bureau facultatifs | Contrôles de bureau exécutés et limites selon la modification et l’environnement disponible | Actions non exécutées, autres plateformes ou approbation de publication |

Les enregistrements source n’incluent pas les ID opaques Cargo fondés sur des chemins et ne fabriquent pas de purl crates.io pour les paquets de l’espace de travail. Les chemins d’artefacts sont relatifs à la racine indiquée ou au SDK Qt. Les autres dépendances externes sont identifiées par hachage et nom de fichier, sans chemin absolu privé du SDK. Les polices utilisent OFL-1.1 ; les sources de marque originales conservent l’Apache-2.0 de la bibliothèque d’actifs. Déclarer les modules Qt ne permet pas de déduire les licences de tous les composants réellement déployés.

<a name="xdzip-automated-verification"></a>

## Vérification automatisée

`deployment_audit.py` ne vérifie que les builds de projet fiables explicitement indiqués, pas les archives utilisateur. macOS utilise otool pour les imports et RPATH ; Windows utilise dumpbin de MSVC pour les imports DLL ordinaires et différés ; Linux utilise ldd, à ne jamais utiliser sur des exécutables d’origine inconnue. Un lien symbolique de fichier sortant de la racine d’artefact indiquée fait échouer la vérification. Les rapports doivent être écrits hors de la racine afin de ne pas inclure leur propre sortie dans l’inventaire.

- `--mode sdk` autorise la résolution vers un SDK Qt externe et indique explicitement que le build n’est pas autonome. Les imports non résolus échouent toujours. La récursion couvre l’artefact et le SDK Qt déclaré ; les autres bibliothèques externes ne sont enregistrées que par hachage et nombre non parcouru, sans considérer leur fermeture vérifiée.
- `--mode deployed` interdit les dépendances externes non système et imports non résolus. Sous Windows, les imports VC++ comme MSVCP et VCRUNTIME doivent venir du paquet ; System32 du poste de développement ne peut masquer leur absence. UCRT et contrats API système restent fournis par le système. Poursuivez les tests de démarrage natifs avec les chemins de recherche Qt de développement supprimés.
- Les imports non résolus, divergences de version, commit ou état dirty entre source et binaire réel, erreurs de hachage ou format du schéma officiel et références de composants dupliquées ou pendantes bloquent la vérification concernée.
- La fermeture des imports natifs ne remplace pas la couverture des plugins chargés par nom ni les contrôles sur machine propre. Des imports validés dans un répertoire Windows déployé ne constituent pas l’acceptation de l’installateur.

Le workflow Qt à la demande sur trois plateformes conserve deux fichiers cdx.json à la racine et deployment-audit.json, sans générer automatiquement de formulaires manuels vides. macOS lit seulement le .app et le SDK construits localement ; les contrôles ordinaires n’exécutent pas macdeployqt et ne signent, ne compressent ni ne téléversent de paquets d’application. Linux fournit Qt/QML/plugins par un [répertoire de runtime indépendant](linux-deployment.md), avec vérification deployed et conteneur sans SDK ; le tar.gz téléchargé conserve les bits exécutables. Seule la base Ubuntu 24.04, x86_64, X11 est actuellement déclarée, pas un installateur Linux universel.

Pour les artefacts Windows fiables du projet, extrayez séparément l’installateur et comparez son répertoire d’application au ZIP portable du même candidat avec `python scripts/check-windows-package-payload.py --installer-app <extracted-app-directory> --portable <portable-package.zip> --output build/review/payload-comparison.json`. Le contrôle compare les hachages par fichier, le manifeste d’installation et les DLL graphiques SDK à exclure. Le chemin de sortie doit être nouveau. Cette comparaison statique n’exécute ni l’installateur ni une mise à jour et ne remplace pas la vérification réelle d’installation.

Exemple macOS local après reconstruction depuis des sources cohérentes :

```sh
python3 scripts/collect-notices.py build/review/wp12/notices
python3 scripts/deployment_audit.py \
  --root build/ui/apps/desktop/xdzip.app \
  --app build/ui/apps/desktop/xdzip.app/Contents/MacOS/xdzip \
  --qt-prefix build/toolchain/Qt/6.8.2/macos --mode sdk \
  --output build/review/wp12/deployment-audit.json
python3 scripts/sbom.py \
  --audit build/review/wp12/deployment-audit.json \
  --source build/review/wp12/notices/source-build.cdx.json \
  --output build/review/wp12/artifact-files.cdx.json
```

Après installation de jsonschema==4.23.0 dans un environnement de vérification isolé, utilisez le Python de cet environnement pour exécuter `scripts/validate-sbom.py` avec les deux fichiers cdx.json ci-dessus. Voir la [documentation jsonschema 4.23](https://python-jsonschema.readthedocs.io/en/v4.23.0/referencing/) pour le registre. Les outils de vérification ne sont pas déployés dans le produit.

<a name="xdzip-attribution-results-and-verification-limits"></a>

## Résultats d’attribution et limites de vérification

L’[attribution des composants réels](runtime-attribution.md) est intégrée : le SPDX du SDK associe fichiers réels, modules Qt et tiers intégrés ; les textes officiels de licence aux hachages épinglés sont livrés ; les différences de hachage SDK/amont et attributions non confirmées sont enregistrées. Les candidats Rust ordinaires d’exécution sont séparés des outils de compilation, développement et proc-macro. Les nouvelles relations de composants sont liées à l’audit réel et fusionnées au SBOM d’artefact, qui reste explicitement incomplet.

Dans l’arborescence installée, associez chaque DLL, framework et plugin livré à son composant amont, sa version, sa licence et ses sources correspondantes. Examinez séparément les composants intégrés à Qt, l’attribution des liens Rust statiques et la provenance de la chaîne d’outils. Utilisez le [guide de revue de publication](../../../developers/qa/release-review.md) pour les décisions sur les candidats.
