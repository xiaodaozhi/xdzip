<a name="xdzip-ace-alz-and-egg-source-and-licensing-gates"></a>

# Fuentes y condiciones de licencia de ACE, ALZ y EGG

[English](../../engineering/ace-alz-egg-format-sources.md) | [简体中文](../../zh-CN/engineering/ace-alz-egg-format-sources.md) | [繁體中文](../../zh-TW/engineering/ace-alz-egg-format-sources.md) | [日本語](../../ja-JP/engineering/ace-alz-egg-format-sources.md) | [한국어](../../ko-KR/engineering/ace-alz-egg-format-sources.md) | [Français](../../fr-FR/engineering/ace-alz-egg-format-sources.md) | [Deutsch](../../de-DE/engineering/ace-alz-egg-format-sources.md) | Español | [Português (Brasil)](../../pt-BR/engineering/ace-alz-egg-format-sources.md) | [Русский](../../ru-RU/engineering/ace-alz-egg-format-sources.md)

En esta página se explica el soporte de xdzip para ACE, ALZ y EGG y las referencias técnicas públicas y limitaciones de licencia en que se basan las decisiones. Los archivos de referencia quedan en `build/toolchain/references`, fuera de las dependencias del producto.

<a name="xdzip-ace"></a>

## ACE

- Nota técnica de Marcel Lemke, *Technical information of the archiver ACE v1.2*, obtenida de `wdlkmpx/unace`, SHA-256 `F645C91348D5DCD560A19CA21DEF849D25C304EFFAACF65B03EFBB0A893E51F9`.
- Implementación independiente `droe/acefile`, commit `11ed076ae19e8228148731cfbc05f019a395b3b9`. SHA-256 de la licencia BSD-2-Clause `AD376BD9695F0E355F5C523443ABF822BB896999A3252AAA614D313CC4F1F5B5`. SHA-256 de la implementación `BC701569AD8521964C62256472AB44976AC57FB46E52B147A21018BC96348F1F`.
- La implementación declara que lee ACE 1.0 y 2.0, incluidos LZ77, DELTA, EXE, PIC, SOUND, contraseñas y volúmenes. Su licencia solo cubre la implementación en Python y declara expresamente que no otorga derechos sobre la tecnología ACE.

La documentación técnica permite definir cabeceras de bloques, CRC, entradas almacenadas sin compresión y la familia LZ77 antigua. No concede permiso comercial sobre la tecnología ACE. Las referencias de la implementación a WinAce y unace no constituyen un conjunto independiente de interoperabilidad. El producto excluye ACE de forma permanente. xdzip no lo implementará ni registrará, y no se añadirán extensiones, detectores, tipos MIME ni asociaciones de ACE.

<a name="xdzip-alz"></a>

## ALZ

- Especificación de formato `alkegi/docs`, commit `36230c34d3056604cc63519e9855a79e894496b7`. SHA-256 de la licencia CC0 `F4E7F373B9B996950337E8D41A4A2939C2D90B7725E9BAF3D5084A22717AD328`.
- SHA-256 de `alz.md` `CBD35E77AB53CFD08C0FF192FA2793438150E4935522608936B1A0942560E237`.
- El documento especifica las firmas de ALZ y de archivos locales, tamaños de ancho variable, CRC-32, nombres UTF-8/CP949, Store, Deflate sin envoltura según RFC 1951 y un flujo BZip2 modificado por ALZ. También especifica el cifrado tradicional de PKWARE y la disposición de los volúmenes divididos.

xdzip admite un conjunto limitado de ALZ de solo lectura. Los archivos normales de un único volumen pueden usar Store o Deflate sin envoltura, sin cifrado y con comprobación exacta de CRC, tamaños y terminación. Las pruebas automáticas cubren el códec, el entorno de ejecución, la CLI y las asociaciones. BZip2 modificado, ZipCrypto y los volúmenes divididos requieren muestras completas independientes y pruebas separadas de recursos y seguridad. Una licencia documental CC0 no acredita por sí sola que todos los detalles de implementación estén libres de obligaciones de patentes o marcas de terceros.

<a name="xdzip-egg"></a>

## EGG

- Documento público del formato `alkegi/docs`, mismo commit `36230c34d3056604cc63519e9855a79e894496b7`, licencia CC0 indicada arriba.
- SHA-256 de `egg.md` `4CA2CB6E8FA9A4F40C30EDE05A5E977BE113A96E1527E28A7B1ED0836D8A43A3`.
- Módulo de referencia `dterracino/UnEgg`, commit `f21e653b02f7ab467bbc16af879e68a48dabcb2b`. SHA-256 del PDF de especificación de EGG `0BB7E6F174CAFB9F6726D70B7DEE8DBF33C034717058B1B0EB2B372D9A8FBB37`.
- SHA-256 de la licencia de código de UnEGG `9B012928756D9243CC549FA0A7D7B5742ED8F69B00A2BE1C964F42EF21F1D8D6` para el paquete de código fuente y `30F2E45148D43D7A77574403807C13AA359EE9ACE841A00B6A5348AD961423F0` para el paquete del módulo.

La licencia de UnEGG exige autorización de ESTsoft para el uso comercial, restringe la redistribución a fines no comerciales y prohíbe usar el módulo para desarrollar un compresor EGG. ESTsoft se reserva todos los derechos. Por ello, el módulo solo sirve como referencia de pruebas. No se puede copiar, enlazar ni incluir como dependencia del producto.

El formato se puede implementar por separado del módulo restringido. xdzip admite una implementación independiente de Store y Deflate sin envoltura en un solo volumen, sin cifrado ni modo sólido. Las pruebas del códec, el entorno de ejecución, la CLI y las asociaciones comprueban tamaños exactos de bloque y CRC-32. BZip2 estándar y las tramas LZMA documentadas aún requieren muestras completas separadas y revisión. AZO, AES/LEA, ZipCrypto y los archivos divididos y sólidos quedan fuera del alcance actual.

<a name="xdzip-required-implementation-gates"></a>

## Requisitos para admitir una implementación

1. Guarda las referencias fuera del árbol del producto y comprueba los hashes fijados antes de usarlas.
2. Prepara muestras completas e independientes de cada método admitido. Una muestra de solo cabecera o solo Store no demuestra interoperabilidad del método.
3. Añade a la vez las entradas de contenedor, entorno de ejecución, CLI y escritorio. Los métodos desconocidos quedan visibles sin permiso para procesar datos.
4. Mantén desactivados contraseñas, volúmenes, estado sólido y objetos especiales hasta que pasen sus comprobaciones propias de fuentes, seguridad y cancelación.
5. Registra la decisión jurídica y las identidades de las muestras en la incidencia antes de modificar el registro público de capacidades.
