<a name="xdzip-bandizip-format-inventory-and-xdzip-feasibility"></a>

# Inventario de formatos de Bandizip y viabilidad en xdzip

[English](../../engineering/bandizip-format-assessment.md) | [简体中文](../../zh-CN/engineering/bandizip-format-assessment.md) | [繁體中文](../../zh-TW/engineering/bandizip-format-assessment.md) | [日本語](../../ja-JP/engineering/bandizip-format-assessment.md) | [한국어](../../ko-KR/engineering/bandizip-format-assessment.md) | [Français](../../fr-FR/engineering/bandizip-format-assessment.md) | [Deutsch](../../de-DE/engineering/bandizip-format-assessment.md) | Español | [Português (Brasil)](../../pt-BR/engineering/bandizip-format-assessment.md) | [Русский](../../ru-RU/engineering/bandizip-format-assessment.md)

Esta evaluación compara los formatos enumerados por Bandizip con el alcance de implementación de xdzip e identifica capacidades que requieren verificación de fuentes, muestras y pruebas automatizadas. El inventario procede de la página del producto Bandizip y sus páginas de ayuda de formatos, consultadas el 2026-11-15:

- La [página del producto Bandizip](https://www.bandisoft.com/bandizip/) enumera: `7Z`, `ACE`, `AES`, `ALZ`, `ARJ`, `BH`, `BIN`, `BR`, `BZ`, `BZ2`, `CAB`, `Compound(MSI)`, `DAA(1.0)`, `DEB`, `EGG`, `GZ`, `IMG`, `ISO`, `ISZ`, `LHA`, `LZ`, `LZH`, `LZMA`, `PMA`, `RAR`, `RAR5`, `SFX(EXE)`, `TAR`, `TBZ/TBZ2`, `TGZ`, `TLZ`, `TXZ`, `UDF`, `WIM`, `XPI`, `XZ`, `Z`, `ZIP`, `ZIPX`, `ZPAQ`, `PEA`, `UU`, `UUE`, `XXE`, `ASAR`, `ZSTD`, `NSIS` y `LZ4`.
- La misma página indica que Bandizip crea `ZIP`, `7Z`, `ZIPX`, `EXE (SFX)`, `TAR`, `TGZ`, `LZH`, `ISO`, `GZ` y `XZ`; la lista más larga es principalmente de extracción.
- La [ayuda de Bandizip sobre ALZ y EGG](https://www.bandisoft.com/bandizip/help/how-to-open-alz-and-egg-files/) describe Deflate, Deflate ofuscado, BZip2 y ZipCrypto de ALZ, y Deflate, BZip2, LZMA, AZO, cifrado AES/LEA, bloques sólidos y archivos divididos de EGG.
- La [ayuda de Bandizip sobre ZPAQ](https://www.bandisoft.com/bandizip/help/what-is-a-zpaq/) describe ZPAQ como un formato de archivo con diario propio y enumera los métodos de compresión que Bandizip puede manejar.
- La [guía de fuentes de imágenes de disco y otros formatos de archivo](disk-image-archive-format-sources.md) registra las comprobaciones de fuentes públicas, licencias, corpus y sala limpia de DAA, ISZ, ZPAQ, PEA, BOND y J2J. No está confirmado que BOND y J2J sean entradas oficiales actuales de archivos Bandizip y solo permanecen como pistas históricas.

<a name="xdzip-current-xdzip-disposition"></a>

## Situación actual en xdzip

La tabla usa estos significados. **Admitido** indica que el registro actual, runtime, CLI y recorrido de escritorio tienen evidencia automatizada para el perfil indicado. **Alias/perfil** indica que la extensión se admite mediante un contenedor existente solo después de validar el contenido. **Candidato** significa que el sufijo puede ser una entrada futura útil, pero no se declara ningún descodificador. **Bloqueado** significa que el formato no puede admitirse responsablemente hasta disponer de una especificación pública del formato, comprobable de manera independiente, y una implementación compatible o algoritmo propio. **Excluido** significa que el proyecto ha decidido explícitamente no admitir el formato; no recibe entrada de registro ni detector salvo que se revise esa decisión de producto.

| Nombre de Bandizip | Estado en xdzip | Alcance actual defendible o motivo |
| --- | --- | --- |
| 7Z | Admitido | Backend 7z independiente en Rust, incluidos los perfiles declarados de contraseña, sólido y volúmenes. |
| ACE | Excluido | Decisión de producto: abandonar ACE permanentemente; no se añadirán descodificador, detector, sufijo, MIME ni asociación. |
| AES | Admitido | AES Crypt v0–v3, perfil de contraseña y HMAC; es la envoltura `.aes`, no archivos cifrados arbitrarios. |
| ALZ | Admitido | Store y Deflate sin procesar ordinarios de un volumen y solo lectura, con tamaños exactos y CRC-32; BZip2 modificado, ZipCrypto y volúmenes divididos quedan fuera. |
| ARJ | Admitido | Store y métodos 1–4 de solo lectura con encabezados completos, tamaños y CRC-32 de miembros. |
| BH | Admitido | Miembros ordinarios `BH 05 07` de solo lectura, método 8 Deflate sin procesar, tamaños exactos y CRC-32. Store, cifrado, volúmenes y otros métodos se excluyen. |
| BIN | Candidato | Extensión ambigua entre disco, ejecutables y datos. Detectar solo un formato interno validado; nunca registrar BIN genérico como descodificador de archivos. |
| BR | Admitido | Flujo Brotli acotado y perfiles TAR.BR. |
| BZ / BZ2 | Admitido | Flujos BZip2 de archivo único y alias TAR.BZ2 declarados. |
| CAB | Admitido | Backend CAB independiente en Rust con Store, MSZip, Quantum, LZX y conjuntos de gabinetes autorizados. |
| Compound(MSI) | Admitido | Estructura CFB de solo lectura, nombres de almacenamiento/flujo y copia acotada de flujos sin procesar para contenido `.msi` validado. Tablas MSI, interpretación de CAB integrado, transformaciones, acciones personalizadas e instalación quedan fuera. |
| DAA(1.0) | Excluido | No compatible permanentemente por decisión de producto; no se admite detector, códec, entrada de registro, lector ni escritor. |
| DEB | Admitido | Envoltura `ar` común, vistas control/data y miembros comprimidos admitidos. |
| EGG | Admitido | Store y Deflate sin procesar ordinarios de un volumen, sin cifrar, no sólidos y de solo lectura, con tamaños exactos de bloque y CRC-32; AZO, BZip2, LZMA, cifrado, división y otros perfiles quedan fuera. |
| GZ | Admitido | Miembros GZIP, incluidos flujos concatenados y TAR.GZ. |
| IMG | Candidato | Sufijo ambiguo de imagen de disco. La admisión ISO/UDF por contenido no implica compatibilidad genérica con IMG. |
| ISO | Admitido | Lectura ISO9660/Joliet/Rock Ridge y creación de imágenes de datos ordinarias ISO9660 + Joliet. |
| ISZ | Excluido | Decisión de producto: abandonar ISZ; no se añadirán detector, registro ni tratamiento de claves. |
| LHA / LZH | Admitido | Niveles LHA 0–3, Store, LH1–LH7, LZS/LZ5 y creación LH5. |
| LZ | Admitido | Lzip v1 y TAR.LZ. |
| LZMA | Admitido | LZMA Alone y TAR.LZMA. |
| PMA | Admitido | Perfiles PM0/PM1/PM2 de solo lectura con estructura CP/M y comprobaciones CRC. |
| RAR / RAR5 | Admitido | Backend RAR independiente en Rust, incluidos formatos antiguos, RAR5, extensiones RAR7, contraseñas y perfiles sólidos y divididos dentro del alcance declarado. |
| SFX(EXE) | Alias/perfil | Los prefijos ZIP SFX se validan con el lector ZIP. Nunca se autoriza extracción ni ejecución genérica de ejecutables. La creación ZIP Store/Deflate propia para Windows x64 está disponible mediante el perfil explícito `.sfx.exe`, sin ejecución automática. |
| TAR | Admitido | TAR y perfiles filtrados TAR.GZ/TAR.BZ2/TAR.LZMA/TAR.LZ/TAR.XZ/TAR.Z/TAR.BR/TAR.LZ4/TAR.ZST enumerados en el registro. |
| TBZ / TBZ2 | Alias/perfil | Alias TAR filtrados por BZip2. |
| TGZ | Alias/perfil | Alias TAR filtrado por GZIP. |
| TLZ | Alias/perfil | Alias TAR filtrado por LZMA. |
| TXZ | Alias/perfil | Alias TAR filtrado por XZ. |
| UDF | Admitido | Imágenes UDF de solo lectura, incluidas las reglas de espacio de nombres de imágenes híbridas. |
| WIM | Excluido | Decisión de producto: no abordar el lector, modelo de recursos y corpus específicos de imágenes que requiere WIM. |
| XPI | Alias/perfil | Contenido ZIP con sufijo XPI; prevalece la validación del contenido. |
| XZ | Admitido | XZ y TAR.XZ con LZMA2, comprobaciones declaradas y filtros admitidos. |
| Z | Admitido | LZW de Unix compress y alias TAR.Z/TAZ. |
| ZIP | Admitido | ZIP/ZIP64, AES-256 y perfiles compatibles de volúmenes, métodos y reescritura. |
| ZIPX | Alias/perfil | ZIPX se admite para métodos ZIP validados como XZ; no es una implementación de contenedor independiente. Actualmente no se ofrece creación ZIPX. |
| ZPAQ | Excluido | Decisión de producto: no implementar la máquina virtual ZPAQL necesaria para este formato sólido con diario. |
| PEA | Admitido | Perfil de solo lectura PEA 1.x de un volumen, UTF-8, sin cifrar, con archivos y directorios almacenados PCOMPRESS0; comprobaciones, cifrado, compresión, volúmenes, creación y modificación quedan fuera. |
| UU / UUE | Admitido | Envolturas UU/UUE acotadas con validación de miembros físicos y líneas. |
| XXE | Admitido | Perfil de envoltura XXE acotado. |
| ASAR | Admitido | Metadatos ordenados, intervalos almacenados y comprobaciones SHA-256 declaradas. |
| ZSTD | Admitido | Tramas Zstandard estándar y perfiles TAR.ZST/TZST. |
| NSIS | Admitido | Solo metadatos estáticos `firstheader`. La detección por contenido valida un overlay PE acotado; `.exe` no es entrada de arrastrar y soltar ni de asociación shell, y no se permite ejecutar PE, scripts, complementos ni cargas útiles. |
| LZ4 | Admitido | Tramas LZ4 estándar y TAR.LZ4. |

Por tanto, el inventario no justifica añadir manejadores genéricos para `BIN`, `IMG` o `SFX(EXE)`. Estos nombres pueden describir datos ejecutables, de disco o instalador, y un sufijo por sí solo no establece capacidad de archivo. NSIS tiene un lector más limitado de metadatos estáticos validados por contenido; nunca concede autoridad de ejecución ni sobre la carga útil y su entrada `.exe` se oculta en arrastrar y soltar y asociaciones shell. Compound(MSI) dispone ahora de un lector CFB validado por contenido; el sufijo `.msi` sigue sin conceder autoridad de ejecución.

Véanse las [fuentes de imágenes de disco y formatos de archivo](disk-image-archive-format-sources.md) para DAA, ISZ, ZPAQ, PEA, BOND y J2J. PEA está disponible en el perfil conservador de solo lectura documentado. ACE, DAA, ISZ y ZPAQ están excluidos por decisión de producto. BOND y J2J se excluyen porque no se confirma su identidad como formatos de archivo Bandizip.

<a name="xdzip-delivery-plan"></a>

## Plan de entrega

1. **Control de inventario y registro.** Mantener una entrada legible por máquina por perfil admitido en `specs/engineering/archive-formats.json`. Generar los consumidores de Rust, C++, JavaScript y asociaciones y ejecutar ambas comprobaciones del registro. Cada formato nuevo recibe contrato y párrafo de aceptación correspondientes.
2. **Contrato del formato antes del código.** Fijar revisiones de fuentes públicas y SHA-256; documentar campos de encabezado, codificación de nombres, terminación física, métodos, sumas de comprobación, identidad de duplicados y límites de recursos. Una herramienta de referencia solo es evidencia si produce muestras propias completas; un fallo o una única muestra Store se registra como limitación.
3. **Pruebas de algoritmo sin procesar.** Implementar el lector acotado independientemente de Qt. Cubrir cargas vacías y límite, cada método admitido, presupuestos exactos de entrada/salida/trabajo/memoria, cancelación, E/S tipada, cambios de origen, truncamiento, bytes posteriores, CRC y estado terminal tras error. Los métodos desconocidos siguen visibles sin autoridad sobre la carga útil.
4. **Admisión de contenedor y producto.** Añadir el crate de códec, detección débil/fuerte del runtime, list/test/extract de CLI, etiquetas de escritorio y muestras de asociación. Verificar metadatos fijados y selección de duplicados físicos y ejecutar las comprobaciones automáticas pertinentes de Qt y diseño. Esta parte de investigación y formatos no necesita aceptación visual manual.
5. **Cola de prioridades.** Están admitidos el subconjunto conservador ALZ Store/Deflate sin procesar, EGG sin cifrar y no sólido Store/Deflate sin procesar, metadatos CFB/Compound(MSI), perfil conservador almacenado PEA y metadatos estáticos NSIS `firstheader`. ACE, DAA, WIM, ISZ, ZPAQ, BOND y J2J quedan permanentemente excluidos por decisión de producto. La creación ZIPX sigue excluida de los escritores actuales. El [escritor SFX propio](sfx-creation.md) está implementado por separado y no se deriva de admitir el alias en lectura.
6. **Límite de publicación.** Mantener los formatos candidatos fuera de la tabla pública de capacidades hasta completar la evidencia de códec, runtime, CLI y escritorio. Registrar métodos no compatibles y limitaciones de herramientas de referencia en la incidencia, sin recurrir silenciosamente a Store u otro contenedor.

El subconjunto BH actual sigue este plan: solo lectura, sin ruta de contraseña ni volúmenes, con lector Deflate acotado independiente, CRC-32 obligatorio y terminación física exacta. El subconjunto Compound(MSI) sigue el mismo límite: validación CFB FAT/DIFAT, árbol de directorios, mini-flujo y ciclos independiente de Qt, y copia de flujos sin procesar solo tras coincidir la instantánea inmutable de metadatos. El subconjunto NSIS se detiene en el registro fijo de metadatos firstheader y devuelve no compatible para acceso a la carga útil. No se ejecuta ningún comportamiento de instalador MSI o NSIS.
