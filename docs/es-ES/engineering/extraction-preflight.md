<a name="xdzip-extraction-preflight-metadata-phase"></a>

# Comprobación previa de extracción: fase de metadatos

[English](../../engineering/extraction-preflight.md) | [简体中文](../../zh-CN/engineering/extraction-preflight.md) | [繁體中文（台灣）](../../zh-TW/engineering/extraction-preflight.md) | [日本語](../../ja-JP/engineering/extraction-preflight.md) | [한국어](../../ko-KR/engineering/extraction-preflight.md) | [Français](../../fr-FR/engineering/extraction-preflight.md) | [Deutsch](../../de-DE/engineering/extraction-preflight.md) | Español | [Português (Brasil)](../../pt-BR/engineering/extraction-preflight.md) | [Русский](../../ru-RU/engineering/extraction-preflight.md)

Este módulo proporciona reglas de comprobación previa de metadatos. No tiene capacidad de autorizar destinos, decodificar, escribir, preparar temporalmente ni confirmar resultados.

<a name="xdzip-implementation-and-boundaries"></a>

## Implementación y límites

`xdzip-extract-plan` depende solo de interfaces de dominio y metadatos, sin Qt, analizadores de formato ni dependencias de terceros adicionales. La entrada es una instantánea de sesión, un conjunto de EntryId físicos y cancelación; la salida es solo un informe acotado, no rutas utilizables para escribir ni un indicador de aprobación de seguridad. `xdzip-runtime` conserva la instantánea; Qt lo llama mediante el puente del hilo de trabajo.

La selección debe ser no vacía, está limitada a 128 ID físicos y todos deben pertenecer a la instantánea actual. Al seleccionar un directorio se expanden todos sus descendientes según los límites de barras de los nombres originales, independientemente de búsqueda, paginación o alcance del directorio actual. Si se seleccionan directorio e hijo, el hijo se cuenta una vez. Los registros físicos con el mismo nombre no se fusionan por ruta. Las estadísticas de directorios cuentan solo registros explícitos y no crean objetos de directorio sintéticos.

La comprobación previa se ejecuta mediante el servicio de comandos y eventos con admisión acotada y cancelación independiente. El entorno verifica la identidad de la fuente abierta antes y después de la inspección. Para ZIP con índice en disco, carga del índice los registros físicos seleccionados y los descendientes necesarios. Un cambio de selección, consulta o sesión borra el informe. Los resultados son estimaciones de los metadatos registrados. La comprobación previa nunca decodifica contenido de miembros, reserva rutas de destino ni autoriza escrituras.

<a name="xdzip-rules-performed"></a>

## Reglas aplicadas

- Los nombres deben poder resolverse como UTF-8, con máximo 4 KiB, 64 componentes y 255 bytes por componente; los nombres originales no se reparan ni reescriben.
- Rechazar rutas absolutas, componentes vacíos, `.`, `..`, barras inversas, dos puntos, caracteres de control y control bidireccional, caracteres inválidos de Windows y espacios o puntos finales en componentes.
- Rechazar nombres comunes de dispositivos Windows, incluidas formas con extensión y dígitos COM/LPT en superíndice, aplicando de forma conservadora un subconjunto portable multiplataforma.
- Informar las advertencias de metadatos existentes y usar los indicadores de capacidad de contenido y contraseña del registro. Un registro cifrado puede informar de un requisito de contraseña compatible o de cifrado no compatible. Un registro de tamaño conocido con contenido no compatible también informa un problema de método. La comprobación usa metadatos y capacidades proyectados por el lector; no descifra, autentica ni decodifica el contenido.
- Comprobar nombres exactamente duplicados, colisiones por comparación Unicode en minúsculas, alias de directorios padre con diferencias de mayúsculas y conflictos archivo/directorio en ambos órdenes de inserción. Un directorio padre implícito puede completarse con uno explícito del mismo nombre; los registros explícitos duplicados de directorio siguen siendo conflictos.
- Informar tamaños declarados por archivo superiores a 8 GiB, totales superiores a 20 GiB y desbordamiento de enteros. Para registros ZIP, una relación de compresión declarada superior a 1000:1 solicita revisión. Los tamaños de directorios no cuentan como bytes de salida.

