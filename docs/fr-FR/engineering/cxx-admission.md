<a name="xdzip-cxx-bridge-dependency-rationale"></a>

# Choix de la passerelle cxx

[English](../../engineering/cxx-admission.md) | [简体中文](../../zh-CN/engineering/cxx-admission.md) | [繁體中文（台灣）](../../zh-TW/engineering/cxx-admission.md) | [日本語](../../ja-JP/engineering/cxx-admission.md) | [한국어](../../ko-KR/engineering/cxx-admission.md) | Français | [Deutsch](../../de-DE/engineering/cxx-admission.md) | [Español](../../es-ES/engineering/cxx-admission.md) | [Português (Brasil)](../../pt-BR/engineering/cxx-admission.md) | [Русский](../../ru-RU/engineering/cxx-admission.md)

Qt et Rust communiquent par une passerelle à propriété typée générée par `cxx`. C++ conserve les objets Qt et Rust conserve la logique de domaine et d’exécution. Les versions de `cxx` et `cxx-build` doivent être strictement identiques, comme l’indiquent [Cargo.lock](../../../Cargo.lock) et la [politique des outils](toolchain.md). cxx ne fournit pas d’ABI stable aux extensions de format externes. Une ABI C écrite à la main réduirait les dépendances et alourdirait la maintenance des chaînes, des fonctions libres et des règles de propriété.

<a name="xdzip-source-and-licensing"></a>

## Source et licences

La source est crates.io, avec [dtolnay/cxx](https://github.com/dtolnay/cxx) en amont. Les empreintes, versions, options, scripts de construction et déclarations de licence proviennent du fichier de verrouillage et de l’[inventaire de la passerelle](../../../specs/engineering/bridge-dependencies.json). Avant une publication, vérifiez et regroupez les notices réellement applicables. Le fichier de verrouillage inclut des dépendances dont les conditions peuvent ne pas correspondre à la cible. L’inventaire des binaires liés, les compilateurs, Qt et les bibliothèques du système doivent être examinés séparément.

<a name="xdzip-build-and-ownership-boundaries"></a>

## Construction et propriété

cxx-build génère les sources et les en-têtes et lance les outils C++. cxx compile la passerelle fournie et link-cplusplus lie la bibliothèque standard C++. Cargo vérifie les dépendances à partir du fichier de verrouillage. Un nouveau script de construction ou une nouvelle option nécessite de réexaminer les accès réseau et l’exécution. Une vérification antérieure ne couvre pas une nouvelle version.

Conservez std/alloc et les autres options requises. Toute réduction demande de revérifier UTF-8, la propagation des erreurs, l’unicité des références, la contre-pression, l’arrêt et l’épuisement des files. Le [contrat d’exécution](product-runtime.md) et la [conception de la passerelle](../design/qt-rust-bridge.md) définissent les interfaces effectives. La vérification des licences suit le [processus de revue de publication](../../../developers/qa/release-review.md).

Les essais, exécutions CI et résultats sont consignés dans le [ticket WP-01](https://github.com/xiaodaozhi/xdzip/issues/1) et ses artefacts Actions.
