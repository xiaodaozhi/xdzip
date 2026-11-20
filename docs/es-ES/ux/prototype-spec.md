<a name="xdzip-v01-interaction-prototype-and-feature-mapping"></a>

# Prototipo de interacción v0.1 y correspondencia de funciones

[English](../../ux/prototype-spec.md) | [简体中文](../../zh-CN/ux/prototype-spec.md) | [繁體中文](../../zh-TW/ux/prototype-spec.md) | [日本語](../../ja-JP/ux/prototype-spec.md) | [한국어](../../ko-KR/ux/prototype-spec.md) | [Français](../../fr-FR/ux/prototype-spec.md) | [Deutsch](../../de-DE/ux/prototype-spec.md) | Español | [Português (Brasil)](../../pt-BR/ux/prototype-spec.md) | [Русский](../../ru-RU/ux/prototype-spec.md)

Estado: borrador interactivo de validación de diseño. Las fuentes están en [design/html](../../../design/html/README.md), con [index.html](../../../design/html/index.html) como entrada. El borrador de Figma sigue siendo una exploración histórica; el refinamiento actual de funciones y estados continúa en el repositorio.

<a name="xdzip-covered-flows"></a>

## Flujos cubiertos

| Estado | Entrada y salida | Requisitos / aceptación |
| --- | --- | --- |
| B01 Navegar | Abrir un archivo de ejemplo; entrar en un directorio con doble clic o Intro y volver por la ruta de navegación | P01/P02/P08, AC-01/02/08 |
| B02 Buscar y seleccionar | Casilla, selección individual, múltiple con ⌘/Ctrl, rango con Shift y seleccionar todo; la búsqueda cubre el directorio actual y descendientes, o todo el archivo desde la raíz | P02/P03, AC-02/03 |
| E01 Confirmar extracción | Botón principal o menú de alcance; destino, directorio con el mismo nombre, alcance y política de conflicto editables | P03/P07, AC-03/07 |
| E02 Conflicto | Entrar ante una colisión con la política de preguntar; continuar tras omitir, cambiar nombre automáticamente o sobrescribir | P03/P07, AC-03/07 |
| T01 Tarea | Inicio → preparación → procesamiento → verificación → resultado; volver a navegar o solicitar detención | P06/P15, AC-06/14 |
| R01 Resultado | Éxito, fallo parcial, cancelación, cambio de fuente; reintentar elementos inacabados tras nueva confirmación | P03/P04/P15, AC-03/04/14 |
| C01 Crear | Añadir/quitar entradas, formato/nivel/volúmenes/contraseña y ubicación; confirmar primero destinos existentes | P04/P05, AC-04/05 |
| S01 Ajustes | Tema, tamaño de texto, densidad, idioma e historial reciente; cerrar vuelve a navegar | P08/P11, AC-08/11 |
| X01 Excepciones | Archivo vacío, sin resultados, carga, corrupción, funciones no admitidas, permisos y fallo de tarea | P05/P07/P15, AC-05/07/14 |

«Anotaciones de diseño» en la cabecera del prototipo proporciona estos ID y las reglas de estado actuales. Los resultados de permisos y espacio dependen del escenario; ejecutar correctamente el prototipo no supera la aceptación técnica P09/P10/P12.

<a name="xdzip-interaction-decisions"></a>

## Decisiones de interacción

<a name="xdzip-selection-and-search"></a>

### Selección y búsqueda

- Cada selección corresponde a un ID de entrada estable, no a un número de fila ordenada. Cambiar consulta o directorio borra la selección para evitar malentendidos por selecciones ocultas.
- Seleccionar todo en la lista actual y extraer todo el archivo son conceptos distintos. Marcar una carpeta se expande a sus archivos descendientes sin duplicados.
- No enviar consultas durante la composición IME; actualizar después de compositionend. Las pruebas de eventos simulados del navegador no sustituyen la aceptación IME nativa en macOS/Windows/Linux.
- Intro o doble clic en un archivo normal extrae una copia temporal y la abre con la asociación del sistema; en un directorio entra en él. La entrada de propiedades permanece. HTML solo demuestra progreso y ocultación automática; no escribe archivos ni llama al sistema. Consulta el [contrato de vista previa](../engineering/content-preview.md) para la vista previa automática de Qt.

