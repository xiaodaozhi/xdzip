<p align="center">
  <img src="design/assets/brand/xdzip-128.png" width="80" height="80" alt="Icono de xdzip">
</p>

<h1 align="center">xdzip</h1>

<p align="center"><a href="README.md">English</a> | <a href="README.zh-CN.md">简体中文</a> | <a href="README.zh-TW.md">繁體中文</a> | <a href="README.ja-JP.md">日本語</a> | <a href="README.ko-KR.md">한국어</a> | <a href="README.fr-FR.md">Français</a> | <a href="README.de-DE.md">Deutsch</a> | Español | <a href="README.pt-BR.md">Português (Brasil)</a> | <a href="README.ru-RU.md">Русский</a></p>

<p align="center"><strong>Comprimir y descomprimir, un poco más fácil</strong></p>

<p align="center">xdzip es un gestor de archivos comprimidos multiplataforma y de código abierto. Sus formatos están implementados en Rust y funcionan sin programas externos de extracción. Permite explorar y buscar entradas, extraer una selección, trabajar con archivos cifrados y divididos en volúmenes y modificar entradas en formatos compatibles.<br>Admite ZIP, 7z, RAR, TAR, CAB, ISO/UDF y otros formatos en Windows, macOS y Linux.</p>

<p align="center">
  <a href="https://github.com/xiaodaozhi/xdzip/actions/workflows/docs.yml"><img src="https://github.com/xiaodaozhi/xdzip/actions/workflows/docs.yml/badge.svg?branch=main" alt="Integración continua"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-blue.svg" alt="Apache License 2.0"></a>
</p>

<p align="center">
  <a href="#xdzip-download-and-install">Descargar</a> / <a href="#xdzip-using-xdzip">Uso</a> / <a href="#xdzip-format-support">Formatos compatibles</a> / <a href="#xdzip-build-from-source">Compilar desde el código fuente</a> / <a href="docs/es-ES/README.md">Documentación</a> / <a href="https://github.com/xiaodaozhi/xdzip/issues">Comentarios</a>
</p>

![Exploración de un archivo comprimido con xdzip](docs/images/main-window.png)

<a name="xdzip-download-and-install"></a>

## Descargar e instalar

