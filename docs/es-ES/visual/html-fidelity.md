<a name="xdzip-html-visual-fidelity-review"></a>

# Revisión de fidelidad visual HTML

[English](../../visual/html-fidelity.md) | [简体中文](../../zh-CN/visual/html-fidelity.md) | [繁體中文](../../zh-TW/visual/html-fidelity.md) | [日本語](../../ja-JP/visual/html-fidelity.md) | [한국어](../../ko-KR/visual/html-fidelity.md) | [Français](../../fr-FR/visual/html-fidelity.md) | [Deutsch](../../de-DE/visual/html-fidelity.md) | Español | [Português (Brasil)](../../pt-BR/visual/html-fidelity.md) | [Русский](../../ru-RU/visual/html-fidelity.md)

Las imágenes de alta fidelidad son la referencia visual; HTML es la implementación editable. Las dimensiones por encima y debajo de la barra de herramientas se miden independientemente a partir de la [ventana principal inicial congelada](../../../design/hifi/screens/main-light.png), sin deducirlas del CSS actual. La presentación actual usa el [diseño claro con pestañas de cápsula](../../../design/hifi/screens/main-capsule-light.png) y el [diseño oscuro](../../../design/hifi/screens/main-capsule-dark.png), exportados desde la [fuente editable](../../../design/hifi/sources/main.html); no sustituyen la medición independiente. La barra de título se acepta según la revisión explícita de layout-scale.json. Los originales tienen bordes suavizados y glifos generados; el número de pruebas no demuestra identidad perfecta de píxeles en toda la página.

<a name="xdzip-fixed-reproducible-conditions"></a>

## Condiciones fijas reproducibles

- El lienzo de origen coincide con el PNG de 1586×992; la ventana mide 1548×940 en el origen `(20,30)`. Las comprobaciones originales de anclajes siguen usando un área del navegador de 1588×1000 con relación de píxeles de dispositivo 1. Recorta la parte superior en `(20,30)-(1568,167)` y el contenido en `(20,167)-(1568,970)` sin estirar el original.
- Los SHA-256 originales están fijados en [header-reference.json](../../../design/html/header-reference.json) y [body-reference.json](../../../design/html/body-reference.json). Cambiar los originales requiere volver a medir, no sustituirlos silenciosamente.
- La fuente sin conexión incluida [Xdzip Design Sans SC](../../../design/assets/fonts/README.md) se mide después de `document.fonts.ready`. Se fijan pesos reales 400–750 para reducir variaciones de ancho por fuentes alternativas de la plataforma.
- No se puede determinar la familia original a partir de una imagen generada. Los contornos de glifos y el suavizado del sistema aún pueden diferir; registra las diferencias por separado y no afirmes haber restaurado la fuente original.

<a name="xdzip-display-size-and-source-pixels"></a>

## Tamaño de presentación y píxeles de origen

La presentación normal usa la [ventana de 1024 píxeles lógicos de ancho](../../visual/README.md), con una conversión uniforme de coordenadas de origen mediante `1024 / 1548`. La ventana principal se muestra a 1024 × aproximadamente 621,81 píxeles CSS; el original usa la misma escala. Esta relación es una convención de diseño explícita, no una deducción sobre Retina o DPI a partir de un PNG.

`display-spec.js` es la única fuente de dimensiones del lienzo y escalas de presentación. El escritorio usa por defecto el tamaño normal y el móvil el ajuste; este se limita al tamaño normal para evitar ampliaciones automáticas en pantallas anchas. El modo de píxeles originales usa `scale=1`, conserva las dimensiones originales y permite desplazamiento tanto en HTML como en la imagen. La relación de píxeles del dispositivo solo afecta a la densidad de rasterización, no al ancho lógico CSS.

No ajustes el original y el HTML a sus contenedores de forma independiente. Los anclajes de cabecera y cuerpo siguen midiéndose en coordenadas de píxeles de origen; la escala de presentación no modifica espaciados ni tamaños de fuente calibrados. La coherencia de tamaño cubre por separado límites reales de pantalla, tamaño natural original, desplazamiento móvil, vistas previas independientes y salidas sin conexión de un archivo. Conserva las pruebas según el [método de verificación](../../../developers/qa/README.md).

