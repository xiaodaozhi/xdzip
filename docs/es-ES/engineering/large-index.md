<a name="xdzip-large-archive-indexing-and-on-demand-browsing"></a>

# Indexación de archivos comprimidos grandes y navegación bajo demanda

[English](../../engineering/large-index.md) | [简体中文](../../zh-CN/engineering/large-index.md) | [繁體中文（台灣）](../../zh-TW/engineering/large-index.md) | [日本語](../../ja-JP/engineering/large-index.md) | [한국어](../../ko-KR/engineering/large-index.md) | [Français](../../fr-FR/engineering/large-index.md) | [Deutsch](../../de-DE/engineering/large-index.md) | Español | [Português (Brasil)](../../pt-BR/engineering/large-index.md) | [Русский](../../ru-RU/engineering/large-index.md)

El límite de capacidad del código para ZIP/ZIP64 ordinario de un solo archivo es de un millón de registros físicos. Los demás formatos y las rutas con volúmenes divididos conservan sus admisiones existentes de 100 000 entradas; el nombre del formato no implica que todas sus combinaciones tengan límites ampliados. Directorios, nombres duplicados y registros que requieren revisión cuentan como entradas físicas; los directorios implícitos de navegación solo añaden filas de visualización.

<a name="xdzip-scanning-and-storage"></a>

## Exploración y almacenamiento

ZIP explora cada registro con el analizador ZIP ordinario: primero termina un muestreo acotado de nombres de codificación heredada en todo el archivo y después hace una segunda pasada con la misma decisión de codificación. Ambas pasadas leen en total como máximo 512 MiB; el directorio central tiene como máximo 256 MiB, los nombres originales suman como máximo 128 MiB y los nombres decodificados y mostrados juntos como máximo 128 MiB. Cada nombre original o decodificado sigue limitado a 4 KiB. Una cola dañada, cancelación o cambio de origen hace fallar todo el resultado sin publicar un índice parcial.

Al alcanzar 64 MiB de metadatos activos o superar 100 000 entradas, migrar los registros individualmente a almacenamiento privado en disco y liberar nombres y registros existentes sin mantener dos índices completos simultáneamente. La tabla de posiciones físicas contiene como máximo un millón de desplazamientos u64. Los metadatos en disco tienen un máximo de 512 MiB; los nombres y contenidos no aparecen en cadenas de diagnóstico. El árbol de directorios sigue limitado a 32 768 nodos, 64 niveles y 4 MiB de nombres. Si se supera el límite, no publicar el árbol; siguen disponibles la navegación en la raíz y la búsqueda física.

Los archivos temporales residen en un directorio `xdzip-index-*` de nombre aleatorio bajo el directorio temporal del anfitrión, legible y modificable solo por el usuario actual y operado mediante identificadores conservados de directorio y archivo ordinario. El cierre normal, sustitución de consulta, fallo y cancelación limpian archivos conocidos y directorios vacíos cuya identidad se verifica. Los restos de una caída se identifican mediante un marcador de 16 bytes que coincide con la identidad aleatoria del directorio y una concesión exclusiva. Una creación posterior explora de forma acotada y limpia solo archivos conocidos sin usar de más de siete días. Se conservan enlaces, identidades no coincidentes y objetos desconocidos. No borrar recursivamente contenido desconocido ni restaurar o reutilizar índices entre ejecuciones.

En Windows, el directorio nuevo debe superar primero las comprobaciones ordinarias de propietario y permisos de modificación antes de proteger su DACL para almacenamiento confidencial. Permisos de modificación heredados para entidades ajenas causan rechazo antes de escribir la concesión o los metadatos; el entorno no usa silenciosamente una ubicación menos privada. Para verificar de forma aislada, dirigir solo `TMP` y `TEMP` del proceso de prueba a un directorio propio nuevo cuya ACL permita modificar únicamente al usuario actual, SYSTEM y Administrators. No cambiar las ACL de directorios reales de usuarios ni relajar las comprobaciones. Verificar por separado la asignación, consultas y fusión reales en disco, concesiones activas, limpieza de archivos conocidos y rechazo de permisos de modificación ajenos.

<a name="xdzip-queries-and-selection"></a>

## Consultas y selección