<a name="xdzip-extraction-and-conflicts"></a>

### Extracción y conflictos

- Mostrar la ruta final en directo; añadir solo una vez el directorio con el mismo nombre. Las rutas de ejemplo vacías o manifiestamente no escribibles bloquean el inicio; repetir la comprobación previa tras cambiar el destino.
- Preguntar por defecto ante conflictos; están disponibles omitir, cambio automático de nombre y sobrescribir. El texto cercano explica las consecuencias de sobrescribir.
- El foco inicial del panel de conflicto está en Omitir; sobrescribir no es la acción predeterminada de Intro. Aplicar a conflictos posteriores se limita a esta tarea; los datos de ejemplo crean un solo conflicto.
- Los cambios de nombre aparecen en el resumen final y los detalles. Omitir no es un fallo; los archivos previos del ejemplo no cuentan como escrituras completadas.

<a name="xdzip-tasks-and-results"></a>

### Tareas y resultados

- Volver a navegar no cancela la tarea; la barra de estado puede reabrirla.
- Solicitar cancelación → deteniendo, con cancelación repetida desactivada → resumen cancelado. La extracción conserva elementos publicados; la creación de archivo no publica resultados intermedios.
- Un disco lleno detiene la tarea y conserva los éxitos. Reintentar selecciona solo lo fallido o inacabado y vuelve a confirmar el plan.
- Los cambios de fuente requieren volver a abrir el archivo, sin reutilizar silenciosamente el índice anterior.
- Una tarea de creación cuenta un archivo final; al cancelar muestra cero archivos creados y conserva claramente el anterior.

<a name="xdzip-archive-creation-and-capabilities"></a>

### Creación de archivos y capacidades

El prototipo ilustra niveles ZIP, empaquetado TAR y opciones de compresión GZIP. Comprueba campos básicos de entrada, nombre y destino, y cambia la extensión con el formato. Las opciones actuales del producto usan el [registro integrado de formatos](../engineering/format-registry.md), incluidos los perfiles de creación [ISO](../engineering/iso-creation.md) y [SFX de Windows](../engineering/sfx-creation.md). Consulta la [interacción de contraseña](password-extraction.md) y el [contrato de volúmenes](../engineering/archive-opening.md) para las opciones admitidas.

<a name="xdzip-visuals-and-adaptation"></a>

### Aspecto y adaptación

Conservar temas gris cálido, azul cobalto y grafito. La altura de fila normal es 40 y la compacta 32. El acento de texto del tema oscuro es independiente del relleno de botones para mantener legible el texto pequeño. En escritorio estrecho, los detalles se convierten en una superposición que puede cerrarse. La revisión móvil contrae la barra lateral y columnas secundarias; los paneles de operación se adaptan a pantallas estrechas y se desplazan.

Las interfaces de operación son ventanas independientes según la [especificación de ventanas de plataforma](platform-windows.md). HTML las simula con dialog: las ventanas de decisión limitan el foco; progreso, resultados y ajustes no son modales; al cerrar se intenta restaurar el foco. Cerrar una ventana de conflicto cancela la tarea en vez de omitir silenciosamente; cerrar progreso solo oculta detalles. Qt usa ventanas reales independientes de nivel superior.

<a name="xdzip-theme-selection"></a>

### Selección de tema

La página de ajustes organiza tres botones de opción nativos, sistema, claro y oscuro, con `fieldset#setting-theme` y la leyenda «Appearance theme». Cada tarjeta usa el recurso compartido `theme-system`, `theme-light` o `theme-dark` y asocia su etiqueta y entrada. Los controles reales participan en clics de tarjeta, cambios por teclado y foco. La selección usa la marca común y sincroniza borde y vista previa de la tarjeta, sin sustituirla por una captura no interactiva.

