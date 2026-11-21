<a name="xdzip-high-fidelity-states-and-implementation-contract"></a>

# Estados de alta fidelidad y contrato de implementación

[English](../../ux/hifi-state-spec.md) | [简体中文](../../zh-CN/ux/hifi-state-spec.md) | [繁體中文](../../zh-TW/ux/hifi-state-spec.md) | [日本語](../../ja-JP/ux/hifi-state-spec.md) | [한국어](../../ko-KR/ux/hifi-state-spec.md) | [Français](../../fr-FR/ux/hifi-state-spec.md) | [Deutsch](../../de-DE/ux/hifi-state-spec.md) | Español | [Português (Brasil)](../../pt-BR/ux/hifi-state-spec.md) | [Русский](../../ru-RU/ux/hifi-state-spec.md)

Esta guía describe el comportamiento, el foco y las acciones de cierre de las ventanas de creación, extracción y tareas, con enlaces a los [originales de estados de operación](../../../design/hifi/README.md) y la [biblioteca compartida de recursos](../../../design/assets/README.md). Los identificadores de la tabla corresponden a mesas de trabajo del diseño y no cambian el protocolo de tareas.

<a name="xdzip-screens-and-task-semantics"></a>

## Pantallas y semántica de las tareas

| Estado / mesa de trabajo | Condición de entrada | Foco y acción predeterminados | Cierre y recuperación |
| --- | --- | --- | --- |
| C01 / 01 | Creación, sin tarea en ejecución | Primer botón de entrada o nombre de archivo; validar entradas, nombre, destino y capacidad antes de crear | Descartar el borrador de esta ventana; sin escritura |
| E02 / 02 | Una instantánea de tarea contiene un aviso de conflicto activo | Omitir, Renombrar y Sobrescribir son decisiones explícitas. Renombrar es la acción principal. Aplicar a conflictos posteriores comienza desmarcado | Cerrar oculta el aviso y deja la tarea esperando. Volver a abrirlo desde la tarea o usar su acción Cancelar para detenerla |
| T01 / 03, 08 | Plan aceptado y evento de ejecución recibido | Volver a explorar y cancelar son botones normales; no se inventa una pausa | Solo se oculta la vista; la entrada de tarea permite reabrirla; la finalización en segundo plano no roba el foco |
| T01 / comprobación de integridad | Mismo componente que el progreso, archivo de solo lectura y fases de verificación | Volver a explorar, cancelar la comprobación y resultado real de la tarea; la demostración HTML y la vista previa del componente Qt siguen identificadas por separado | Cerrar no cancela; las comprobaciones reales entran en la cola de segundo plano desde la barra de herramientas |
| R01 / 04 | La tarea tiene elementos publicados y sin terminar | Reintentar selecciona solo los elementos sin terminar y vuelve a confirmar la planificación | Cerrar la vista no elimina archivos ya confirmados |
| S01 / 05 | Abrir ajustes y elegir una de seis pestañas | Todas las ediciones son accesibles por teclado; los cambios se aplican inmediatamente al tener éxito y las direcciones deben validarse | Cerrar conserva los ajustes aplicados; un fallo conserva los valores anteriores |
| R02 / 06 superior | Recibido el estado terminal único de éxito | La extracción puede abrir la carpeta de destino por elección explícita; la creación abre resultados compatibles solo dentro de la aplicación, nunca una carpeta del sistema automáticamente. Cero fallos usa un estado neutro | Cerrar detalles sin repetir la notificación de finalización |
| T02 / 06 centro | Cancelación aceptada, limpieza sin terminar | Cancelación repetida desactivada; sigue siendo posible volver a explorar | Se puede ocultar; no se puede mostrar «Cancelado» antes de tiempo |
| R03 / 06 inferior | Recibido el estado terminal de cancelación | Conservar elementos publicados; si se permite reintentar, volver a planificar el alcance sin terminar | Cerrar no revierte elementos publicados |
| C02 / 07 | La mesa de trabajo muestra un archivo de salida existente | Un conflicto real de creación utiliza las reglas de nombre seguro y publicación de la tarea activa | Cerrar el aviso deja la tarea esperando. Renombrar, Reemplazar o Cancelar explícitamente sigue el resultado de la tarea |

