<a name="xdzip-lz4-stream-contract"></a>

# Contrato de los flujos LZ4

[English](../../engineering/lz4-streams.md) | [简体中文](../../zh-CN/engineering/lz4-streams.md) | [繁體中文（台灣）](../../zh-TW/engineering/lz4-streams.md) | [日本語](../../ja-JP/engineering/lz4-streams.md) | [한국어](../../ko-KR/engineering/lz4-streams.md) | [Français](../../fr-FR/engineering/lz4-streams.md) | [Deutsch](../../de-DE/engineering/lz4-streams.md) | Español | [Português (Brasil)](../../pt-BR/engineering/lz4-streams.md) | [Русский](../../ru-RU/engineering/lz4-streams.md)

El perfil de tramas estándar sigue las especificaciones del autor para el [formato de trama 1.6.5](https://github.com/lz4/lz4/blob/v1.11.1/doc/lz4_Frame_format.md), el [formato de bloque del 2026-10-26](https://github.com/lz4/lz4/blob/v1.11.1/doc/lz4_Block_format.md) y [XXH32](https://github.com/Cyan4973/xxHash/blob/v0.8.3/doc/xxhash_spec.md). Los algoritmos del producto son implementaciones independientes en Rust. Las herramientas de referencia y las evidencias de ejecución generadas se guardan localmente en `build/`. Solo se versionan patrones de entrada reproducibles del proyecto. Consulta los [flujos de un solo archivo](single-file-streams.md) para los permisos del host y la publicación.

<a name="xdzip-admission-and-framing"></a>

## Admisión y tramas

Las tramas estándar de versión 1 validan los bits reservados, los valores BD de 4 a 7, el XXH32 de la cabecera, el tamaño opcional del contenido, cada XXH32 declarado para bloques o contenido y el marcador final. Los datos de bloque no pueden superar el máximo declarado, entre 64 KiB y 4 MiB. Los bloques independientes reinician el historial. Los enlazados conservan solo los 64 KiB anteriores. Los bloques sin comprimir, incluso de longitud cero, se someten a las mismas comprobaciones y límites. Los perfiles con diccionario externo y las tramas antiguas quedan fuera del alcance. Declarar un diccionario nunca autoriza a leer otro archivo.

Las tramas estándar concatenadas y las tramas omitibles intercaladas se limitan por la entrada física, los presupuestos de la tarea, la cancelación y un máximo de 100 000 tramas o bloques. Un archivo independiente válido contiene al menos una trama de datos estándar. Los datos finales arbitrarios o una entrada con solo tramas omitibles provocan un fallo. La detección puede examinar hasta 8 MiB de entrada comprimida real antes de producir un prefijo de contenido de 512 bytes. Una identificación inequívoca o un TAR reconocido como dañado impide cambiar a otro formato tras un fallo. El escaneo completo valida todas las tramas antes de publicar el tamaño observado y la entrada virtual.

<a name="xdzip-block-processing-and-creation"></a>

## Procesamiento de bloques y creación

Las longitudes se calculan con aritmética comprobada. Las coincidencias necesitan un desplazamiento distinto de cero dentro del historial. Los bytes solapados se copian en orden ascendente y respetan el límite de salida declarado. La última secuencia contiene solo literales y se ignora su semibyte de coincidencia. Un flujo con coincidencias conserva como mínimo cinco literales finales. Su última coincidencia empieza como mínimo doce bytes descodificados antes del final del bloque. La lectura produce fragmentos acotados y se detiene tras un fallo. El historial pasa al bloque siguiente después de validar el final exacto del bloque. Las operaciones de E/S del host, la cancelación, los excesos de cuota y los perfiles incompatibles tienen categorías de error distintas.

Un escritor admitido debe emitir coincidencias LZ4 reales, usar bloques sin comprimir solo cuando la compresión no reduce el tamaño e incluir sumas de comprobación de bloque y contenido. Los niveles ofrecidos limitan el trabajo de búsqueda de coincidencias. En cada nivel debe poder descodificarse de forma independiente contenido con varios bloques. Los búferes de tramas y coincidencias se autorizan antes de asignarlos. La creación sigue usando tamaños exactos de entrada congelados, salida temporal a cargo del host, verificación completa y comprobaciones finales del origen y la salida. LZ4 ordinario recibe un único archivo normal. Una composición TAR solo se declara cuando el lector y el escritor TAR compartidos y las comprobaciones automáticas correspondientes la cubren. Registrar una extensión no habilita reescritura, cifrado ni volúmenes para ningún perfil.

La verificación usa `format_declared_checks` porque las tramas pueden omitir la suma de comprobación del contenido. Una trama sin suma de comprobación se informa sin afirmar que su contenido está verificado. Los límites de recursos, la aceptación real por plataforma, la validez del formato y la autorización de publicación se evalúan por separado.