La búsqueda y la ordenación producen un orden de filas completo e inmutable. Las claves se escriben en secuencias privadas de fusión de hasta 8 MiB cada una, con un máximo de 128 secuencias y 512 MiB de almacenamiento temporal. Las consultas finales guardan solo tokens de fila, sin otra copia completa de nombres. Las comparaciones conservan los enteros originales, las reglas existentes de nombre y tipo y la identidad física estable. Todos los modos sitúan directorios explícitos e implícitos antes que archivos; la dirección se aplica dentro de cada grupo. Los tamaños desconocidos siguen a los conocidos dentro de su grupo; el orden descendente no cambia el orden de identidad de duplicados. Se comprueba la cancelación durante la exploración y alrededor de secuencias y fusión, sin alterar las relaciones de orden en el comparador.

El puente transfiere como máximo 128 filas y 64 KiB de nombres por lote. Las consultas de hasta 1 024 filas conservan el método de confirmación completa existente. Los modelos Qt mayores mantienen el número lógico completo de filas y cargan solo las posiciones visibles, con un máximo de 1 024 filas residentes en caché. Las filas no cargadas no pueden enviar acciones de entrada. Cada página diferida queda vinculada a sesión, consulta, generación, revisión y desplazamiento solicitado; las páginas obsoletas no pueden entrar en resultados nuevos.

La selección entre páginas usa hasta 4 096 rangos acotados de la consulta actual inmutable. Seleccionar todo abarca toda la consulta actual; salir de la caché no modifica la selección. Antes de confirmar la extracción, Rust congela los rangos como identidades físicas o de navegación, que se mantienen aunque cambie la consulta. Las selecciones que superan el límite de 100 000 entradas de la tarea se rechazan explícitamente en vez de reducirse en silencio. Actualmente no se muestra el tamaño total de selecciones no residentes. Una copia masiva al portapapeles más allá del contenido residente devuelve un límite de recursos en vez de copiar silenciosamente una lista incompleta.

<a name="xdzip-task-boundaries"></a>

## Límites de tareas

Los archivos ordinarios seleccionados de un índice en disco pueden extraerse tras volver a validar completamente el origen, planificar de forma segura, verificar la entrada completa y aplicar la confirmación autorizada existente. Un plan congelado sigue limitado a 100 000 entradas y 64 MiB de metadatos; los límites de archivos y bytes realmente decodificados siguen el [contrato de extracción](zip-safe-extraction.md). La expansión de directorios no puede omitir nombres sin resolver; los enlaces seleccionados se rechazan en esta configuración. El lector ZIP vuelve a comprobar los desplazamientos físicos de todo el directorio central, conserva solo las ubicaciones de datos seleccionadas y mantiene las comprobaciones de solapamiento, cabecera local, tamaño, CRC, autenticación y cancelación.

Los archivos comprimidos que usan índice en disco aún no admiten modificación ni comprobación del archivo completo; una proyección vacía en memoria no puede tratarse como archivo completo. Los límites existentes de creación, otros formatos, volúmenes divididos y reescritura no cambian. Consulta por separado la [vista previa de contenido](content-preview.md). El listado físico y la selección CLI reutilizan el mismo índice; la [paginación explícita](command-line.md#xdzip-archive-behavior) obtiene el rango físico completo de archivos grandes. Las respuestas sin paginación mantienen presupuestos de 100 000 entradas y 16 MiB de datos serializados o texto y fallan por completo si se superan.

<a name="xdzip-verification"></a>

## Verificación

`scripts/probe-large-index.py` genera un ZIP64 real en `build/` con nombres chinos, duplicados y directorios anidados y llama al `xdzip-m2-probe` real. La sonda lee cada lote de consulta y registra por separado exploración, ordenación, recuperación, bytes en disco y métricas de memoria disponibles en la plataforma. El generador de muestras no construye en memoria una matriz de un millón de nombres. Alcance, fallo, cancelación, limpieza, páginas obsoletas y salida real de archivos seleccionados siguen los [requisitos de aceptación](../../../developers/qa/acceptance.md); los resultados permanecen en incidencias, localmente o en artefactos CI. Las mediciones son evidencia de la máquina y carga reales, no garantías universales de rendimiento.