Esto describe la validación de diseño HTML. Los ajustes actuales están en memoria y seguir el sistema solo se resuelve al seleccionarlo. Los cambios de tema de sistema durante la ejecución, la persistencia entre arranques y el guardado inmediato y tratamiento de fallos de ajustes Qt del producto requieren verificación independiente.

<a name="xdzip-verification-and-remaining-boundaries"></a>

## Verificación y límites restantes

El script de navegador es scripts/check-html-design.py. Comprueba clics reales, estadísticas, condiciones desactivadas, acoplamiento de formatos, recuperación, desbordamiento móvil y funcionamiento sin conexión en un solo archivo. Los informes acompañan el paquete exportado del prototipo; las pruebas de navegador usan solo Chromium.

La entrega actual de la ventana principal incluye dos sesiones independientes de archivos de ejemplo, lista/cuadrícula, orden por campo, árbol recursivo, rutas jerárquicas, menús contextuales/desplegables/submenús y estados de botones. Consulta el [contrato de componentes e interacción](../../ux/main-window-components.md) y el [método de verificación](../../../developers/qa/README.md). `scripts/check-html-controls.py` los comprueba con entradas reales del navegador; la [página de revisión de componentes](../../../design/html/components.html) usa la misma implementación que la interfaz principal.

HTML solo verifica estados de ejemplo; no demuestra planificación real de tareas, autorización, carreras de rutas, CRC ni accesibilidad de plataforma. Qt integra creación/extracción reales, arrastrar y soltar, selectores de sistema, ajustes persistentes y sesiones independientes en pestañas opcionales o varias ventanas. El alcance real de múltiples formatos, la indexación de un millón de entradas en ZIP normales de un archivo y la vista previa limitada aparece respectivamente en [motores de formato](../engineering/native-codecs.md), [indexación grande](../engineering/large-index.md) y [vista previa](../engineering/content-preview.md); la aceptación se registra en los issues correspondientes. Los ajustes HTML conservan el estado de demostración solo dentro de la página, no entre arranques, y no prueban seguimiento nativo del tema de sistema ni aceptación de accesibilidad.

Estos límites no cambian las capacidades objetivo del diseño. «Completado» en el prototipo describe solo un estado de tarea de demostración y no demuestra la entrega de compresión o extracción reales.

<a name="xdzip-current-directory-queries-and-card-revisions"></a>

## Consultas del directorio actual y revisiones de tarjetas

La entrada raíz es «Root» y solo lista archivos y carpetas directos. La búsqueda encuentra descendientes del alcance actual y muestra rutas completas dentro del archivo. Listas y cuadrículas normales muestran solo el nombre final; copia y navegación conservan rutas completas. A la derecha de la ruta de navegación se muestra el número de filas actuales. El título del árbol es «Archive», centrado verticalmente con la ruta; al pasar sobre el árbol no aparece información flotante.

El resaltado al pasar cubre toda la fila. Las cuadrículas tienen aproximadamente 16 píxeles lógicos de relleno horizontal; una casilla de 8 píxeles arriba a la izquierda aparece al pasar y permanece visible al seleccionarse. Los menús de botones están a 4 unidades del botón y usan sombra suave. El botón para reabrir la vista previa es cuadrado de lado 28. Los parámetros están en layout-scale.json, directory-tree.json y controls.json; la verificación real sigue el [método general](../../../developers/qa/README.md).

<a name="xdzip-settings-window"></a>

## Ventana de ajustes

Ajustes conserva los títulos con iconos, tarjetas de tema, densidad, historial reciente y explicación de privacidad originales, y añade tamaño de texto e idioma. Las vistas previas de tema y fuente pueden cancelarse; Guardar y Restaurar valores usan borradores. HTML conserva el estado de demostración solo en la página actual. En la demostración HTML, la selección de idioma muestra una configuración que surte efecto en el próximo inicio de Qt; ni la página de revisión ni el contenido del archivo de ejemplo prueban la cobertura de traducción del producto. Para el cambio de idioma, recursos incluidos y persistencia actuales de Qt, consulta [ajustes y localización](settings-localization.md).
