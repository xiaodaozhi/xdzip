<a name="xdzip-arj-compressed-streams-and-container-admission"></a>

# Flujos comprimidos ARJ y admisión del contenedor

[English](../../engineering/arj-streams.md) | [简体中文](../../zh-CN/engineering/arj-streams.md) | [繁體中文](../../zh-TW/engineering/arj-streams.md) | [日本語](../../ja-JP/engineering/arj-streams.md) | [한국어](../../ko-KR/engineering/arj-streams.md) | [Français](../../fr-FR/engineering/arj-streams.md) | [Deutsch](../../de-DE/engineering/arj-streams.md) | Español | [Português (Brasil)](../../pt-BR/engineering/arj-streams.md) | [Русский](../../ru-RU/engineering/arj-streams.md)

`ArjReader` ofrece decodificación bruta acotada de métodos ARJ 1–4. Store pertenece al contenedor, no a este lector. `xdzip-codec-arj` proporciona estructura física completa y entrega de miembros verificada por CRC sobre una fuente propiedad del host. `ArjCodec` conecta perfiles admitidos al entorno, CLI y navegación, comprobación y extracción de escritorio. El [registro de formatos](../../../specs/engineering/archive-formats.json) declara el límite de solo lectura; creación, reescritura, contraseñas y volúmenes no están disponibles.

<a name="xdzip-host-admission-and-names"></a>

## Admisión del host y nombres

Una propuesta de cabecera principal de doce bytes nunca publica formato ni filas. El entorno conserva primero la prioridad de contenedores fuertes validados y ZIP/SFX y después examina completamente los metadatos ARJ propuestos bajo el registro compartido de entrada de sondeos débiles. Un marcador TAR bruto no validado se desambigua mediante admisión ARJ completa. Las hipótesis inválidas no publican nada; errores tipados del host, cancelación y cuotas siguen siendo determinantes.

El adaptador conserva ordinales físicos de miembros, bytes originales de método y nombres brutos. Los nombres marcados ANSI y los OEM forman grupos de codificación separados que comparten un límite de 1 MiB de muestreo de campos completos, sin selección por configuración regional ambiental. Decodificar el nombre multibyte completo antes de interpretar separadores. Solo los hosts de tipo DOS sin indicador de ruta portable convierten barras inversas en barras normales. Las barras inversas literales portables o Unix siguen siendo sospechosas. Los directorios explícitos reciben barra final; nombres sospechosos o no resueltos, métodos desconocidos y perfiles de objetos no admitidos siguen visibles sin autoridad sobre el contenido. Los nombres decodificados se limitan a 4 096 bytes y las asignaciones retenidas de visualización/decodificación a 16 MiB, o límites menores del llamante. No se restauran atributos.

La apertura de contenido vuelve a proyectar y comparar el conjunto completo de entradas congeladas antes de entregarlo. Cada registro seleccionado debe coincidir exactamente con su fila física; las selecciones no admitidas y cualquier fallo de decodificación invalidan la instancia de contenido. Comprobaciones de identidad del host, verificación previa, preparación privada, cancelación y publicación final conservan la semántica común del entorno. Las comprobaciones `supported_content` decodifican por completo todos los miembros normales admitidos y verifican tamaños exactos y CRC-32 obligatorio. Las filas no admitidas impiden el éxito completo. El texto antiguo se entrega como bytes del archivo cubiertos por CRC, sin conversión de saltos de línea del host.

<a name="xdzip-sources-and-reference-boundary"></a>

## Fuentes y límite de referencias

