<a name="xdzip-repository-layout-and-dependencies"></a>

# Estructura del repositorio y dependencias

[English](../../engineering/repository-layout.md) | [简体中文](../../zh-CN/engineering/repository-layout.md) | [繁體中文](../../zh-TW/engineering/repository-layout.md) | [日本語](../../ja-JP/engineering/repository-layout.md) | [한국어](../../ko-KR/engineering/repository-layout.md) | [Français](../../fr-FR/engineering/repository-layout.md) | [Deutsch](../../de-DE/engineering/repository-layout.md) | Español | [Português (Brasil)](../../pt-BR/engineering/repository-layout.md) | [Русский](../../ru-RU/engineering/repository-layout.md)

Los archivos Cargo.toml y CMakeLists.txt de la raíz definen los módulos actuales del producto. La tabla describe los directorios existentes. Las divisiones previstas en el diseño de arquitectura siguen siendo planes.

| Directorio | Responsabilidades y dependencias |
| --- | --- |
| apps/desktop | Qt/QML, controladores C++ ligeros, modelos, ajustes e interacción con el sistema. Accede al núcleo a través del puente. |
| crates/xdzip-cli | Argumentos, terminal, JSON, instalación administrada y host de actualizaciones sin Qt. Depende directamente del entorno de ejecución, la E/S segura y el núcleo de actualizaciones. Delega la interpretación de formatos. |
| crates/xdzip-domain | Tipos puros e invariantes. Sin dependencia de Qt ni de códecs concretos. |
| crates/xdzip-codec-api | Contratos de capacidades, entrada prestada, salida codificada y cancelación. Depende del dominio. |
| crates/xdzip-compression | Algoritmos y envolturas compartidos para Deflate/Deflate64/GZIP, LZMA/LZMA2, BZip2, PPMd7, MSZIP, LZX y Quantum. Sin dependencias de terceros en producción ni autoridad sobre rutas o publicación. Véase el [contrato](../../engineering/independent-rust-codecs.md). |
| crates/xdzip-crypto | Hashes SHA/HMAC y BLAKE de memoria fija, independientes del formato y con estado sensible borrable. Reutiliza primitivas admitidas. Otros módulos se encargan de las contraseñas y la publicación. |
| crates/xdzip-codec-zip, xdzip-codec-tar | Implementaciones ZIP y TAR/GZIP sin rutas de salida arbitrarias. |
| crates/xdzip-codec-lha, xdzip-codec-arj, xdzip-codec-arc, xdzip-codec-bh | Perfiles independientes de estructuras y entradas para [LHA](../../engineering/lha-containers.md)/[PMA](pma-streams.md), [ARJ](arj-streams.md), [ARC](arc-streams.md) y [BH](bh-containers.md), sin autoridad sobre el sistema de archivos. El registro de formatos determina su disponibilidad en el producto. |
| crates/xdzip-codec-7z | Contenedor 7z propio, grafos de métodos, filtros, composición AES, creación en flujo y reescritura acotada. No controla rutas de archivos. |
| crates/xdzip-codec-cab | Contenedor CAB propio de solo lectura con volúmenes autorizados por el host y algoritmos compartidos. Conserva nombres en bruto, declaraciones UTF-8 y registros ordinarios con nombres duplicados. La detección de volúmenes y la publicación corresponden al host. Véanse el [contrato](../../engineering/independent-rust-codecs.md#cab只读规格) y las [fuentes](../../engineering/cab-format-sources.md). |
| crates/xdzip-codec-optical | Lectores ISO9660/Joliet/Rock Ridge/UDF y escritor ISO9660/Joliet para imágenes de datos nuevas. Usa flujos sin controlar rutas. |
| crates/xdzip-codec-bridge | Límite CXX ligero entre el proceso aislado y los códecs Rust de 7z, RAR, CAB y medios ópticos. Flujos autorizados, límites de recursos y cancelación. |
| crates/xdzip-codec-native, tools/native-codec | Proceso aislado para los módulos de Rust de 7z/RAR/medios ópticos/CAB. Una capa ligera de C++ proporciona solo el protocolo y los servicios de calendario y codificación del sistema. El receptor nativo comprueba de manera independiente la salida ya contabilizada y el trabajo auxiliar. Véase el [contrato de E/S](native-codecs.md). |
| crates/xdzip-extract-plan | Comprobación previa de metadatos e informes acotados sin autorización de destino ni capacidad de escritura. |
| crates/xdzip-io | Árboles de entradas, autorización e identidad nativas, preparación segura y confirmación de escrituras. |
| crates/xdzip-runtime | Sesiones, índices acotados en memoria o disco, consultas, selección, vistas previas acotadas, tareas de creación y extracción y planificación. |
| crates/xdzip-qt-bridge | Puente de tipos cxx, dos manejadores y ciclo de vida. Depende del entorno de ejecución. |
| crates/xdzip-plugin-protocol | CBOR y tramas acotadas con la biblioteca estándar. No controla rutas, procesos ni Qt. |
| crates/xdzip-plugin-host, examples/sdk | Ensayo independiente del SDK de solo lectura, E/S controlada, requisitos para el aislamiento nativo y par Python independiente. Queda fuera de la detección de formatos y de las dependencias de ejecución del producto. |
| tools | Sondas independientes para el puente, los formatos, la E/S y el recuento de procesos. La interfaz gráfica tiene puntos de entrada propios. |
| scripts, .github | Generación, comprobaciones, herramientas de despliegue, CI y plantillas de colaboración. |
| release-notes | Notas revisables de cada versión y metadatos de versión y compilación. Los informes de ejecución y los paquetes generados se guardan aparte. |
| design/hifi, design/html, design/assets | Originales de alta fidelidad, diseños interactivos y recursos de entrada para el entorno de ejecución. |
| docs | Diseños mantenidos en Markdown, contratos técnicos, métodos de verificación reutilizables e imágenes necesarias. Los informes de ejecución se guardan aparte. |
| specs | Inventarios de diseño visual, terminología y dependencias legibles por máquina. Se usan en los recursos del producto, los generadores y las comprobaciones. |
| third_party | Fuentes de terceros y materiales de licencia conservados con el código. |

Los módulos independientes xdzip-index y xdzip-platform y las herramientas completas de conformidad para formatos externos forman parte de la arquitectura futura. Ahora el índice está en el entorno de ejecución. Las operaciones de archivos por plataforma están en io y los adaptadores de escritorio. El módulo plugin-host existente sirve al [ensayo del SDK de solo lectura](sdk-trial.md). Las pruebas se mantienen sobre todo con los módulos Rust, en apps/desktop/tests y en las comprobaciones de scripts. Se evita crear módulos vacíos sin implementación.

El producto dirige CAB siempre a Rust. Los controladores Cab oficiales y su LZX/Quantum específico para CAB quedan fuera de la compilación. Las llamadas siguen usando el mismo proceso aislado sin nuevos ejecutables, opciones de prueba ni rutas alternativas en caso de error. codec-api define la contabilización anticipada de tramas completas. Los contenedores conservan el crédito por trama. Los receptores nativos y el entorno de ejecución verifican sus propios créditos y límites de salida por separado. Véase el [contrato interno](../../engineering/independent-rust-codecs.md#cab只读规格).

La [arquitectura del sistema](../../design/system-architecture.md) explica las dependencias. Evita dependencias inversas del dominio hacia el entorno de ejecución, notificaciones directas de los códecs a QML y decisiones sobre los textos de interfaz en las capas de plataforma. Guarda los enlaces generados, muestras, informes y resultados de compilación en build/, target/ o dist/ fuera de los directorios de código. [Instalación y asociaciones](../../engineering/installers-associations.md#兼容基线的长期输入) indica qué entradas deben conservarse para comprobar compatibilidad histórica.

`tools/image-preview/` es un proceso interno de imágenes estáticas basado en QtGui/QtSvg. Recibe contenido acotado y verificado y devuelve datos RGBA acotados. No mantiene rutas del archivo comprimido ni del destino. Consulta las plataformas y el comportamiento ante fallos en el [contrato de vistas previas](content-preview.md).
