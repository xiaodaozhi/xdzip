<a name="xdzip-adaptive-filename-encoding-contract"></a>

# Contrato de codificación adaptativa de nombres de archivo

[English](../../engineering/filename-encoding.md) | [简体中文](../../zh-CN/engineering/filename-encoding.md) | [繁體中文](../../zh-TW/engineering/filename-encoding.md) | [日本語](../../ja-JP/engineering/filename-encoding.md) | [한국어](../../ko-KR/engineering/filename-encoding.md) | [Français](../../fr-FR/engineering/filename-encoding.md) | [Deutsch](../../de-DE/engineering/filename-encoding.md) | Español | [Português (Brasil)](../../pt-BR/engineering/filename-encoding.md) | [Русский](../../ru-RU/engineering/filename-encoding.md)

<a name="xdzip-resolution-and-identity"></a>

## Resolución e identidad

La decodificación se realiza en la capa de formatos Rust independiente de Qt. No usa páginas de códigos del sistema anfitrión, idioma de interfaz, conjeturas sobre el sistema de origen, contenido de archivos ni servicios de red. En ZIP, raw_name conserva los bytes originales del campo de nombre. TAR conserva el nombre original elegido por PAX/GNU tras quitar el `./` inicial según las reglas existentes. El EntryId físico es independiente del nombre. decoded_name almacena la escritura UTF-8 que requiere transcodificación o una declaración adicional; name_encoding conserva el origen de la resolución y name_bytes proporciona el nombre efectivo congelado. display_name es solo presentación escapada y no puede ser una ruta de salida.

ZIP resuelve los nombres en este orden:

1. Si está activada la marca UTF-8, exigir UTF-8 válido en el campo original; no reparar una declaración dañada con una codificación antigua.
2. En caso contrario leer Info-ZIP Unicode Path 0x7075 y aceptar la versión 1 con CRC32 coincidente del nombre original y UTF-8 completo y válido. Ignorar versiones desconocidas y CRC no coincidentes según la especificación. Marcar para revisión UTF-8 inválido con CRC coincidente, campos válidos duplicados o estructura dañada.
3. Sin una extensión utilizable, si todos los nombres no declarados del grupo son UTF-8 válido, aceptar UTF-8 sin marcar.
4. En caso contrario aplicar detección estadística a escala del archivo comprimido a los nombres no ASCII no declarados, incluyendo GBK/GB18030, Big5, Shift-JIS, EUC-JP, coreano y otros juegos admitidos. La hipótesis occidental predeterminada de ZIP vuelve a su valor histórico CP437. Si se detecta GBK, usar el decodificador estricto GB18030 compatible.

Las cabeceras TAR ordinarias, nombres largos GNU y rutas PAX marcadas explícitamente hdrcharset=BINARY usan la misma estrategia para nombres no declarados. Las rutas PAX ordinarias requieren UTF-8 válido; la detección no debe ocultar declaraciones dañadas. No se admiten valores explícitos de hdrcharset distintos de UTF-8 y BINARY. TAR no hereda el valor CP437 de ZIP. TAR.GZ usa la misma resolución de nombres que TAR.