La comparación Unicode en minúsculas no es plegado completo de mayúsculas ni equivalencia NFC/NFD del sistema de archivos. Cualquier componente no ASCII añade un aviso de que todavía hay que comprobar equivalencias y colisiones en el sistema de archivos de destino. Incluso un informe sin hallazgos no autoriza la salida.

<a name="xdzip-resources-and-failures"></a>

## Recursos y fallos

Comprobar como máximo 100 000 registros de la instantánea; los nombres seleccionados o expandidos suman como máximo 4 MiB. Las claves de comparación y grafías originales almacenadas en el índice de colisiones suman como máximo 8 MiB, sin contar la sobrecarga del contenedor. La profundidad de recorrido está acotada y se comprueba la cancelación en cada registro externo. Los informes cuentan todos los hallazgos, pero como máximo 64 problemas representativos cruzan el puente, con solo ID físicos y razones estables, sin texto arbitrario de error del archivo comprimido.

Una selección vacía, ID ajenos, límites del índice o cancelación devuelven errores, no informes parcialmente satisfactorios. Los totales desbordados muestran «No disponible» en lugar de truncarse a cero. Los hallazgos básicos solo se informan; no sobrescriben, renombran, ignoran ni omiten automáticamente.

Los tamaños sin comprimir desconocidos informan la razón estable `unknown_size`. `declared_bytes=None` significa total desconocido y es distinto de `Some(0)` para un archivo vacío conocido. Los valores desconocidos no inventan hallazgos por tamaño total ni por método desconocido; un desbordamiento real sigue produciendo un hallazgo de límite total. Las tareas de salida rechazan por separado registros de tamaño desconocido antes de decodificar, preparar temporalmente o interactuar con contraseñas.

<a name="xdzip-desktop-entry-point"></a>

## Entrada de escritorio

Selecciona entradas y pulsa «Comprobar los metadatos antes de extraer…». Los resultados usan una `OperationWindow` superior independiente y no modal, con la ventana principal como propietaria transitoria y la barra de título nativa conservada; la aplicación no dibuja los botones de ventana de macOS. Cerrar resultados solo cierra esa ventana. El cierre de la ventana principal usa cancelación cooperativa y no espera a los trabajadores en el hilo gráfico.

<a name="xdzip-boundary-with-actual-extraction"></a>

## Límite con la extracción real

La [extracción segura real](zip-safe-extraction.md) implementa independientemente el cómputo de salida, la verificación CRC y estructural, autorización de destino, nuevas comprobaciones de rutas por plataforma, conflictos, preparación temporal y confirmaciones de archivos. No está limitada a las 128 selecciones de la comprobación previa y admite planes de 100 000 registros físicos y selecciones de directorios virtuales. Un informe previo no se reutiliza como permiso de escritura. Al iniciar una tarea deben obtenerse de nuevo la autorización de destino, validarse las fuentes y comprobarse el contenido real según la [especificación de seguridad de archivos](../design/file-security.md). Los fallos nativos y defectos conocidos se siguen en los contratos e incidencias correspondientes.

<a name="xdzip-window-and-result-presentation"></a>

## Ventana y presentación de resultados

La comprobación previa usa una ventana de operación compartida de 491 píxeles lógicos de ancho, cuerpo adaptable, altura máxima de área de cliente de 600, lista larga de problemas desplazable y pie fijo accesible. Los límites de autorización de las fases de datos y metadatos no cambian. Consulta los [parámetros de comprobación previa](../../../specs/visual/preflight-reference.json) para parámetros y fuente de diseño.

La ubicación de la explicación se revisó el 2026-10-15: trasladar «Este informe solo comprueba metadatos…» al final del cuerpo desplazable con texto explicativo de 12 puntos. El pie ya no contiene esa explicación; mantiene solo la acción Volver y coincide en altura con un pie compartido que solo contiene acciones al mismo tamaño de fuente, independientemente de la longitud del informe.
