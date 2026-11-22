<a name="xdzip-product-version-and-build-identity"></a>

# Versión del producto e identidad de la compilación

[English](../../engineering/versioning.md) | [简体中文](../../zh-CN/engineering/versioning.md) | [繁體中文](../../zh-TW/engineering/versioning.md) | [日本語](../../ja-JP/engineering/versioning.md) | [한국어](../../ko-KR/engineering/versioning.md) | [Français](../../fr-FR/engineering/versioning.md) | [Deutsch](../../de-DE/engineering/versioning.md) | Español | [Português (Brasil)](../../pt-BR/engineering/versioning.md) | [Русский](../../ru-RU/engineering/versioning.md)

El campo opcional `prerelease_number` separa la secuencia de preversiones del número de compilación. Por ejemplo, alpha.1 puede usar la compilación 14 y el número de compilación seguir aumentando en alpha.2. Si el campo falta, se conserva la interpretación `channel.build`. El [protocolo de publicación](release-protocol.md) especifica manifiestos y nombres de archivos.

[VERSION.json](../../../VERSION.json) es la única fuente mantenida de versión, canal y número de compilación. La compilación lee los valores actuales. Consulta el [registro de cambios](../../../CHANGELOG.es-ES.md) para las versiones y las [incidencias](https://github.com/xiaodaozhi/xdzip/issues) para las entregas y comprobaciones. Incrementar la versión no aprueba una publicación.

Las comprobaciones de compatibilidad usan bases históricas fijas y la compilación actual para verificar actualizaciones, vuelta a una versión anterior y ajustes. El [script del producto anterior](../../../scripts/prepare-previous-product.py) mantiene la base del producto completo. El [script de compatibilidad de ajustes](../../../scripts/check-settings-compat.py) mantiene la base de ajustes. Los commits y versiones cambiantes no se duplican aquí. El esquema de configuración y la versión del producto se gestionan por separado.

- `version` sigue `major.minor.patch` sin ceros iniciales. Los cambios incompatibles, las funciones compatibles y las correcciones usan sus componentes respectivos. También hay que explicar los límites de compatibilidad en 0.x.
- `channel` admite development, alpha, beta, rc y stable. Las preversiones se muestran como `version-channel-abbreviation.build`, con development abreviado a dev. La versión estable muestra la versión base.
- `build` es un entero de 1 a 65535. Increméntalo para una compilación de entrega externa. La compilación local no lo incrementa automáticamente y el incremento no crea automáticamente una etiqueta ni una publicación. Cada componente numérico también está limitado a 65535 para los recursos de Windows.
- CMake lee JSON directamente. `python3 scripts/sync-version.py` sincroniza las versiones derivadas del espacio de trabajo Cargo. La CI usa `--check`. Los paquetes internos de Rust heredan la versión del espacio de trabajo. Actualiza Cargo.lock después y ejecuta `cargo check --workspace --locked`.
- El recurso numérico de Windows de cuatro componentes contiene la versión base y el número de compilación. La versión textual conserva el canal. En macOS, `CFBundleShortVersionString` contiene la versión base, `CFBundleVersion` el número de compilación y la versión larga conserva el canal. Sigue la [documentación de Apple](https://developer.apple.com/documentation/bundleresources/information-property-list/cfbundleversion). Usa el formato numérico permitido y evita commits Git en campos numéricos del sistema.
- En Windows, `FileDescription` vale `xdzip Archive Manager` en lugares como el Administrador de tareas. `ProductName` sigue siendo `xdzip`. El nombre del ejecutable y la identidad de las asociaciones son independientes del nombre visible. Las comprobaciones de compilación leen los recursos del EXE real.
- Cada compilación de escritorio genera un commit Git abreviado y un indicador de cambios sin confirmar. Sin Git se muestra un valor desconocido. No se incluyen rutas de la máquina, nombres de usuario ni marcas de tiempo inestables. La cabecera solo se actualiza si cambia su contenido. Compilar de nuevo tras el commit produce la identidad limpia correcta.

Las versiones del módulo QML, del protocolo del puente y del esquema de ajustes son independientes de la versión del producto.

`xdzip --version`, Configuración → Acerca de y la aplicación de vista previa leen la misma información de compilación. La CLI procesa los argumentos de versión antes de crear la interfaz gráfica. Acerca de muestra versión, compilación, commit, Qt utilizado, sistema y arquitectura. Indica el canal de desarrollo y los cambios sin confirmar. El JSON copiado no contiene archivos personales, rutas, nombres de máquina ni credenciales. El botón del proyecto solo abre el navegador si el usuario lo pulsa.

El icono grande de Acerca de usa el PNG de 512 px de la plataforma.

Acerca de usa los parámetros lógicos de Configuración sin pie. Admite los [idiomas de interfaz incluidos](../ux/settings-localization.md), temas claro y oscuro, y tamaños 100%, 125% y 150%. Consulta el diseño de Acerca de. Los valores nativos de Qt y del commit se mantienen desconocidos si no están disponibles.

Las comprobaciones usan `scripts/check-build-version.py` para la CLI y los recursos nativos, `scripts/test-version.py`, `BrowserTest::versionAboutWindow`. Las capturas y medidas quedan en `build/review/about/`. Las fuentes, los metadatos del sistema y las versiones en ejecución deben coincidir. El número de versión no acredita por sí solo la firma, la aceptación manual ni la compatibilidad.
