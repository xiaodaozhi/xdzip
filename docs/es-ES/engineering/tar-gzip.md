<a name="xdzip-targzip-implementation-contract"></a>

# Contrato de implementación de TAR/GZIP

[English](../../engineering/tar-gzip.md) | [简体中文](../../zh-CN/engineering/tar-gzip.md) | [繁體中文](../../zh-TW/engineering/tar-gzip.md) | [日本語](../../ja-JP/engineering/tar-gzip.md) | [한국어](../../ko-KR/engineering/tar-gzip.md) | [Français](../../fr-FR/engineering/tar-gzip.md) | [Deutsch](../../de-DE/engineering/tar-gzip.md) | Español | [Português (Brasil)](../../pt-BR/engineering/tar-gzip.md) | [Русский](../../ru-RU/engineering/tar-gzip.md)

Este contrato define los límites de navegación, creación y extracción segura de TAR/TAR.GZ. Los resultados de plataforma y la aceptación manual se registran en incidencias o artefactos de CI. Conserva la autorización de entrada, preparación temporal, conflictos, cancelación y confirmación del archivo completo de la [extracción segura](zip-safe-extraction.md) y la [creación ZIP](zip-creation.md).

<a name="xdzip-scope-and-capabilities"></a>

## Alcance y capacidades

Se admiten navegación, creación y extracción segura de TAR y TAR.GZ. TAR es un contenedor y GZIP un filtro de compresión; la lectura detecta el contenido en lugar de elegir un backend por extensión. El comportamiento ZIP sigue siendo compatible. No se habilitan expansión anidada automática, cifrado ni TAR dividido. GZIP ordinario de un solo archivo usa el [contrato de flujo](single-file-streams.md) independiente.

Las cabeceras TAR, números, PAX, codificación en flujo y reescritura usan una capa de registros Rust independiente. El encuadre GZIP, CRC-32 y Deflate LZ77/Huffman usan el módulo compartido `xdzip-compression`. `tar` y `flate2` son solo referencias para pruebas de desarrollo y no entran en las dependencias de producción de estos crates. Consulta el [contrato de formatos propios](../../engineering/independent-rust-codecs.md) para especificaciones, fuentes y límites.

TAR admite archivos normales y directorios y analiza rutas PAX y nombres largos GNU. No se pueden extraer enlaces simbólicos o físicos, dispositivos, FIFO, archivos dispersos ni tipos desconocidos. Todos los nombres de salida siguen pasando por PortablePath y la comprobación previa de conflictos del anfitrión; las bibliotecas de formato no reciben rutas ni autoridad de extracción. No se restauran permisos de directorio, propietarios ni modos especiales. Las fechas son metadatos de visualización sin garantía de restauración.

<a name="xdzip-bounded-streaming"></a>

## Procesamiento en flujo acotado

Las codificaciones antiguas de nombres, nombres largos GNU y declaraciones PAX UTF-8/BINARY se resuelven uniformemente según el [contrato de codificación](filename-encoding.md). Los índices se publican solo tras el análisis completo; durante él se desconocen los recuentos y totales. Los metadatos mantienen límites de 100 000 entradas, 4 096 bytes por nombre y presupuestos acumulados de nombres. La entrada codificada del flujo y el trabajo de decodificación tienen cada uno un máximo de 24 GiB; una cabecera de extensión GNU/PAX tiene como máximo 64 KiB y cada entrada hasta cuatro cabeceras de extensión. PAX global solo acepta comentarios sin semántica de archivo, por compatibilidad con git archive. Se comprueban íntegramente frente a la longitud declarada del registro y se descartan sin heredarse a las entradas. Los campos globales path, size, mtime, linkpath, sparse y desconocidos siguen rechazándose expresamente; no se admiten enlaces largos GNU ni variantes dispersas. Los comentarios globales también cuentan para las cuatro cabeceras por entrada y los presupuestos de bytes del flujo, y no hacen válidas extensiones locales incompletas. Se normaliza el prefijo habitual `./` de nombres TAR; los marcadores raíz no son entradas de salida. Lecturas y saltos secuenciales también comprueban cancelación y presupuestos de bytes; el posicionamiento no evita las comprobaciones de flujos corruptos. GZIP debe llegar a EOF y verificar CRC e ISIZE. Varios miembros solo forman un flujo continuo de bytes TAR. Los restos no nulos, truncamientos y corrupción fallan. Se rechazan índices parciales no verificados.