Las reglas siguen [ZIP APPNOTE 6.3.10](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT), [POSIX pax](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/pax.html) y el [estándar WHATWG Encoding](https://encoding.spec.whatwg.org/). Un campo Unicode Path puede faltar en la cabecera local, pero un valor válido proporcionado debe coincidir con el nombre efectivo congelado. La comprobación de datos sigue verificando nombre original, cabeceras local y central, CRC, tamaño, autenticación de contraseña y terminación del flujo.

CAB conserva los bytes originales del nombre y el atributo UTF-8. El UTF-8 explícito se valida estrictamente; los demás nombres reutilizan la detección acotada a escala del archivo sin páginas de códigos del sistema. Las barras inversas de CAB son separadores de directorio del formato, pero solo se normalizan a barras normales después de la decodificación completa, evitando dividir el byte posterior multibyte 0x5C de Shift-JIS. Los bytes originales no cambian y las rutas efectivas pasan los mismos controles de seguridad y colisión. Los nombres ISO que no son Joliet conservan bytes originales y usan la misma resolución acotada, evitando la conversión OEM del anfitrión para Rock Ridge. Joliet/UDF usan nombres Unicode proporcionados por el procesador y los validan estrictamente; los caracteres de reemplazo con pérdida no se tratan como nombres originales.

<a name="xdzip-detection-boundaries-and-resources"></a>

## Límites y recursos de detección

El detector lee hasta 1 MiB de nombres completos no declarados que no son UTF-8 y separa los registros con saltos de línea para que secuencias multibyte truncadas no se combinen entre nombres. Las declaraciones Unicode explícitas y extensiones inválidas no son muestras. Cuando una muestra contiene nombres que no son UTF-8, aplicar de forma coherente el resultado a escala del archivo a los nombres no declarados, evitando que fragmentos de GBK casualmente válidos como UTF-8 dividan un directorio en dos nombres. Solo si falla la decodificación estricta de un nombre individual puede probarse UTF-8 válido o un resultado local para ese nombre. La detección de ISO-2022-JP está desactivada; no se interpretan estados de escape en los nombres. La decodificación estricta no inserta caracteres de reemplazo, elimina BOM, convierte barras, normaliza Unicode ni repara rutas.

Los nombres originales suman como máximo 16 MiB. Cada nombre original o decodificado tiene como máximo 4096 bytes, y los nombres decodificados y su presentación escapada juntos como máximo 16 MiB. Las asignaciones por entrada y acumuladas de la extensión Unicode ZIP se comprueban durante el análisis, no al final del archivo. Resolución, detección y conversión individual comprueban la cancelación. Los datos ZIP conservan como máximo otros 16 MiB para una tabla de verificación de nombres efectivos; una decodificación posterior no puede cambiar nombres congelados.

La detección estadística no puede recuperar de forma única nombres cortos arbitrarios o codificaciones antiguas mezcladas, ni identificar texto corrupto ya guardado como Unicode válido por otro programa. Unicode declarado y codificación antigua pueden coexistir en un archivo comprimido; varias codificaciones sin declarar se detectan según el mejor esfuerzo. Actualmente no hay selector manual de codificación. Los nombres que no pueden decodificarse estrictamente, tienen declaraciones dañadas o rutas anómalas permanecen escapados en pantalla y marcados para revisión, sin conceder extracción, apertura temporal, exportación ni modificación. No se promete identificar correctamente todos los archivos históricos. Si hay nombres sin resolver puede desconocerse su pertenencia real a directorios, por lo que se rechaza extraer mediante expansión de directorios para no omitir elementos silenciosamente de un resultado satisfactorio. Los archivos regulares resueltos y seleccionados explícitamente aún pueden extraerse individualmente.

<a name="xdzip-end-to-end-use-and-saving"></a>

## Uso integral y guardado

El árbol de directorios, índice de hijos directos, búsqueda recursiva, expansión de selección de directorios, precomprobación, extracción, apertura temporal, exportación externa y modificación usan el mismo nombre efectivo. Después de decodificar se siguen comprobando PortablePath, alias Unicode NFD o de mayúsculas, nombres duplicados y autorización de salida. El byte posterior multibyte 0x5C de Shift-JIS no es un separador de directorio. Salvo los separadores de formato CAB y LHA, las barras inversas realmente decodificadas siguen rechazándose.

Tras planificar, añadir y mover en ZIP, TAR y TAR.GZ ordinarios vuelve a declarar como UTF-8 nombres de codificación antigua, extensión Unicode y UTF-8 sin marcar, conservando posiciones originales y semántica de omisión del plan lógico de movimiento. ZIP conserva los datos comprimidos y TAR usa rutas PAX UTF-8 explícitas. Tras escribir, analizar de nuevo, verificar cada nombre y contenido y efectuar la sustitución existente en el mismo volumen. Si al reescribir TAR se elimina una declaración BINARY de una ruta PAX y esto cambiaría la interpretación de otro texto PAX no UTF-8, fallar explícitamente y conservar el archivo original. Los archivos cifrados y divididos usan los mismos nombres al leer y extraer; las restricciones de añadir y mover no cambian.

<a name="xdzip-reproducible-acceptance"></a>

## Aceptación reproducible

Usar datos de prueba propios de bytes originales y archivos comprimidos codificados independientemente con la biblioteca estándar Python, cubriendo UTF-8 sin marcar, GBK, GB18030 de cuatro bytes, Big5, Shift-JIS con 0x5C, coreano, ruso, ZIP CP437 y Unicode Path. Verificar nombres completos, bytes originales, directorios, búsqueda, selección, precomprobación, bytes reales de salida, apertura temporal, exportación, añadir y mover en archivos ordinarios, AES y volúmenes divididos. Comprobar también declaraciones dañadas, prioridad de extensiones, rechazo de rutas, colisiones Unicode o de mayúsculas, cancelación y presupuestos.

scripts/check-filename-encoding.py extrae ZIP/TAR/TAR.GZ generados por Python mediante una sonda real del servicio. Python también lee independientemente archivos reescritos por Rust para verificar marcas UTF-8/PAX, nombres y contenido. Los datos de prueba proceden solo de constantes del código, no de archivos de usuarios. Los resultados reales y la aceptación manual pendiente se registran en incidencias y artefactos build/CI, no como estado aprobado en este contrato.

LHA conserva los límites originales de los campos dirname/filename y los separadores originales `0xff` de dirname. La detección acotada compartida toma muestras de cada campo completo por separado. Decodificar estrictamente cada componente antes de interpretar barras inversas decodificadas y componer `/`; los límites originales impiden tomar bytes multibyte de otro campo. La expansión de directorios y la salida usan este mismo nombre efectivo. Consulta el [contrato LHA](lha-containers.md).
