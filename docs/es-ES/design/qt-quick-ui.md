<a name="xdzip-qt-quick-desktop-interface"></a>

# Interfaz de escritorio de Qt Quick

[English](../../design/qt-quick-ui.md) | [简体中文](../../zh-CN/design/qt-quick-ui.md) | [繁體中文](../../zh-TW/design/qt-quick-ui.md) | [日本語](../../ja-JP/design/qt-quick-ui.md) | [한국어](../../ko-KR/design/qt-quick-ui.md) | [Français](../../fr-FR/design/qt-quick-ui.md) | [Deutsch](../../de-DE/design/qt-quick-ui.md) | Español | [Português (Brasil)](../../pt-BR/design/qt-quick-ui.md) | [Русский](../../ru-RU/design/qt-quick-ui.md)

El escritorio usa Qt Quick para la presentación y la interacción. Los controladores C++ traducen las acciones del usuario y mantienen el estado de la vista. El entorno de ejecución Rust interpreta los archivos comprimidos y controla las reglas de las tareas y los permisos sobre archivos. La [arquitectura del sistema](system-architecture.md) y el [contrato del puente](qt-rust-bridge.md) explican estas llamadas.

<a name="xdzip-windows-sessions-and-models"></a>

## Ventanas, sesiones y modelos

| Componente | Responsabilidad actual |
| --- | --- |
| `Browser.qml` y `SessionPage.qml` | Pantalla de inicio, pestañas de archivos comprimidos, ruta y búsqueda, navegación por directorios, lista o cuadrícula, propiedades y acciones. Una página muestra el estado actual del controlador e invoca comandos. |
| `ApplicationSessions` | Dirige las solicitudes de apertura a sesiones de exploración independientes. Coordina las pestañas o ventanas, el acceso compartido a las tareas y el cierre de la aplicación. El modo de apertura se aplica a las solicitudes futuras. Cerrar una página libera la vista, y el controlador de una tarea puede seguir activo. |
| `BrowserController` y `ArchiveTableModel` | Mantienen la selección, el estado de la lista y la identidad del resultado. Los resultados pequeños se incorporan como modelos completos. Los grandes exponen el número lógico de filas con un máximo de 1024 filas residentes y cargan las páginas que faltan. Las filas sin cargar no permiten acciones sobre las entradas. |
| `TaskListModel` y ventanas de operación | Presentan instantáneas acotadas de tareas, progreso, resultados y la decisión pendiente. La identidad de escritorio compuesta dirige cada comando de tarea al controlador responsable. |
| Ajustes y adaptadores de plataforma | Mantienen la apariencia, el idioma, el tamaño del texto y el comportamiento de la aplicación. Los adaptadores nativos gestionan el área de notificación, la activación, los diálogos de archivo, las asociaciones y la entrega de transferencias según el sistema operativo. |

[Entrada de escritorio](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/browser_main.cpp), [Página de sesión](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/ui/SessionPage.qml), [Controlador y modelos](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/BrowserController.h), [Sesiones de aplicación](../engineering/application-sessions.md), [Ajustes e idiomas](../ux/settings-localization.md)

<a name="xdzip-browsing-and-operation-flow"></a>

## Exploración y operaciones

La apertura publica una instantánea completa de los metadatos. Cada búsqueda, ordenación o navegación nueva lleva su propia identidad de resultado. Una página obsoleta no puede reemplazar el modelo actual. Durante una consulta, la vista muestra el resultado ya incorporado mientras el controlador lo considere válido. El transporte del núcleo envía hasta 128 filas y 64 KiB de nombres por lote. [Datos de dominio](domain-data.md), [Indexación de archivos comprimidos grandes](../engineering/large-index.md)

Abrir un directorio navega a su interior. Haz doble clic sobre una entrada de archivo normal compatible o usa la opción de abrir para iniciar la apertura temporal. La vista de propiedades incluye una acción de vista previa independiente. Los avisos de conflicto o contraseña mantienen la identidad de la operación en curso. El acceso a las tareas muestra el progreso y los resultados aun cuando se ha cerrado la página de exploración. [Exploración](../user-guide/browse.md), [Apertura temporal](../engineering/archive-opening.md), [Vista previa del contenido](../engineering/content-preview.md), [Ventanas de tareas](../ux/task-windows.md)

Soltar un archivo puede abrir un archivo comprimido o añadir entradas para su creación, según el destino. Los movimientos internos admitidos y el arrastre al exterior usan tareas de preparación y transferencias específicas de la plataforma. La plataforma determina las capacidades de transferencia, incluidos el contenido asíncrono y las escrituras en el destino. [Transferencias de archivos comprimidos](../engineering/archive-transfers.md)

<a name="xdzip-shared-controls-and-native-boundaries"></a>

## Controles compartidos y límites nativos

Las pantallas de operación reutilizan `OperationButton`, `OperationFooterButton`, `OperationFrame`, `OperationSelect` y `OperationSegmentedGroup`. Los componentes compartidos definen las medidas y la interacción. Los controles comunes de selección dirigen ratón, teclado y accesibilidad al mismo modelo de estado. El texto y los datos de archivo siguen las reglas de tipografía y localización de la [norma de diseño](../../visual/design-standard.md). Los nombres de archivo comprimido se conservan como datos y se muestran de forma segura.

Las ventanas de operación independientes siguen el registro de ventanas y las reglas propias de modalidad y cierre. macOS usa controles nativos de la barra de título. Windows y Linux usan adaptadores de sistema. El [contrato de ventanas de plataforma](../ux/platform-windows.md) define la aceptación de ventanas reales y el [contrato de integración de escritorio](../ux/desktop-integration.md) explica el área de notificación y la activación.

<a name="xdzip-design-and-verification-scope"></a>

## Alcance del diseño y de la verificación

El código y las pruebas describen el comportamiento del producto. Los diseños HTML de alta fidelidad ilustran la disposición y los estados simulados para su revisión. Los cambios en el foco nativo, los menús, el IME, las acciones de accesibilidad, la escala de pantalla y los gestores de archivos requieren pruebas en las plataformas afectadas. Las configuraciones de escritorio y las ventanas de operación nuevas requieren su propia implementación y aceptación. [Flujo de diseño visual](../visual/design-workflow.md), [Ingeniería de calidad](quality-engineering.md)