<a name="xdzip-canvas-and-content-scrolling"></a>

## Lienzo y desplazamiento del contenido

El HTML integrado en la herramienta de revisión es un lienzo de tamaño fijo con `overflow:hidden` en el documento raíz. Así se evita que las barras clásicas consuman dimensiones del lienzo y se activen entre sí. La página de revisión se desplaza verticalmente; el lienzo exterior solo se desplaza horizontalmente cuando falta ancho. `scrollbar-gutter:stable` reserva el ancho de la barra clásica para que una altura umbral no cambie repetidamente la escala de ajuste; las vistas previas independientes usan `auto`. Las listas, propiedades largas y ventanas de operación conservan desplazamiento en sus contenedores; los lienzos de origen abiertos directamente y sin integración conservan el desplazamiento de página. Esto no cambia dimensiones del lienzo, ventana ni componentes.

Playwright usa `--hide-scrollbars` por defecto, por lo que las matrices anteriores de capturas y tamaños no cubrían barras clásicas reales. Las comprobaciones de tamaño y vista previa retiran ahora ese argumento y verifican reserva de barra raíz, visibilidad de los cuatro bordes de ventana, desplazamiento horizontal exterior y acceso al último botón de propiedades largas. Un lienzo 1 px demasiado pequeño y un caso negativo sin reglas de integración reproducen y detectan la competencia por espacio de barras. Conserva las capturas de reproducción y corrección en artefactos de ejecución según el [método de verificación](../../../developers/qa/README.md); esto no demuestra el aspecto de las barras en todos los navegadores o sistemas.

<a name="xdzip-header-corrections"></a>

## Correcciones de cabecera

| Zona | Antes | Corregido según el original |
| --- | --- | --- |
| Pestaña activa | Inicio x299, ancho 244 | x293, ancho 246 |
| Pestaña inactiva | x557, ancho 244 | x542, ancho 262 |
| Separación de pestañas | 14 px | 3 px; se permite aproximadamente 1 px de incertidumbre por bordes suavizados |
| Botones de añadir y cerrar | Disposiciones verticales separadas con centros a 5 px | Misma fila de pestañas, con aproximadamente 1 px de desplazamiento óptico respecto al original |
| Controles de cierre | Un botón y un span sin botón | Misma estructura de icono/nombre/cierre y espacios explícitos |
| Icono de ajustes | Gráfico visible pequeño y desplazado arriba a la izquierda | Centro `(1532,61)`; examinar el gráfico real, no solo el contenedor SVG |
| Borde inferior de pestaña activa | Se superponía a la línea de la barra de título | Se une a la superficie blanca de la barra de herramientas, con transición del hombro izquierdo definida por separado |
| Buscar y extraer | Límites horizontales y verticales desplazados | Comparar respectivamente con `(906,101,449,51)` y `(1368,101,183,51)` |
| Iconos izquierdos de la barra | Disposición determinada por tamaños y separaciones genéricos | Posicionar y verificar frente a anclajes gráficos originales |

Los espacios y la tipografía de cabecera se centralizan en las secciones de barra de título, pestañas y barra de herramientas de `hifi.css`, sin depender de dimensiones genéricas `.icon-button` ni de altura de línea heredada. Los temas claro y oscuro comparten geometría. Las decoraciones de título Windows/Linux siguen siendo demostraciones de navegador; las ventanas nativas del producto tienen aceptación independiente.

<a name="xdzip-content-below-the-toolbar"></a>

## Contenido bajo la barra de herramientas

