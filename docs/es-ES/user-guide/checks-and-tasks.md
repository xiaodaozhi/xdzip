# Verificar archivos comprimidos y gestionar tareas

[English](../../user-guide/checks-and-tasks.md) | [简体中文](../../zh-CN/user-guide/checks-and-tasks.md) | [繁體中文（台灣）](../../zh-TW/user-guide/checks-and-tasks.md) | [日本語](../../ja-JP/user-guide/checks-and-tasks.md) | [한국어](../../ko-KR/user-guide/checks-and-tasks.md) | [Français](../../fr-FR/user-guide/checks-and-tasks.md) | [Deutsch](../../de-DE/user-guide/checks-and-tasks.md) | Español | [Português (Brasil)](../../pt-BR/user-guide/checks-and-tasks.md) | [Русский](../../ru-RU/user-guide/checks-and-tasks.md)

<a name="xdzip-test-an-archive"></a>

## Verificar un archivo comprimido

1. Abre el archivo comprimido y elige **Verificar** en la barra de herramientas.
2. Introduce la contraseña si se te solicita y espera el resultado. La tarea comprueba el contenido compatible sin escribir archivos extraídos.
3. Si hay un fallo, revisa el motivo y la entrada afectada, si aparece. Algunos formatos proporcionan sumas de verificación o autenticación. Otros comprueban la estructura y la legibilidad. Un resultado correcto solo cubre las comprobaciones que admite el formato. [Comprobaciones de integridad](../engineering/archive-checking.md)

Hay archivos comprimidos que no se pueden verificar. Por ejemplo, un ZIP con índice en disco admite extraer archivos seleccionados dentro de los límites de la tarea. De NSIS solo se muestran los metadatos del instalador. Consulta [límites de los archivos grandes](../engineering/large-index.md) y [formatos compatibles](../../../README.es-ES.md#xdzip-format-support).

<a name="xdzip-progress-cancellation-and-results"></a>

## Progreso, cancelación y resultados

Abre el menú de tareas para volver a una operación en curso o consultar sus resultados. Cerrar la pestaña de un archivo comprimido no cancela la tarea en segundo plano. Puede que tengas que responder a una solicitud de contraseña o resolver un conflicto antes de que continúe.

Elige **Cancelar** en la ventana de la tarea para solicitar la detención. Espera a que llegue al estado final. Los archivos que ya se hayan guardado de forma definitiva pueden permanecer en el destino. Amplía los detalles para distinguir elementos completados, omitidos y pendientes. Consulta las cantidades y los motivos antes de decidir qué repetir. Borrar el historial de tareas completadas elimina sus entradas y conserva los archivos en el disco. [Ventana de tareas](../ux/task-windows.md)
