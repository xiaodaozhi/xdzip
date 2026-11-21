# Funcionalidad, componentes y contrato de interacción de la ventana principal

[English](../../ux/main-window-components.md) | [简体中文](../../zh-CN/ux/main-window-components.md) | [繁體中文](../../zh-TW/ux/main-window-components.md) | [日本語](../../ja-JP/ux/main-window-components.md) | [한국어](../../ko-KR/ux/main-window-components.md) | [Français](../../fr-FR/ux/main-window-components.md) | [Deutsch](../../de-DE/ux/main-window-components.md) | Español | [Português (Brasil)](../../pt-BR/ux/main-window-components.md) | [Русский](../../ru-RU/ux/main-window-components.md)

Los controles de la ventana principal proporcionan aspecto y comportamiento, con estados explícitos normal, cursor encima, pulsado, foco de teclado, desactivado, seleccionado, ocupado y error. Qt habilita acciones según las capacidades reales de exploración, creación, extracción, [comprobación de integridad](../engineering/archive-checking.md) y [vista previa automática del contenido](../engineering/content-preview.md).

Los nombres de archivos, directorios y archivos comprimidos, así como las rutas, usan preferentemente la fuente predeterminada del sistema al inicio, con alternativas de plataforma para glifos ausentes. El texto fijo de la interfaz sigue usando el subconjunto incluido. El árbol de directorios y las vistas previas de arrastre se miden con sus respectivas fuentes de texto; consulte el [estándar visual](../visual/design-standard.md#界面字体与用户数据).

## Funciones y componentes

| Área | Componentes compartidos / compuestos | Interacción y capacidad |
| --- | --- | --- |
| Barra de título del sistema | `Browser`, `ArchiveTabs`, marco de plataforma | macOS usa los botones nativos. Windows usa botones integrados, redimensionamiento nativo y decoración DWM. En Ubuntu X11 los botones integrados y esquinas redondeadas del estado normal siguen el diseño X11. Consulte [ventanas de plataforma](platform-windows.md) |
| Pestañas de archivo | ArchiveTabs, ArchiveTab, botones de icono | Pestañas nuevas por defecto, ventanas nuevas opcionales; pestañas inactivas de grosor normal, que se estrechan y después se desplazan horizontalmente con accesos a desplazamiento y lista de pestañas. Arrastre dentro de la ventana y Ctrl+Shift+Page Up/Down cambian el orden; al cerrar se conservan tareas en segundo plano |
| Barra de herramientas | `ArchiveToolbar`, `ArchiveAction`, `CommandMenu` | Abrir, Nuevo, Comprobar, Más y el botón dividido de extracción usan el mismo envío de comandos que menús y teclado. La comprobación previa está disponible en el menú de extracción para selecciones admisibles |
| Búsqueda | SearchField, acción de borrar, indicación de ámbito | No enviar durante la composición IME; buscar en el directorio actual; borrar no cambia el directorio; el atajo enfoca |
| Resumen del archivo | ArchiveSummary, distintivo de formato, barra de estadísticas | Distinguir listo, leyendo, vacío y fallo de lectura; los valores desconocidos no son cero |
| Navegación de directorios | DirectoryTree, DirectoryRow, Disclosure, líneas de conexión | Expansión multinivel, espacios de flecha vacíos en hojas, directorio actual y foco independientes, rutas completas accesibles, marcador de antepasado contraído y acciones contextuales de directorio |
| Encabezado del área de archivos | PaneHeader, Breadcrumb, ViewToggle | Navegación jerárquica de ruta, cambio lista/cuadrícula y menú de ordenación; altura y centro vertical compartidos |
| Lista y cuadrícula | EntryView, SelectableEntry, casilla, SortHeader | Selección simple, múltiple y por rango, seleccionar todo, borrar, invertir y teclas de flecha. Doble clic/Intro entra en directorios o extrae temporalmente archivos ordinarios y los abre con la asociación del sistema; las propiedades siguen disponibles en el menú contextual o los detalles |
| Menú contextual de archivo | ContextMenu, CommandMenuItem, Submenu | Abrir se aplica solo al elemento pulsado con el botón derecho, como el doble clic; conservar la selección múltiple existente. Un clic derecho en un elemento no seleccionado lo selecciona primero por sí solo. Habilitar extracción, propiedades, copia de nombre/ruta y navegación de directorios según la selección |
| Menú contextual de zona vacía | ContextMenu, RadioMenuItem, CheckboxMenuItem | Selección, vista y ordenación de la lista actual y panel de detalles; no borrar arbitrariamente la selección |
| Menú contextual de encabezado | Menú de ordenación | Campo, dirección y restauración del orden del archivo; ordenar todos los resultados de la consulta, no solo los visibles |
| Propiedades / vista previa | `InspectorPanel` en la página o ventana independiente de detalles | Muestra estado de archivo, selección o archivo comprimido. El contenido se lee cuando se solicita. Por debajo de 928 píxeles lógicos el panel integrado se contrae y la acción de detalles abre una ventana separada |
| Estado y tareas | Barra de estado, botón de tareas y `CommandMenu` | Muestra número de seleccionados y tamaños conocidos. El menú de tareas usa el historial de toda la aplicación. Las tareas activas tienen un recuento separado y continúan al cerrar su ventana de progreso |
| Indicaciones y ventanas de operación | Tooltip, Notice, Toast, ventanas independientes de plataforma | Nombres completos y motivos de desactivación; Escape, retorno del foco, cancelación y cierre siguen la semántica de cada acción |

Lista y cuadrícula comparten un estado vacío. Una búsqueda enviada sin resultados sugiere otro término. Sin búsqueda, un archivo sin registros se identifica como vacío; en otro caso, el mensaje solo describe el contenido visible de la carpeta actual y no deduce que un índice de directorios limitado sea completo. La lectura y los errores conservan sus estados reales; el texto de búsqueda no enviado no cambia los mensajes del resultado actual. La guía antes de abrir un archivo no enumera una lista desactualizada de formatos.

## Comandos y menús unificados

Botones, atajos, menús contextuales y desplegables invocan los mismos comandos y vuelven a comprobar la capacidad actual antes de ejecutarlos. Los menús fijan el objeto y el ámbito de selección al abrirse; se cierran cuando cambia el archivo o la consulta o desaparece el objeto, para que el comando de una entrada antigua no afecte a un archivo nuevo.

Los menús comparten cuatro columnas para espacio de icono o marca, nombre, atajo y flecha de submenú, con separadores entre grupos de acciones. Incluya solo acciones pertinentes al contexto: los directorios ofrecen entrada y expansión, los archivos ofrecen detalles y una selección múltiple conserva el ámbito de lote. Las acciones que no pueden ejecutarse permanecen desactivadas, con motivos disponibles solo en las descripciones accesibles; no muestre sublíneas explicativas ni información emergente, ni sustituya la desactivación por «próximamente» al pulsar.

Cuando cambia el tamaño de fuente, la altura de la etiqueta del menú sigue las métricas reales y permanece centrada verticalmente en la fila. La primera presentación debe cumplir un error de centro máximo de medio píxel lógico sin depender de una pasada posterior de diseño para corregir un espacio de texto antiguo.

Las filas normales, de casilla, de opción y de submenú y los separadores son componentes compartidos. Ordenación y vista usan elementos de opción para el estado actual; el panel de detalles usa un elemento de casilla. Las acciones peligrosas nunca se activan automáticamente por defecto; un clic derecho o abrir un menú no ejecutan ninguna acción.

## Entrada y ciclo de vida de menús

- Abajo/arriba mueven entre elementos disponibles; Inicio/Fin van al primero/último; Intro/Espacio activan, derecha entra en un submenú e izquierda vuelve al padre.
- Escape cierra un nivel cada vez; al cerrar la raíz devuelve el foco al control activador o a la entrada original. Tab cierra el menú y continúa el orden normal de foco de la página. Se admite búsqueda por primer carácter del nombre; la composición IME no activa la búsqueda de menú.
- El cursor encima resalta sin ejecutar. Los submenús se abren tras un retraso y conservan el elemento padre al entrar. Ratón y teclado pueden seguirse sin dejar varios resaltados o menús raíz activos.
- Los clics exteriores cierran sin robar el foco al destino exterior. El desplazamiento del área principal, redimensionar la ventana, entrar en ventanas de operación o cambiar de archivo cierra los menús; desplazar el propio menú no los cierra.
- Evite los bordes inferior y derecho del área visible, abriendo a la izquierda o hacia arriba si es necesario. Posicione en coordenadas del host distinguiendo dimensiones lógicas, píxeles originales de origen, DPR y escala de revisión.
- Mayús+F10/tecla Menú abre el menú contextual de la entrada actual. La pulsación táctil prolongada y el clic derecho comparten contexto y no destruyen la selección múltiple como haría un clic simple.

## Estados de control y accesibilidad

| Estado | Aspecto | Comportamiento |
| --- | --- | --- |
| Normal | Conservar borde, radio, icono y línea base del original aprobado | Los cambios de estado no modifican dimensiones del control |
| Cursor encima | Relleno pálido; el botón principal usa un color principal más intenso | Las indicaciones retrasadas no cubren los destinos; el toque no conserva el estado de cursor |
| Pulsado | Relleno más definido que con cursor encima | Sin movimiento de texto o icono; una pulsación cancelada no ejecuta |
| Foco | Sin contorno, borde grueso ni brillo adicionales | Conservar orden Tab y activación de teclado; clic exterior, Escape e Intro dejan el foco del control |
| Desactivado | Colores explícitos de texto y gráfico desactivados conservando los contornos | Sin respuesta a clic o atajo; la descripción accesible explica el motivo |
| Seleccionado | Fondo o marca indica claramente el estado | Sincronizar `aria-selected` / `aria-checked` / Qt Accessible |
| Ocupado | El estado y ámbito desactivado siguen la tarea | Evitar envíos duplicados; permitir exploración durante trabajo en segundo plano |
| Error | Texto adyacente explica el problema y la recuperación | El color no es la única información y no se fabrica un éxito |

Los iconos compartidos proceden de `design/assets`. Los controles implementan texto, marcas, fondos, líneas de conexión y posición de menús; los recortes en píxeles originales sirven para comparar. Qt usa la misma especificación lógica.

## Aceptación

El número y los botones de acción a la derecha del área de archivos forman un grupo de anchura fija. Lista, cuadrícula, ordenación y accesos visibles a detalles permanecen en 28×28 píxeles lógicos con separación 6.4; ampliar la ventana solo da más espacio a la ruta de navegación. El menú Más de la barra contiene solo Configuración; el orden original, la comprobación previa de metadatos de extracción, seleccionar todo y los detalles usan accesos dedicados existentes.

Las páginas de inicio y archivo comparten una barra de estado de al menos 32 píxeles lógicos según statusbarHeight en layout-scale.json. El crecimiento de fuente de accesibilidad usa la altura real del botón de tareas y márgenes verticales iguales, sin cambiar la altura al cambiar de página. Ambos botones usan task-list en un espacio de icono centrado de 16×16 píxeles lógicos definido por main.interaction.taskIconSize, sin cambios por página o escala de fuente.

La barra de estado del archivo muestra el número seleccionado; con selección no vacía, añade separador y tamaño solo cuando se conoce el total. Si un directorio virtual o archivo de tamaño desconocido lo hace desconocido, muestra solo el número sin separador extra ni `—`. Un tamaño conocido de cero bytes sigue mostrándose; el texto de bienvenida no cambia.

El borde derecho de la casilla de lista permanece a 12 píxeles lógicos del espacio de icono; icono, nombre y encabezado Nombre se alinean. Los tamaños de fuente de accesibilidad cambian las métricas reales del texto sin ampliar ese espacio. Las casillas de filas y Seleccionar todo del encabezado conservan el modelo de selección existente.

Las cinco columnas pueden redimensionarse arrastrando el borde derecho, con un área activa de 8 píxeles lógicos; doble clic restaura el valor predeterminado de esa columna. Nombre llena el espacio restante por defecto; las demás empiezan con los valores del diseño original. Las dos columnas de tamaño crecen por defecto para que etiquetas medidas y flechas de ordenación quepan sin recorte. Las anchuras del usuario se comparten entre pestañas y ventanas durante la ejecución y se restablecen al salir; las anchuras lógicas permanecen en memoria, no en configuración ni DPR. El mínimo de Nombre es 160 y el de otras columnas 64, creciendo proporcionalmente para fuentes de accesibilidad; el desbordamiento se desplaza horizontalmente. Encabezados y contenido comparten una fuente de anchura y el arrastre no activa ordenación. Los indicadores ascendente/descendente preceden las etiquetas Tamaño original y Tamaño comprimido alineadas a la derecha y siguen a las demás, siempre a 6 píxeles lógicos del texto mostrado. Las columnas de ordenación activas reservan espacio para la flecha en el lado correspondiente; al estrechar manualmente se eliden primero las etiquetas y las flechas no pueden solaparse con el texto o columnas vecinas. Los encabezados numéricos permanecen alineados a la derecha con el contenido.

Los iconos de tipo de archivo se generan para selectores Qt desde la [tabla de indicaciones de extensión](../../../specs/visual/file-types.json), que cubre documentos ofimáticos, hojas de cálculo, presentaciones, audio, vídeo, archivos comprimidos, instaladores, fuentes e imágenes de disco. Las extensiones no distinguen mayúsculas y los tipos desconocidos usan un icono genérico de documento. Esto describe solo el aspecto, no contenido fiable ni soporte de exploración, creación o extracción del formato.

Use ratón y teclado reales para comprobar entradas, evasión de bordes de menú, ida y vuelta de submenús, retención de selección, restauración de foco, protección de acciones desactivadas, aislamiento de pestañas, cambio de vista y jerarquía de directorios. Capture controles reales con temas claro/oscuro, decoraciones de tres plataformas, fuentes normales/grandes y escalas. La alineación inicial existente del título, barra, lista y vista previa no debe sufrir regresiones por el trabajo de menús.

Registre cobertura funcional, capturas reales y capacidades no integradas según el [método general de verificación](../../../developers/qa/README.md), con conclusiones en las incidencias correspondientes y pruebas en build/ o artefactos CI. Las mesas estáticas, el comportamiento del navegador y la verificación de Qt en ejecución se describen por separado y no se sustituyen entre sí.

Los menús Qt usan explícitamente `Popup.Item` con delegados compartidos para que AppKit no ignore el estilo personalizado; las ventanas de operación siguen siendo ventanas independientes de nivel superior. `CommandTooltip` usa parámetros compartidos de indicaciones y texto sin formato para que los nombres de archivos no se interpreten como texto enriquecido; oculte las indicaciones cuando los menús obtengan el foco. Los colores JSON compartidos de ocho dígitos usan CSS RGBA; el generador QML convierte a QColor ARGB. Verifique color y opacidad conjuntamente.

El árbol de directorios y los estados de cursor sobre Abrir, Nuevo, Comprobar, Más y Extraer de la barra principal no tienen información emergente flotante. Las acciones de menú desactivadas tampoco muestran indicación ni información emergente; los demás botones usan indicaciones compartidas retrasadas. La raíz y los subdirectorios ordinarios enumeran solo archivos y carpetas directos, mientras la búsqueda cubre recursivamente el ámbito actual. Los nombres ordinarios muestran el último componente; la búsqueda muestra rutas completas. El cursor encima cubre toda la fila de lista; las cuadrículas mantienen margen horizontal fijo y casillas al pasar el cursor o seleccionar. Los menús usan sombras suaves en todos los lados y una separación de 4 píxeles lógicos respecto a los botones que los invocan. Conserve pruebas separadas de interacción y escala según el [método general](../../../developers/qa/README.md).

Todos los modos de ordenación mantienen directorios antes de archivos. Ascendente y descendente se aplican dentro de cada grupo. La ordenación por tipo agrupa archivos sin extensión aparte de los directorios y conserva el orden estable de identidad para tipos iguales. Ordene la consulta completa antes de dividir lotes de transferencia internos.

## Selección, tareas e indicaciones

Seleccionar todo mantiene el foco de detalles en el último elemento de la lista actual, con metadatos iguales a la selección individual. Borrar la selección oculta cuatro propiedades y «Extraer este archivo»; ese botón usa estados compartidos de cursor, pulsación y foco. Seleccionar todo sigue aplicándose a la consulta actual completa sin cambiar los contratos de directorio o identidad física.

«Tareas N» en la parte inferior muestra el número del historial conservado; el recuento activo solo se usa para indicaciones y lógica de salida. Cada entrada del menú de tareas muestra creación/extracción y número a la izquierda y color de estado a la derecha; la activación con ratón o teclado vuelve a abrir la misma tarea. Los menús de tareas comparten densidad, tamaño de fuente y límites de desplazamiento con otros menús. Sin selección, las acciones de ámbito seleccionado y comprobación previa en el menú de ámbito de extracción están desactivadas sin líneas adicionales de indicación.

Las indicaciones flotantes aparecen solo después de que el puntero permanezca continuamente inmóvil dentro del mismo control durante 1200 ms. Movimiento, salida, pulsación o apertura de un menú cancelan la visualización y los temporizadores antiguos; pasar entre controles requiere una nueva espera completa, no un despertar rápido de la indicación anterior. Preferir encima del control; usar debajo si no hay espacio superior suficiente para evitar que los límites empujen la información emergente sobre el control y causen oscilación del estado de cursor. El CommandTooltip compartido de Qt usa un temporizador explícito de inmovilidad. El tiempo de indicaciones de menús nativos y bandeja está controlado por el sistema y no se afirma que pueda controlarse con QML. El tiempo no escala con tamaño de fuente, DPR ni densidad; la geometría visual permanece igual.

Las pestañas de archivo también muestran nombres completos como texto sin formato en la información emergente compartida con retraso de inmovilidad; pulsar, reordenar o abrir el menú de pestaña o la lista de pestañas la oculta. Las entradas globales de tareas cierran primero su menú y después abren solo la ventana de tarea, sin cambiar de pestaña de archivo ni reabrir páginas de archivo cerradas.

La página inicial usa un cuerpo desplazable fijado arriba y centrado horizontalmente. Los errores y advertencias aparecen debajo de las indicaciones de formato con texto rojo e iconos semánticos sin desplazar el contenido superior. Al cerrar una ventana principal residente en la bandeja se borran sus pestañas; la restauración normal muestra solo la página inicial. Al minimizar se conservan las pestañas; las tareas en segundo plano siguen disponibles globalmente.

Consulte [arrastrar y soltar y modificación de archivos](../engineering/archive-transfers.md) para los depósitos bidireccionales en la lista, confirmación de adición al directorio actual y copia externa en tres plataformas.

Los movimientos dentro del archivo usan arrastre interno. Los destinos de carpeta en lista/cuadrícula y árbol de directorios usan el relleno de selección existente; la respuesta inmediata reutiliza la geometría de información emergente, con cuatro líneas como máximo y limitada al área cliente actual. Los directorios seleccionados, sus descendientes, posiciones originales, búsqueda y destinos cifrados o divididos tienen explicaciones explícitas de rechazo sin una nueva ventana de confirmación. Los conflictos de nombre usan la ventana de conflictos de archivos y las tareas de movimiento tienen un nombre distinto. Consulte el diseño de movimiento dentro del archivo y el [contrato de arrastrar y soltar](../engineering/archive-transfers.md) para transacciones y límites.
