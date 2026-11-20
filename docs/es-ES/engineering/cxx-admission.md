<a name="xdzip-cxx-bridge-dependency-rationale"></a>

# Motivos para usar el puente cxx

[English](../../engineering/cxx-admission.md) | [简体中文](../../zh-CN/engineering/cxx-admission.md) | [繁體中文](../../zh-TW/engineering/cxx-admission.md) | [日本語](../../ja-JP/engineering/cxx-admission.md) | [한국어](../../ko-KR/engineering/cxx-admission.md) | [Français](../../fr-FR/engineering/cxx-admission.md) | [Deutsch](../../de-DE/engineering/cxx-admission.md) | Español | [Português (Brasil)](../../pt-BR/engineering/cxx-admission.md) | [Русский](../../ru-RU/engineering/cxx-admission.md)

Qt y Rust se conectan mediante un puente de propiedad tipada generado por `cxx`. C++ conserva los objetos Qt y Rust conserva la lógica del dominio y del entorno de ejecución. `cxx` y `cxx-build` deben tener exactamente la misma versión, según [Cargo.lock](../../../Cargo.lock) y la [política de herramientas](toolchain.md). cxx no proporciona una ABI estable para extensiones de formato de terceros. Una ABI C escrita a mano reduciría dependencias y aumentaría el mantenimiento de cadenas, funciones libres y reglas de propiedad.

<a name="xdzip-source-and-licensing"></a>

## Origen y licencias

El origen es crates.io y el proyecto original es [dtolnay/cxx](https://github.com/dtolnay/cxx). El archivo de bloqueo y el [inventario de dependencias del puente](../../../specs/engineering/bridge-dependencies.json) registran sumas, versiones, características, scripts de compilación y licencias. Antes de publicar, comprueba y reúne los avisos realmente aplicables. El archivo de bloqueo incluye dependencias cuyas condiciones de destino pueden no cumplirse. El inventario de binarios enlazados, compiladores, Qt y bibliotecas del sistema se comprueba por separado.

<a name="xdzip-build-and-ownership-boundaries"></a>

## Compilación y límites de propiedad

cxx-build genera fuentes y cabeceras y llama a las herramientas de compilación C++. cxx compila el puente incluido y link-cplusplus enlaza la biblioteca estándar C++. Cargo verifica las dependencias adquiridas contra el archivo de bloqueo. Los scripts nuevos o las características opcionales requieren volver a comprobar el acceso a la red y la ejecución. Las comprobaciones previas no establecen la seguridad de otra versión.

Conserva std/alloc y las demás características necesarias. Al reducirlas hay que volver a comprobar UTF-8, propagación de errores, identificadores únicos, contrapresión, cierre y vaciado. El [contrato de ejecución](product-runtime.md) y el [diseño del puente](../design/qt-rust-bridge.md) definen las interfaces efectivas. Para las licencias se sigue el [proceso de revisión de publicación](../../../developers/qa/release-review.md).

Las pruebas, las ejecuciones de CI y los resultados se registran en la [incidencia WP-01](https://github.com/xiaodaozhi/xdzip/issues/1) y en sus artefactos de Actions.
