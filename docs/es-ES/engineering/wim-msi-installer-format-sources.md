<a name="xdzip-wim-msi-and-installer-format-sources-and-licenses"></a>

# Fuentes y licencias de WIM, MSI y formatos de instaladores

[English](../../engineering/wim-msi-installer-format-sources.md) | [简体中文](../../zh-CN/engineering/wim-msi-installer-format-sources.md) | [繁體中文（台灣）](../../zh-TW/engineering/wim-msi-installer-format-sources.md) | [日本語](../../ja-JP/engineering/wim-msi-installer-format-sources.md) | [한국어](../../ko-KR/engineering/wim-msi-installer-format-sources.md) | [Français](../../fr-FR/engineering/wim-msi-installer-format-sources.md) | [Deutsch](../../de-DE/engineering/wim-msi-installer-format-sources.md) | Español | [Português (Brasil)](../../pt-BR/engineering/wim-msi-installer-format-sources.md) | [Русский](../../ru-RU/engineering/wim-msi-installer-format-sources.md)

Esta página describe las referencias técnicas, restricciones de licencia y requisitos para admitir WIM, MSI y formatos de instaladores. Las referencias permanecen en `build/toolchain/references` o enlazan a especificaciones públicas; ninguna es una dependencia del producto.

<a name="xdzip-wim"></a>

## WIM

- Implementación de referencia: `ebiggers/wimlib`, commit `cd5e231c348c255ae5088873b5a66ee0eb96fa07`.
- El repositorio ofrece GPLv3+ para el proyecto y una opción LGPL-2.1+ condicionada para la biblioteca. La opción LGPL depende de que las dependencias de terceros, como `libntfs-3g`, no impongan condiciones GPL.
- WIM es un contenedor de imágenes, no un archivo comprimido ordinario. Un lector con límites debe procesar tablas de recursos, flujos de metadatos, recursos divididos en bloques, compresión Xpress/LZX/LZMS, tablas de integridad, metadatos de enlaces físicos y selección de imágenes sin montar ni restaurar permisos de seguridad.

La implementación de referencia sirve para estudiar el formato y los conjuntos de muestras, pero la política actual que excluye dependencias de ejecución de terceros impide enlazarla con xdzip. WIM está **excluido por decisión del producto**: el proyecto no asumirá el lector específico para imágenes, el modelo de amplificación por descompresión, los recursos de integridad, la semántica de enlaces físicos ni el corpus completo necesarios para admitirlo. No se permite ninguna entrada de registro, MIME, entorno de ejecución o CLI mientras no se reconsidere expresamente esta decisión.

<a name="xdzip-compound-file-and-msi"></a>

## Compound File y MSI

- Microsoft [MS-CFB] es la principal especificación pública: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/53989ce4-7b05-4f8d-829b-d08d6148375b . La página publica revisiones en PDF/DOCX y un aviso de propiedad intelectual que permite implementar la tecnología descrita; las licencias de patentes siguen sujetas a los programas aplicables de Microsoft.
- Antes de interpretar contenido específico de MSI, un lector CFB debe gestionar la asignación de sectores y FAT/DIFAT, el recorrido del árbol rojo-negro de directorios, la asignación de mini-streams, los límites de tamaño de los flujos y la detección de ciclos.

Un lector independiente de metadatos CFB es técnicamente viable y ya está implementado en la parte de solo lectura `xdzip-codec-cfb`. Antes de exponer filas de almacenamientos y flujos, valida las cabeceras CFB 3.0/4.0, incluidas las bases de sector de 512 y 4096 bytes, las cadenas FAT/DIFAT, el árbol de directorios, la asignación de mini-streams, los límites de tamaño de los flujos y los ciclos. La copia de flujos sin procesar solo está disponible cuando coincide la instantánea inmutable de metadatos. Las tablas MSI, los archivos cabinet integrados, las acciones personalizadas y las transformaciones quedan fuera del contrato: no se ejecutan instaladores, no se interpretan acciones personalizadas, no se instalan paquetes ni se escriben cambios de vuelta. La extensión `.msi` no concede permiso de ejecución. Las pruebas automatizadas del códec y del entorno de ejecución cubren las muestras y la evidencia de ejecución.

<a name="xdzip-nsis-and-other-sfx"></a>

## NSIS y otros SFX

- Fuente de referencia de NSIS: `kichik/nsis`, commit `a96303eae3115221471bb127fa916a13d8bf9ecd`.
- Su archivo `COPYRIGHT` indica zlib/libpng para la mayor parte del código y el compresor zlib, bzip2 para el módulo bzip2 y Common Public License 1.0 para el módulo LZMA.
- Los archivos NSIS son instaladores ejecutables con scripts y contenido integrado. Un analizador debe permanecer fuera del límite de ejecución, acotar los intervalos de cabecera y contenido y separar los métodos de compresión del comportamiento del instalador.
- El código público define el registro fijo `firstheader`: `flags`, `0xDEADBEEF`, las palabras de firma `NullsoftInstallSystem`, `length_of_header` y `length_of_all_following_data`. Un lector estático puede localizarlo y validarlo sin abrir el programa de arranque PE, pero aún necesita un modelo independiente para los bloques de cabecera comprimidos y muestras completas de instaladores propios.

NSIS está **admitido únicamente para metadatos estáticos**. El códec solo explora registros alineados a 512 bytes dentro de una zona adicional PE acotada, acepta los dos valores mágicos `firstheader` registrados, limita las marcas y longitudes y exige que el intervalo posterior declarado quepa en la entrada física. Después publica una fila de metadatos `NSIS/firstheader`. No descomprime la cabecera del instalador ni expone ningún byte del contenido. La entrada de registro `.exe` no es deliberadamente candidata a arrastrar y soltar ni a una asociación del sistema; prevalece la detección por contenido. El analizador es independiente de Qt y de las API de ejecución de instaladores. Nunca ejecuta el programa de arranque PE, scripts, páginas personalizadas, complementos ni contenido extraído. Los demás formatos SFX solo se mantienen como alias cuando su archivo comprimido interno supera la validación de contenido.

<a name="xdzip-admission-gates"></a>

## Requisitos de admisión

1. Fijar las revisiones del código y los hashes de los textos de licencia antes de usar una herramienta de referencia.
2. Crear muestras completas y redistribuibles para cada contenedor y método de compresión admitido; una muestra que solo contenga la cabecera no es suficiente.
3. Mantener el análisis de CFB/WIM/NSIS independiente de Qt y de las API de instalación, montaje, registro o ejecución de scripts.
4. Añadir entradas al entorno de ejecución, la CLI y el escritorio solo después de superar las pruebas de identidad física exacta, entradas duplicadas, límites de ciclos, límites acumulados de entrada, trabajo y salida, cancelación, modificación del origen y fallo terminal.
5. Mantener los componentes de referencia GPL/LGPL/CPL en build/toolchain, salvo que una decisión independiente sobre dependencias y licencias cambie expresamente la política del producto.
