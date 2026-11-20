<a name="xdzip-qt-rust-bridge-and-application-service"></a>

# Puente Qt/Rust y servicio de aplicación

[English](../../design/qt-rust-bridge.md) | [简体中文](../../zh-CN/design/qt-rust-bridge.md) | [繁體中文](../../zh-TW/design/qt-rust-bridge.md) | [日本語](../../ja-JP/design/qt-rust-bridge.md) | [한국어](../../ko-KR/design/qt-rust-bridge.md) | [Français](../../fr-FR/design/qt-rust-bridge.md) | [Deutsch](../../de-DE/design/qt-rust-bridge.md) | Español | [Português (Brasil)](../../pt-BR/design/qt-rust-bridge.md) | [Русский](../../ru-RU/design/qt-rust-bridge.md)

El escritorio Qt llama al entorno Rust mediante el puente `cxx` enlazado. Cada `BrowserController` posee una `RuntimeConnection` con una instancia de servicio independiente. El puente transporta comandos tipados y resultados con propiedad transferida dentro del proceso de escritorio. La CLI usa directamente el mismo servicio Rust. [Arquitectura del sistema](../../design/system-architecture.md), [Sesiones de escritorio](../../engineering/application-sessions.md)

<a name="xdzip-handles-threads-and-ownership"></a>

## Identificadores, hilos y propiedad

`new_runtime()` inicia los hilos Rust de comandos y tareas. El `Runtime` temporal entrega un `CommandClient` y un `EventReceiver`. El hilo de interfaz usa el identificador de comandos. Un hilo receptor Qt dedicado posee el identificador de eventos y espera en `next_event()`. El hilo de comandos Rust posee el archivo comprimido actual y su índice. El hilo de tareas separado posee el trabajo preparado durante su ejecución.

CXX transporta tipos por valor y contenedores Rust propietarios. El código Qt receptor conserva o convierte estos valores mientras tiene su propiedad. Ningún identificador de archivo comprimido o archivo nativo cruza a Qt. Cuando los ID o recuentos exactos de bytes llegan a QML, el controlador usa cadenas para valores que necesitan conservar los 64 bits. Las cabeceras generadas y la biblioteca estática se compilan junto al host. Esta interfaz pertenece a la compilación del producto. El proceso de trabajo incluido usa un [protocolo de proceso](../engineering/native-codecs.md) separado.

[Declaraciones y conversiones del puente](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-qt-bridge/src/lib.rs), [Conexión Qt](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/RuntimeConnection.cpp), [Servicio Rust](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/service.rs)

<a name="xdzip-request-query-and-task-identities"></a>

## Identidades de solicitudes, consultas y tareas

| Identidad | Propósito |
| --- | --- |
| ID de solicitud | El servicio Rust asigna un ID al admitirla, dentro de la conexión. `Submission` devuelve `Accepted`, `Busy`, `Closed`, `Invalid` o `Exhausted`. Los rechazos devuelven ID cero. |
| ID de sesión | Identifica la instantánea del archivo abierto. Los comandos que actúan sobre él deben coincidir. Cada nueva apertura recibe un ID de sesión nuevo. |
| Identidad y desplazamiento de consulta | Una página se vincula a sesión, ID de consulta, generación de interfaz, revisión y desplazamiento. El controlador valida estos valores antes de aceptar filas. |
| ID de entrada física o navegación | Una entrada física conserva su identidad al ordenar y paginar. Los directorios sintetizados tienen identidades propias. Un índice de fila, nombre visible o rango de consulta nunca autoriza acceder a una ruta del sistema de archivos. |
| ID de tarea y aviso | Un ID de tarea pertenece a su conexión. Un aviso identifica una respuesta pendiente de conflicto o contraseña dentro de esa tarea. La entrada de tarea de la aplicación también conoce su controlador propietario. |
| Secuencia de eventos | El receptor asigna una secuencia creciente por conexión al tomar eventos. Qt descarta eventos cuya secuencia no sea posterior a la última consumida. Las actualizaciones de tareas comparten la secuencia y usan el ID de solicitud original para correlación. |