«Usar nombre nuevo» solo propone un nombre; al comenzar realmente se vuelve a comprobar si existe el destino. Comprobar el nombre no constituye la autorización final de escritura. Una creación con reemplazo debe generar y verificar por completo un archivo temporal antes de volver a comprobar las condiciones del destino para publicarlo. Una cancelación que llega después del punto de confirmación linealizado muestra la finalización según el resultado del núcleo, sin convertir el éxito en cancelación.

<a name="xdzip-creation-capabilities-and-field-errors"></a>

## Capacidades de creación y errores de campo

- Los cuatro niveles ZIP corresponden a Store/Deflate 1/6/9, con 2 normal por defecto. Se pueden crear TAR y TAR.GZ; cambiar de formato sincroniza las extensiones y borra la contraseña y la división en volúmenes. TAR desactiva el nivel; los niveles TAR.GZ 0/1/2/3 corresponden a GZIP 0/1/6/9. El cambio de formato sincroniza la extensión sin sobrescribir destinos silenciosamente.
- Sin entrada, con nombre vacío, destino inválido o permiso ausente, Crear está desactivado y el motivo aparece junto al campo. El texto de error participa en la lectura de pantalla en lugar de limitarse a cambiar el color del borde.
- El núcleo trata las entradas duplicadas y la expansión de directorios; la interfaz solo muestra resúmenes de selección. Los iconos de imagen en creación indican el tipo, no son vistas previas.
- ZIP y 7z muestran sus opciones compatibles de contraseña y volúmenes. 7z también puede cifrar nombres de archivo. El registro de formatos indica las capacidades reales de escritura. [Creación ZIP](../engineering/zip-creation.md), [registro de formatos](../engineering/format-registry.md)

<a name="xdzip-task-data-and-exceptions"></a>

## Datos de tareas y excepciones

El ejemplo A tiene tres archivos: 8,4 + 6,2 + 12,8 = 27,4 MB. Con dos éxitos se guardan 14,6 MB y el PDF restante mide 12,8 MB; no deben confundirse los recuentos de elementos correctos, omitidos y sin terminar. El ejemplo B es un archivo completo de 30 archivos y 77,5 MB; 46,5 MB son el 60%, no el mismo momento del ejemplo A.

Los porcentajes proceden del total conocido de bytes y la medición real. Un total desconocido usa progreso indeterminado sin inventar tiempo restante. Volver a estimar la velocidad tras cambiar de fase; los 6,2 MB/s y aproximadamente 5 segundos ilustrados son solo un ejemplo de fase estable. Los recuentos distinguen archivos terminados y en proceso, sin usar el ordinal de la entrada actual como número de éxitos.

Disco lleno: detener y conservar los elementos publicados; volver a planificar tras liberar espacio o cambiar el destino. Cambio de origen: reintentar consiste en volver a abrir el archivo; no se pueden reutilizar los índices antiguos. Solo cancelación: los elementos sin terminar usan ámbar o neutro, sin informar de corrupción. Los códigos de error aparecen en detalles; las pantallas normales no muestran pilas de llamadas ni términos internos del puente.

<a name="xdzip-immediately-applied-settings"></a>

## Ajustes aplicados inmediatamente

Tema, tamaño de letra, densidad, idioma, modo de apertura y opciones de privacidad se guardan y aplican inmediatamente tras un cambio satisfactorio. Borrar el historial reciente ocurre de inmediato sin eliminar los archivos originales. Si falla, se conservan los ajustes anteriores y se muestra un mensaje. Acerca de y las actualizaciones están en la página Acerca de de ajustes; las asociaciones de archivos requieren confirmación del sistema y reflejan resultados reales. La página de red guarda las direcciones validadas de proxy y réplica. Véase el [contrato de ajustes](settings-localization.md).

Los presupuestos de salida predeterminados del anfitrión existente son 8 GiB por archivo y 20 GiB por tarea; la ventana de ajustes actual no permite editar cuotas. Si se añaden límites posteriormente, validar enteros positivos y desbordamiento, asegurar que el límite por archivo no supere el de la tarea y aplicarlos solo a planes nuevos. No cambiar la autorización inmutable de tareas activas ni eludir las comprobaciones de espacio.