El cuerpo comprende tarjeta del archivo, árbol de directorios, barra de ruta, cabeceras y filas de tabla, selección, vista previa y barra inferior de estado. Registra límites de columnas, alturas de fila, rellenos, centros de iconos y trazos visibles y compruébalos frente a la [referencia independiente](../../../design/html/body-reference.json). Las tolerancias de geometría estricta y trazos aislados de iconos son 2 px; los contornos de texto solo se registran para revisión manual. El tamaño óptico de carpeta es una revisión explícita: el nuevo contorno se amplía uniformemente alrededor del 25,7 % en recursos compartidos. Se conservan cinco ejemplos antiguos para comparación y se comprueban frente a nuevos límites registrados independientemente; los demás anclajes originales siguen sin cambios.

La revisión del cuerpo usa recortes de contenido de igual tamaño y conserva comparaciones regionales de barra lateral, lista, vista previa y estado. Las capturas antes/después, commits y mediciones pertenecen solo a los artefactos del issue o Actions correspondiente, no a la especificación como resultados de una ejecución concreta. La revisión móvil se realiza por zonas y no debe cambiar el diseño de escritorio para ocultar diferencias.

El original selecciona un elemento dejando vacío Seleccionar todo e indica ahorro del 26 % mientras llena la barra aproximadamente al 56 %; son estados estáticos contradictorios. HTML conserva respectivamente la selección parcial `indeterminate` y el llenado real del 26 % como excepciones semánticas documentadas. Las comprobaciones de vista previa cubren desbordamiento normal y desplazamiento de contenido largo; las pruebas de interacción existentes cubren selección, navegación, orden, búsqueda y acciones de ejemplo. Las comprobaciones geométricas no sustituyen la verificación funcional.

Guarda por separado revisión del cuerpo, comparaciones regionales de igual tamaño y mediciones según el [método de verificación](../../../developers/qa/README.md). Conserva las explicaciones sobre fuentes originales desconocidas, bordes suavizados generados, iluminación y diferencias de suavizado; estas pruebas HTML no demuestran la calibración correspondiente del producto Qt.

<a name="xdzip-checks-and-delivery"></a>

## Comprobaciones y entrega

Ejecuta:

```sh
python scripts/build-html-design.py
python scripts/check-html-header.py
python scripts/check-html-body.py
python scripts/check-html-preview.py
python scripts/check-html-references.py
python scripts/check-html-scale.py
python scripts/check-html-design.py
python scripts/build-html-header-review.py
python scripts/build-html-body-review.py
```

Las pruebas de cabecera usan anclajes originales independientes: los límites y centros DOM se desvían como máximo 1 px y los límites reales de trazos de iconos en regiones aisladas como máximo 2 px. El trazo blanco de la flecha desplegable tiene tolerancia de 1 px; el hombro derecho de pestaña muestrea superficie blanca y curva original con tolerancia de un canal de 10/255 para el suavizado. Los contornos de glifos solo se miden y registran. Las superposiciones y diferencias incluyen variaciones de suavizado, iluminación y glifos; no conviertas la diferencia total de píxeles en una similitud inventada.

El archivo de cabecera sin conexión es `dist/xdzip-header-review.html` y el de cuerpo `dist/xdzip-body-review.html`. Ambos alternan entre original, antes, después, superposición transparente y diferencias de píxeles; la revisión del cuerpo añade selección de zona móvil. La interfaz de un archivo es `dist/xdzip-hifi-design.html`.

Ejecuta estas comprobaciones HTML y revisiones sin conexión localmente cuando se necesiten; no forman parte de la CI predeterminada del producto de escritorio. Investiga fallos de anclaje e inspecciona superposiciones incluso si pasa la geometría. Las pruebas HTML de la ventana principal no sustituyen la calibración de otras páginas o del producto Qt ni la aceptación nativa de escritorio.

La densidad normal de ventana principal y menú sigue la [norma de interfaz](../../visual/design-standard.md). Las proporciones de tamaño de fuente de menús y listas Qt deben verificarse juntas. Al revisar HTML localmente, comprueba el zoom real de página según el [método de verificación](../../../developers/qa/README.md); coincidir con coordenadas antiguas de la imagen de origen no lo sustituye.