<a name="xdzip-commands-in-the-current-bridge"></a>

## Comandos del puente actual

| Comando o método | Propósito y resultado ordinario |
| --- | --- |
| `Open`, `OpenWithPassword` | Abrir y examinar la fuente elegida. Una respuesta correcta incluye la primera `Page` y datos de navegación acotados. Un directorio cifrado puede requerir contraseña antes de abrir una sesión. |
| `Browse` | Iniciar una consulta o continuarla con identidad y desplazamiento coincidentes. Cada comando devuelve como máximo una `Page`, limitada a 128 filas y 64 KiB de nombres. |
| `FreezeSelection`, `Preflight` | Resolver selectores actuales a ID físicos o de navegación con propiedad en `Selected`, o resumir hallazgos de metadatos en `Preflight`. La instantánea y el alcance seleccionado se vuelven a validar para la operación real. |
| `Preview` | Resolver una entrada física. Si tiene éxito, devolver bytes `Preview` acotados y con propiedad, junto a la identidad de entrada y sesión. |
| `CheckTarget`, `InspectInputs` | Devolver el número de restos en `TargetChecked` después de que el entorno abra y limpie un espacio de sondeo. La comprobación de creación usa sesión cero. `Inputs` informa de metadatos superficiales en orden de entrada, sin recorrer directorios recursivamente. Ninguna respuesta concede permiso para confirmar. |
| `Extract`, `Create`, `Check`, `OpenEntry` | Validar la intención y poner una tarea en cola. `TaskAccepted` contiene el ID asignado. El entorno y la capa de E/S autorizan el destino y confirman el resultado. |
| `Update`, `Move`, `Export`, `PublishExport`, `ExportManifest` | Las ediciones del archivo y la publicación de datos entran en la cola de tareas pesadas. Tras admitir `Update` o `Move`, el servicio descarta la instantánea actual para que solicitudes posteriores no usen sus antiguos ID de entrada. `ExportManifest` devuelve metadatos relativos de transferencia como respuesta ordinaria. |
| `Snapshot`, `CloseSession` | Devolver el ID de sesión del archivo actual o cero. Cerrar la sesión indicada descarta su instantánea de navegación mientras las entradas de tareas conservadas por separado pueden seguir activas. |
| `cancel`, `cancel_task`, `decide_task`, `respond_password`, `release_export`, `shutdown` | Métodos de control fuera de la cola ordinaria. La cancelación de una solicitud tiene ID propio y `Finished`; los controles de tarea actúan sobre la tarea existente. |

El servicio Rust expone otras operaciones específicas de la CLI como `Catalog` y `SelectMembers`. El puente valida el tamaño de entrada y los valores de enumeración antes del envío. Por ejemplo, las rutas de creación se limitan a 4 KiB cada una y 4 MiB en total. El entorno valida la sesión y la semántica de la operación, mientras la capa de E/S abre archivos y obtiene capacidades de salida. [Contrato del servicio de ejecución](../engineering/product-runtime.md), [Contrato CLI](../engineering/command-line.md), [Autorización de archivos](file-security.md)

<a name="xdzip-event-delivery-and-backpressure"></a>

## Entrega de eventos y contrapresión

Una solicitud ordinaria admitida produce como máximo un resultado de datos seguido de un `Finished` bajo entrega normal. Un error se representa con un `Finished` que lleva un código. El controlador espera a que termine correctamente antes de publicar la página o vista previa correspondiente. Un resultado reemplazado se descarta mientras su solicitud termina igualmente. La ruta de índices grandes almacena hasta 1 024 filas en Qt y solicita las páginas que faltan. Congela una selección entre páginas antes de presentar la confirmación de extracción. [Indexación de archivos grandes](../engineering/large-index.md), [Vista previa](../engineering/content-preview.md)

