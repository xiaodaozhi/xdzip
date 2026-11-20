# Compiler XDZip et y contribuer

[English](../../contributing/README.md) | [简体中文](../../zh-CN/contributing/README.md) | [繁體中文](../../zh-TW/contributing/README.md) | [日本語](../../ja-JP/contributing/README.md) | [한국어](../../ko-KR/contributing/README.md) | Français | [Deutsch](../../de-DE/contributing/README.md) | [Español](../../es-ES/contributing/README.md) | [Português (Brasil)](../../pt-BR/contributing/README.md) | [Русский](../../ru-RU/contributing/README.md)

Consultez les [règles de contribution](../../../CONTRIBUTING.md) et cherchez ou ouvrez un ticket pour la modification. Décrivez le comportement attendu, les plateformes concernées et un petit exemple que vous pouvez reproduire. Utilisez des fichiers de test appartenant au projet ou redistribuables. L’[organisation du dépôt](../engineering/repository-layout.md) indique où se trouve chaque partie de l’implémentation.

<a name="xdzip-build-the-part-you-need"></a>

## Compiler la partie nécessaire

Lancez les commandes depuis la racine du dépôt. Sous Windows, utilisez `python` dans un environnement de développement MSVC x64. Sous macOS, utilisez un terminal Apple Silicon natif et Xcode Command Line Tools. Sous Ubuntu, suivez la liste des dépendances de la plateforme. Le [guide de compilation](../../engineering/developer-build.md#environment-and-entry-points) indique les prérequis et les versions de Rust et de Qt fixées dans le dépôt.

Pour une première compilation de bureau :

```sh
python3 scripts/desktop.py prepare --install-rust --install-qt
python3 scripts/desktop.py doctor --for-stage build
python3 scripts/desktop.py build
```

Les options de préparation utilisent rustup existant pour installer la chaîne d’outils fixée et téléchargent le SDK Qt fixé si nécessaire. Réutilisez les dépendances partagées de `build/toolchain/` si elles sont déjà prêtes. Avec un SDK Qt compatible installé ailleurs, donnez la même valeur `--qt-prefix` aux étapes concernées. Si le changement exige les tests du bureau, exécutez `python3 scripts/desktop.py test` sur une compilation terminée de la même configuration. Pour les suites complètes du programme auxiliaire, consultez la [préparation des fichiers de test](../../engineering/developer-build.md).

Pour travailler sur la CLI autonome, compilez puis testez la CLI et son programme auxiliaire :

```sh
python3 scripts/cli.py build
python3 scripts/cli.py test
```

La sortie par défaut est `build/cli`. Il faut la chaîne Rust fixée, CMake et la chaîne C++ de la plateforme. Sous Ubuntu, il faut aussi `libssl-dev` et `pkg-config`. La [spécification de compilation de la CLI](../../engineering/command-line.md#build-and-command-installation) donne les détails par plateforme.

<a name="xdzip-verify-the-affected-behavior"></a>

## Vérifier le comportement concerné

Réutilisez une compilation correspondante si elle existe. Pour une modification de documentation, commencez par les vérifications ci-dessous et examinez la sortie des générateurs concernés. Compilez toute l’application de bureau seulement si la modification ou les vérifications nécessaires l’exigent.

```sh
python3 scripts/check-docs.py
git diff --check
```

| Modification | Code et spécification | Vérifications |
| --- | --- | --- |
| Documentation et exemples | Actualisez les traductions, les liens et les ancres. Le README à la racine et `docs/` existent en dix langues. Les guides de `developers/` utilisent l’anglais américain et le chinois simplifié. Vérifiez les commandes modifiées avec leur spécification et un petit fichier de test propre au projet. | Lancez la vérification de documentation ci-dessus. Ouvrez les liens de section modifiés et examinez la sortie générée si un générateur utilise la page. |
| CLI | Commencez par `crates/xdzip-cli` et suivez l’environnement Rust partagé. [Spécification de la CLI](../../engineering/command-line.md) | Lancez les vérifications de la CLI autonome sur votre compilation. Si des commandes ou sorties changent, vérifiez stdout, stderr, le code de sortie et les fichiers écrits, y compris dans un cas d’échec approprié. |
| Domaine Rust, formats ou tâches d’archive | Commencez par `xdzip-domain`, le codec concerné, `xdzip-runtime` ou `xdzip-io`. Pour une commande du bureau, suivez `BrowserController` et `RuntimeConnection`. [Registre des formats](../engineering/format-registry.md), [exécution](../engineering/product-runtime.md), [sécurité des fichiers](../design/file-security.md) | Lancez `cargo fmt --all -- --check` et les tests des crates concernés. Vérifiez le chemin réel, y compris l’échec, l’annulation ou l’écriture définitive selon le cas. Si le chemin utilise le programme auxiliaire natif, lancez les [tests de ce programme](../../../developers/qa/runtime-worker-tests.md). Après modification du registre, exécutez `python3 scripts/generate-format-registry.py`, puis la même commande avec `--check`, et enfin `python3 scripts/test-format-registry.py`. |
| Fenêtres QML, contrôles ou localisation | Commencez dans `apps/desktop/ui`, `resources/i18n` et `specs/visual`. Avant de modifier une fenêtre, lisez la [spécification visuelle](../../visual/design-standard.md) et le [processus de conception](../visual/design-workflow.md), puis consignez son contrat dans le [registre des fenêtres](../../../specs/visual/window-registry.json). | Pour les fenêtres, lancez `python3 scripts/check-window-contracts.py --self-test`, `python3 scripts/check-operation-components.py` et les tests bilingues concernés à plusieurs tailles de texte. Pour la localisation, lancez `python3 scripts/check-ui-i18n.py`. Vérifiez l’interaction native ou le rendu sur la plateforme concernée lorsque nécessaire. |
| Installation, intégration au système ou mises à jour | Consultez le [guide de compilation et de déploiement](../../engineering/developer-build.md), la [spécification des associations](../../engineering/installers-associations.md) et celle des [mises à jour](../../engineering/software-update.md). | Vérifiez le chemin d’installation ou d’enregistrement concerné, l’identité du fichier réel, les erreurs et la récupération. La [politique de CI](../../../developers/engineering/ci-policy.md) indique le workflow approprié. |

<a name="xdzip-describe-the-evidence"></a>

## Décrire les vérifications

Dans la PR, indiquez le changement de comportement, les commandes et plateformes testées et les vérifications restantes. Précisez les résultats hors écran, sur le bureau natif et à l’installation. Le [guide de vérification](../../../developers/qa/README.md) précise la portée de chacun. Les spécifications sont dans la [référence technique](../reference/README.md), les procédures de publication dans le [manuel de maintenance](../../../developers/README.md).
