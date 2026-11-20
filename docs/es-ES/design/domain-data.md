<a name="xdzip-domain-model-and-archive-data"></a>

# Modelo de dominio y datos de archivos comprimidos

[English](../../design/domain-data.md) | [简体中文](../../zh-CN/design/domain-data.md) | [繁體中文](../../zh-TW/design/domain-data.md) | [日本語](../../ja-JP/design/domain-data.md) | [한국어](../../ko-KR/design/domain-data.md) | [Français](../../fr-FR/design/domain-data.md) | [Deutsch](../../de-DE/design/domain-data.md) | Español | [Português (Brasil)](../../pt-BR/design/domain-data.md) | [Русский](../../ru-RU/design/domain-data.md)

El entorno de ejecución posee la sesión del archivo comprimido, el índice completado y la última consulta de esa sesión. Qt mantiene una vista del resultado y la CLI puede leer registros físicos mediante su comando de catálogo independiente. Ambos envían operaciones al mismo servicio Rust. [Arquitectura del sistema](system-architecture.md), [Puente Qt/Rust](qt-rust-bridge.md)

<a name="xdzip-identity-and-source-version"></a>

## Identidad y versión del origen

| Identidad o datos | Representación y alcance actuales |
| --- | --- |
| `SessionId` | Identidad de ejecución distinta de cero para un archivo comprimido abierto. Cada nueva apertura correcta crea una sesión nueva. Las rutas de archivo se usan para entrada y enrutamiento de interfaz. |
| `SourceVersion` | El entorno registra identidad, longitud y marcas de tiempo disponibles del archivo regular de cada parte admitida. Vuelve a comprobar el objeto abierto y la ruta correspondiente en los límites pertinentes de lectura y escritura. La detección de cambios se realiza según el mejor esfuerzo. |
| `EntryId` | Identifica un registro físico del archivo comprimido dentro de una sesión. Los registros con igual nombre mantienen identificadores distintos. |
| Identidad de navegación | Representa un directorio indexado. Un directorio virtual sintetizado a partir de descendientes puede aparecer en árbol o lista sin identificador de entrada física. |
| `QueryKey` | Combina sesión, identificador de consulta, generación y revisión de instantánea. El desplazamiento indica una posición de ese resultado. |
| Identidad de petición, tarea y aviso | El servicio asigna números de petición y tarea dentro de una conexión. Un aviso pertenece a una tarea concreta. La entrada compartida de tareas de escritorio usa la identidad de sesión del controlador junto al número de tarea. |

Los tipos de dominio exigen identidades de archivo comprimido y consulta distintas de cero. En el puente nativo los valores Rust tipados se representan mediante datos CXX generados. QML recibe cadenas opacas para identificadores que necesitan los 64 bits. Un número de fila es solo una posición en una consulta concreta. [Identidades de dominio](../../../crates/xdzip-domain/src/identity.rs), [Implementación del origen](../../../crates/xdzip-runtime/src/source.rs), [Sesiones de aplicación](../../engineering/application-sessions.md)

<a name="xdzip-physical-entries-and-output-names"></a>

## Entradas físicas y nombres de salida

`xdzip-codec-api::Entry` contiene formato, identidad física, representaciones del nombre, tamaños, indicadores de tipo de archivo, datos de modificación y campos de método y cifrado específicos del formato. Se almacenan por separado los bytes originales del nombre, su escritura UTF-8 decodificada opcional, la decisión de codificación y el texto de visualización escapado. Para la salida, el entorno valida el nombre efectivo con la política de rutas portables y el destino autorizado. Un nombre sin resolver puede permanecer visible mientras se rechaza la extracción de ese registro o directorio.

El tamaño original tiene un indicador independiente `size_known`, que distingue archivos de tamaño cero conocido de tamaños desconocidos. Algunos contenedores comprimidos carecen de tamaño comprimido por entrada. Los valores de método se interpretan según el formato. Los datos de tiempo conservan también las convenciones del formato, como la hora DOS local de ZIP; un tiempo ausente sigue siendo desconocido. `payload_supported()` determina si el registro actual puede decodificarse. La salida también depende de la operación elegida, estado del origen, nombre y reglas del sistema de archivos. [Campos de entrada](../../../crates/xdzip-codec-api/src/lib.rs), [Tratamiento de nombres](../engineering/filename-encoding.md), [Autorización de archivos](file-security.md)

<a name="xdzip-index-query-and-page-lifecycle"></a>

## Ciclo de vida de índice, consulta y página

