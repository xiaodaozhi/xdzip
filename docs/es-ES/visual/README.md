# Especificaciones visuales de xdzip

[English](../../visual/README.md) | [简体中文](../../zh-CN/visual/README.md) | [繁體中文](../../zh-TW/visual/README.md) | [日本語](../../ja-JP/visual/README.md) | [한국어](../../ko-KR/visual/README.md) | [Français](../../fr-FR/visual/README.md) | [Deutsch](../../de-DE/visual/README.md) | Español | [Português (Brasil)](../../pt-BR/visual/README.md) | [Русский](../../ru-RU/visual/README.md)

Entradas de especificación: [norma de diseño de interfaz](../../visual/design-standard.md), [flujo obligatorio de diseño](design-workflow.md) y [contrato de densidad y unidades](../../../specs/visual/layout-scale.json). La nueva verificación de proporciones de la ventana principal y los menús se registra mediante el [método general](../../../developers/qa/README.md).

Los originales actuales están en [design/hifi](../../../design/hifi/README.md); la entrega HTML interactiva está en [design/html](../../../design/html/README.md). Las propuestas tempranas no adoptadas no son la base actual. No dependa del historial reorganizado de main para localizar diseños antiguos; rigen los originales y el manifiesto actuales.

[tokens.json](../../../specs/visual/tokens.json) contiene parámetros de tema e implementación compartida; las entradas visuales legibles por máquina están en `specs/visual/`. La geometría y fuentes de la ventana principal se generan desde [qt-main-reference.json](../../../specs/visual/qt-main-reference.json), congelado independientemente a partir de originales de alta fidelidad y mediciones HTML aceptadas, con escala 1024/1548. La base anterior de ancho 1280 se convirtió a densidad normal. Redondeo, capturas reales y capacidades sin implementar se registran por separado según el [método de verificación](../../../developers/qa/README.md); los parámetros de implementación no pueden convertirse en su propia base de aceptación del diseño.

## Fundamento de investigación y adaptación

