<a name="xdzip-sources-and-licenses-for-disk-images-and-other-archive-formats"></a>

# Fuentes y licencias de imágenes de disco y otros formatos de archivo

[English](../../engineering/disk-image-archive-format-sources.md) | [简体中文](../../zh-CN/engineering/disk-image-archive-format-sources.md) | [繁體中文](../../zh-TW/engineering/disk-image-archive-format-sources.md) | [日本語](../../ja-JP/engineering/disk-image-archive-format-sources.md) | [한국어](../../ko-KR/engineering/disk-image-archive-format-sources.md) | [Français](../../fr-FR/engineering/disk-image-archive-format-sources.md) | [Deutsch](../../de-DE/engineering/disk-image-archive-format-sources.md) | Español | [Português (Brasil)](../../pt-BR/engineering/disk-image-archive-format-sources.md) | [Русский](../../ru-RU/engineering/disk-image-archive-format-sources.md)

Esta página describe referencias técnicas públicas, restricciones de licencia y requisitos de soporte para DAA, ISZ, ZPAQ, PEA, BOND, J2J y formatos relacionados. Una descripción pública del formato no concede permiso para distribuir un descodificador y una implementación de referencia no es necesariamente una dependencia redistribuible del producto. Ninguna de las referencias siguientes se enlaza con xdzip. Antes de que un formato entre en el registro del producto deben establecerse revisión de fuente, texto de licencia, muestras completas, límites de recursos y comportamiento ante fallos.

<a name="xdzip-daa-10"></a>

## DAA 1.0

- **Especificación pública:** No se encontró una especificación DAA 1.0 publicada por el proveedor. Descripciones públicas identifican DAA como imagen de disco/archivo propietaria de PowerISO y describen un encabezado, tabla de longitudes de bloques y bloques ISO comprimidos independientemente. Los registros públicos cubren variantes zlib y LZMA y versiones internas posteriores como 110, pero no definen un contrato 1.0 estable.
- **Implementación de referencia:** `daa2iso` 0.1.7e, paquete Debian/Fedora bajo GPL-2.0-or-later. La fuente solo es útil para investigación histórica. Actualmente no se puede fijar un commit Git upstream ni una especificación del proveedor desde un repositorio público de autoridad.
- **Licencia y redistribución:** La implementación es GPL-2.0-or-later. Su licencia no concede derechos sobre la tecnología propietaria DAA y el proyecto no tiene un registro independiente de permiso para distribuir un descodificador en xdzip.
- **Muestras completas:** Las muestras públicas suelen ser imágenes de disco o archivos multipartes sin procedencia ni versión fijadas. Actualmente no hay un corpus redistribuible admitido que cubra encabezados antiguos, versión 110, orden multipartes, zlib, LZMA, ofuscación, sumas de comprobación y bloques malformados.
- **Viabilidad de sala limpia:** Una reconstrucción ISO acotada podría ser técnicamente posible tras establecer un formato estable y una decisión jurídica. La ingeniería inversa basada solo en un descodificador GPL no basta como evidencia para una implementación de producto.
- **Límite de capacidad:** No compatible permanentemente por decisión de producto. No añadir detector, códec, registro, tipo MIME, asociación, lector ni escritor DAA. La investigación histórica no autoriza su admisión futura.
- **Conclusión:** **Excluido permanentemente por decisión de producto.** xdzip no seguirá con DAA. Los hallazgos de fuentes, licencias y muestras son investigación histórica, no requisitos pendientes de entrega.

<a name="xdzip-isz"></a>

## ISZ

