<a name="xdzip-7z-format-sources-and-independent-implementation"></a>

# Fuentes del formato 7z e implementación independiente

[English](../../engineering/sevenzip-format-sources.md) | [简体中文](../../zh-CN/engineering/sevenzip-format-sources.md) | [繁體中文](../../zh-TW/engineering/sevenzip-format-sources.md) | [日本語](../../ja-JP/engineering/sevenzip-format-sources.md) | [한국어](../../ko-KR/engineering/sevenzip-format-sources.md) | [Français](../../fr-FR/engineering/sevenzip-format-sources.md) | [Deutsch](../../de-DE/engineering/sevenzip-format-sources.md) | Español | [Português (Brasil)](../../pt-BR/engineering/sevenzip-format-sources.md) | [Русский](../../ru-RU/engineering/sevenzip-format-sources.md)

Este documento recoge fuentes reutilizables y límites de implementación. La aceptación de la migración y las capacidades restantes se siguen en la [incidencia #50](https://github.com/xiaodaozhi/xdzip/issues/50); el encaminamiento Rust fijado y la eliminación de la dependencia oficial del producto no acreditan automáticamente la aceptación multiplataforma, manual o de entrega. Las reglas generales de sustitución están en el [contrato de implementación independiente](../../engineering/independent-rust-codecs.md).

<a name="xdzip-specifications-and-reference-identity"></a>

## Especificaciones e identidad de referencia

Los campos del contenedor, enteros de longitud variable, grafos de métodos, propiedades y asignaciones de archivos siguen los documentos oficiales `DOC/7zFormat.txt` y `DOC/Methods.txt`. La referencia fijada es el [archivo oficial de código 7-Zip 26.05](https://github.com/ip7z/7zip/releases/download/26.05/7z2605-src.tar.xz), SHA-256 `0b6080b8c3e7a579161d2d495224c0e51c467576cb4c7048b0ab485ef44cc22e`. Los SHA-256 de los dos documentos son respectivamente `8ec976b73ca88710286be1e991e49dbab485d9c0bf2410a80fa87e657850f82b` y `b16ea5fdc50345fa2cb031f06fe410cea0b089f74a4138ac8141b9e2d816ffc7`.

El flujo de bits LZMA también consulta el [paquete oficial de especificación](https://www.7-zip.org/a/lzma-specification.7z) de Igor Pavlov del 2015-06-14, SHA-256 del archivo `4529aca392d6b9598f72086e7d136b1e05d2863bf7a757836d3fd4f49e623b58`; el texto de especificación tiene SHA-256 `6a4d441c2fa1d8d37b0189c7147a4b5424d5cdb122a7fc1fbebd022864ed54fb`. Solo se leen texto del formato, matemáticas de la especificación y pseudocódigo; el decodificador de referencia del paquete no se usa como entrada para portar a Rust. Descargar documentos o muestras no concede nuevas licencias de redistribución.

Deflate genérico sigue RFC 1951 y comparte implementación con ZIP, GZIP y MSZIP. Deflate64 selecciona explícitamente reglas extendidas en esa implementación: historial de 64 KiB, códigos de distancia 30/31 y código de longitud 285; Deflate/MSZIP ordinarios no cambian. La sección de Deflate mejorado de [PKWARE APPNOTE](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT) es solo una visión general. Las diferencias concretas del flujo se complementan con análisis factual de la referencia fijada y verificación independiente de lectura; la visión general no debe presentarse como especificación completa. Las matemáticas AES-CBC y los vectores independientes siguen [NIST SP 800-38A](https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf). Las propiedades 7z, derivación de contraseñas UTF-16LE y relleno cero son reglas adicionales del formato y no se deducen de ZIP AES ni PKCS#7. El manual de BZip2 y el nombre de formato PPMd no especifican por completo flujos históricos, reinicios de modelo ni terminación; no puede afirmarse que estas implementaciones se obtienen solo leyendo manuales.

<a name="xdzip-reference-analysis-and-original-coding"></a>

## Análisis de referencias y código original

Una persona analista lee selectivamente los siguientes grupos del archivo original fijado para completar campos omitidos, estados matemáticos, reinicios por capacidad, terminación y relaciones de errores. Las entregas contienen solo hechos y requisitos de identificación, sin código de referencia, reescritura línea por línea, diseños de clases o matrices, descomposición de funciones, flujo de control copiado ni comentarios traducidos. Los algoritmos Rust los organizan independientemente implementadores que no han leído las implementaciones de referencia correspondientes; una persona analista no implementa un algoritmo que haya analizado. Quien implementa el contenedor tampoco ha leído su implementación oficial y programa de forma independiente a partir de documentación y hechos entregados.

| Tema de análisis | Alcance de referencia en el archivo fijado | Distinción de licencia |
| --- | --- | --- |
| LZMA1/LZMA2 | `C/LzmaDec.c/.h`, `C/Lzma2Dec.c/.h`, partes de rango, bloque/reinicio y propiedades de codificadores y envolturas C++ Lzma/Lzma2 | Estos archivos C declaran dominio público; las envolturas C++ usan los términos predeterminados del archivo |
| Detalles del contenedor | Atributos externos, herencia de sumas de subflujos y propiedades de archivos en `CPP/7zip/Archive/7z/7zIn.cpp` | LGPL-2.1-or-later predeterminado del archivo |
| 7z AES | `CPP/7zip/Crypto/7zAes.cpp/.h`, longitud de bloque, contraseña, relleno y finalización en MyAes, FilterCoder y envolturas del códec 7z | LGPL-2.1-or-later predeterminado; la implementación AES subyacente no se lee ni se porta |
| BZip2 | BZip2Decoder, Const, Crc y tratamiento diferenciado de errores en envolturas 7z ordinaria y bz2 independiente | LGPL-2.1-or-later predeterminado; no asignar la licencia de otra implementación por el nombre del formato |
| PPMd7 | Estados matemáticos, capacidad virtual y reinicios en `C/Ppmd7.c/.h`, `Ppmd7Dec.c`, `Ppmd7Enc.c`; restricciones de propiedades y terminación en la envoltura C++ Ppmd | Estos archivos C declaran dominio público; las envolturas C++ usan los términos predeterminados. No se copian diseños de contexto, asignador ni flujo de control |
| Deflate64 | Partes de modo de 64 bits de `CPP/7zip/Compress/DeflateConst.h` y DeflateDecoder, Deflate64Register y admisión de propiedades 7z | LGPL-2.1-or-later predeterminado; la entrega cubre solo diferencias de formato respecto a Deflate ordinario |
| ISA, Delta, Swap | Semántica escalar en `C/Bra.c`, `Bra86.c`, `Bra.h`, `Delta.c`, `SwapBytes.c` y envolturas C++ de registro de métodos, propiedades y cola | Estos archivos C declaran dominio público; las envolturas C++ usan por separado los términos predeterminados. La estrategia SIMD no es regla de formato |
| BCJ2 | Cuatro flujos y estado matemático en `C/Bcj2.c/.h`; envolturas C++ Bcj2Coder de finalización del decodificador y tamaño | Los archivos C declaran dominio público; las envolturas C++ usan los términos predeterminados. El cuerpo del codificador no se analiza ni copia |

Conservar con la evidencia de desarrollo las identidades del archivo fijado y sus archivos, el alcance real de lectura y los resúmenes factuales entregados. Volver a comprobar al actualizar la referencia; números de línea antiguos no demuestran el comportamiento de una versión nueva. Las constantes de especificación, identificadores y relaciones matemáticas pueden usarse para implementar el formato; optimizaciones internas y comportamiento históricamente indefinido no conceden permisos al producto. Las declaraciones de dominio público no abarcan todo el archivo, documentación, envolturas C++ ni otras muestras.

Esto no afirma una sala limpia estricta ni ausencia de riesgo jurídico. Las fuentes de terceros, entregas históricas con el motor oficial y herramientas de prueba independientes siguen sujetas a sus propios términos. El producto actual no compila el motor oficial; consulta [licencias](licensing.md) y [código correspondiente](source-delivery.md).

<a name="xdzip-interoperability-boundaries"></a>

## Límites de interoperabilidad

El éxito entre nuestro escritor y lector no sustituye una referencia independiente. Las pruebas verifican por separado salidas conocidas de compresores independientes, lectura oficial de nuestros flujos comprimidos, consumo de toda la entrada, terminación, CRC, corrupción, cuotas, cancelación y errores de origen. La verificación de componentes y la aceptación del anfitrión, proceso y plataformas se registran por separado.

Las tolerancias históricas de compatibilidad requieren confirmación individual. Por ejemplo, un nodo 7z BZip2 puede aceptar bits no usados no nulos en el último byte y exactamente cuatro bytes repetidos al final de un bloque sin byte adicional de cuenta, pero sigue requiriendo CRC correcto, tamaño declarado y terminación completa del flujo. Esto no admite bytes adicionales ni flujos concatenados. Los bits del último byte y los bytes adicionales no son el mismo tipo de relleno.

PPMd7 distingue perfiles de terminación para longitudes externas conocidas y desconocidas. Tras la salida exacta, un nodo 7z de longitud conocida comprueba el estado terminal del rango y el intervalo comprimido sin consumir un símbolo END adicional; el lector compartido para longitud desconocida exige END. Los reinicios de capacidad del modelo definidos por el formato y la admisión de memoria del anfitrión son aspectos separados: la memoria insuficiente no debe provocar un reinicio de formato inventado. Las entradas de comparación independientes incluyen flujos con reinicio por capacidad generados por el codificador C oficial fijado. El generador y el código original de referencia son solo herramientas de prueba locales y no dependencias de producción.
