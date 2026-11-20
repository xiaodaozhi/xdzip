<a name="xdzip-product-runtime-and-qt-lifecycle"></a>

# Entorno de ejecución del producto y ciclo de vida de Qt

[English](../../engineering/product-runtime.md) | [简体中文](../../zh-CN/engineering/product-runtime.md) | [繁體中文](../../zh-TW/engineering/product-runtime.md) | [日本語](../../ja-JP/engineering/product-runtime.md) | [한국어](../../ko-KR/engineering/product-runtime.md) | [Français](../../fr-FR/engineering/product-runtime.md) | [Deutsch](../../de-DE/engineering/product-runtime.md) | Español | [Português (Brasil)](../../pt-BR/engineering/product-runtime.md) | [Русский](../../ru-RU/engineering/product-runtime.md)

Esta guía explica cómo la aplicación de escritorio se conecta al entorno Rust mediante comandos y eventos, incluidos la contrapresión de colas, el cierre y las compilaciones por plataforma. Consulta la [navegación de solo lectura](../../engineering/readonly-browser.md) y la [extracción segura](zip-safe-extraction.md) para sus capacidades concretas.

<a name="xdzip-implemented-services"></a>

## Servicios implementados

`xdzip-runtime::service` implementa el servicio del producto. La interfaz toma `CommandClient` una sola vez y un hilo receptor Qt dedicado toma `EventReceiver` una sola vez. El hilo de comandos Rust posee el archivo abierto de la conexión y la instantánea de navegación. Un hilo de tareas Rust separado recibe el trabajo preparado con la fuente y los ajustes necesarios y publica actualizaciones. Los identificadores de archivos comprimidos y los puntos de entrada de exploración síncrona permanecen en Rust. Consulta el [puente Qt/Rust](../design/qt-rust-bridge.md) para la ABI y la [planificación de tareas](../design/task-scheduling.md) para propiedad y admisión.

Los comandos principales incluyen Open, Browse, Preflight, CloseSession y Snapshot; toda la navegación y comprobación previa reales existentes los usan. Cada solicitud ordinaria aceptada recibe un ID que no se reutiliza, devuelve como máximo un evento de datos acotado y después exactamente un Finished; los errores devuelven Finished directamente. Las páginas llevan identidad de sesión, solicitud y desplazamiento. Cada RuntimeConnection tiene una sesión y solicitudes pendientes de la interfaz en serie. Las consultas incluyen además ID completo de consulta, generación de la interfaz y revisión de la instantánea; consulta el [contrato de navegación ZIP](zip-readonly-browsing.md). Las solicitudes entre sesiones devuelven stale_session.

La admisión ordinaria de la interfaz usa try_lock sin esperar espacio en la cola y valida rutas de 4 KiB, búsqueda de 1 KiB, 128 elementos seleccionados, ID de directorio de 21 bytes y códigos de orden 0–12. Un rechazo no consume ID; Busy puede reintentarse. La cancelación tiene un canal de control separado de hasta 128 elementos y su propio acuse Finished; el acuse no garantiza que el destino no haya terminado ya. El cierre conserva un RequestId independiente e idempotente y reserva el último ID disponible para funcionar incluso si se agotan los ID ordinarios o se llena la cola.

El controlador Qt conserva solo un comando pendiente de admisión. Un Busy transitorio se reintenta en el bucle de eventos mediante un temporizador de una sola ejecución de 1 ms, hasta 32 intentos; solo una saturación sostenida solicita reintentar. No es un plazo de finalización. La cancelación, el cierre o la desconexión retiran inmediatamente los comandos no aceptados; no hay temporizador de reintento en reposo. Los argumentos inválidos y el agotamiento de ID se notifican por separado, sin presentarlos como ocupación. Open se desactiva tras la desconexión.

<a name="xdzip-bounded-transport"></a>

## Transporte acotado

- Hay como máximo 128 solicitudes ordinarias pendientes; la capacidad solo se libera cuando el receptor toma Finished. Se reservan dos espacios de eventos por solicitud para que los datos no desplacen los estados terminales.
- Los eventos de datos Rust tienen un presupuesto acumulado de contenido de 32 MiB, incluidas imágenes de hasta 16 MiB y sus contenedores. La publicación espera al superar el presupuesto; la cancelación o el cierre despiertan la espera y pueden emitir un estado terminal cancelado. Un evento individual demasiado grande devuelve resource_limit.
- Los comandos Rust conservan un límite acumulado de contenido de 8 MiB; el exceso se rechaza al admitirlo y el mayor presupuesto de eventos de imagen no lo modifica.
- El buzón Qt admite hasta 256 eventos y 32 MiB de contenido. Como máximo 128 espacios son datos ordinarios; los demás se reservan para eventos terminales o de control. No crea una llamada Qt por cada evento Rust.
- Una cadena de notificación procesa hasta 32 eventos o aproximadamente 4 ms por turno y programa otro si queda trabajo; el tratamiento de un evento individual no se puede interrumpir. Las colas vacías dejan de programarse y el receptor espera en una variable de condición.
- Page de Open también lleva el índice completo acotado de directorios, hasta 32 768 nodos y 4 MiB de nombres/prefijos, marcado para sustituir mediante `replace_directories`; un índice vacío también sustituye explícitamente al anterior. Browse no vuelve a enviarlo; `navigation_limited` indica expresamente una indexación reducida. Rust contabiliza bytes de nombres y prefijos; el buzón Qt también cuenta las cadenas de ID de nodo y padre generadas por el puente. Contenedores de nodos y profundidad tienen límites separados. Aplicar juntos la página y el modelo de directorios solo después del Finished correcto de la solicitud correspondiente.
- `total` de Page cuenta las filas de la lista actual; `archive_total` conserva por separado el número de registros físicos del archivo. La navegación normal muestra archivos y carpetas directos del directorio actual; la búsqueda devuelve recursivamente coincidencias físicas. Los directorios sin registro explícito usan `navigation_id` y `physical=false`; las claves de fila Qt son `dN`, los ID físicos originales permanecen separados y los nodos virtuales no pueden enviar Preflight. Los metadatos conservan rutas completas; se muestra el nombre final al navegar y la ruta completa al buscar.
- Los presupuestos de cadenas no son RSS: contenedores, instantáneas y un evento en conversión requieren espacio adicional, pero entradas y páginas siguen acotadas. Los lotes del puente contienen como máximo 128 elementos. Consulta el [contrato de navegación ZIP](zip-readonly-browsing.md) para la base original de 100 000 entradas y la [indexación de archivos grandes](large-index.md) para la ampliación posterior de ZIP normales de un solo archivo.

