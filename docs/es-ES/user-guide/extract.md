# Extraer todos los archivos o solo los seleccionados

[English](../../user-guide/extract.md) | [简体中文](../../zh-CN/user-guide/extract.md) | [繁體中文（台灣）](../../zh-TW/user-guide/extract.md) | [日本語](../../ja-JP/user-guide/extract.md) | [한국어](../../ko-KR/user-guide/extract.md) | [Français](../../fr-FR/user-guide/extract.md) | [Deutsch](../../de-DE/user-guide/extract.md) | Español | [Português (Brasil)](../../pt-BR/user-guide/extract.md) | [Русский](../../ru-RU/user-guide/extract.md)

<a name="xdzip-from-the-archive-window"></a>

## Desde la ventana del archivo comprimido

1. Abre el archivo comprimido. El botón principal de la barra extrae los elementos seleccionados o el archivo completo si no hay selección. Pulsa la flecha que hay junto al botón para elegir **Extraer todo** o **Extraer selección**.
2. Confirma qué quieres extraer y elige un destino con permisos de escritura. Si la opción está disponible, puedes crear una carpeta con el nombre del archivo comprimido.
3. Elige cómo resolver los conflictos. **Preguntar** pone la operación en pausa hasta que decidas. **Omitir** conserva el archivo existente. **Cambiar nombre** guarda el archivo nuevo por separado. **Sobrescribir** sustituye el archivo existente después de verificarlo, si la operación es compatible.
4. Inicia la extracción y comprueba el resultado. Si se produce un error o cancelas, los archivos ya extraídos pueden quedarse en el destino. Antes de repetir la operación, consulta el resultado para saber qué queda por extraer. [Verificaciones, tareas y resultados](checks-and-tasks.md)

El menú de extracción también incluye **Comprobar los metadatos antes de extraer**. Selecciona explícitamente hasta 128 entradas reales del archivo comprimido. La comprobación informa, por ejemplo, de nombres inseguros, métodos incompatibles y límites declarados. Si el formato lo permite, puedes extraer una carpeta implícita aunque solo se muestre para navegar. La comprobación previa no lee los datos ni concede permiso de escritura. [Comprobación previa y límites de extracción](../engineering/extraction-preflight.md)

En archivos ZIP grandes con índice en disco puedes extraer archivos normales seleccionados dentro de los límites de la tarea. Si el archivo comprimido completo supera el límite de extracción, selecciona un grupo más pequeño. Si cambia el archivo de origen o se sustituye, ábrelo de nuevo y vuelve a seleccionar los elementos. [Límites de los archivos grandes](../engineering/large-index.md)

<a name="xdzip-from-the-file-manager"></a>

## Desde el gestor de archivos

En Windows 11, abre el menú clásico con **Mostrar más opciones**. La edición de Microsoft Store puede mostrar acciones también en el menú moderno. En Ubuntu se usa la extensión de Nautilus. En macOS, activa la extensión de Finder en los ajustes del sistema o usa el servicio registrado de Finder. Si no aparece el menú, abre el archivo comprimido con xdzip y extráelo desde su ventana.

Desde el menú del gestor puedes extraer en el directorio actual de cada archivo, crear una carpeta separada por archivo, dejar que xdzip calcule el destino o elegir una carpeta. El modo automático solo evita crear una carpeta adicional si el contenido validado ya contiene exactamente una carpeta en el nivel superior. Un lote muestra el resultado de cada archivo comprimido y continúa si uno falla. Al cancelar, se detienen las operaciones posteriores y se pide detener la que está en curso. [Acciones y ajustes del gestor de archivos](../engineering/shell-actions.md)