`TaskAccepted` y `Finished` pertenecen a la entrega ordinaria de solicitudes. `TaskUpdate` usa espacios pendientes separados por tarea y lleva el ID original de solicitud, el ID de tarea y el ID de sesión de origen. Una actualización puede llegar antes de `TaskAccepted` o después de `Finished`. El controlador correlaciona una tarea temprana con la solicitud original y dirige las actualizaciones posteriores a esa tarea. Una actualización posterior nunca vuelve a finalizar la solicitud ordinaria. Los espacios pendientes conservan la última instantánea de cada tarea. El progreso puede agruparse y la instantánea terminal permanece hasta que el receptor la toma. Los avisos aparecen en las instantáneas mientras la tarea espera su respuesta. [Estado de tareas y cancelación](task-scheduling.md)

- **Rust.** Por conexión, la admisión ordinaria permite 128 solicitudes pendientes y un presupuesto de 8 MiB para bytes de comandos en cola. El presupuesto de contenido de eventos ordinarios es 32 MiB. Las solicitudes aceptadas reservan espacio para resultado y finalización. Los acuses de cancelación de solicitudes tienen una cola acotada propia y el cierre no necesita espacio en ella.
- **Qt.** El buzón contiene hasta 256 eventos, hasta 128 eventos con contenido y 32 MiB de contenido contabilizado. Si se agota la capacidad, el receptor espera a que la interfaz vacíe el buzón. Una única cadena de notificación en cola procesa hasta 32 eventos o 4 ms por turno. La notificación llega al hilo de interfaz y los eventos conservan la propiedad de sus datos mientras esperan.

<a name="xdzip-prompts-failures-and-shutdown"></a>

## Avisos, fallos y cierre

- Las decisiones de conflicto usan `decide_task`. Las contraseñas de tareas usan `respond_password` y un ID de aviso de contraseña distinto. Ambos validan el aviso actual sin respuesta. La respuesta de contraseña entra en almacenamiento Rust que se borra al liberarse y nunca aparece en resultados ni eventos de progreso. `OpenWithPassword`, `Preview` y la creación cifrada llevan secretos de entrada específicos de la operación que el puente mueve a los tipos de contraseña Rust correspondientes. [Gestión de contraseñas](../engineering/password-extraction.md)
- Los códigos estables describen resultados de admisión, solicitud y tarea. Por ejemplo, `stale_session`, `stale_query`, `source_changed`, `unsafe_target`, `target_changed`, `disk_full` y `password_required`. El controlador traduce los códigos para mostrarlos. Un cambio de fuente invalida la instantánea de navegación correspondiente. `RuntimeFault` y desconexiones inesperadas invalidan el trabajo pendiente de la interfaz con `runtime_unavailable`; la pérdida de conexión puede impedir que lleguen los eventos terminales originales.
- `shutdown` cierra la admisión y cancela el trabajo aceptado, incluidas tareas que esperan una respuesta. El receptor Qt sigue vaciando eventos mientras terminan los hilos Rust. El receptor Rust espera su fin antes de entregar `ShutdownReady`, y después el puente entrega `End`. Qt informa de desconexión limpia tras vaciar el buzón. Si la conexión se destruye antes de tiempo, cierra el buzón y despierta al receptor sin esperar su fin en el hilo de interfaz. [Cierre de tareas](task-scheduling.md), [Ciclo de vida del entorno](../engineering/product-runtime.md)

Para cambios en este límite, la [sonda del puente](https://github.com/xiaodaozhi/xdzip/blob/main/tools/bridge-probe/README.md), las pruebas del servicio Rust y las [pruebas del controlador de escritorio](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/tests/browser_test.cpp) cubren capas distintas. Usa la [guía de compilación](../../engineering/developer-build.md) para localizar los objetivos pertinentes.
