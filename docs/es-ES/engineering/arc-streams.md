<a name="xdzip-arc-containers-and-compressed-stream-profiles"></a>

# Contenedores ARC y perfiles de flujos comprimidos

[English](../../engineering/arc-streams.md) | [简体中文](../../zh-CN/engineering/arc-streams.md) | [繁體中文（台灣）](../../zh-TW/engineering/arc-streams.md) | [日本語](../../ja-JP/engineering/arc-streams.md) | [한국어](../../ko-KR/engineering/arc-streams.md) | [Français](../../fr-FR/engineering/arc-streams.md) | [Deutsch](../../de-DE/engineering/arc-streams.md) | Español | [Português (Brasil)](../../pt-BR/engineering/arc-streams.md) | [Русский](../../ru-RU/engineering/arc-streams.md)

`ArcReader` implementa la decodificación bruta acotada de métodos ARC 3–9. Los métodos 1 y 2 son perfiles Store del contenedor. `xdzip-codec-arc` y `ArcCodec` conectan la estructura completa y CRC-16 obligatorio al entorno, CLI y navegación, comprobación y extracción de escritorio. El [registro de formatos](../../../specs/engineering/archive-formats.json) declara este alcance de solo lectura; creación, modificación, contraseñas y volúmenes no están disponibles.

<a name="xdzip-container-and-host-admission"></a>

## Admisión del contenedor y el host

El perfil admitido usa la estructura ARC 5.x normal; no se reclaman metadatos adicionales ARC 6.x ni extensiones Spark. Cada miembro comienza con `0x1a` y su byte de método original. El método 1 usa 25 bytes; los demás métodos distintos de cero usan la cabecera normal de 29 bytes. El campo de nombre de trece bytes debe contener un nombre no vacío terminado en NUL. Todos los enteros son little endian: longitud comprimida en 15, fecha/hora DOS en 19/21, CRC-16 obligatorio en 23 y longitud decodificada normal en 25. El Store antiguo hereda la longitud comprimida. Ambos métodos Store requieren longitudes iguales. Los rangos comprimidos deben caber, el siguiente marcador es obligatorio y el terminador exacto de dos bytes `1a00` debe coincidir con EOF físico. No se admiten resincronización, búsqueda de prefijo, relleno ni archivos vecinos.

El contenedor conserva todos los duplicados físicos y métodos desconocidos. Las lecturas de metadatos tienen límite estricto de 64 MiB, hasta 100000 registros, nombres brutos acotados por el llamante y límite de entrada física de 24 GiB. SHA-256 vincula cada byte de cabecera y el terminador, incluido el relleno de nombre ignorado, al reabrir un `Index` exacto. Los hashes de cabecera no autentican el contenido. Cada lector tiene registros acumulados de 24 GiB de entrada real y bytes decodificados y 96 GiB de trabajo, reserva trabajo por bytes de CRC por tabla antes de decodificar y termina ante cualquier fallo sin devolución de presupuesto ni publicación de índice parcial.

El host conserva la prioridad de contenedores fuertes y ZIP/SFX y valida completamente propuestas ARC débiles bajo el registro compartido de sondeos. Los nombres se decodifican como campos completos antes de convertir barras inversas DOS, usando la decisión compartida y acotada de codificación del archivo. Los bytes originales e ID de método permanecen disponibles. Los nombres sospechosos/no resueltos y métodos desconocidos siguen visibles sin autoridad sobre el contenido. ARC no tiene perfiles admitidos de directorios, enlaces, restauración de modo ni objetos externos. El XOR antiguo no tiene indicador de cifrado; la validación de flujo, tamaño y CRC sigue siendo obligatoria sin adivinar contraseñas ni recurrir a alternativas. Este perfil no reclama registro MIME dedicado en Linux.

