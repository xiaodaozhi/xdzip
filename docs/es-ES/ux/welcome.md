<a name="xdzip-main-window-welcome-page"></a>

# Página de bienvenida de la ventana principal

[English](../../ux/welcome.md) | [简体中文](../../zh-CN/ux/welcome.md) | [繁體中文](../../zh-TW/ux/welcome.md) | [日本語](../../ja-JP/ux/welcome.md) | [한국어](../../ko-KR/ux/welcome.md) | [Français](../../fr-FR/ux/welcome.md) | [Deutsch](../../de-DE/ux/welcome.md) | Español | [Português (Brasil)](../../pt-BR/ux/welcome.md) | [Русский](../../ru-RU/ux/welcome.md)

El diseño de alta fidelidad está confirmado. Qt implementa apertura y creación reales; la aceptación manual de escritorio se registra por separado. Consulta el diseño de estados de bienvenida y [welcome-reference.json](../../../specs/visual/welcome-reference.json). Los colores y la densidad de la ventana conservan los tokens y layout-scale existentes sin cambiar las normas globales. Cuatro PNG son exportaciones de originales dibujados mediante código, no capturas del producto en ejecución.

<a name="xdzip-information-and-layout"></a>

## Información y disposición

El titular de inicio es «打包与解压，都简单一点。» en chino y «Pack and unpack, a little simpler.» en inglés; el catálogo español usa «Comprime y extrae con facilidad.». El texto complementario describe navegar primero, extraer lo necesario y comprimir para compartir.

El área cliente predeterminada mide 1024×622, mínimo 832×544; área de título 48, cápsula 32 y barra inferior 32. El grupo de contenido de ancho 608 se centra solo horizontalmente y comienza 28 píxeles lógicos bajo el borde superior disponible, sin centrado vertical. El titular mide 30, o 28 en una ventana estrecha; descripción 13, título de acción 16 y descripción de tarjeta 12. Dos tarjetas de acción miden 296 de ancho por 124 de alto, con separación 16, relleno 20 y radio 8. Son tarjetas de entrada de bienvenida, no sustitutos de los botones compartidos de ventanas de operación.

La barra de título reutiliza por completo los componentes existentes de la ventana principal, conservando marca, pestaña inicial, añadir pestaña y ajustes de apariencia; bienvenida no cambia su disposición, dimensiones, fuentes ni iconos. El cuerpo no muestra árbol de archivos vacío, cabecera de tabla, búsqueda ni barra de extracción; la navegación aparece tras abrir un archivo. La acción principal es «Abrir un archivo comprimido», la secundaria «Crear archivo comprimido»; después del área de arrastre aparecen los tipos admitidos. Cada categoría muestra su número de extensiones registradas. «Archivos comprimidos» está abierta inicialmente. Activar otra categoría cambia la vista previa; activar la misma la contrae. El color de acento y el peso marcan la categoría activa. La vista previa muestra extensiones en una fila de etiquetas con relleno ligero y borde, con la fuente normal de interfaz. Una elipsis indica desbordamiento; pasar sobre la fila muestra sufijos de la categoría elegida. «Asociaciones de archivos» abre los controles existentes en Ajustes. El mensaje de procesamiento local de la barra inferior describe dónde se procesa, no garantiza seguridad. El número de tareas usa el historial real; cero en el diseño representa historial vacío de primer uso.

No mostrar archivos recientes de ejemplo, tamaños ficticios ni capacidades de formato que el producto no tenga. Los controles de semáforo indican solo la posición de decoración macOS; la implementación usa mecanismos de ventana de la plataforma. Windows y Linux conservan su decoración sin copiar los semáforos.

<a name="xdzip-interaction-design"></a>

## Diseño de interacción

