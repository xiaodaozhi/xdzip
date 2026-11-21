# Instalar XDZip y abrir un archivo comprimido

[English](../../user-guide/install.md) | [简体中文](../../zh-CN/user-guide/install.md) | [繁體中文](../../zh-TW/user-guide/install.md) | [日本語](../../ja-JP/user-guide/install.md) | [한국어](../../ko-KR/user-guide/install.md) | [Français](../../fr-FR/user-guide/install.md) | [Deutsch](../../de-DE/user-guide/install.md) | Español | [Português (Brasil)](../../pt-BR/user-guide/install.md) | [Русский](../../ru-RU/user-guide/install.md)

Descarga el paquete para tu sistema desde la [última versión estable en GitHub](https://github.com/xiaodaozhi/xdzip/releases/latest) o usa Microsoft Store en Windows. La página de GitHub indica el nombre de archivo y la versión, y ofrece `SHA256SUMS` para verificar las descargas directas. Consulta los requisitos en la [política de compatibilidad](../engineering/support-policy.md).

| Sistema de escritorio | Instalación o inicio |
| --- | --- |
| Windows 11 x64 | Busca **XDZip** en Microsoft Store o ejecuta el archivo `-setup.exe` de GitHub para instalarlo en tu cuenta de usuario. El instalador de GitHub no está firmado. Para usar el ZIP portátil, extrae todo el paquete e inicia `xdzip.exe` desde la carpeta extraída. |
| macOS 14 o posterior, Apple Silicon | Abre el DMG y arrastra `xdzip.app` a **Aplicaciones**. Si descargas un `.app.zip`, primero extráelo y luego mueve la aplicación a esa carpeta. Ejecuta la copia de Aplicaciones. Las compilaciones de GitHub tienen firma de integridad ad hoc y carecen de notarización. Si macOS muestra un aviso, comprueba el origen de la descarga y la suma de verificación. |
| Ubuntu 24.04 LTS x64, X11, glibc 2.39 o posterior | Instala el paquete `.deb` mediante el gestor de paquetes. Si usas el `.tar.gz` portátil, extrae todos los archivos y ejecuta `./xdzip` desde la carpeta de ejecución extraída. Mantén el lanzador junto a los archivos de ejecución. |

La aplicación de escritorio no es compatible con Windows ARM64, los Mac con Intel, Linux ARM64 ni Wayland. La herramienta independiente de línea de comandos de Linux funciona sin X11 y requiere OpenSSL 3 y los certificados del sistema. Para usarla, elige un archivo cuyo nombre comience por `xdzip-cli-` y sigue la [guía rápida de la línea de comandos](../integration/README.md).

<a name="xdzip-open-an-archive"></a>

## Abrir un archivo comprimido

Usa **Abrir un archivo comprimido** en XDZip o arrastra el archivo a la página de bienvenida. En el gestor de archivos, elige **Abrir con** si el paquete instalado está registrado. Para elegir XDZip como aplicación predeterminada de un tipo de archivo, selecciónalo en los ajustes del sistema. La edición de Microsoft Store incluye **Abrir aplicaciones predeterminadas de Windows** en la página de asociaciones de archivos. Las aplicaciones predeterminadas y las acciones de compresión del menú contextual se configuran por separado. Consulta [extracción desde el gestor de archivos](extract.md#xdzip-from-the-file-manager) e [instalación y asociaciones](../engineering/installers-associations.md).

Después, [explora los archivos y obtén vistas previas](browse.md).
