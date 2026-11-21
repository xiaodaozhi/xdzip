# Compilar y contribuir a XDZip

[English](../../contributing/README.md) | [简体中文](../../zh-CN/contributing/README.md) | [繁體中文](../../zh-TW/contributing/README.md) | [日本語](../../ja-JP/contributing/README.md) | [한국어](../../ko-KR/contributing/README.md) | [Français](../../fr-FR/contributing/README.md) | [Deutsch](../../de-DE/contributing/README.md) | Español | [Português (Brasil)](../../pt-BR/contributing/README.md) | [Русский](../../ru-RU/contributing/README.md)

Lee la [política de contribución](../../../CONTRIBUTING.md) y busca o abre una incidencia para el cambio. Describe el comportamiento esperado, las plataformas afectadas y un ejemplo sencillo que se pueda reproducir. Usa archivos de prueba propios del proyecto o redistribuibles. La [estructura del repositorio](../engineering/repository-layout.md) indica dónde está cada parte de la implementación.

<a name="xdzip-build-the-part-you-need"></a>

## Compilar la parte necesaria

Ejecuta los comandos desde la raíz del repositorio. En Windows, usa `python` en un entorno de desarrollo MSVC x64. En macOS, usa un terminal Apple Silicon nativo y las herramientas de línea de comandos de Xcode. En Ubuntu, sigue la lista de dependencias de la plataforma. La [guía de compilación](../engineering/developer-build.md#environment-and-entry-points) enumera los requisitos y las versiones de Rust y Qt fijadas por el repositorio.

Para la primera compilación de escritorio:

```sh
python3 scripts/desktop.py prepare --install-rust --install-qt
python3 scripts/desktop.py doctor --for-stage build
python3 scripts/desktop.py build
```

Las opciones de preparación instalan las herramientas fijadas mediante el rustup existente y descargan el SDK de Qt fijado si hace falta. Reutiliza las dependencias compartidas de `build/toolchain/` si ya están preparadas. Si usas un SDK de Qt compatible en otra ubicación, pasa el mismo `--qt-prefix` a las fases correspondientes. Si el cambio exige las pruebas de escritorio, ejecuta `python3 scripts/desktop.py test` sobre una compilación terminada con la misma configuración. Consulta la [preparación de datos de prueba](../engineering/developer-build.md) para las suites completas del proceso auxiliar.

Para trabajar en la CLI autónoma, compila y verifica la CLI y su proceso auxiliar:

```sh
python3 scripts/cli.py build
python3 scripts/cli.py test
```

La salida predeterminada es `build/cli`. Para compilar la CLI se necesitan las herramientas fijadas de Rust, CMake y el compilador C++ de la plataforma. En Ubuntu también se requieren `libssl-dev` y `pkg-config`. Consulta la [especificación de compilación de la CLI](../engineering/command-line.md#xdzip-build-and-command-installation) para ver los detalles de cada plataforma.

<a name="xdzip-verify-the-affected-behavior"></a>

## Verificar el comportamiento afectado

Reutiliza una compilación adecuada si existe. Para un cambio de documentación, empieza por las comprobaciones de abajo y verifica la salida de los generadores afectados. Compila todo el escritorio cuando lo exijan el cambio o las comprobaciones necesarias.

```sh
python3 scripts/check-docs.py
git diff --check
```

| Cambio | Código y especificación | Qué ejecutar o comprobar |
| --- | --- | --- |
| Documentación y ejemplos | Actualiza las traducciones, enlaces y anclajes correspondientes. El README raíz y `docs/` tienen diez ediciones lingüísticas. Las guías de `developers/` usan inglés de Estados Unidos y chino simplificado. Usa la especificación correspondiente y prueba los comandos modificados con un archivo pequeño propio del proyecto. | Ejecuta la comprobación documental anterior. Abre los enlaces de sección modificados y comprueba la salida generada cuando un generador use la página. |
| CLI | Empieza en `crates/xdzip-cli` y sigue el entorno compartido de Rust. [Especificación de la CLI](../engineering/command-line.md) | Ejecuta las comprobaciones de la CLI autónoma en tu compilación. Si cambian comandos o resultados, verifica stdout, stderr, el código de salida y los archivos escritos, también en un caso de fallo aplicable. |
| Dominio Rust, formatos o tareas sobre archivos | Empieza en `xdzip-domain`, el códec correspondiente, `xdzip-runtime` o `xdzip-io`. Para un comando de escritorio, sigue `BrowserController` y `RuntimeConnection`. [Registro de formatos](../engineering/format-registry.md), [ejecución](../engineering/product-runtime.md), [seguridad de archivos](../design/file-security.md) | Ejecuta `cargo fmt --all -- --check` y las pruebas de los crates afectados. Comprueba la ruta real y los casos de fallo, cancelación o escritura definitiva pertinentes. Si usa el proceso nativo de formatos, ejecuta las [pruebas del proceso auxiliar](../../../developers/qa/runtime-worker-tests.md). Si cambia el registro, regenera con `python3 scripts/generate-format-registry.py`, comprueba con `--check` y ejecuta `python3 scripts/test-format-registry.py`. |
| Ventanas QML, controles o localización | Empieza en `apps/desktop/ui`, `resources/i18n` y `specs/visual`. Antes de cambiar una ventana, lee la [especificación visual](../visual/design-standard.md) y el [proceso de diseño](../visual/design-workflow.md) y registra su contrato en el [registro de ventanas](../../../specs/visual/window-registry.json). | Para ventanas, ejecuta `python3 scripts/check-window-contracts.py --self-test`, `python3 scripts/check-operation-components.py` y las pruebas bilingües de diseño afectadas con distintos tamaños de texto. Ejecuta `python3 scripts/check-ui-i18n.py` para localización. Cuando sea necesario, verifica la interacción o el aspecto nativos en la plataforma afectada. |
| Instalación, integración o actualizaciones | Consulta la [guía de compilación y despliegue](../engineering/developer-build.md), la [especificación de asociaciones](../engineering/installers-associations.md) y la de [actualizaciones](../engineering/software-update.md). | Comprueba la instalación o el registro afectados, la identidad real del paquete, los errores y la recuperación. Usa la [política de CI](../../../developers/engineering/ci-policy.md) para encontrar el flujo de la plataforma correspondiente. |

<a name="xdzip-describe-the-evidence"></a>

## Describir las pruebas

En la PR indica qué comportamiento ha cambiado, qué comandos y plataformas se han probado y qué comprobaciones quedan pendientes. Especifica los resultados sin pantalla, del escritorio nativo y de instalación. La [guía de verificación](../../../developers/qa/README.md) explica el alcance de cada prueba. Las especificaciones de los subsistemas están en la [referencia técnica](../reference/README.md) y los procedimientos de publicación, en el [manual de mantenimiento](../../../developers/README.md).