El entorno abre el origen y completa su análisis acotado antes de publicar el archivo comprimido y la primera página. El análisis puede cancelarse. Una cola dañada, contraseña ausente o cambio de origen impide publicar un índice parcial. La navegación tiene un presupuesto independiente. Si el árbol de directorios lo supera, todavía pueden ofrecerse navegación de raíz y búsqueda física. ZIP/ZIP64 ordinario de un solo archivo puede contener hasta un millón de registros mediante la ruta privada en disco. Otros formatos y rutas divididas mantienen sus propios límites.

`Archive::query_page` acepta directorio, texto, modo de ordenación, desplazamiento e identidad de consulta esperada. Una consulta nueva comienza en desplazamiento cero con una generación nueva. El entorno construye el orden completo del resultado y mantiene una consulta actual inmutable por archivo comprimido. Las páginas resuelven identidades de filas a partir de ese orden. Búsqueda y ordenación mantienen las identidades originales de los registros. Cada modo agrupa directorios antes de archivos, desempata de forma determinista por identidad y sitúa tamaños desconocidos después de conocidos dentro de cada grupo.

El tipo genérico de dominio `PageRequest` valida hasta 256 filas. El transporte de ejecución de escritorio solicita hasta 128 filas con presupuesto de nombres de 64 KiB. Qt confirma íntegramente resultados pequeños y mantiene como máximo 1 024 filas residentes para uno grande. Las respuestas con sesión, consulta, generación, revisión o desplazamiento distintos no pueden sustituir los datos actuales. [Implementación de consultas](../../../crates/xdzip-runtime/src/query.rs), [Listado y ordenación](../../../crates/xdzip-runtime/src/listing.rs), [Índices grandes](../engineering/large-index.md)

<a name="xdzip-selections-and-operation-input"></a>

## Selecciones y entrada de operaciones

El modelo Qt guarda por identificador las selecciones de resultados cargados y, para resultados grandes, como rangos acotados del resultado actual. Seleccionar todo cubre ese resultado. Una tarea puede distinguir el archivo comprimido completo de un conjunto explícitamente seleccionado de resultados de búsqueda. Rust resuelve selectores entre páginas con la clave completa de consulta mediante `FreezeSelection` para confirmar la extracción. La expansión de directorios resuelve descendientes reales y elimina identificadores físicos duplicados. Superar el límite de la operación devuelve un error. La CLI resuelve rutas de miembros e identificadores físicos mediante `SelectMembers`.

Extracción, comprobación y edición preparan trabajos específicos con el origen y metadatos conservados necesarios. La creación recibe opciones y después analiza y valida un `InputTree` independiente. La precomprobación de metadatos puede describir problemas potenciales sin conceder autoridad de salida a una tarea. Antes de escribir, el entorno vuelve a comprobar entradas y obtiene la autorización pertinente. Una vez que un trabajo posee su selección, los cambios en la interfaz no alteran su alcance. [Modelo de escritorio](../../../apps/desktop/BrowserController.cpp), [Comandos de ejecución](../../../crates/xdzip-runtime/src/service.rs), [Planificación de tareas](task-scheduling.md)

<a name="xdzip-results-and-data-lifetime"></a>

## Resultados y duración de los datos

| Datos | Duración |
| --- | --- |
| Archivo comprimido, consulta e índice en disco actuales | Pertenecen a la sesión del servicio. Sustituir la consulta o cerrar la sesión libera los datos privados asociados, salvo recursos que los trabajos conservan independientemente. El índice de metadatos en disco se reconstruye al reabrir el archivo comprimido. |
| Instantáneas de tareas | El entorno publica instantáneas acotadas con estado real, recuentos de bytes y elementos, estado de limpieza y lista limitada de detalles. El escritorio mantiene disponibles los resultados mediante el controlador propietario durante la sesión de la aplicación. |
| Preferencias de usuario | Se gestionan por separado de índices y resultados de tareas. La conservación y privacidad se documentan en los contratos de configuración y privacidad. |
| Espacios de trabajo y diarios | La preparación privada de salida y los registros relativos de publicación permiten limpiar e inspeccionar. Un reinicio no repite una escritura incompleta del archivo comprimido. |

[Campos de instantánea](../../../crates/xdzip-runtime/src/extraction/mod.rs), [Publicación de archivos](file-security.md), [Configuración y localización](../ux/settings-localization.md), [Ciclo de datos y privacidad](../engineering/privacy.md)

<a name="xdzip-design-options-outside-the-current-implementation"></a>

## Opciones de diseño fuera de la implementación actual

La publicación incremental de índices consultables, un plan reutilizable con revisión y token de inicio propios, y un historial persistente de tareas paginado son opciones de diseño. Los comandos actuales preparan y encolan sus propios trabajos acotados, mantienen una consulta actual y devuelven instantáneas de tareas limitadas. La recuperación persistente también requeriría un diseño separado para validar origen y salida.
