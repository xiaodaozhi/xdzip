<a name="xdzip-requirements-and-verification-traceability"></a>

# Trazabilidad de requisitos y verificaciones

[English](../../design/requirements-traceability.md) | [简体中文](../../zh-CN/design/requirements-traceability.md) | [繁體中文](../../zh-TW/design/requirements-traceability.md) | [日本語](../../ja-JP/design/requirements-traceability.md) | [한국어](../../ko-KR/design/requirements-traceability.md) | [Français](../../fr-FR/design/requirements-traceability.md) | [Deutsch](../../de-DE/design/requirements-traceability.md) | Español | [Português (Brasil)](../../pt-BR/design/requirements-traceability.md) | [Русский](../../ru-RU/design/requirements-traceability.md)

Los [requisitos del producto](product-requirements.md) definen los identificadores y su alcance actual. Cada fila indica qué comprobar cuando cambia el comportamiento. Los resultados, las plataformas y las limitaciones se registran con el cambio correspondiente. Los [casos de aceptación](../../../developers/qa/acceptance.md) señalan los puntos de entrada para las verificaciones.

<a name="xdzip-requirement-matrix"></a>

## Matriz de requisitos

| Requisito | Referencia de diseño | Aspectos de verificación |
| --- | --- | --- |
| P01 Explorar sin extraer | [Dominio](domain-data.md), [Escritorio](qt-quick-ui.md) | T01 Abrir un archivo comprimido real y confirmar que la exploración no produce archivos extraídos. |
| P02 Explorar, consultar y seleccionar | [Dominio](domain-data.md), [Rendimiento](performance.md) | T02 Identidad de entradas homónimas, consultas estables, páginas obsoletas, almacenamiento de índices grandes y selección entre páginas. |
| P03 Extraer el ámbito previsto | [Autorización de archivos](file-security.md), [Tareas](task-scheduling.md) | T03 Expansión de directorios, duplicados, cambio de nombre, omisión, sobrescritura y resultados exactos en disco. |
| P04 Crear un archivo comprimido | [Formatos](codec-extension-sdk.md), [Autorización de archivos](file-security.md) | T04 Pruebas independientes de creación y lectura del perfil modificado, verificación temporal y protección de archivos existentes. |
| P05 Respetar las capacidades de los formatos | [Puente](qt-rust-bridge.md), [Formatos](codec-extension-sdk.md) | T05 Registro, acciones propias de cada sesión y rechazo en ejecución de perfiles incompatibles. |
| P06 Limitar tareas y permitir su cancelación | [Puente](qt-rust-bridge.md), [Tareas](task-scheduling.md) | T06 Colas llenas, envío de solicitudes de decisión, concurrencia entre cancelación y escritura definitiva, y cierre asíncrono. |
| P07 Proteger archivos y recursos | [Autorización de archivos](file-security.md), [Formatos](codec-extension-sdk.md) | T07 Comprobaciones nativas aplicables de enlaces e identidad, validación de rutas, cuotas y objetos protegidos. |
| P08 Ofrecer un escritorio accesible | [Escritorio](qt-quick-ui.md) | T08 Comportamiento de modelos y controles, teclado nativo, IME, accesibilidad, escala y foco en las plataformas afectadas. |
| P09 Limitar y medir recursos | [Rendimiento](performance.md) | T09 Compilación comparable, métricas del árbol de procesos, inicio, ordenación, velocidad de salida o tamaño del paquete según corresponda. |
| P10 Mantener Qt fuera de la lógica de formatos | [Arquitectura](../../design/system-architecture.md), [Formatos](codec-extension-sdk.md) | T10 Núcleo sin Qt, selección explícita de adaptadores y regresiones específicas de formatos. |
| P11 Proteger secretos y datos | [Autorización de archivos](file-security.md), [Privacidad](../engineering/privacy.md) | T11 Uso delimitado de contraseñas, diagnósticos sin datos sensibles y acceso intencionado a la red. |
| P12 Auditar la entrega | [Calidad](quality-engineering.md), [Publicación](build-release.md) | T12 Entradas fijadas e identidad de artefactos, inventario, licencias, confianza del paquete y comprobaciones de instalación aplicables. |
| P13 Controlar las extensiones externas | [Formatos](codec-extension-sdk.md), [Ensayo del SDK](../engineering/sdk-trial.md) | T13 Interoperabilidad y permisos nativos del ensayo. La instalación de extensiones en el producto sigue siendo una opción de diseño. |
| P14 Limitar vista previa y apertura | [Escritorio](qt-quick-ui.md), [Autorización de archivos](file-security.md) | T14 Vista previa automática, contenido compatible, límites del proceso y salidas temporales privadas. |
| P15 Informar del resultado real | [Puente](qt-rust-bridge.md), [Autorización de archivos](file-security.md), [Tareas](task-scheduling.md) | T15 Cambios de origen o destino, fallos y cancelación después de escrituras confirmadas, limpieza e incertidumbre. |

<a name="xdzip-cross-cutting-verification"></a>

## Verificación transversal

| Grupo | Cambios aplicables | Aspectos de verificación |
| --- | --- | --- |
| T16 Propiedad y ciclo de vida del puente | Duración del puente, la conexión o el controlador | Identidad de la solicitud actual, activación del receptor, tareas conservadas y destrucción segura. |
| T17 Interoperabilidad y errores de formato | Cambios de formato o códec | Muestra válida independiente cuando corresponda, truncamiento o perfiles incompatibles y salida acotada. |
| T18 Instalación, actualización y eliminación | Cambios de plataforma o canal de distribución | Paquete real, dependencias en un entorno limpio, identidad de instalación, transferencia de la actualización y eliminación o recuperación admitida. |

<a name="xdzip-representative-workflows"></a>

## Flujos representativos

1. Abrir un ZIP y buscar en él, fijar los resultados seleccionados, extraer y comparar los bytes de salida. «Extraer todo» sigue abarcando todo el archivo comprimido.
2. Extraer entradas homónimas o en conflicto con una política definida y comprobar los nombres y estados resultantes.
3. Cancelar antes y durante la escritura definitiva, verificar que se informan los archivos completados y que los archivos comprimidos ya verificados se conservan si la creación falla antes de guardarse.
4. Cambiar el origen o el destino en una prueba controlada y comprobar el resultado real. Tras una interrupción anormal, examinar los restos sin suponer que se repiten las operaciones automáticamente.
5. Provocar un fallo del proceso de formato incluido o del ensayo independiente del SDK según su contrato y comprobar que nunca se publican datos incompletos como resultado correcto.

Las comprobaciones estáticas de redacción, formato y enlaces validan la documentación. Las pruebas con muestras, sin pantalla, automatizadas en entorno nativo y manuales tienen sus propios alcances. [Ingeniería de calidad](quality-engineering.md), [Guía de verificación](../../../developers/qa/README.md)