En Windows 11 x64, busca **XDZip** en Microsoft Store. Para descargar directamente u obtener la CLI autónoma, elige tu sistema en la [última versión estable de GitHub](https://github.com/xiaodaozhi/xdzip/releases/latest).

| Sistema | Entorno compatible | Descarga de GitHub |
| --- | --- | --- |
| Windows | Versiones de Windows 11 aún mantenidas, x86_64 | `setup.exe` por usuario o `.zip` portátil |
| macOS | macOS 14 o posterior, Apple Silicon | `.dmg` o paquete de aplicación `.app.zip` |
| Linux | Ubuntu 24.04 LTS, x86_64, X11, glibc 2.39 o posterior | Paquete `.deb` o directorio de ejecución `.tar.gz` |

Los archivos de escritorio empiezan por `xdzip-<version>-`. Los de la [CLI](#xdzip-command-line) autónoma empiezan por `xdzip-cli-<version>-`. Usa `SHA256SUMS` en la página de la versión para verificar la descarga directa. El instalador de Windows de GitHub no está firmado. La descarga de macOS usa una firma ad hoc de integridad y no está notarizada. Consulta la [guía de instalación](docs/es-ES/user-guide/install.md) para iniciar cada paquete y configurar las asociaciones de archivos.

Microsoft Store administra las actualizaciones de su edición y la publica independientemente de GitHub. Para las descargas de GitHub, comprueba si hay actualizaciones en **Acerca de xdzip**. La instalación desde la aplicación está disponible para las instalaciones de Windows por usuario que cumplan los requisitos, las aplicaciones de macOS instaladas en Aplicaciones y las instalaciones mediante DEB en Ubuntu. Actualiza las copias portátiles manualmente. Consulta [ajustes y actualizaciones](docs/es-ES/user-guide/settings-and-updates.md) y la [política de compatibilidad](docs/es-ES/engineering/support-policy.md).

**Nota sobre las versiones.** Esta página describe las funciones del código fuente actual. Para los paquetes publicados, consulta las notas de la versión correspondiente. Las compilaciones a partir del código fuente son versiones experimentales de desarrollo.

<a name="xdzip-using-xdzip"></a>

## Usar xdzip

Abre un archivo comprimido para explorar sus carpetas, buscar y ordenar entradas y elegir qué extraer. Las carpetas aparecen primero en ambos sentidos de ordenación. La interfaz ofrece vistas de lista y cuadrícula, propiedades de archivos, selección del idioma en Ajustes, temas claro y oscuro, y ajustes de tamaño de texto y densidad. El idioma predeterminado sigue el del sistema cuando es compatible y usa inglés de Estados Unidos en los demás casos. Un ZIP o ZIP64 ordinario de un solo archivo puede indexar hasta un millón de registros para la navegación. La mayoría de los demás formatos tienen límites inferiores. La extracción, la verificación del archivo completo y la modificación tienen límites independientes. Consulta el [manual de usuario](docs/es-ES/user-guide/README.md) y la [indexación de archivos grandes](docs/es-ES/engineering/large-index.md).

- **Vista previa del contenido.** Con la tarjeta de propiedades visible, selecciona un solo archivo para previsualizar automáticamente texto sin formato, imágenes de mapa de bits o SVG estático compatibles. macOS también admite HEIC y HEIF. La vista previa aparece una vez leído todo el contenido y realizadas las comprobaciones previstas por el formato. Puedes cancelar la operación. Consulta los [formatos y límites de la vista previa](docs/es-ES/engineering/content-preview.md), incluidos los tamaños y el presupuesto de descodificación.
- **Extraer y abrir.** Elige el destino y la regla para los conflictos de nombres, consulta el progreso y solicita la cancelación si es necesario. Al hacer doble clic en un archivo normal se extrae una copia a un directorio temporal privado antes de abrirla con el sistema. Los cambios en la copia no se escriben de vuelta en el archivo comprimido. Las copias en caché se eliminan al abrir un archivo en una operación posterior, una vez transcurridos siete días.
- **Crear y modificar.** Crea archivos ZIP, TAR, TAR.GZ, GZIP, XZ, TAR.XZ, LZ4, TAR.LZ4, Zstandard, TAR.ZST, LZH y 7z e imágenes de datos ISO9660/Joliet. Windows x64 también permite crear ZIP autoextraíbles. Añade, reemplaza, mueve, renombra o elimina entradas en los archivos regulares de un único archivo que cumplan los requisitos. También puedes arrastrar archivos fuera de la lista o soltarlos en ella y confirmar que quieres añadirlos. Consulta la tabla para conocer los formatos y límites.
- **Verificar.** Comprueba un archivo completo en segundo plano sin escribir archivos. Las comprobaciones dependen del formato. TAR, ISO, UDF y Compound File (MSI), por ejemplo, permiten comprobar la estructura y la legibilidad sin sumas de comprobación del contenido por archivo. Consulta la [verificación de integridad](docs/es-ES/engineering/archive-checking.md).
- **Varios archivos comprimidos.** Se abren en pestañas nuevas de forma predeterminada. También puedes configurarlos para que se abran en ventanas nuevas. La sesión de inicio comparte el proceso de la aplicación y su icono en la bandeja. Las tareas en segundo plano siguen accesibles desde la lista de tareas al cerrar las vistas de navegación.
- **Acciones del gestor de archivos.** En las ediciones instaladas de Windows 11, usa **Mostrar más opciones** para el menú clásico. La edición de Microsoft Store también puede aparecer en el menú moderno. En Ubuntu usa Archivos (Nautilus) con la extensión instalada. En macOS, activa la extensión de Finder en los ajustes del sistema o usa el servicio de Finder registrado. Consulta la [extracción desde el gestor de archivos](docs/es-ES/user-guide/extract.md#xdzip-from-the-file-manager).

Al crear archivos comprimidos o añadir archivos se omiten los archivos `.DS_Store` y las carpetas `__MACOSX` y `_MACOSX`. La extracción comprueba las rutas y los permisos del destino, gestiona los conflictos de nombres y prepara los archivos en una ubicación temporal. Los formatos compatibles también verifican el contenido. El tratamiento de objetos especiales y enlaces depende del formato y del sistema. Consulta las [reglas de extracción segura](docs/engineering/zip-safe-extraction.md).

<a name="xdzip-format-support"></a>

## Formatos compatibles

Consulta la tabla para saber si xdzip puede abrir, crear o modificar un archivo. «Modificar» incluye añadir, reemplazar, mover, renombrar o eliminar entradas dentro del archivo existente. La lectura y la verificación dependen del contenido real, el método de compresión y el tipo de registro. Al abrir un archivo por su extensión se inicia la detección. Para conocer los límites detallados, consulta el [registro de formatos integrados](docs/es-ES/engineering/format-registry.md) y la [guía de verificación de integridad](docs/es-ES/user-guide/checks-and-tasks.md).

| Formato | Explorar, extraer y verificar | Crear | Modificar |
| --- | --- | --- | --- |
| ZIP y archivos candidatos ZIPX/XPI/APK/IPA/CBZ | Store, Deflate, Deflate64, BZip2, LZMA y XZ, con perfiles AES-256 y multivolumen admitidos | ZIP con Store, Deflate, AES-256 y volúmenes numerados | Archivos ZIP/ZIPX aptos que usan los métodos indicados |
| TAR, TAR.GZ (incluido `.tgz`) | Archivos normales y carpetas | Compatible. TAR agrupa archivos sin comprimirlos | Archivos regulares de un solo archivo que cumplan los requisitos |
| GZIP (`.gz`) | Un archivo lógico, incluidos los miembros GZIP concatenados. Un TAR dentro de GZIP muestra su árbol TAR | Un archivo normal, Deflate 0/1/6/9 | No compatible |
| BZip2, LZMA Alone, Lzip v1 | Un archivo normal, dentro de los límites de flujo y memoria | No compatible | No compatible |
| TAR.BZ2, TAR.LZMA, TAR.LZ | Archivos normales y carpetas | No compatible | Archivos que cumplan los requisitos |
| XZ, TAR.XZ (incluido `.txz`) | Comprobaciones declaradas, LZMA2, Delta/BCJ y flujos concatenados | LZMA2 en los niveles 1/6/9, XZ con un archivo normal | TAR.XZ |
| LZ4, TAR.LZ4 | Tramas estándar, bloques independientes o enlazados, concatenación y comprobaciones declaradas | LZ4 en los niveles 1/6/9, LZ4 ordinario con un archivo normal | TAR.LZ4 |
| Zstandard, TAR.ZST (incluido `.tzst`) | Tramas estándar, ventanas admitidas, Huffman/FSE, desplazamientos repetidos y comprobaciones declaradas | Zstandard en los niveles 1/6/9, Zstandard ordinario con un archivo normal | TAR.ZST |
| 7z | Contraseñas, nombres de archivo cifrados, archivos sólidos y volúmenes numerados | Cifrado, archivos sólidos y volúmenes numerados | Archivos compatibles sin cifrar y de un solo volumen |
| DEB | Miembros ar habituales, vistas TAR de control y datos, perfiles admitidos de GZIP/XZ/Zstandard/BZip2/LZMA | No compatible | No compatible |
| LHA/LZH | Cabeceras de nivel 0–3, Store, LH1–LH7, LZS/LZ5 y CRC declarados | LH5 en los niveles 1/6/9, cabeceras de nivel 2, archivos normales y carpetas | Archivos aptos de nivel 2 con CRC por entrada |
| [PMA](docs/es-ES/engineering/pma-streams.md) | Nivel 0 con PM0/PM1/PM2, estructura CP/M exacta y CRC declarados | No compatible | No compatible |
| [ARJ](docs/es-ES/engineering/arj-streams.md) | Store y métodos 1–4, cabeceras verificadas y CRC-32 completo de los miembros | No compatible | No compatible |
| [ARC](docs/es-ES/engineering/arc-streams.md) | Store antiguo y nuevo, métodos 3–9, estructura exacta y CRC-16 completo de los miembros | No compatible | No compatible |
| [BH](docs/es-ES/engineering/bh-containers.md) | Miembros ordinarios `BH 05 07`, método 8 Deflate sin envoltura, tamaños exactos y CRC-32 | No compatible | No compatible |
| [ALZ](docs/es-ES/engineering/ace-alz-egg-format-sources.md) | Miembros normales de un solo volumen con Store o Deflate sin envoltura, tamaños exactos y CRC-32 | No compatible | No compatible |
| [EGG](docs/es-ES/engineering/ace-alz-egg-format-sources.md) | Miembros sin cifrar ni sólidos de un solo volumen con Store o Deflate sin envoltura, tamaños exactos y CRC-32 | No compatible | No compatible |
| [PEA](docs/es-ES/engineering/disk-image-archive-format-sources.md) | PEA 1.x de volumen único, nombres UTF-8, archivos y carpetas PCOMPRESS0 almacenados sin cifrar. Sin comprobaciones de objetos, flujos ni volúmenes | No compatible | No compatible |
| [Compound File (MSI)](docs/es-ES/engineering/wim-msi-installer-format-sources.md#xdzip-compound-file-and-msi) | Explora almacenamientos y extrae flujos sin interpretar. Comprueba la estructura y la legibilidad | No compatible | No compatible |
| [NSIS (`.exe`)](docs/es-ES/engineering/wim-msi-installer-format-sources.md#xdzip-nsis-and-other-sfx) | Solo metadatos estáticos `firstheader`. No permite extraer ni verificar el archivo completo | No compatible | No compatible |
| ASAR | Archivos y carpetas almacenados, rangos de datos compartidos y SHA-256 declarados de archivos y bloques | No compatible | No compatible |
| Z, TAR.Z (incluido `.taz`) | Unix compress LZW, diccionarios de 9–16 bits y perfiles admitidos de CLEAR y alineación | No compatible | TAR.Z |
| Brotli, TAR.BR | Flujos ordinarios RFC 7932, ventanas de 10–24 bits y todas las transformaciones del diccionario estático | No compatible | TAR.BR |
| AES Crypt (`.aes`) | Versiones 0–3, contraseña de apertura, HMAC de todo el texto cifrado y reglas de longitud específicas de la versión | No compatible | No compatible |
| UU/UUE, XXE | Líneas tradicionales con longitud declarada y envolturas begin-base64 admitidas, también con varios miembros | No compatible | No compatible |
| RAR | Formatos antiguos, RAR5 y extensiones de RAR 7, incluidas contraseñas y archivos sólidos y multivolumen | No compatible | No compatible |
| CAB | Store, MSZip, Quantum, LZX y volúmenes en la misma carpeta | No compatible | No compatible |
| ISO | ISO9660, Joliet y Rock Ridge | Imagen de datos ISO9660 + Joliet | No compatible |
| UDF | UDF | No compatible | No compatible |
| ZIP SFX | ZIP autoextraíbles validados | Windows x64, Store/Deflate, `.sfx.exe` | No compatible |

- ZIP AES-256 cifra solo el contenido. Los nombres de archivo siguen visibles. No se admiten AES-128/192 ni ZipCrypto tradicional. Los archivos divididos se pueden leer con `.zip.001`, PKZIP `.z01` y convenciones relacionadas. La creación usa volúmenes numerados. Consulta [extraer con contraseña](docs/es-ES/engineering/password-extraction.md) y [abrir archivos divididos](docs/es-ES/engineering/archive-opening.md).
- Los flujos independientes BZip2, LZMA Alone y Lzip v1 son de solo lectura. LZMA carece de suma de comprobación del contenido. La comprobación abarca la estructura, los tamaños y la legibilidad. Consulta [especificación de flujos de un solo archivo](docs/es-ES/engineering/single-file-streams.md) para las extensiones, las entradas, los límites de memoria y la admisión.
- XZ ejecuta las comprobaciones de contenido que declara cada flujo. Los flujos que declaran None solo permiten comprobar la estructura y la legibilidad. ZIPX lee los métodos ZIP indicados. Las variantes PPMd, JPEG y WavPack siguen fuera del alcance. Al editar ZIP/ZIPX se conservan los datos comprimidos y el método de las entradas que permanecen. Las entradas nuevas usan Deflate.
- Modificar un 7z reconstruye el archivo. El conjunto de trabajo para los datos conservados y añadidos tiene un límite de 512 MiB. Los archivos nuevos leen la entrada como flujo y no están sujetos a ese límite de reconstrucción. Los ZIP y 7z cifrados o divididos en volúmenes no pueden modificarse. Consulta la [modificación de archivos comprimidos](docs/engineering/archive-transfers.md).
- RAR, CAB y UDF son de solo lectura. La [creación de imágenes ISO](docs/es-ES/engineering/iso-creation.md) admite archivos normales y carpetas, un solo volumen, archivos de menos de 4 GiB y nombres Joliet de hasta 64 caracteres UCS-2 por componente. No admite imágenes de arranque, escritura de UDF o Rock Ridge ni modificación de imágenes. Leer una imagen nunca la monta ni la ejecuta. Consulta la [especificación de RAR](docs/engineering/rar-format-sources.md) y la [especificación de los motores de formato](docs/es-ES/engineering/native-codecs.md).
- La [creación de SFX](docs/es-ES/engineering/sfx-creation.md) produce un extractor ZIP autónomo para Windows x64 sin Qt ni una instalación de xdzip. Permite elegir el destino, tratar conflictos y cancelar, y nunca ejecuta los archivos extraídos. No admite contraseñas, volúmenes ni comandos de inicio automático. Los `.exe` genéricos quedan fuera de las asociaciones de archivos.
- DEB muestra `members/`, `control/` y `data/` como vistas de exportación. Extraer todo incluye los miembros sin procesar y el contenido expandido. No se instala nada ni se ejecutan scripts. Control admite datos sin comprimir, GZIP, XZ y Zstandard. Datos admite también BZip2 y LZMA. La verificación confirma la legibilidad del contenedor y las comprobaciones declaradas, sin autenticar el paquete. Consulta la [especificación de DEB](docs/es-ES/engineering/deb-containers.md).
- Z y TAR.Z permiten comprobar la estructura y la legibilidad, sin sumas de comprobación. No se pueden crear archivos Z independientes ni reparar flujos antiguos dañados. Consulta [especificación de Unix compress](docs/es-ES/engineering/compress-streams.md).
- UU/UUE y XXE comprueban la estructura y la legibilidad sin sumas de comprobación. Las rutas y modos de las cabeceras no conceden permisos de escritura ni de ejecución. La codificación GNU de nombres y la reparación de archivos en varias partes quedan fuera del [perfil de envolturas de texto](docs/es-ES/engineering/text-envelopes.md).
- Abrir o extraer un ASAR de solo lectura no inicia su aplicación. Los datos externos `unpacked` y los enlaces declarados son visibles, pero están fuera del alcance. No se accede a carpetas vecinas ni se restaura el permiso de ejecución. Los hashes opcionales detectan daños. Sin hash solo se comprueba la estructura y la legibilidad. Consulta la [especificación de ASAR](docs/es-ES/engineering/asar-containers.md).
- NSIS solo se reconoce si la superposición PE contiene un registro `firstheader` válido y acotado. xdzip muestra esa fila de metadatos estáticos. Nunca ejecuta el iniciador PE, el script de instalación, complementos ni datos integrados. Tampoco descomprime la cabecera del instalador ni permite extraer esos datos. La entrada `.exe` queda oculta en las asociaciones y en arrastrar y soltar. El soporte se limita a este perfil.
- La creación LHA/LZH utiliza LH5, cabeceras de nivel 2 y nombres UTF-8. Las herramientas antiguas de Windows pueden mostrar los nombres con su página de códigos. Se pueden modificar los archivos aptos de nivel 2. Los nombres antiguos se descodifican antes de interpretar los separadores. La descodificación del contenido verifica los CRC presentes. Las entradas antiguas sin CRC solo permiten comprobar la estructura y la legibilidad. No se admiten contraseñas, volúmenes ni reparación. PMA utiliza un perfil separado de solo lectura. Consulta [especificación de LHA](docs/engineering/lha-containers.md).
- ARJ es de solo lectura. Los miembros binarios y de texto antiguo entregan los bytes cubiertos por el CRC sin convertir los saltos de línea. No se admiten contraseñas, volúmenes, perfiles de recuperación o seguridad, enlaces ni objetos especiales. Los métodos desconocidos siguen visibles e impiden que la verificación del archivo completo tenga éxito.
- ARC es de solo lectura. Los archivos normales se procesan en secuencia con una terminación exacta. No se admiten la descodificación con contraseña, los volúmenes, SFX ni las extensiones Spark. Los métodos desconocidos siguen visibles sin alias ZIP e impiden que la verificación completa tenga éxito. El cifrado XOR antiguo carece de indicador en la cabecera. La descodificación exige un flujo válido, el tamaño exacto y el CRC, sin recurrir a contraseñas ni reparación.
- BH admite para lectura el perfil `BH 05 07` de método 8 Deflate sin envoltura. Excluye métodos desconocidos, cifrado, carpetas, volúmenes, datos finales y prefijos SFX.

- Los ZIP grandes que usan el índice en disco no admiten actualmente ni la verificación del archivo completo ni la modificación. Aún puedes extraer archivos normales seleccionados dentro de los límites de la tarea. Consulta la [indexación de archivos grandes](docs/es-ES/engineering/large-index.md).
- Actualmente no se admiten archivos dispersos en TAR ni extensiones de terceros. Los GZIP ordinarios y los candidatos XPI/APK/IPA/CBZ derivados de ZIP siguen la [especificación de flujos de un solo archivo](docs/es-ES/engineering/single-file-streams.md). Las codificaciones antiguas se detectan cuando es posible. Los nombres cortos o las codificaciones mezcladas no siempre se pueden identificar de forma inequívoca. Consulta la [codificación de nombres](docs/es-ES/engineering/filename-encoding.md).

- AES Crypt requiere la contraseña de apertura antes de mostrar metadatos. Todos los MAC del texto cifrado deben ser válidos. Las versiones 0–2 dejan sin autenticar el resto de la longitud final y las extensiones no otorgan permisos. Consulta la [especificación de AES Crypt](docs/es-ES/engineering/aescrypt-envelopes.md).

- La detección de Brotli y TAR.BR sin firma inequívoca exige validar todo el contenido. El formato no incluye suma de comprobación. La modificación de TAR.BR escribe metabloques estándar sin comprimir y puede aumentar el tamaño. Los diccionarios compartidos y las extensiones de ventana grande no se admiten en [perfil de Brotli admitido](docs/es-ES/engineering/brotli-streams.md).

<a name="xdzip-command-line"></a>

## Línea de comandos

La CLI autónoma funciona sin Qt ni sesión gráfica. Puede listar, extraer, crear, ampliar y probar archivos comprimidos, y generar JSON. Descomprime el archivo de distribución `xdzip-cli-<version>-` y ejecuta `./bin/xdzip --help` en macOS o Linux, o `.\bin\xdzip.exe --help` en Windows PowerShell. Mantén el proceso auxiliar incluido junto al ejecutable. La [guía rápida de la CLI](docs/es-ES/integration/README.md) explica los primeros comandos y cómo instalarla en PATH. También puedes compilarla desde el código fuente.

```sh
git clone https://github.com/xiaodaozhi/xdzip.git
cd xdzip
python3 scripts/cli.py build
python3 scripts/cli.py test
build/cli/bin/xdzip --help
build/cli/bin/xdzip list archive.zip --json
build/cli/bin/xdzip extract archive.zip -C output
```

En Windows, usa `python` y añade `.exe` al nombre del ejecutable. En Linux, la ejecución requiere OpenSSL 3 y los certificados del sistema. Para compilar se necesitan también `libssl-dev` y `pkg-config`. Consulta la [guía de la CLI](docs/engineering/command-line.md) para ver el uso de contraseñas, el modo no interactivo, los códigos de salida, la instalación del comando y los límites de actualización.

<a name="xdzip-build-from-source"></a>

## Compilar desde el código fuente

La interfaz de escritorio usa Qt Quick/QML y se conecta al núcleo Rust mediante una capa fina de C++. Los formatos, las consultas y las tareas son independientes de Qt. Los lectores de 7z, RAR, CAB, ISO y UDF y el escritor de 7z usan procesos auxiliares aislados. La creación de ISO se ejecuta en el proceso principal de Rust. El SFX autónomo de Windows usa cuadros de diálogo del sistema sin Qt.

La documentación está en `docs/`. Los parámetros visuales, la terminología y las listas de dependencias que usan los generadores y las comprobaciones se encuentran en `specs/`. Los diseños de alta fidelidad se encuentran en `design/hifi/`.

Instala Python 3.10+, Git, CMake 3.24+, Ninja, rustup y el compilador y SDK de tu plataforma. A continuación, descarga el código fuente y compílalo. Si ya has clonado el repositorio para la CLI, empieza por `prepare`.

```sh
git clone https://github.com/xiaodaozhi/xdzip.git
cd xdzip
python3 scripts/desktop.py prepare --install-rust --install-qt
python3 scripts/desktop.py doctor
python3 scripts/desktop.py build
python3 scripts/desktop.py test
```

En Windows, usa un entorno de desarrollo MSVC x64. En macOS, usa un terminal Apple Silicon nativo y Xcode Command Line Tools. La cadena de herramientas predeterminada de Qt 6.8.2 está en `build/toolchain/`. Usa `--qt-prefix` si ya tienes Qt instalado. La [guía de compilación](docs/engineering/developer-build.md) cubre dependencias, inicio, empaquetado y verificación. La CLI y el escritorio usan puntos de entrada de compilación independientes. La compilación de escritorio también incluye una CLI privada cuyo punto de entrada debe activarse de forma explícita.

Los paquetes de desarrollo son experimentales. También puedes descargar los archivos de desarrollo de Windows y Linux de ejecuciones manuales correctas del [flujo de componentes de Qt](https://github.com/xiaodaozhi/xdzip/actions/workflows/qt-components.yml). La CI habitual no genera paquetes de desarrollo automáticamente.

<a name="xdzip-documentation-and-contributing"></a>

## Documentación y contribuciones

| Tema | Dónde empezar |
| --- | --- |
| Uso de escritorio y scripts | [Manual de usuario](docs/es-ES/user-guide/README.md), [integración por línea de comandos](docs/es-ES/integration/README.md) |
| Diseño e implementación del producto | [Referencia técnica](docs/es-ES/reference/README.md) |
| Privacidad y tratamiento de datos | [Política de privacidad](PRIVACY.md), [versión en chino](PRIVACY.zh-CN.md) |
| Compilar y contribuir | [Guía de contribución](docs/es-ES/contributing/README.md) |
| Cambios y trabajo previsto | [Registro de cambios](CHANGELOG.md), [incidencias de GitHub](https://github.com/xiaodaozhi/xdzip/issues) |
| Contribuciones | [Normas de contribución](CONTRIBUTING.md), [código de conducta](CODE_OF_CONDUCT.md) |
| Políticas del proyecto | [Gobernanza](GOVERNANCE.md), [seguridad](SECURITY.md) |

Agradecemos los comentarios y las contribuciones. Al informar de un error, incluye la versión o el commit, el sistema y la arquitectura, los pasos para reproducirlo y el resultado obtenido. Genera tus propias muestras y evita incluir información sensible. Comunica los problemas de seguridad en privado mediante [Report a vulnerability](https://github.com/xiaodaozhi/xdzip/security/advisories/new).

<a name="xdzip-license"></a>

## Licencia

El contenido original del proyecto tiene la [licencia Apache 2.0](LICENSE). Consulta [COPYRIGHT](COPYRIGHT). Qt, los códecs, las fuentes y las demás dependencias están sujetos a sus propias licencias. La [documentación de licencias](docs/es-ES/engineering/licensing.md) enumera fuentes y avisos. Las versiones oficiales no incluyen un archivo independiente con el código fuente correspondiente. Consulta la [entrega de código fuente](docs/engineering/source-delivery.md). Se mantienen las obligaciones aplicables sobre código fuente, reenlazado y avisos. Cada versión candidata requiere su propia revisión. Los registros históricos están en [#4](https://github.com/xiaodaozhi/xdzip/issues/4) y [#53](https://github.com/xiaodaozhi/xdzip/issues/53).