El [proyecto ARJ de código abierto](https://arj.sourceforge.net/) proporciona [fuente 3.10.22](https://arj.sourceforge.net/files/arj-3.10.22.tar.gz), SHA-256 `589e4c9bccc8669e7b6d8d6fcd64e01f6a2c21fe10aad56a83304ecc3b96a7db`, y su [paquete Win32 3.10](https://arj.sourceforge.net/files/arjw_310.exe), SHA-256 `e5cc5d8f82c5f08246e1c0a8c5be77e5f796c2dd30f987e6bbb27a2c1df1e371`. La fuente incluye el texto GNU GPL versión 2. Las observaciones limitadas del formato usan `defines.h`, `decode.c` y `arj_arcv.c`; la referencia nativa permanece sin modificar y bajo sus condiciones originales, solo en build/toolchain.

La comparación independiente usa [7-Zip](https://www.7-zip.org/) oficial, con observaciones acotadas de su [manejador ARJ](https://github.com/ip7z/7zip/blob/main/CPP/7zip/Archive/ArjHandler.cpp), blob Git `da890f279d35c00bc61d332ce55f4d96ecd6d373`. No se afirma una sala limpia estricta. La implementación Rust reutiliza primitivas propias de bits, prefijos y LZ y expresa directamente los campos unarios rápidos. Las fuentes de referencia, binarios y motores de compresión no entran en las dependencias del producto. Las identidades exactas de componentes y entradas propias se registran con las [muestras](../../../tests/fixtures/arj/README.md).

<a name="xdzip-static-methods-13"></a>

## Métodos estáticos 1–3

Los métodos 1–3 comparten el modelo Huffman/LZ estático, aunque los codificadores nativos pueden elegir búsquedas distintas. Sus recuentos de comandos de bloque de 16 bits, alfabeto de longitudes de 19 símbolos, alfabeto literal/longitud de 510 símbolos, alfabeto de posición de 17 símbolos y recuento de posición de cinco bits reutilizan la implementación acotada de prefijos LZH mediante un perfil ARJ explícito. No adquieren los campos de árbol antiguos de LH3 ni un identificador público de método LHA. Los recuentos de bloque cero no se admiten.

Los símbolos literales son 0–255; los demás dan longitudes `symbol - 253`, de 3 a 256. El símbolo de posición cero da desplazamiento cero; en otro caso el desplazamiento es `2^(symbol - 1)` más esa cantidad de bits bajos. La distancia real es desplazamiento más uno, como máximo 26624, y debe referirse a bytes ya emitidos. Un anillo de 32 KiB implementa el historial lógico acotado; su asignación mayor no amplía la admisión. Los espacios iniciales de LHA permanecen separados y no pueden inicializar la salida ARJ. El retorno circular y las copias solapadas mantienen semántica secuencial por byte. Árboles canónicos completos, campos de símbolo único, secuencias de ceros y profundidad de código de 16 bits conservan la validación y contabilización del prefijo compartido.

<a name="xdzip-fast-method-4"></a>

## Método rápido 4

El flujo usa primero el bit más significativo. Un campo de longitud cuenta de 0 a 7 bits iniciales a uno, con parada en cero salvo en el máximo. Leer esa cantidad de bits bajos y sumar `2^count - 1`. Cero indica un literal de ocho bits; un valor distinto de cero indica una copia del valor más dos, de 3 a 256 bytes. El puntero cuenta de forma similar de 0 a 4 bits iniciales a uno, con ancho bajo `9 + count` y base `512 * (2^count - 1)`. Desplazamiento más uno da distancia máxima 15872. Cada referencia debe apuntar a historial ya emitido, usando un anillo de 16 KiB con solapamiento y retorno circular; no se permite reparar historial inicial.

<a name="xdzip-limits-and-failures"></a>

## Límites y fallos

Los perfiles estáticos admiten 32 KiB de historial más 64 KiB para entrada/tablas/espacio temporal. La decodificación rápida admite 16 KiB de historial más 32 KiB para entrada/estado/espacio temporal. Tamaños comprimido y decodificado tienen cada uno un límite estricto de 24 GiB, el trabajo de 96 GiB; los presupuestos menores del llamante prevalecen. Las lecturas de salida son de hasta 8192 bytes. Operaciones de prefijo, trabajo de historial y copias independientes de la fuente conservan puntos de sondeo cooperativo. `work_remaining()` admite el registro acumulado de trabajo real del contenedor.

No se pueden superar las longitudes declaradas. Los bytes comprimidos lógicos deben terminar exactamente; se ignora el relleno final menor de un byte, bytes completos adicionales o entrada ausente fallan y no se inventan bytes cero EOF. Solo se reintenta `Interrupted` sin carga adicional. Los errores tipados conservan su primera carga y terminan; cualquier error de fuente, datos malformados, cuota o cancelación invalida el lector. Los lectores brutos no tienen autoridad sobre rutas, credenciales, ejecución, publicación en el sistema de archivos ni CRC.

<a name="xdzip-bounded-physical-containers"></a>

## Contenedores físicos acotados

`inspect` valida un archivo en desplazamiento cero, anchura de primera cabecera principal 30/34, anchuras de archivo 30/34/46 y bloques base completos de hasta 2600 bytes. Nombres y comentarios terminan exactamente, el campo de posición del nombre no autoriza truncar y todos los CRC-32 de bloques base y extendidos deben pasar. Cada registro admite hasta 256 bloques extendidos con longitudes de 16 bits. Los atributos opacos de archivo se validan e incluyen en la identidad de metadatos, sin aplicarlos ni ejecutarlos. El terminador exacto `60 ea 00 00` debe terminar la fuente física; no se infieren prefijo SFX, relleno ni cola de recuperación.

Lecturas de metadatos se limitan a 64 MiB, entradas físicas a 100000, nombres brutos a 4096 bytes cada uno y 16 MiB en total, conservando límites menores del llamante. Ningún fallo posterior publica un índice parcial. SHA-256 cubre todos los metadatos físicos y el terminador, incluidos comentarios/modificadores/atributos ignorados; `Reader::open` vuelve a examinar y compara esto, cada registro y la longitud física antes de producir salida. Los errores de fuente siguen siendo terminales y no se convierten en conjeturas de datos malformados. Las marcas de tiempo de host Unix usan solo el calendario opcional del host; las proyecciones ausentes son cero y los errores de llamada terminan.

El perfil de miembro admite versiones requeridas hasta 11, hosts DOS/Unix/OS2/Win95/WinNT, métodos 0–4 binarios o de texto antiguo sin cifrar y directorios Store vacíos. Métodos no admitidos, versiones futuras, tipos especiales, modos especiales Unix y campos de cifrado/continuación/capítulo permanecen explícitos y no pueden decodificarse. No se admiten perfiles principales de volumen/protección/seguridad/doble nombre/cifrado/capítulo ni estructuras de archivo de doble nombre. Modos y atributos nunca autorizan permisos o creación de archivos especiales. El texto antiguo se entrega como bytes cubiertos por tamaño y CRC del archivo, igual que 7-Zip; ARJ Win32 nativo amplía LF a CRLF para el perfil de texto. No se hace conversión de saltos de línea dependiente del host.

Un lector conserva 24 GiB de entrada real acumulada, incluido el nuevo examen de metadatos, 24 GiB de salida decodificada y 96 GiB de trabajo. El trabajo por pasos de bits CRC de la salida declarada se reserva antes de prestar el resto al decodificador con estado. Los miembros correctos contabilizan exactamente esa reserva más el trabajo real de construcción/modelo/Store; un fallo termina sin devolución. La contabilidad del sumidero del llamante incluye la construcción y no comienza después de inicializar. Todas las lecturas siguen limitadas a 8192 bytes, se comprueban instantáneas e índices físicos y cualquier fallo de contenido/CRC/sumidero/cancelación/cuota invalida la instancia.

<a name="xdzip-required-container-and-host-admission"></a>

## Admisión requerida del contenedor y el host

Cabeceras principales y de archivo ARJ, sus estructuras acotadas de primera cabecera, CRC-32, cadenas de cabeceras extendidas, rangos comprimidos exactos y marcador final de longitud cero deben validarse antes de publicar un índice completo. Los ID físicos conservan duplicados. Nombres, páginas de código, separadores, directorios, tipos de texto antiguo o binario, objetos especiales y cada método desconocido requieren proyección y admisión de contenido explícitas; un marcador reconocido no lo autoriza.

Cada CRC de archivo declarado debe pasar tras la decodificación completa. Las cabeceras y campos opcionales de seguridad no implican autenticidad criptográfica. Contraseñas, métodos cifrados, volúmenes, entradas continuadas, capítulos, colas de recuperación/seguridad y envolturas autoextraíbles requieren perfiles y pruebas separados. Ningún script, comando automático, atributo externo, ruta vecina ni indicador ejecutable concede autoridad. Congelación de fuente, extracción seleccionada, presupuestos acumulados de entrada/trabajo/salida, preparación privada, cancelación, protección de archivos antiguos y publicación final siguen el [contrato de extracción](zip-safe-extraction.md).

El generador de muestras crea solo entradas binarias propias, registra métodos solicitados y reales y lee independientemente la salida completa mediante ARJ nativo y 7-Zip. Una alternativa nativa Store sigue siendo Store. Solo normaliza marcas de tiempo sin cifrar, modificador no usado derivado del tiempo y CRC de cabecera correspondientes; bytes comprimidos y CRC de contenido no se modifican. Las entradas nativas vacías producen miembros Store y no prueban un flujo comprimido vacío codificado. La admisión del producto también requiere automatización real de entorno/CLI/Qt y de candidatos/diseños aplicables según el [alcance de QA](../../../developers/qa/README.md).