TAR.GZ no dispone de tamaños comprimidos por entrada. La admisión por alta relación compara el total declarado seleccionado con el tamaño completo de la fuente fijada. Por encima de 1000:1 y 1 MiB sigue siendo necesaria autorización expresa, que no evita las cuotas de trabajo de análisis ni de salida. La comprobación previa de metadatos no inventa relaciones de compresión por entrada.

La extracción procesa las entradas seleccionadas en orden físico. Saltar contenido no seleccionado se contabiliza y puede cancelarse; no se promete lectura aleatoria. PayloadReader solo recibe la fuente y el receptor del anfitrión. El anfitrión sigue verificando versiones de origen en los límites de análisis, decodificación y confirmación. La confirmación por entrada y la semántica de fallo de creación completa siguen el contrato seguro de E/S existente.

<a name="xdzip-creation-and-interface"></a>

## Creación e interfaz

La ventana de creación habilita los segmentos TAR y TAR.GZ existentes sin cambiar la densidad. Al cambiar de formato se actualizan las extensiones. TAR deshabilita el nivel de compresión; ambos formatos TAR deshabilitan y borran contraseña y valores de volúmenes divididos. TAR.GZ usa niveles GZIP 0/1/6/9. El tamaño comprimido por entrada es desconocido y se muestra con una raya, no como 0 bytes. Rust deriva uniformemente la carpeta de destino con el nombre del archivo comprimido a partir del nombre de origen y la proporciona al puente acotado para la vista previa. TAR.GZ elimina toda la extensión compuesta sin distinguir mayúsculas; un nombre base vacío recurre a archive. La interfaz no deriva otro destino; PortablePath y la raíz autorizada lo vuelven a validar. Las etiquetas de formato de la ventana principal se adaptan al formato real y al ancho del texto. Las fechas TAR se muestran en UTC y permanecen desconocidas fuera del intervalo de visualización existente. La creación escribe explícitamente rutas PAX UTF-8 para Unicode o nombres largos y tamaño PAX para archivos grandes, evitando que lectores de referencia adivinen la codificación local. Fija el árbol de entrada, escribe en flujo, termina TAR/GZIP, vuelve a analizar y decodifica y verifica cada entrada; publica el conjunto solo después del éxito. El cambio de nombre por conflicto conserva toda la extensión compuesta.

<a name="xdzip-verification-and-limits"></a>

## Verificación y límites

Cubre interoperabilidad bidireccional con tar del sistema/Python tarfile, rutas largas, Unicode, directorios vacíos, duplicados, PAX/GNU, entradas especiales, escape de rutas, truncamiento, CRC, límites de recursos, cancelación y protección de archivos antiguos. La creación, reapertura y extracción reales de Qt reutilizan las ventanas existentes de tareas y conflictos. La evidencia real solo se genera en build/ o artefactos de CI.

Elige las verificaciones adicionales de escritorio y accesibilidad afectadas según la [verificación general](../../../developers/qa/README.md); la automatización nativa y las comprobaciones fuera de pantalla no sustituyen las manuales. Los resultados y límites reales van en la incidencia correspondiente. La aceptación de escritorio existente está en [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5); los bloqueos puntuales de permisos o entorno no se conservan en este contrato. La revisión histórica de licencias de `v0.1.0` está en [#4](https://github.com/xiaodaozhi/xdzip/issues/4) y [#53](https://github.com/xiaodaozhi/xdzip/issues/53); los futuros candidatos necesitan nuevas incidencias de publicación.

La entrada de creación CLI admite .tgz como alias del mismo formato que .tar.gz, con la misma codificación GZIP, verificación y confirmación. El GZIP ordinario sin TAR utiliza el códec independiente de un solo archivo.