- **Especificación pública:** EZB Systems, *ISZ File Format Specification*, versión 1.00, revisada el 2006-07-03, documenta la firma `IsZ!`, encabezado fijo, tabla opcional de definición de segmentos, tabla de definición de bloques y bloques de datos. Los campos incluyen tamaño y total de sectores, modo de contraseña, tamaño de segmento, número y tamaño de bloques, longitud de puntero, número de segmento y desplazamientos de tablas y datos. Los indicadores de bloque identifican bloques cero, almacenados, zlib y BZip2. La especificación se reproduce en [la copia del foro ImgBurn](https://forum.imgburn.com/topic/14260-isz-images-supporting/#comment-115553).
- **Implementaciones de referencia:** Filtro ISZ de CDEmu `libmirage`, commit `39fcffe09739e1dc40bef4781646f9ffb8b7dc4e`, y `IszImageStream` de Play-, commit `5bb010f27d8e2309f37b1f8eba0f74ecd546b29b`. Ambos son solo material de análisis de referencia. `Helias/isz-tool` es un lector Python, pero en este entorno no se pudo obtener un commit inmutable de autoridad ni un registro de licencia.
- **Licencia y redistribución:** La sección 8 de la especificación EZB indica que incorporar tecnología ISZ a un producto requiere contactar con EZB para obtener una licencia gratuita sujeta a condiciones. Un texto público o lector independiente con licencia permisiva no elimina ese requisito de licencia de producto. CDEmu y Play- no son dependencias de xdzip.
- **Muestras completas:** Herramientas públicas pueden generar bloques Store, zlib y BZip2 válidos, pero una matriz redistribuible completa necesita imágenes de un volumen y segmentadas, bloques cero, todas las longitudes de puntero, bloques finales exactos, modos de contraseña, ofuscación de tablas, truncamiento, desplazamientos y cambios del origen. No hay tal corpus propio registrado en el repositorio.
- **Viabilidad de sala limpia:** El subconjunto sin cifrar, de un volumen y solo lectura es técnicamente separable de la navegación ISO. Una primera implementación podría descodificar los cuatro tipos documentados de bloque en una salida virtual `image.iso`, sin montaje ni ejecución de arranque. Sin embargo, autorización jurídica, referencia inmutable fijada y conjunto completo de muestras deben preceder al código.
- **Límite de capacidad:** Hasta cumplir esos requisitos, no registrar `.isz`, no inferir soporte de `IsZ!` ni ofrecer contraseña, AES, segmentación, escritura o navegación óptica directa. Si se admite después, el descodificador debe validar encabezado/versión, tablas de punteros y segmentos, reglas XOR/desofuscación, límites de bloques y datos, aritmética del tamaño lógico, terminación completa de bloques, cancelación y límites de entrada/trabajo/salida.
- **Conclusión:** **Excluido por decisión de producto.** xdzip no seguirá con ISZ ni contactará con EZB para una licencia de producto en este paquete de trabajo. No se puede añadir detector `.isz` ni registro sin revisar la decisión.

<a name="xdzip-zpaq"></a>

## ZPAQ

- **Especificación pública:** Matt Mahoney, *The ZPAQ Open Standard Format for Highly Compressed Data*, versiones 2.03 y 2.06. El formato almacena un programa de descompresión en bytecode ZPAQL y admite bloques sólidos, diario, deduplicación, registros SHA-1, cifrado en niveles posteriores y archivos multipartes.
- **Implementación de referencia:** `zpaq/zpaq`, etiqueta `6.22`, y las fuentes `libzpaq` correspondientes. La etiqueta es una referencia de investigación fijada para esta evaluación; antes de implementar deben capturarse el hash del commit y el hash del texto de licencia.
- **Licencia y redistribución:** La implementación de referencia y sus archivos de configuración son GPLv3. No pueden enlazarse al producto actual bajo la política que excluye runtimes de archivo de terceros. El estándar público permite análisis de sala limpia, pero la máquina virtual de descodificación y el corpus de configuración requieren implementación y revisión separadas.
- **Muestras completas:** Un corpus útil debe cubrir bloques de niveles 1 y 2, programas ZPAQL arbitrarios, actualizaciones de diario, deduplicación, registros cifrados y multipartes, verificación SHA-1 y límites malformados de máquina virtual o bloques. El repositorio no tiene ese corpus propio.
- **Viabilidad de sala limpia:** Posible en teoría, pero es un proyecto de máquina virtual y contabilidad de recursos, no un pequeño analizador de contenedores. Los límites seguros deben contabilizar interpretación de bytecode, memoria del modelo, salida descomprimida, historial y estado de deduplicación.
- **Límite de capacidad:** Un primer subconjunto futuro sería de solo lectura, sin cifrar, sin multipartes ni diario, con un subconjunto de máquina virtual declarado. No se implica creación, actualización, contraseñas ni extracción sólida ilimitada.
- **Conclusión:** **Excluido por decisión de producto.** xdzip no implementará la máquina virtual ZPAQL ni aceptará un perfil ZPAQ que dependa de ella. No se puede añadir detector `.zpaq` ni registro sin revisar la decisión.

<a name="xdzip-pea"></a>

## PEA

- **Especificación pública:** PeaZip, especificación *Pack, Encrypt, Authenticate*, que cubre PEA 1.x y PEA 2.0 aún propuesto, flujos, controles de volúmenes y objetos, marcadores finales y niveles PCOMPRESS. Se publica como [`pea_help.pdf`](https://peazip.github.io/pea_help.pdf) y resume en [pea-file-format.html](https://peazip.github.io/pea-file-format.html). La página oficial actual libera explícitamente las especificaciones PEA y PCOMPRESS al dominio público.
- **Implementación de referencia:** `peazip/PeaZip`, commit `81b31f7e487b462980a52ac0f2e0b3cf700fe799`, incluidos `peazip-sources/dev/pea_utils.pas` y `unit_pea.pas`. La implementación es LGPLv3 y las unidades criptográficas de Wolfgang Ehrhardt incluidas usan la licencia zlib/libpng. La URL de publicación de fuentes es [peazip-11.4.0.src.zip](https://github.com/peazip/PeaZip/releases/download/11.4.0/peazip-11.4.0.src.zip). La fuente fijada sirve solo para análisis de referencia.
- **Licencia y redistribución:** La especificación del formato en sí no bloquea la licencia de producto, pero el motor LGPLv3 y las dependencias criptográficas no pueden copiarse ni enlazarse por defecto al producto actual. Una implementación Rust de sala limpia debe usar la descripción de formato de dominio público y algoritmos seleccionados independientemente; el binario de referencia sigue siendo solo para pruebas.
- **Muestras completas:** El perfil admitido tiene muestras Store completas propias que cubren archivos, directorios, nombres UTF-8 válidos, datos vacíos y no vacíos, truncamiento, bytes posteriores, cambios de origen y límites. Un corpus más amplio debe incluir PEA 1.x y 2.0, múltiples flujos, límites de volúmenes, niveles PCOMP, fallos de autenticación, encabezados cifrados, marcadores finales malformados y tamaños declarados grandes; esos perfiles siguen fuera del alcance del producto.
- **Viabilidad de sala limpia:** Un lector Store/Deflate sin cifrar y de un flujo es técnicamente plausible, pero autenticación y cifrado deben tratarse como trabajo de seguridad separado. El diseño de integridad del formato impide presentar el mero análisis estructural como verificación del contenido.
- **Límite de capacidad:** Una admisión futura empezaría en solo lectura, sin contraseña, cifrado, varios volúmenes, creación ni modificación. Cada flujo extraído requeriría verificar el resumen declarado y la autenticación terminal antes de publicar.
- **Conclusión:** **Admitido como perfil conservador de solo lectura.** xdzip admite actualmente PEA 1.x, un volumen físico, nombres UTF-8, archivos/directorios almacenados PCOMPRESS0 y ninguna comprobación de objeto, flujo o volumen. Contraseñas, flujos autenticados o cifrados, algoritmos de suma, PCOMPRESS1–3, volúmenes divididos, PEA 2.0, creación y modificación siguen sin soporte. La referencia LGPL sigue siendo solo para pruebas y no se enlaza al producto.

<a name="xdzip-bond"></a>

## BOND

- **Especificación pública:** No se encontró una especificación, firma, versión o contrato de compresión de archivos Bandizip BOND. Notas recientes de Bandizip mencionan vulnerabilidades en «algunos archivos BOND», pero no definen el formato. Las bases públicas de extensiones `.bond` describen principalmente datos con esquema Microsoft Bond, un formato de serialización distinto.
- **Implementación de referencia:** No se encontró ninguna con identidad de formato de archivo de autoridad, revisión fija y licencia redistribuible.
- **Licencia y redistribución:** Desconocida. No se puede admitir producto ni descodificador mientras se desconozcan propietario del formato, derechos tecnológicos y condiciones de referencia.
- **Muestras completas:** Ninguna de una fuente fiable y versionada. Una extensión por sí sola no identifica una muestra aceptable.
- **Viabilidad de sala limpia:** No puede evaluarse sin una firma y suficientes muestras completas que distingan un archivo comprimido de datos Microsoft Bond u otra carga binaria.
- **Límite de capacidad:** Sin detección, registro de sufijo, navegación, extracción, comprobación, creación, contraseña ni volúmenes.
- **Conclusión:** **Excluido por decisión de producto y no confirmado como formato de archivo Bandizip.** No se añadirá detector `.bond` ni registro en este paquete de trabajo.

<a name="xdzip-j2j"></a>

## J2J

- **Especificación pública:** No se encontró especificación, firma, versión ni registro de desarrollador de archivos J2J. Listas históricas de funciones Bandizip mencionan J2J solo para extracción, mientras que las páginas oficiales actuales no dan definición técnica. Otros resultados J2J describen conjuntos de datos job-to-job o herramientas JPEG-to-JPEG sin relación.
- **Implementación de referencia:** No se encontró ninguna con revisión fija y licencia vinculable a un lector de archivos Bandizip.
- **Licencia y redistribución:** Desconocida. Las páginas de extensiones de terceros y listas antiguas de software no conceden una licencia.
- **Muestras completas:** No existe en el repositorio una matriz de muestras fiable y redistribuible. Un nombre `.j2j` no establece la identidad de un contenedor.
- **Viabilidad de sala limpia:** No evaluable hasta documentar firma, modelo de registros y comportamiento de compresión/cifrado y disponer de un corpus completo.
- **Límite de capacidad:** Sin detección, registro de sufijo, navegación, extracción, comprobación, creación, contraseña ni volúmenes.
- **Conclusión:** **Excluido por decisión de producto y no confirmado como formato de archivo Bandizip.** No se añadirá detector `.j2j` ni registro en este paquete de trabajo.

<a name="xdzip-requirements-for-format-support"></a>

## Requisitos de soporte de formatos

1. Registrar una especificación de autoridad o protocolo de sala limpia revisado jurídicamente. Un blog, base de extensiones o fuente de un descodificador por sí solos no bastan.
2. Fijar cada fuente de investigación a un commit inmutable o hash de contenido y registrar el texto de licencia aplicable.
3. Crear una matriz de muestras completa y redistribuible para cada método, versión, modo de volumen, estado de cifrado y condición terminal declarados. Muestras solo de encabezado o Store no demuestran interoperabilidad.
4. Definir un modelo acotado de recursos para lecturas de entrada, metadatos, trabajo de descompresión, memoria, salida lógica, historial, estado criptográfico y cancelación. Rechazar desplazamientos malformados, ciclos, desbordamientos y cambios de origen antes de publicar.
5. Mantener cada descodificador independiente de Qt y de API de montaje, arranque, instalador, scripts, registro y ejecución arbitraria de procesos.
6. Conectar runtime, CLI, asociaciones y documentación de usuario solo después de que códec, identidad de instantánea, comportamiento exacto de datos y clasificación de fallos superen la automatización específica.
7. Un candidato bloqueado sigue visible como bloqueado en la evaluación. No debe recibir sufijo, entrada MIME ni capacidad del producto solo porque Bandizip o una lista de terceros lo nombre.

El control del repositorio `python scripts/test-m10-source-gates.py` comprueba que las seis secciones bilingües conservan los campos de evidencia necesarios, que cada conclusión sigue explícitamente bloqueada, excluida o admitida y que ningún candidato bloqueado o excluido ha entrado en `specs/engineering/archive-formats.json`.