- La bienvenida puede aparecer en el primer inicio, tras cerrar la última pestaña de archivo o tras fallar una apertura sin ningún archivo navegable. Las tareas en segundo plano continúan y no se reinician.
- Abrir llama al selector de archivos del sistema; cancelar conserva la bienvenida. Si tiene éxito cambia a la pestaña real del archivo; si falla muestra un error recuperable y no fabrica una lista satisfactoria.
- Crear abre la ventana de creación existente; cancelar conserva la bienvenida. Los atajos propuestos son ⌘O/⌘N en macOS y Ctrl+O/Ctrl+N en Windows/Linux. Comprobar las asignaciones reales antes de implementar; este diseño no afirma que estén integradas.
- Arrastrar una selección admisible resalta el área con borde de acento de 2 píxeles lógicos y relleno de selección. Al salir vuelve al estado normal. Consulta [Integración real de Qt](#xdzip-real-qt-integration) para la apertura efectiva.
- Se puede pulsar toda la tarjeta. El orden de teclado tras la barra de título es abrir, crear y tareas. El foco no añade contorno y sigue la regla general de pérdida de foco de la ventana. Pasar y pulsar usan accentHover y accentPressed compartidos, no solo opacidad.
- Durante la apertura, la tarjeta sustituye su título por un indicador ocupado y «Abriendo…» e impide temporalmente abrir de nuevo y soltar; el aspecto desactivado usa accentDisabled y accentDisabledText. Que la ventana de creación pueda ejecutarse a la vez depende de las reglas del host; la bienvenida no cambia la semántica de tareas. Los estados ocupado, desactivado y error son contratos de implementación no representados en los cuatro originales estáticos.
- Los menús de ajustes y tareas conservan ancho 240, fuente 13 y altura mínima de fila 32, proporción 1:1 con la fuente de cuerpo 13; bienvenida no introduce otro estilo de menú. La fuente de tarjeta 16 es título de entrada, no base para comparar densidad de menú.

<a name="xdzip-verification-and-boundaries"></a>

## Verificación y límites

Cubrir temas claro/oscuro, ventana mínima, estados de arrastre y proporciones menú/barra de título, comprobando DPR 1, 1,25 y 2 y el zoom real del navegador. Una captura reducida no sustituye la disposición ni el escalado CSS sustituye el zoom del navegador. Resultados, imágenes y mediciones geométricas solo pertenecen a artefactos locales o de CI; el estado se registra en el issue correspondiente.

Qt usa pruebas de regresión de bienvenida y navegador y abre archivos por la entrada real del producto. Las pruebas automáticas no sustituyen la verificación nativa de arrastrar y soltar, lector de pantalla o IME.

<a name="xdzip-real-qt-integration"></a>

## Integración real de Qt

WelcomePage y WelcomeAction expresan solo disposición e intención. La bienvenida aparece al iniciar y al cerrar un archivo; la apertura correcta entra en la sesión correspondiente. Un archivo vacío sigue siendo un estado de navegación válido. La carga inicial desactiva abrir repetidamente y crear, muestra «Abriendo…» y permite cancelar la carga. Un fallo conserva la bienvenida y el error real para volver a seleccionar. Cerrar un borrador de creación no crea una tarea. Las listas de tareas en segundo plano no se reinician.

Abrir y los atajos de sistema reutilizan chooseArchive; crear reutiliza CreateWindow. El selector nativo sigue siendo una ventana del sistema y cancelarlo no crea sesión. Qt dirige según el modo guardado de pestaña/ventana, reutiliza preferentemente una página inicial vacía y activa sesiones existentes para archivos duplicados.

Con el gestor de sesiones de escritorio, la bienvenida puede abrir hasta 128 archivos locales en un arrastre. Cada elemento debe coincidir con un sufijo generado para arrastrar y soltar o un patrón admitido de volumen dividido. La entrada de desarrollo de sesión única acepta un elemento. Se rechazan directorios y URL no locales. Estas comprobaciones eligen una vía de apertura; el entorno Rust sigue identificando el formato real y validando la fuente. Usa **Abrir un archivo comprimido** para formatos con entrada solo de apertura explícita. Consulta el [registro de formatos](../engineering/format-registry.md).

Las dimensiones llegan desde welcome-reference.json a Theme mediante generate-qml-theme.py. El texto más grande amplía tarjetas y cuerpo; el cuerpo se desplaza y muestra automáticamente los elementos con foco de teclado. Estado y título crecen independientemente sin Item.scale. Qt usa iconos PNG compartidos y el gráfico existente de trazos claros/oscuros para la acción principal; los recursos de ejecución no se recortan de los originales.

Las entradas de automatización son tst_welcome.qml, welcomeOpensRealArchivesAndReturns en browser_test.cpp y scripts/check-qt-welcome.py. Los flujos Qt bajo demanda en tres plataformas comprueban apertura real, cierre, recuperación de fallos y 36 elementos geométricos con DPR 1, 1,25 y 2, además de ancho de menú 240 y fuente 13. Arrastre nativo, selector de archivos, lector de pantalla e IME se verifican según necesidad; los resultados fuera de pantalla no se presentan como operaciones de escritorio no realizadas.

La bienvenida solo cambia el contenido y reutiliza el mismo componente Qt ArchiveTabs. `screens/welcome-titlebar-*.png` conserva el diseño de la barra de título. La regresión compara la geometría de la barra, pestañas y botón de apariencia antes y después de abrir un archivo.

Estado de carga: la entrada de cancelación de apertura inicial pasó a una superposición flotante del cuerpo. El progreso desconocido usa una barra activa sin porcentajes simulados; consulta el [contrato de superposición de carga](welcome-opening.md). Se conservan los mapas de bits originales; el diseño dinámico ofrece `opening=1`.

Los errores y avisos aparecen bajo las indicaciones de formato con el color de peligro y el icono de advertencia compartidos. Mostrar, ajustar, borrar o sustituir un error no cambia las coordenadas superiores del titular ni de las tarjetas; el contenido largo se desplaza en el cuerpo. Volver a abrir la ventana principal tras cerrarla a la bandeja muestra una sola página inicial y no cancela tareas en segundo plano. Los PNG antiguos conservan sus bytes históricos y su antigua disposición centrada no debe sustituir el contrato de posición superior.
