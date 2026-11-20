<a name="xdzip-additional-interaction-and-copy-specification"></a>

# Especificación adicional de interacción y textos

[English](../../ux/interaction-spec.md) | [简体中文](../../zh-CN/ux/interaction-spec.md) | [繁體中文](../../zh-TW/ux/interaction-spec.md) | [日本語](../../ja-JP/ux/interaction-spec.md) | [한국어](../../ko-KR/ux/interaction-spec.md) | [Français](../../fr-FR/ux/interaction-spec.md) | [Deutsch](../../de-DE/ux/interaction-spec.md) | Español | [Português (Brasil)](../../pt-BR/ux/interaction-spec.md) | [Русский](../../ru-RU/ux/interaction-spec.md)

La [especificación de estados](hifi-state-spec.md) amplía el diseño de la ventana principal y la confirmación de extracción con creación, conflictos, tareas, resultados, ajustes y comparaciones por plataforma. El [contrato de bienvenida](welcome-opening.md) cubre el inicio y el progreso de apertura. Los diseños y medidas vigentes siguen el [registro de ventanas](../../../specs/visual/window-registry.json). Los contratos técnicos y de interacción indican qué se ha implementado y comprobado visualmente en el entorno nativo.

| Situación | Presentación y acción | Continuación |
| --- | --- | --- |
| Primer inicio | Abrir o crear un archivo comprimido. Permitir arrastrar archivos locales. | Evitar escanear todos los discos o asociar automáticamente todos los formatos. |
| Lectura de directorio | Cantidad leída y progreso indeterminado. Total desconocido. | Se puede cerrar la sesión. Marcar los resultados parciales como incompletos. |
| Archivo vacío | «Este archivo comprimido está vacío». Mantener sus propiedades. | Abrir otro archivo. Deshabilitar la extracción y explicar el motivo. |
| Sin resultados | Mostrar el término y el alcance de búsqueda. | Limpiar la búsqueda. Indicar que no hay coincidencias sin presentar el archivo comprimido como vacío. |
| Función no compatible | Indicar el motivo concreto de formato, algoritmo o cifrado. | Conservar metadatos legibles y detener salidas parciales inseguras. |
| Creación | Entradas, formato, destino, nivel y comprobación previa. | Mostrar errores junto a los campos. Conservar los archivos existentes por defecto. |
| Conflicto de nombres | Mostrar ambos elementos, Omitir, Cambiar nombre y Sobrescribir, y el alcance de decisiones posteriores. | Cerrar el aviso deja la tarea en espera. Vuelve a abrirlo o cancela desde la tarea. |
| Tarea completada | Recuentos de éxitos, omisiones, fallos y resultados guardados. | Tras extraer, abrir el destino si se pide. La creación permite abrir un archivo único compatible o el primer volumen en la aplicación. Para reintentar hay que volver a planificar. |
| Fallo parcial | Conservar archivos guardados y explicar los fallos. | Mantener en el destino los archivos del usuario ya escritos. |
| Cerrar la aplicación | Número de tareas activas. Esperar o cancelar y salir. | El núcleo confirma la limpieza. Mantener la interfaz operativa durante el cierre. |
| Configuración | Tema del sistema, claro u oscuro, densidad, tamaño, idioma, historial y área de notificación. | Aplicar tras guardar. Ante un fallo, conservar los valores anteriores. Las cuotas no se pueden editar en la configuración actual. |
| Extensiones de formato | El producto usa formatos integrados. Un [ensayo de SDK externo](../engineering/sdk-trial.md) prueba aparte la ejecución de solo lectura. | Consulta el [registro](../engineering/format-registry.md) para ver las operaciones disponibles. |

El botón de extracción sigue la selección. Su menú ofrece los dos alcances según las capacidades actuales. La búsqueda cubre el directorio actual y sus descendientes. Seleccionar un solo archivo compatible muestra automáticamente una [vista previa de texto o imagen](../engineering/content-preview.md) en el panel de propiedades visible. Si no hay anchura para el panel, puede abrirse en una ventana aparte. Los tamaños grandes amplían el contenido y permiten desplazarse hasta los controles de confirmación.

El mensaje explica qué pasó y qué puede hacer el usuario. Los códigos van en los detalles. Evita mostrar pilas de llamadas o términos de IPC en los mensajes principales. Muestra nombres peligrosos como texto seguro y conserva los mensajes de complementos sin interpretarlos como HTML. Los atajos, IME, foco y lectores de pantalla siguen el [diseño de Qt Quick](../design/qt-quick-ui.md).
