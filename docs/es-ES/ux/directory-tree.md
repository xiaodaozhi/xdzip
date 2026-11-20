# Árbol de directorios B: diseño, recursos e integración

[English](../../ux/directory-tree.md) | [简体中文](../../zh-CN/ux/directory-tree.md) | [繁體中文](../../zh-TW/ux/directory-tree.md) | [日本語](../../ja-JP/ux/directory-tree.md) | [한국어](../../ko-KR/ux/directory-tree.md) | [Français](../../fr-FR/ux/directory-tree.md) | Español | [Português (Brasil)](../../pt-BR/ux/directory-tree.md)

Estado: **la interacción principal de directorios está integrada en los ejemplos HTML y en la exploración de solo lectura de Qt; los recursos de estados límite están pendientes de integración**. La interacción sigue el [contrato de exploración de solo lectura](../engineering/readonly-browser.md). La exploración normal usa hijos directos, la búsqueda cubre recursivamente el ámbito actual y la autorización de salida se trata por separado de la lectura.

Las tres mesas de trabajo actuales son [expansión multinivel](../../../design/hifi/screens/directory-tree-capsule.png), [estados de nodos e interacción](../../../design/hifi/screens/directory-tree-states.png) y [límites de navegación y estados vacíos](../../../design/hifi/screens/directory-tree-boundaries.png). La [galería compartida](../../../design/hifi/index.html#components) muestra las mesas completas y la [biblioteca de recursos](../../../design/assets/index.html#directory-tree) proporciona los recortes individuales. La expansión multinivel se exporta desde la [fuente editable](../../../design/hifi/sources/main.html?tree=expanded), con la barra de título sincronizada con las pestañas actuales en cápsulas. Las listas superior y central de la mesa solo aportan contexto y no redefinen la barra de título, la barra de herramientas ni la tabla de archivos. El [original histórico multinivel](../../../design/hifi/screens/directory-tree.png) permanece intacto como fuente de 26 recortes y referencia independiente de medición.

Las mesas están ampliadas y las escalas de las tarjetas son distintas. El texto, los radios y las dimensiones en píxeles de las imágenes no pueden definir directamente los tamaños de los controles; los [parámetros de componentes](../../../specs/visual/directory-tree.json) registran explícitamente las dimensiones lógicas implementadas y el manifiesto de recursos registra por separado las coordenadas físicas de recorte. La implementación usa una anchura lógica de 1024 y una transformación compartida del lienzo de origen; no amplíe de nuevo todo el conjunto de componentes.

Qt usa directamente los valores lógicos siguientes. El diseño principal HTML existente asigna una ventana de 1548 píxeles de origen a 1024 píxeles lógicos según [display-spec.js](../../../design/html/display-spec.js): `source CSS = logical / (1024/1548)`. Así, una altura de fila de 32 corresponde a 48.375 píxeles CSS de origen, una fuente de 11.2 a 16.93125, un espacio de icono de 16 a 24.1875 y una sangría de 12.8 a 19.35. La escala de vista previa se aplica uniformemente a toda la ventana después de la conversión. El DPR solo determina la densidad de muestreo y la alineación de líneas y no interviene en la conversión de tamaños lógicos.

## Estructura del árbol y recuentos

El índice completo proporciona directorios explícitos e implícitos. Una hoja significa **sin subdirectorios**, no sin archivos. Si Glacier Lake contiene un subdirectorio, tiene su propia flecha de expansión, independiente de la de su padre.

```text
Root                               ← current directory
▼ 01 Photos                     12
  ▼ Glacier Lake                 6
    ▸ Lakeside Trail             3  ← deeper directory, not expanded
      Snow Mountain View          3  ← files only, retains empty arrow slot
  ▸ Mountain Sunrise             4
    Other Photos                 2  ← no subdirectories
▸ 02 Documents                   8
  03 Maps                        4  ← no subdirectories
```

El archivo de ejemplo contiene 12 fotos, 8 documentos, 4 mapas y 6 archivos raíz; la lista raíz muestra tres directorios y seis archivos directos sin expandir descendientes. Los descendientes de ramas contraídas también se enumeran por completo en el archivo de parámetros para no adornar las hojas con flechas falsas. El directorio vacío explícito de otra mesa es un ejemplo de estado independiente, no añadido a este ejemplo de 30 entradas.

Los números de directorio representan todos los **registros físicos del archivo** descendientes bajo el prefijo, incluidos nombres duplicados pero excluido el registro de directorio idéntico al prefijo. No son recuentos de subdirectorios y no cambian con la búsqueda por nombre. Las filas del árbol no muestran recuentos. El resumen del archivo conserva el recuento completo de registros físicos, mientras que el lado derecho del título de la lista muestra el número de filas del directorio actual; sus definiciones son distintas. Las listas normales muestran el nombre final y los resultados de búsqueda la ruta completa dentro del archivo.

Las flechas dependen solo de los subdirectorios conocidos: apuntan a la derecha con el padre contraído y hacia abajo con él expandido. Los nodos que contienen solo archivos o están vacíos no tienen flecha, pero los elementos del mismo nivel conservan idénticos espacios de flecha e icono y comienzo del texto. Durante la carga del índice completo, no se deben adivinar flechas, recuentos ni estructura por nodo ni añadir estados de carga asíncronos por nodo.

## Diseño, fuentes y geometría dinámica

| Elemento | Tamaño lógico / comportamiento normal |
| --- | --- |
| Anchura predeterminada de la barra lateral | 193.16; el límite estructural de la ventana principal se redondea a 193 |
| Altura de fila | Mínimo 32; crece según la fórmula de parámetros con fuentes mayores |
| Sangría por nivel | 12.8; independiente del tipo de nodo |
| Área activa / espacio gráfico de flecha | Al menos 19.2×19.2 / 14.4×14.4; las hojas conservan el mismo espacio |
| Espacio de icono de carpeta | 16×16; el SVG compartido de 24 unidades se escala proporcionalmente y sustituye el espacio 18 del árbol actual |
| Espacio entre icono y texto / margen final | 6.4 / 16; sin recuento visible |
| Texto principal | 11.2; Qt redondea a 11 y usa la fuente del mismo origen |
| Recuento de registros | Se conserva en los datos y en la semántica accesible; no se muestra en filas ni se reserva columna |
| Directorio actual | Fondo azul pálido, esquinas derechas de 6.4, borde izquierdo recto e indicador de ancho 2.4 y altura completa |
| Foco de teclado | Sin contorno o barra de foco adicionales ni cambio del espacio de fila; se conserva el indicador de directorio actual |
| Líneas de jerarquía | Anchura 1; jerarquía discreta sobre el fondo |
| Marcador de directorio actual oculto | Punto de 4.8 unidades en un espacio transparente de 19.2 unidades, centrado en la fila del nombre |

Las líneas de conexión se dibujan dinámicamente a partir de la cadena de antepasados visibles y los indicadores de último hermano. Las líneas verticales del padre atraviesan los subárboles expandidos hasta el centro del último hermano; la última conexión termina en L y no continúa al siguiente grupo de nivel superior. El centro X de cada línea vertical se alinea con la punta X de la flecha descendente del padre, también con fuentes grandes de accesibilidad. Las líneas interiores terminan de forma independiente; una imagen genérica de línea no puede abarcar todas las filas. Alinee los segmentos con el DPR de destino; texto, flechas y carpetas comparten el centro vertical de la fila.

La región de jerarquía se desplaza horizontalmente. La anchura usa solo los nombres reales, la sangría y el margen final de 16 de los nodos visibles expandidos. Los nombres permanecen completos sin superposición de números. Recalcule la anchura al contraer para que los nodos profundos ocultos no dejen espacio vacío; el indicador izquierdo permanece fijo al borde del área visible. Al pasar el cursor por una fila no aparece una información emergente flotante; las rutas completas están disponibles en las descripciones accesibles y los detalles. El desplazamiento vertical ocurre solo dentro del área de directorios; las barras no rodean el borde de la ventana principal.

Todos los directorios usan el mismo recurso de carpeta. Los puntos de rama, iconos rellenos de información y advertencia y animaciones segmentadas de carga tienen SVG claros/oscuros y PNG 1×/2×/3×; líneas, marcos de foco, fondos, esqueletos, texto y avisos son componentes dinámicos. Los parámetros oscuros reutilizan el tema global con colores adicionales para líneas y marcadores; **el original oscuro completo del árbol sigue pendiente** y las exportaciones de iconos oscuros no sustituyen la mesa completa. Las conclusiones existentes de representación y aceptación de escritorio se registran por separado en las incidencias y artefactos correspondientes.

Registre la coherencia de interacción entre HTML y Qt según el [método de verificación](../../../developers/qa/README.md). Las mesas y recortes conservan el historial del diseño original; los recuentos de filas, el foco y las esquinas actuales siguen esta página y la especificación lógica.

## Interacciones y estados independientes

La expansión, el directorio actual, el estado de cursor y el foco de teclado se guardan de forma independiente y pueden coexistir. El directorio actual también es distinto de la selección de casillas de archivos de la derecha.

La raíz es una entrada navegable por separado que permanece siempre expandida y no participa en la expansión o contracción normal de nodos. Cuando se abre su menú con clic derecho, tecla Menú o Mayús+F10, «Contraer subdirectorios» permanece desactivado sin línea de explicación ni información emergente visible. La descripción accesible es «根目录始终展开» / «Root is always expanded»; la ausencia de una fila de árbol normal no implica que no haya subdirectorios. La expansión o contracción de padres ordinarios depende de los subdirectorios reales; las hojas ordinarias conservan la descripción accesible «This node has no subdirectories». Los motivos solo son accesibles, no indicaciones visibles. Abrir el menú o pulsar elementos desactivados no debe cambiar el directorio, la selección de archivos ni la expansión del árbol.

| Acción | Resultado |
| --- | --- |
| Pulsar la flecha | Expandir o contraer solo ese nodo; la consulta de directorio, selección de archivos y detalles no cambian |
| Pulsar el nombre o Intro | Entrar usando el ID de directorio de la sesión actual; tras sustituir correctamente la consulta, restablecer la consulta del directorio actual, selección de archivos y detalles según el contrato existente |
| Arriba / abajo | Mover el foco entre filas visibles sin cambiar automáticamente la lista de archivos |
| Derecha | Expandir un padre contraído y, si ya está expandido, pasar al primer hijo; las hojas permanecen en su sitio |
| Izquierda | Contraer un padre expandido; de lo contrario, ir al padre. Los nodos superiores permanecen en su sitio |
| Contraer un antepasado del directorio actual | Mantener el directorio actual y la lista; el foco oculto vuelve al antepasado visible más cercano |
| Nueva sesión de archivo | Borrar los ID antiguos, expansión, foco y puntos de rama; no reutilizar el estado de otro archivo |

Por ejemplo, con la ruta `01 Photos / Glacier Lake`, contraer `01 Photos` mantiene Glacier Lake a la derecha. La fila Photos muestra solo un punto azul al final, sin recibir el fondo de directorio actual de Glacier Lake; tampoco la raíz pasa a ser el directorio actual. El punto azul aparece en el **antepasado contraído visible más cercano**; cada directorio actual oculto tiene un marcador de rama. Al volver a expandir se restaura el fondo actual de Glacier Lake.

El árbol completo es una única entrada de navegación por teclado. Los nodos conservan semántica accesible de ID estable, nombre, nivel, expansión, posición y recuento de registros. Las hojas no se anuncian como expandibles; carpetas, líneas y puntos decorativos no se anuncian de forma redundante. Las rutas actuales ocultas tienen descripciones accesibles y no dependen solo del azul. Las teclas de flecha y los clics en flechas no deben propagarse a la activación de la fila.

## Cobertura de estados y recuperación

| Estado | Árbol de directorios | Lista de archivos / recuperación |
| --- | --- | --- |
| Lectura | Mensaje de carga, recuento desconocido; el esqueleto es un recurso futuro; navegación desactivada | Seguir el estado global de carga; sustituir el árbol una vez después de publicar el índice completo |
| Archivo vacío | Sin número en la fila raíz; «Este archivo está vacío» | Lista vacía; se puede abrir otro archivo |
| Solo archivos raíz | Sin número en la fila raíz; «No hay subdirectorios» | Mostrar seis archivos normalmente; no convertir toda la página en estado vacío |
| Límite del índice de directorios superado | No publicar medio árbol; mostrar «El directorio supera el límite de visualización» | Continúan raíz, búsqueda y exploración de directorios explícitos admisibles según las capacidades actuales |
| Fallo de lectura | Atenuar la navegación y mostrar un error explícito | Volver a abrir el archivo; no interpretar el fallo como cero o directorio vacío |
| Nodo con subdirectorios | Flecha independiente en cada nivel | Expandir no entra en ese directorio |
| Hoja no vacía / vacía | Recuento conservado en datos; ninguna muestra recuento de fila ni flecha | El nombre sigue siendo activable; se puede explorar un directorio vacío |
| Nombre largo / anidamiento profundo | Nombre completo, ruta accesible, desplazamiento horizontal y anchura según nodos visibles | El filtrado de rutas sigue siendo coherente |
| Fuente al 150 % (200 % es solo un caso histórico de esfuerzo) | Espacios y filas crecen; las líneas se centran de nuevo | Nombres, flechas y foco no quedan recortados |
| Directorio actual oculto | El antepasado visible más cercano muestra un punto de rama | La lista conserva la ruta actual; al expandir se restaura la fila actual |

Los límites de navegación y los fallos globales de lectura son estados distintos; los motivos, acciones disponibles y fuentes de recuento no pueden compartir una plantilla de «directorio vacío». Los presupuestos existentes de 32.768 nodos, 64 niveles y 4 MiB de nombres proceden del contrato de solo lectura; el diseño no los aumenta ni elude.

## Entrega de recursos e integración real

Los 26 recortes en píxeles originales están en `design/assets/reference/tree-*.png`, cada uno con origen, SHA-256, `x,y,width,height` y propósito. `component_reference_coverage` apunta individualmente a vectores compartidos o componentes dinámicos. No describa imágenes estáticas de filas completas como controles interactivos ni elimine fondos blancos de los recortes para fabricar transparencia.

| Parte | Recurso / dibujo | Integración actual |
| --- | --- | --- |
| Carpeta, flechas derecha/abajo | `folder`, `chevron-right`, `chevron-down` de recursos compartidos | Recursos compartidos existentes disponibles; iconos y dimensiones B usados por filas HTML recursivas y delegados Qt |
| Carga, información, advertencia | Nuevos `tree-spinner`, `tree-info`, `tree-warning` | Parte de ocho SVG claros/oscuros y 24 PNG transparentes; conservar las formas rellenas y segmentadas de las mesas en vez de usar iconos de contorno existentes |
| Rama actual oculta | Nuevo `tree-active-branch` | Seis PNG transparentes entregados para dos temas; HTML y Qt usan el recurso para marcar el antepasado contraído visible más cercano |
| Líneas, fondo actual, foco, esqueleto, avisos | Dibujados dinámicamente con dimensiones y temas del archivo de parámetros | Estilo y lógica especificados; no se entrega un segundo conjunto de controles de mapa de bits |
| Mesas oficiales | `hifi/screens/directory-tree*.png` | Conservar los bytes de los originales confirmados para comparar el diseño |

HTML y Qt integran líneas de conexión, puntos de rama, espacios de carpeta de 16 unidades y menús contextuales de directorio. HTML usa una jerarquía de ejemplo recursiva completa; Qt usa el índice real acotado. Contraer el antepasado actual no envía una consulta. Los esqueletos de carga y los iconos específicos de información y error siguen siendo recursos de diseño; la mesa oscura completa está pendiente. La integración de iconos no demuestra la aceptación de lectores de pantalla o HiDPI en cada plataforma; las conclusiones manuales existentes siguen [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5).

## Aceptación tras la integración

1. Capture la región real de directorios con la misma anchura lógica de 1024, configuración de fuente, DPR y datos de estado, y compárela con la región correspondiente de la mesa confirmada y la especificación. Registre la relación de escala y conserve por separado el juicio visual y las comprobaciones estructurales.
2. Alinee los inicios de flecha, icono y nombre en cada nivel, sin columna numérica de fila. Iconos, texto, puntos y flechas comparten el centro de fila. La desviación geométrica objetivo es de un píxel físico como máximo; registre la plataforma y la fuente alternativa real ante diferencias.
3. Expanda al menos tres niveles, incluida una rama profunda contraída y una hoja no vacía. Acepte la continuidad y terminación de las líneas de conexión hasta los últimos hermanos y a través de subárboles expandidos.
4. Compruebe los estados actual, cursor y foco por separado y juntos. Los clics en flechas no envían consultas de directorio ni borran la selección de archivos; la activación con Intro sigue la regla de restablecimiento tras una consulta satisfactoria.
5. Con Glacier Lake como directorio actual, contraiga Photos: la ruta de la lista permanece, la raíz no se selecciona y Photos solo muestra un punto de rama. Vuelva a expandir para restaurar la fila actual. La navegación por teclado sigue siendo posible con el foco oculto.
6. Compruebe archivos vacíos, archivos raíz, directorios vacíos explícitos, fallo de lectura y límite de directorios con un índice real acotado; compruebe por separado recuentos, desactivación y acciones de recuperación.
7. Con nombres largos, fuentes al 100 %/125 %/150 % y el caso histórico de esfuerzo al 200 %, ventana mínima y profundidad máxima permitida del índice, los nombres permanecen completos, el desplazamiento horizontal afecta solo a la jerarquía y las rutas completas son accesibles. El desplazamiento del árbol no mueve el resumen del archivo ni las estadísticas inferiores.
8. Complete la aceptación de representación real clara/oscura, densidad normal/Retina/alta de Windows, teclado y lector de pantalla. Superar las comprobaciones de imágenes o la generación de recursos no sustituye estas verificaciones en ejecución.

Según el [método de verificación](../../../developers/qa/README.md), compruebe por separado originales, recortes, recursos y el comportamiento real de ratón, teclado, menú y árbol. Indique explícitamente la accesibilidad y las diferencias de plataforma no probadas.