Los eventos de control y ordinarios se toman alternativamente para que acuses continuos de cancelación no bloqueen los estados terminales ordinarios. Al entregar se asignan números de secuencia crecientes. Qt descarta secuencias antiguas; los acuses de control y request_id obsoletos no modifican la vista actual.

Los sobres `Task`/`TaskUpdate` del canal independiente de tareas conservan el request_id original de creación. Los registros de correlación permanecen en espacios acotados de control de tareas y se liberan al entregar el estado terminal de la tarea. Pueden preceder a los datos de aceptación o seguir al Finished de la solicitud. No consumen un segundo espacio de datos ordinarios ni liberan o finalizan de nuevo la solicitud; vinculan correctamente los marcadores de preparación al TaskId real y la sesión de origen. Este ID no es un ID de solicitud de cancelación; el control de tareas sigue usando TaskId.

<a name="xdzip-shutdown-destruction-and-failure"></a>

## Cierre, destrucción y fallos

Cerrar un archivo o una ventana puede dejar la aplicación y sus tareas ejecutándose en la bandeja. Al salir o reiniciar, el coordinador detiene la admisión, solicita cancelación y vacía eventos mientras Rust libera las sesiones. El receptor espera a los dos hilos Rust antes de devolver `ShutdownReady`. El escritorio espera a que termine el receptor Qt y se vacíe el buzón antes de salir. El hilo de interfaz nunca espera a hilos que estén esperando operaciones de archivo o entrega de eventos.

El control de cancelación y cierre usa secciones críticas breves con operaciones de estado acotadas, independientes de la capacidad de la cola de datos. La cancelación cooperativa no interrumpe una lectura bloqueada por el sistema operativo y no garantiza un cierre en tiempo real estricto. Las solicitudes de solo lectura aquí no producen transacciones de salida. Las confirmaciones de creación/extracción, la limpieza temporal y los resultados de salida siguen por separado los contratos de [creación](zip-creation.md) y [extracción segura](zip-safe-extraction.md).

Si un QObject se destruye antes de tiempo, cerrar el buzón, despertar al receptor y detener el servicio. El trabajo en segundo plano solo mantiene colas compartidas y el identificador del receptor Rust, no punteros al controlador. Qt cancela automáticamente las notificaciones de objetos destruidos. QPointer también protege la vida útil si una llamada de consumo de eventos destruye la conexión de forma síncrona. Al abandonar la recepción, no esperar los hilos en el destructor; el trabajo termina cooperativamente. Solo un ShutdownReady normal acredita que la espera a los hilos terminó.

Si un manejador de lectura de un proceso de trabajo encuentra un panic que permite desenrollar la pila, el límite lo captura, descarta todo el servicio, envía RuntimeFault y termina las solicitudes incompletas con runtime_unavailable; la sesión no se reutiliza. Esto define solo el fallo de lectura y no promete recuperar transacciones de creación o extracción a medio terminar. Los elementos confirmados, resultados inciertos y limpieza de escrituras siguen sus contratos. Una asignación irrecuperable, aborto del proceso o bloqueos internos envenenados no garantizan la entrega terminal. Ante EOF inesperado o fallo de conexión, Qt termina uniformemente los estados pendientes para evitar ocupación permanente.

<a name="xdzip-verification-and-build"></a>

## Verificación y compilación

Consulta la [verificación general](../../../developers/qa/README.md). Las pruebas del núcleo Rust no necesitan Qt; los casos Qt de ventana, modelo, estrés y cierre se compilan y ejecutan de forma nativa en tres plataformas. La CI de macOS verifica arm64 y el inicio de Cocoa en el entorno SDK sin desplegar ni subir aplicaciones; pueden producirse DMG de desarrollo locales bajo petición expresa. Windows/Linux despliegan Qt incluido y verifican la ejecución sin SDK; consulta la [guía de compilación](../../engineering/developer-build.md). Estas comprobaciones no constituyen autorización de publicación oficial.

Las consultas iniciales asignan una identidad de resultado inmutable; los lotes posteriores deben coincidir en todos los campos y parámetros de QueryKey. Qt cancela solicitudes antiguas y agrupa la última intención manteniendo las comprobaciones de secuencia e ID. Un lote también limita los nombres a 64 KiB y no vuelve a filtrar ni ordenar. La aceptación histórica se limita a las capacidades disponibles entonces.

La capa de aplicación puede crear varias conexiones independientes mientras el entorno de dominio aísla las sesiones por conexión. Consulta el [contrato de sesiones](../../engineering/application-sessions.md) para la identidad global de tareas y la admisión secuencial de tareas pesadas.