<a name="xdzip-window-layout-and-asset-mapping"></a>

## Correspondencia de ventanas, diseño y recursos

Todas las ventanas de operación siguen la [especificación de ventanas por plataforma](platform-windows.md). Las barras de título nativas de macOS no usan semáforos recortados. La capa de plataforma verifica la posición y los estados desactivados de los botones de título en Windows/Linux, los menús del sistema y el movimiento. Los controles a la derecha en imágenes comparativas de Linux no son una promesa fijada en el código para todos los entornos de escritorio.

Las dimensiones actuales siguen [layout-scale.json](../../../specs/visual/layout-scale.json), el [registro de ventanas](../../../specs/visual/window-registry.json) y la [norma de diseño](../visual/design-standard.md), en sustitución de propuestas iniciales demasiado grandes. Creación, extracción, conflicto, progreso y resultados comparten anchura 491; ajustes usa 612 como diferencia registrada por su función. Título 20, cuerpo 13, botones de pie 13 y texto auxiliar 12. La altura depende del contenido y del límite de cada ventana; el cuerpo se desplaza. Las ventanas de operación con pie lo mantienen fijo y accesible; ajustes no tiene pie. Ajustes ofrece tamaños de letra 100%/125%/150%; no deducir dimensiones de componentes del escalado global ni de píxeles físicos de capturas.

| Uso | Recursos añadidos | Partes compartidas / dinámicas |
| --- | --- | --- |
| Título de creación, añadir entradas | archive-add, image-file, remove | folder, file-pdf, new |
| Conflicto, fallo parcial | document-warning, status-warning, info | file-pdf; tarjetas comparativas y texto dibujados por QML |
| Progreso en ejecución | document-download, image-file, info | Barras de progreso y números dibujados dinámicamente |
| Éxito, omisión, cancelación, parada | status-success, status-skipped, status-stopped, spinner, clock | El control Qt gira spinner |
| Ajustes | settings-tile, theme-system | Vistas previas de tema, interruptores y entradas numéricas son componentes; settings es un icono pequeño |

Los recursos están en `design/assets/`. `reference/` conserva recortes en píxeles originales para comparar la apariencia; `svg/` y `png/` son recursos transparentes de ejecución y no deben llamarse capas originales. Todos los SVG usan lienzos de 24×24; los PNG se exportan a 24×24, 48×48 y 72×72, manteniendo el mismo tamaño lógico de visualización. Las listas HTML usan un espacio de 48×48 en coordenadas de origen; Qt usa aproximadamente 31,75×39,69 píxeles lógicos. Los iconos PDF, texto, código y Markdown comparten centro y altura visible de 22 píxeles, con anchuras de contorno distintas; las carpetas se centran en el mismo lienzo. Los SVG oscuros son adaptaciones de recursos; el comportamiento oscuro completo aún requiere comprobaciones reales de Qt y contraste.

<a name="xdzip-directory-tree-addition"></a>

## Incorporación del árbol de directorios

La navegación B01 añade la [especificación B del árbol de directorios](directory-tree.md), con tres mesas de trabajo formales y 26 recortes compartidos. Las flechas de nodo dependen de los subdirectorios; expansión, directorio actual y foco de teclado son independientes. La carga, los archivos vacíos, los archivos raíz, los límites de navegación, los fallos y las rutas actuales ocultas se especifican por separado; el contrato de dominio de solo lectura existente no cambia. Qt integran la navegación principal, las líneas de conexión y los indicadores de rama actual oculta; los esqueletos y los iconos específicos de límites siguen siendo recursos de diseño. Véase la especificación del árbol para el alcance de integración.

<a name="xdzip-changes-and-revalidation"></a>

## Cambios y nueva validación

Los estados principales se han usado en Qt. Los cambios posteriores siguen comprobando los diseños originales y los parámetros revisados, la correspondencia entre controles y funciones, los componentes compartidos y los contratos de ventana, y verifican las plataformas afectadas. Los diseños o las comprobaciones estáticas por sí solos no demuestran funcionalidad nativa ni accesibilidad. Registrar las conclusiones reales en la incidencia correspondiente; esta lista no reinicia la aceptación histórica.