1. [Interfaz oficial de Bandizip para Windows](https://www.bandisoft.com/bandizip/screenshots/): conservar árbol de directorios, tabla de archivos, entrada clara de extracción y barra de estado; simplificar la barra grande y colorida en texto con iconos de líneas finas y destacar una acción principal.
2. [Interfaz oficial de Bandizip para macOS](https://www.bandisoft.com/bandizip.mac/screenshots/): revisar la demostración oficial de navegación de archivos y el panel de creación de archivos; mantener el flujo de explorar antes de elegir contenido y destino de extracción.
3. [Instrucciones de extracción de Bandizip](https://www.bandisoft.com/bandizip/help/how-to-unzip/): abrir un archivo no lo extrae automáticamente. Las tareas comienzan solo después de que el usuario confirme destino, alcance y gestión de conflictos.
4. [Interfaz oficial de Transmit](https://panic.com/transmit/): tomar como referencia la densidad de información del gestor de archivos, la navegación de rutas y los separadores discretos.
5. [Renovación de diseño de Linear de 2026](https://linear.app/now/behind-the-latest-design-refresh): tomar como referencia barras laterales más discretas, navegación compacta, bordes suaves y fondos neutros que destaquen el contenido central.

La investigación se realizó el 2026-10-11 con capturas, demostraciones y artículos oficiales públicos; no se afirma que representen la versión instalada más reciente de cada producto. Los materiales de investigación de terceros permanecen en un espacio experimental separado y no se incluyen en este repositorio.

## Sistema visual

Use fondos gris cálido, acciones principales cobalto y un tema oscuro grafito. No sustituya la jerarquía de información con grandes degradados o fondos esmerilados. Las listas normales de archivos mantienen una estructura plana clara; las sombras se reservan para superposiciones. La izquierda responde «¿Dónde?», el centro «¿Qué hay aquí?» y la derecha «¿Qué está seleccionado?».

HTML y Qt comparten glifos y métricas de [Xdzip Design Sans SC](../../../design/assets/fonts/README.en.md); Qt usa TTF de la misma fuente y alternativas de plataforma para nombres de archivo fuera del subconjunto. El texto de la lista principal mide 13, cabeceras y barra de herramientas 12 y títulos de panel 13 píxeles lógicos, redondeados a píxeles enteros de fuente Qt. Los nombres de archivo conservan nombres accesibles completos; el truncamiento visual usa puntos suspensivos y una indicación al pasar el cursor con el nombre completo. Los iconos de archivo comparten un espacio de aproximadamente 31.75 píxeles lógicos sin compensación de tamaño por tipo.

La ventana principal inicial mide 1024 × 622 y el mínimo es 832 × 544 píxeles lógicos. Las medidas de origen de barra lateral y vista previa son aproximadamente 193.16 / 257.98, redondeadas a los límites estructurales Qt 193 / 258. Las filas de directorio miden aproximadamente 43 y las de archivo 44.65. Los títulos de panel tienen altura 46, cabeceras de tabla 34 y búsqueda y botón principal aproximadamente 33.74. El árbol de directorios B conserva por separado altura 32 y espacio de icono 16. Por debajo de ancho 928 la vista previa se contrae y se abre mediante un botón en una ventana nativa independiente. Los ajustes ofrecen tamaño de fuente 100%/125%/150%; el 200% histórico se lee como 150%. Las fuentes grandes usan distribución real, desplazamiento y una ventana independiente de detalles en lugar de escalado global para ocultar desbordamientos. El arrastre de paneles y la persistencia de su ancho no están implementados.

## Interacción de la ventana principal

- Qt admite sesiones de archivo independientes y crea pestañas tipo cápsula o ventanas principales según el modo de apertura guardado; volver a abrir activa una sesión existente. Las páginas conservan ruta, selección, orden y desplazamiento; las tareas continúan al cerrar páginas. Véase el [contrato de sesiones](../engineering/application-sessions.md). El comportamiento de varias pestañas HTML solo valida el diseño.
- El botón principal es «Extraer todo» sin selección y «Extraer selección» con selección; el desplegable siempre ofrece el otro alcance.
- La tabla de archivos admite ordenación, selección múltiple, selección por intervalo y navegación de teclado. Un doble clic en una carpeta entra en ella; en un archivo ordinario extrae una copia temporal y la abre con el sistema. La tarjeta de propiedades muestra automáticamente una vista previa de un único archivo compatible. Véase el [contrato de vista previa](../engineering/content-preview.md) para alcance y seguridad. Las fotos de ejemplo HTML no son resultados de decodificación reales.
- La búsqueda cubre recursivamente el directorio actual y sus descendientes; la búsqueda desde raíz cubre todo el contenido. Los resultados muestran rutas completas dentro del archivo, mientras las listas ordinarias de directorio muestran los nombres finales.
- Los detalles pueden cerrarse y volver a abrirse. La vista previa requiere lectura asíncrona y límites de recursos; los elementos no compatibles muestran propiedades y entrada de extracción. Los resultados tardíos de vista previa se descartan tras cancelar la tarea.
- La barra de estado muestra el número seleccionado y tamaño original, estado del índice y número de entradas del historial de tareas reabribles. El número de tareas activas controla por separado avisos y protección de salida. Un índice incompleto se marca como «Leyendo» sin inventar un recuento completo.

## Interacción del panel de extracción

El panel agrupa archivo, destino, alcance de selección, política de conflictos y comportamiento al completar. La selección de ejemplo es 8.4 + 6.2 + 12.8 = 27.4 MB. La lista de archivos es un resumen; las listas mayores ofrecen «Ver todo».

El destino predeterminado es el último directorio. «Crear carpeta con el nombre del archivo» actualiza inmediatamente la vista previa de la ruta final para evitar anidamientos redundantes. Muestre texto para errores de permisos de ruta, espacio libre y lectura del archivo; la comprobación es un estado distinto y el espacio no confirmado es «Desconocido».

El valor predeterminado ante conflictos es Preguntar; también hay Omitir, renombrado automático y Sobrescribir. Señale claramente las consecuencias de sobrescribir. Durante la ejecución muestre la etapa real, número de archivos procesados, progreso en bytes y velocidad cuando pueda calcularse; use progreso indeterminado si se desconocen los totales. Tras la cancelación muestre «Deteniendo» hasta que el proceso de fondo confirme que se ha detenido.

## Estados de componentes y accesibilidad

| Componente | Estados y respuesta requeridos |
| --- | --- |
| Botón principal | Normal, cursor encima, pulsado, foco de teclado, deshabilitado y enviando; impedir inicios duplicados |
| Fila de archivo | Normal, cursor encima, seleccionada, seleccionada sin foco de ventana y foco de teclado; la selección cambia más que el color del texto |
| Búsqueda | Vacía, entrada, buscando, sin resultados y borrar; Escape retira el foco conservando el texto |
| Vista previa | Sin selección, cargando, cargada, no compatible, error y cancelada |
| Tarea | Esperando, ejecutándose, esperando contraseña o decisión de conflicto, deteniéndose, completada, fallo parcial y fallo |
| Panel modal | Restricción de foco, Escape cierra un panel no enviado y el foco vuelve al botón que lo abrió |

Tras implementar en Qt verifique el contraste WCAG AA del texto para los colores del tema. La apariencia y salida del foco siguen la norma de diseño actual sin dibujar anillos de foco adicionales. Proporcione nombres y anuncios de estado para lectores de pantalla y respete la preferencia del sistema de movimiento reducido. Las transiciones ordinarias sugeridas son 120–160 ms; no anime continuamente efectos decorativos de la lista de archivos.

## Restricciones de implementación Qt Quick

Qt usa `Theme` compartido y dimensiones generadas. `ArchiveTabs`, `ArchiveToolbar` y las secciones de lista, cuadrícula y directorio de `SessionPage` comparten la distribución principal. `InspectorPanel`, `ExtractWindow`, `CreateWindow` y `TaskWindow` conservan sus propios límites de datos y distribución. QML presenta estado e intención del usuario. El runtime Rust es responsable del índice, lectura y escritura de archivos y validación de formatos.

macOS, Windows y Linux comparten distribución de contenido y modelos de interacción; controles de ventana, menús y atajos siguen las convenciones de la plataforma. El marco macOS ilustrado no exige que las demás plataformas copien los botones de semáforo. El desenfoque grande en tiempo real no es requisito de implementación; una máscara estática puede reproducir la atenuación del fondo en capturas.

Revise la ventana principal en [claro](../../../design/hifi/sources/main.html?theme=light) u [oscuro](../../../design/hifi/sources/main.html?theme=dark), junto con [estados de tareas](../../../design/hifi/sources/task-windows.html), [ajustes](../../../design/hifi/sources/settings.html) e [integración de escritorio](../../../design/hifi/sources/desktop-integration.html). Use el [registro de ventanas](../../../specs/visual/window-registry.json) para localizar otros estados o excepciones de tamaño. Una página de diseño puede mostrar el contenido de una ventana o menú nativo. La decoración, activación, escala y comportamiento de menú reales requieren la plataforma correspondiente.

## Revisión de ventanas de plataforma

Los paneles de operación pasan uniformemente a ser [ventanas de plataforma independientes](../ux/platform-windows.md). El botón de cierre en el área de contenido, el panel redondeado y la máscara de las antiguas ilustraciones de extracción se sustituyen por una barra de título independiente y decoración de plataforma; se conservan colores, tipografía y estructura del contenido. Los iconos actuales de tipo de archivo, la alineación de listas y las barras de título de las tres plataformas tienen un prototipo interactivo como material de revisión complementario.

## Originales y biblioteca de recursos compartidos

Los [originales completos](../../../design/hifi/README.md) están en `design/hifi/screens/`, incluidos la ventana principal, confirmación de extracción y ocho paneles de estados de operación. Véase el [contrato de estados](../ux/hifi-state-spec.md) para funciones y recuperación. Conserve los bytes originales; originales y recortes no deben modificarse para ajustarse a la implementación.

La [biblioteca de recursos compartidos](../../../design/assets/README.md) reúne los recursos de la ventana principal y estados de operación en lugar de dividirlos por lote de entrega. SVG es la única fuente gráfica de iconos transparentes de runtime, incrustados directamente en HTML y renderizados como PNG de múltiples resoluciones de la misma fuente en Qt. Cuarenta y cinco recortes de comparación de píxeles originales en `reference/` conservan imágenes originales, recortes y fondos; tres fotos de ejemplo HTML están en `demo/`. Un manifiesto identifica sus fines para evitar que recortes con fondo se confundan con capas transparentes.

Todos los iconos transparentes comparten un lienzo 24×24. Los iconos PDF, texto, código y Markdown comparten centro y altura visible de 22 píxeles, manteniendo distintos anchos de silueta y estilos internos. Las carpetas usan una silueta visible horizontal centrada de 22×aproximadamente 18.86. Esta restricción de altura se aplica a iconos de runtime, no a recortes originales. Los PNG 1×/2×/3× son 24×24, 48×48 y 72×72; la escala no cambia el tamaño lógico. Las listas HTML usan espacios de 48×48 en coordenadas de origen; Qt usa aproximadamente 31.75×39.69 píxeles lógicos tras la conversión equivalente. Elimine ampliación, traslación y compensación de trazo CSS por tipo.

Las migas de pan de la lista y los títulos de vista previa/detalles de la derecha comparten altura de barra, tamaño de fuente y centro vertical del texto. HTML usa variables compartidas de barra de título; Qt usa `PaneHeader` y `Theme.mainPaneTitleSize`. Los títulos permanecen sobre el contenido desplazable y las fuentes grandes aumentan juntas las alturas de ambos paneles. Las mediciones reales se conservan en artefactos de ejecución según el [método de verificación](../../../developers/qa/README.md).

Las vistas previas fotográficas HTML comienzan justo bajo el título del panel, con la parte superior alineada con la cabecera de la lista de archivos y sin espacio superior adicional. Cuando aparecen indicaciones de búsqueda, alinee según el espacio que ocupan realmente. Mantenga la altura de imagen existente y el espaciado inferior. La nueva comprobación de alineación fotográfica cubre dos imágenes de ejemplo, temas claro/oscuro, tres decoraciones de ventana y cambios de búsqueda.

Las cabeceras y valores de tamaño original y comprimido de la lista se alinean a la derecha; la cabecera y contenido de tipo se alinean a la izquierda, con relleno de contenido compartido por columna. HTML mantiene anchos de columna y alturas de fila originales: relleno derecho de columnas de tamaño de 26 y 22 píxeles de origen y relleno izquierdo de columna de tipo de 28 píxeles de origen. La nueva verificación registra los bordes de distribución de texto de todas las filas visibles.

Registre por separado la silueta de icono, las dimensiones del recorte en píxeles originales y las dimensiones del lienzo de runtime. No es necesario estirar carpetas para darles forma de papel; los SVG no deben deformarse para llenar espacios. Haga cambios visuales en la fuente gráfica, revíselos en la galería de recursos y sincronice ambas implementaciones.

## Revisión del árbol de directorios B (2026-10-12)

Tres paneles formales definen expansión multinivel, interacción de nodos y límites de navegación; véase la [especificación del árbol](../ux/directory-tree.md). Veintiséis recortes de píxeles originales entran en `assets/reference` compartido; los puntos de ramas, iconos de estado rellenos e indicadores segmentados usan SVG para dos temas y PNG en tres resoluciones. Las líneas de conexión, fondos de estado, foco y texto se dibujan dinámicamente desde los [parámetros de componentes](../../../specs/visual/directory-tree.json); estos no sustituyen automáticamente los tokens globales. Los paneles son explicaciones visuales ampliadas; las dimensiones lógicas exactas y rectángulos de píxeles de origen se registran por separado. HTML y Qt integran directorios recursivos, líneas de conexión y marcadores de rama actual oculta; los iconos y esqueletos específicos de límites siguen siendo recursos de diseño y queda pendiente un panel oscuro completo.

## Revisión de la barra del diseño principal actual (2026-10-12)

Las sugerencias generales anteriores de controles ya no rigen la barra principal. Las medidas actuales del original son altura de 51 píxeles de origen, fuente de 18 e icono de 32, convertidas por 1024/1548 a aproximadamente altura 33.74, fuente entera 12 y lienzo de icono 21.17. La altura de la región de barra es 53. La conversión anterior de ancho 1280 con 42.17/15/26.46 fue sustituida por densidad normal; las ventanas ordinarias de operación conservan sus mapeos registrados.

Las acciones ordinarias izquierda/derecha y las acciones de búsqueda/principal forman grupos. Las fuentes grandes pueden aumentar la altura y la búsqueda puede reducirse para mantener visibles todas las acciones derechas con tamaño mínimo de ventana. El texto deshabilitado no recibe opacidad de grupo; el botón principal tiene su propio fondo deshabilitado. El botón principal de extracción conserva la apariencia deshabilitada de divisor y flecha; la extracción real y los menús de alcance siguen la capacidad de runtime. Sin selección, el alcance seleccionado y la comprobación previa de metadatos están deshabilitados; la comprobación de integridad sigue la capacidad del archivo. Registre la nueva validación según el [método general](../../../developers/qa/README.md).

## Marca de la aplicación e iconos de plataforma

Los iconos de documentos de sistema para cada formato usan la [especificación de iconos de asociación](file-association-icons.md) y la [página de revisión de alta fidelidad](../../../design/hifi/sources/file-association-icons.html). Esta familia reutiliza la geometría de marca inferior con etiquetas de formato, acentos y simplificaciones de tamaño pequeño diferenciados como recursos de diseño para una futura integración por formato.

La marca usa cuatro bandas entrelazadas y una parte inferior abierta en U; su geometría se mantiene en `scripts/brand_assets.py`. La marca independiente, el icono de aplicación y el maestro de documento ZIP están en `design/assets/brand/`. Los elementos UI `app-mark`, `archive` y `archive-grey` identifican la aplicación, el archivo azul y el archivo gris, generados todos desde la misma geometría de marca. La aplicación usa un exterior azul redondeado y los documentos papel con una esquina doblada: comparten identidad mientras los marcos cumplen funciones distintas.

La marca actual mantiene una relación esbelta 2:3 y no se estira al ancho de la marca anterior. Su base usa una curva de curvatura continua con simetría rotacional en las cuatro esquinas, aumentando suavemente la curvatura desde cero en bordes rectos hasta el punto medio diagonal, evitando un centro biselado. Iluminación y base comparten una trayectoria; no superponga trazos de radios distintos ni líneas de brillo abiertas. Las insignias de documento usan la misma definición de contorno.

Esta es una revisión explícita de marca nueva, no una reproducción indistinguible del icono de cremallera ZIP de los antiguos originales de alta fidelidad. Diez originales completos, 45 referencias de iconos de píxeles originales y tres fotos de ejemplo conservan los bytes originales. El nuevo `brand/identity-reference.png` generado es una referencia de revisión de la dirección actual de marca, no un recurso de runtime. El manifiesto principal registra la cobertura de referencias y etiqueta explícitamente las referencias de la marca anterior como funciones históricas de diseño.

Los iconos pequeños de interfaz conservan lienzos 24×24. Los iconos de plataforma exportan PNG transparentes de 16/24/32/48/64/128/256/512/1024 píxeles. Los tamaños 16/24/32 simplifican texto ilegible e iluminación de imagen grande mientras mantienen las bandas entrelazadas y la estructura reconocible en U. Los iconos de aplicación y documento ZIP ofrecen cada uno ICO de siete tamaños e ICNS con once representaciones de densidad. No amplíe indefinidamente un icono UI de 24 unidades para uso del sistema ni reduzca sombras grandes hasta producir ruido a tamaño pequeño.

Los iconos de aplicación del sistema están integrados en ventanas Qt, paquetes macOS, recursos de ejecutables Windows y lanzadores de escritorio/directorios hicolor Linux. Los formatos de plataforma del documento ZIP se usan en instaladores y asociaciones; las solicitudes de apertura del sistema se dirigen a la navegación real mediante enrutamiento unificado. Véase el [contrato de instalación y asociación](../engineering/installers-associations.md). Registre solo candidatos de apertura sin cambiar valores predeterminados del usuario. Registre la verificación de plataforma según el [método general](../../../developers/qa/README.md).

Las imágenes ausentes de tema, éxito, cancelación, conflicto y progreso se completan desde referencias de píxeles originales como recursos semánticos distintos. La ilustración grande de cancelación usa un símbolo cuadrado de parada; el tema de sistema conserva la estructura completa de ventana. HTML usa entradas de opción nativas, mientras Qt usa IntentRadioButton para dirigir acciones de ratón, teclado y accesibilidad a un único modelo de selección; los recursos solo gestionan la apariencia. Registre por separado comprobaciones de generación, visualización en plataforma de destino y diferencias con los originales; una aceptación existente no sustituye los resultados de esta revisión.

## Especificación de controles interactivos

[controls.json](../../../specs/visual/controls.json) define columnas de menú, alturas de fila, radios, colores, foco y tiempos de indicaciones; [directory-tree.json](../../../specs/visual/directory-tree.json) define la jerarquía de directorios. Ambos generan parámetros de control HTML y QML Theme; las comprobaciones evitan duplicar manualmente dimensiones y temas. Las proporciones de píxeles lógicos y de origen del diseño principal proceden de `display-spec.js`; la [página de revisión de componentes](../../../design/html/components.html) usa dimensiones lógicas 1:1 mientras el lienzo principal usa coordenadas de origen. Las listas de funciones y contratos de entrada están en la [descripción de componentes de la ventana principal](../ux/main-window-components.md); el renderizado y comportamiento reales se registran en issues y artefactos correspondientes según el [método de verificación](../../../developers/qa/README.md).
