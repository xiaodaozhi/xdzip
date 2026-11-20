<a name="xdzip-read-only-zip-browsing-contract"></a>

# Contrato de navegación ZIP en modo de solo lectura

[English](../../engineering/zip-readonly-browsing.md) | [简体中文](../../zh-CN/engineering/zip-readonly-browsing.md) | [繁體中文](../../zh-TW/engineering/zip-readonly-browsing.md) | [日本語](../../ja-JP/engineering/zip-readonly-browsing.md) | [한국어](../../ko-KR/engineering/zip-readonly-browsing.md) | [Français](../../fr-FR/engineering/zip-readonly-browsing.md) | [Deutsch](../../de-DE/engineering/zip-readonly-browsing.md) | Español | [Português (Brasil)](../../pt-BR/engineering/zip-readonly-browsing.md) | [Русский](../../ru-RU/engineering/zip-readonly-browsing.md)

El contrato define las entradas directas de la raíz, la búsqueda en subcarpetas, la identidad física estable y el orden que muestra primero las carpetas. La navegación ZIP lee los metadatos sin descodificar el contenido de las entradas ni comprobar su CRC. No extrae archivos. Los ZIP normales de gran tamaño pueden usar un [índice privado de metadatos en disco](large-index.md).

Consulta la [indexación de archivos comprimidos grandes](large-index.md) para conocer el índice actual de ZIP normales grandes y el modelo de Qt bajo demanda. Esta página conserva el límite inicial de 100 000 entradas y los requisitos de identidad de consulta. Ampliar la capacidad no elimina las pruebas de errores, cancelación o identidad.

<a name="xdzip-browsing-scope-and-limits"></a>

## Alcance y límites de navegación

- Metadatos ZIP/ZIP64 de hasta 100 000 registros físicos. El [host de volúmenes divididos](archive-opening.md) convierte los volúmenes antes de leerlos. El códec sigue rechazando los metadatos multidisco sin convertir.
- Hasta 4 KiB por nombre, 16 MiB en total para los nombres en bruto, 16 MiB en total para los nombres de presentación descodificados y escapados, 64 MiB de lectura total y 256 KiB de búfer de escaneo. Los tamaños declarados de 64 bits se conservan como enteros exactos.
- Índice de directorios de hasta 32 768 nodos, 64 niveles y 4 MiB de nombres y prefijos en total. Si se supera un límite, se descarta todo el índice de navegación y se explica la funcionalidad reducida. No se publica un árbol incompleto.
- Se reutilizan los índices de entradas directas y los resultados. Los lotes de una misma consulta no vuelven a filtrar ni a ordenar. Los nombres de cada lote están limitados a 64 KiB. La interfaz muestra el directorio actual completo sin botones de paginación.
- Cada página valida la sesión, el ID de consulta, la generación de la interfaz, la revisión de la instantánea y el desplazamiento. Un cambio de consulta cancela el trabajo anterior y conserva la solicitud más reciente. Las respuestas obsoletas no sustituyen a los resultados nuevos.
- Se comprueban la longitud, la fecha de modificación, el dispositivo e inodo de Unix y el número de volumen e índice de archivo de Windows antes y después del escaneo, al consultar y en la verificación previa. Ante `source_changed`, las consultas de Qt y la verificación previa vacían las listas, la selección, el árbol de directorios y los detalles invalidados. Vuelve a abrir el archivo comprimido para crear una sesión. La detección es de mejor esfuerzo y no constituye una instantánea atómica del sistema de archivos ni una autorización de escritura segura.

<a name="xdzip-acceptance-exit-criteria"></a>

## Criterios de aceptación

Un ZIP64 generado con 100 000 entradas debe completar el escaneo, la ordenación de todos los resultados, la recuperación, la navegación de carpetas, la lista completa de Qt y la selección. Todos los lotes comparten la misma identidad de consulta. Comprueba ZIP64 dañados, volúmenes divididos, límites de nombres, visualización y lectura, cancelación, respuestas obsoletas y modificaciones o reemplazos del origen. Registra la duración, los recursos, la cola de eventos y el número de delegados reales. Si se validan bajo demanda las compilaciones de tres plataformas o los paquetes de desarrollo para Apple Silicon, registra sus resultados por separado.

Filtra las ejecuciones de CI del repositorio por head_sha y por el evento real. Las comprobaciones habituales se ejecutan al hacer push o abrir una PR. Foundation y Qt se ejecutan en modo completo manualmente cuando hace falta. Registra los resultados y las comprobaciones pendientes en la incidencia correspondiente. Una lista vacía para otro evento no acredita el estado de la comprobación.