La apertura de contenido vuelve a proyectar y comparar entradas congeladas; los ID seleccionados identifican filas físicas y deben coincidir exactamente. Siguen siendo determinantes la comprobación previa, identidad congelada de la fuente, preparación privada, cancelación, protección de archivos antiguos y publicación final. La comprobación completa `supported_content` decodifica realmente cada miembro normal admitido, valida tamaño exacto y CRC-16 y falla ante filas no admitidas. Una selección física compatible puede extraerse aunque tenga un vecino no admitido.

<a name="xdzip-sources-and-references"></a>

## Fuentes y referencias

La adaptación Unix original de ARC mantenida por Howard Chu y Anibal Monsalve Salazar está fijada al [commit `70f0b93516d0ee222db5cf1241aad0db1b721c0d`](https://github.com/ani6al/arc/tree/70f0b93516d0ee222db5cf1241aad0db1b721c0d). Su `Arcinfo` recoge la descripción de estructura ARC 5.12 de Robert Freed. Las observaciones limitadas del formato usan `arcunp.c`, `arcusq.c` y `arclzw.c`. Los archivos `LICENSE` y `COPYING` de los autores recogen la licencia GPL. Los algoritmos de referencia se compilan sin cambios solo en build/toolchain con un adaptador de prueba propio. No se compilan ni despliegan con el producto.

La comparación independiente usa nomarch 1.4 de Russell Marks, GPL versión 2 o posterior. Su paquete fuente original está en el [depósito fuente de Debian](https://deb.debian.org/debian/pool/main/n/nomarch/), SHA-256 `fe20da34e0d3ba0cf6388701f44ac22224cf65130ddbc5fcbc27bc4949a6e1ad`, 22411 bytes, coincidente con el descriptor de fuente del mantenedor. No se afirma una sala limpia estricta. El código del producto expresa los campos del formato con primitivas propias de bits, entrada, trabajo y asignación. El [registro de muestras propias](../../../tests/fixtures/arc/README.md) distingue salida del codificador nativo, entradas de formato elaboradas, comparaciones independientes de bytes completos y límites de las referencias.

<a name="xdzip-method-scope"></a>

## Alcance de los métodos

| Método | Perfil del formato | Etapa posterior |
| --- | --- | --- |
| 1 | Estructura obsoleta de 25 bytes con longitud original implícita | Store |
| 2 | Estructura normal de 29 bytes | Store |
| 3 | Bytes con secuencias DLE `0x90` y DLE escapado | Ninguna |
| 4 | Árbol squeeze de hijos con signo explícitos, códigos de bit menos significativo primero y símbolo EOF 256 | RLE del método 3 |
| 5 | Códigos fijos de 12 bits con bit más significativo primero, diccionario hash original | Ninguna |
| 6 | El mismo diccionario hash original | RLE del método 3 |
| 7 | Códigos fijos de 12 bits, hash mejorado | RLE del método 3 |
| 8 | Byte inicial de ancho máximo 12, LZW agrupado de 9–12 bits con bit menos significativo primero | RLE del método 3 |
| 9 | Sin cabecera de ancho, LZW agrupado de 9–13 bits con bit menos significativo primero | Ninguna |

Los métodos desconocidos, extensiones Spark, contraseñas y estructura SFX requieren admisión separada del contenedor. Un nombre común de algoritmo no autoriza el orden de bits, diccionario, alineación, reinicio o etapa RLE de otro método.

<a name="xdzip-rle-and-squeeze-trees"></a>

## RLE y árboles squeeze

Un byte normal se emite y se convierte en historial de repetición. DLE seguido de cero emite un DLE literal sin cambiar el historial. Un recuento distinto de cero emite `count - 1` copias del byte normal anterior; un recuento sin historial es inválido. El escape debe estar completo y el tamaño decodificado exacto limita cada repetición. El estado RLE sobrevive a cadenas del diccionario y bloques de salida del llamante.

Squeeze comienza con un número de nodos little endian de hasta 256, seguido de dos hijos con signo de 16 bits por nodo. Los hijos no negativos designan nodos internos; los valores negativos codifican el símbolo `-child - 1`, de 0 a 256. La raíz es cero; la forma antigua sin nodos tiene hijos EOF implícitos. El grafo completo debe ser un árbol con raíz sin ciclos, hijos internos compartidos, nodos internos inaccesibles ni aristas fuera de rango, y EOF debe ser alcanzable. Recorrer primero bits menos significativos hasta EOF. La entrada comprimida debe terminar en el mismo byte que ese terminador; se ignoran bits finales no usados. Decodificar hasta el tamaño declarado no sustituye la comprobación EOF.

<a name="xdzip-fixed-dictionaries"></a>

## Diccionarios fijos

El diccionario de 4096 posiciones inserta inicialmente cadenas de bytes 0–255 con predecesor `0xffff`. Los códigos identifican posiciones físicas hash. El hash antiguo usa los doce bits centrales del cuadrado de `((prefix + suffix) mod 65536) | 0x0800`. El método 7 multiplica en cambio la suma de 16 bits por 15073 y toma los doce bits bajos. Estas constantes definen identificadores históricos del formato.

Las colisiones siguen la cadena de posiciones existente, comienzan a buscar 101 posiciones después de su último elemento y luego buscan linealmente con retorno circular. Cada sondeo y expansión de prefijo está acotado y se contabiliza. Cada nueva cadena es la anterior más el primer byte de la actual. Un código aún inexistente solo se admite para la posición KwKwK exacta recién insertada. Los diccionarios llenos dejan de crecer. Los códigos de doce bits colocan primero el medio byte alto entre bytes, a diferencia de perfiles LSB posteriores. El último medio byte no usado se ignora; bytes de código ausentes o bytes completos adicionales son inválidos.

<a name="xdzip-dynamic-dictionaries"></a>

## Diccionarios dinámicos

El primer código debe ser un byte. El código 256 es CLEAR y 257 inicia las adiciones secuenciales al diccionario. El ancho crece en umbrales del diccionario hasta el máximo del método. Los códigos se agrupan de ocho; un cambio de ancho o CLEAR termina el grupo anterior, consume sus bits de alineación no usados e inicia un grupo nuevo. Los bits de alineación no están especificados. CLEAR reinicia la siguiente posición y el ancho, requiere un literal posterior y no puede tomar una cadena anterior al reinicio. No se suministran códigos EOF ni bytes sintéticos.

El crecimiento del diccionario, KwKwK, operación con ancho máximo y reinicios adaptativos son independientes de la etapa RLE. Las entradas nativas de referencia con LZW forzado establecen cada ancho y reinicios reales. Las entradas CLEAR elaboradas cubren independientemente cada una de las ocho posiciones del grupo anterior.

<a name="xdzip-budgets-and-failures"></a>

## Presupuestos y fallos

Las longitudes comprimida y decodificada tienen cada una límite estricto de 24 GiB y el trabajo de 96 GiB. Los límites menores del llamante prevalecen. La admisión de memoria privada completa es 16 KiB para método 3, 32 KiB para método 4, 64 KiB para métodos 5–7 y 96 KiB para métodos 8–9. Estos límites incluyen el búfer de entrada de 8192 bytes, diccionarios, expansión inversa, validación del grafo y estado.

Se contabilizan inicialización del diccionario, bits consumidos, aristas de árbol, sondeos hash, expansión de prefijo, entrega de pila y trabajo de salida/repetición. `work_remaining()` incluye trabajo del constructor para un registro acumulado del contenedor. El sondeo cooperativo cubre trabajo dependiente e independiente de entrada. Las lecturas de salida son de hasta 8192 bytes. Entrada corta, errores tipados del host, agotamiento de cuota, cancelación, códigos/árboles inválidos, datos finales y discrepancia de tamaño decodificado terminan la instancia; el primer error conserva su carga y lecturas posteriores no pueden reanudarse. Los lectores completados devuelven EOF estable sin gastar más trabajo.

El CRC-16 obligatorio del contenido y la estructura física completa pertenecen al contenedor ARC. El éxito bruto por sí solo no autentica contenido ni concede autoridad sobre sistema de archivos, rutas, procesos, archivos vecinos o publicación.
