<a name="xdzip-file-authorization-and-safe-publication"></a>

# Autorización de archivos y publicación segura

[English](../../design/file-security.md) | [简体中文](../../zh-CN/design/file-security.md) | [繁體中文（台灣）](../../zh-TW/design/file-security.md) | [日本語](../../ja-JP/design/file-security.md) | [한국어](../../ko-KR/design/file-security.md) | [Français](../../fr-FR/design/file-security.md) | [Deutsch](../../de-DE/design/file-security.md) | Español | [Português (Brasil)](../../pt-BR/design/file-security.md) | [Русский](../../ru-RU/design/file-security.md)

El host controla qué archivos pueden abrirse y dónde puede publicarse la salida verificada. Un motor recibe lectores y sumideros acotados, también cuando se ejecuta en el proceso de archivos aislado. Los nombres, tamaños declarados, sumas y contenidos del archivo comprimido son datos de entrada. Las comprobaciones de archivos del host se aplican independientemente de las del formato. [Arquitectura del sistema](system-architecture.md), [Protocolo del proceso de archivos](../engineering/native-codecs.md)

<a name="xdzip-authorizing-the-source-and-destination"></a>

## Autorizar origen y destino

- **Origen del archivo comprimido.** `Source::open` abre la ruta seleccionada mediante `xdzip-io::open_regular_readonly` y comprueba el tipo del archivo abierto. En Unix, la apertura de clasificación no bloquea, de modo que un FIFO no puede detenerla. Una ruta elegida por el usuario puede resolverse mediante un enlace a un archivo normal. El descubrimiento de volúmenes divididos está acotado y depende del formato. El host congela las partes vecinas admitidas y comprueba sus identidades y versiones. Un códec solo puede acceder a los flujos suministrados. [Implementación de origen](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/source.rs), [Gestión de volúmenes](../engineering/archive-opening.md)
- **Entradas de creación.** `InputTree` recorre las entradas elegidas con metadatos e identidades de archivos acotados. Las lecturas abren miembros de ese árbol comprobando sus versiones. El directorio privado de preparación actual se excluye de su propio examen de entrada. Un cambio detectado detiene el trabajo. Las comprobaciones de identidad, tamaño y tiempo detectan cambios con el mejor esfuerzo, pero no constituyen una instantánea completa del contenido. [Árbol de entrada](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-io/src/input.rs), [Creación de archivos](../engineering/zip-creation.md)
- **Destino.** `Grant` conserva el objeto directorio elegido y su identidad. La extracción también pasa las identidades de todos los volúmenes de origen como archivos protegidos. Creación y edición comprueban las identidades congeladas de entrada o del archivo original mediante sus propias rutas de tarea. Se accede a subdirectorios y preparación mediante el directorio retenido y se vuelven a comprobar identidades de padres antes de publicar. `CheckTarget` y la comprobación previa de extracción pueden proporcionar información útil antes de iniciar la tarea. La autorización de escritura se comprueba otra vez durante la operación. [Autorizaciones de destino](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-io/src/target.rs), [Comprobación previa de extracción](../engineering/extraction-preflight.md)

<a name="xdzip-name-mapping-platform-protection-and-links"></a>

## Asignación de nombres, protección por plataforma y enlaces

1. **Validar la escritura relativa al archivo comprimido.** El entorno resuelve la selección congelada, elimina el prefijo del subárbol seleccionado cuando corresponda y usa `PortablePath` para aceptar componentes UTF-8 relativos. Se rechazan rutas absolutas o de unidad, caracteres de control, separadores ambiguos, componentes padres, nombres de dispositivo reservados y espacios o puntos finales. Una escritura validada es solo una entrada para la capacidad de destino.
2. **Planificar colisiones antes de crear salida.** La extracción compara alias comunes de mayúsculas y Unicode mediante claves NFD en minúsculas. Escrituras ambiguas de directorios y colisiones archivo/directorio hacen fallar la preparación. Los alias de archivos normales se asignan a una escritura común para aplicar la política de conflicto. La publicación nativa exclusiva sigue decidiendo si el nombre solicitado está disponible en el sistema de archivos real. La escritura visible y la identidad física de la entrada permanecen separadas.
3. **Recorrer el árbol autorizado.** La salida desciende desde una capacidad de directorio retenida y abre directorios hijos sin seguir enlaces. Se vuelven a comprobar la raíz autorizada, identidades de padres y hoja esperada para sobrescribir. Se rechaza un destino sustituido o enlace inseguro. Una última comprobación antes de renombrar en el mismo volumen reduce carreras. El renombrado normal no garantiza comparación e intercambio atómico del contenido frente a un proceso con autoridad de la misma cuenta.

| Plataforma | Comprobaciones de objetos de salida |
| --- | --- |
| Linux y macOS | El propietario debe ser el usuario efectivo actual. Se rechazan permisos de escritura de grupo y otros, y un archivo normal existente debe tener un enlace físico. macOS también comprueba la ACL nativa, incluidos permisos de modificación heredados. |
| Windows | Comprobar puntos de reanálisis, propietario, ID de archivo, número de enlaces físicos para archivos normales y DACL de los objetos retenidos. Las reglas efectivas y solo de herencia deben limitar la modificación a entidades de confianza. Puede mantenerse el uso compartido existente de solo lectura. |
| Texto sin cifrar privado o espacio temporal | Comprobaciones adicionales de confidencialidad excluyen lectores ajenos. En Windows un directorio privado de caché creado exclusivamente puede recibir una DACL protegida. Los destinos de extracción normales elegidos por el usuario conservan su política de lectura compartida. |

La salida de enlaces simbólicos ZIP se limita a un grafo Unix admitido. El entorno valida el grafo seleccionado y las hojas que deben estar ausentes antes de publicar enlaces, confirmando primero los destinos normales. La tarea nunca sigue un enlace creado por el archivo comprimido para escribir contenido. Otros perfiles no admitidos de enlaces o archivos especiales fallan explícitamente. Los modos POSIX normales pueden restaurar bits seguros de lectura/ejecución en la preparación, además de lectura/escritura del propietario. Se excluyen bits de privilegio y escritura externa. La extracción no restaura marcas de tiempo tras confirmar.

[Analizador de rutas portables](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-domain/src/path.rs), [Planificación de extracción](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/extraction/plan.rs), [Comprobaciones de objetos privados](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-io/src/private.rs), [Detalles de enlaces y plataformas](../engineering/zip-safe-extraction.md)

<a name="xdzip-staging-verification-and-commit"></a>

## Preparación, verificación y confirmación

Una autorización de destino crea un espacio privado exclusivo en el mismo volumen. El host mide la salida del códec en su archivo de preparación. Antes de publicar deben tener éxito la decodificación, el tamaño esperado y las comprobaciones de integridad o autenticación que proporcione el formato. `Stage::finish` vacía y sincroniza el archivo y conserva su versión comprobada. El host vuelve a comprobar preparación y destino al confirmar. La sincronización de datos y visibilidad atómica de un archivo no demuestran durabilidad de metadatos de directorio ante cortes de energía.

| Operación | Semántica de publicación |
| --- | --- |
| Extraer o publicar una promesa de archivo | Publicar cada archivo elegible tras verificarlo. Sin sobrescribir, crear de forma exclusiva un enlace físico en el mismo volumen y después eliminar el enlace privado de preparación. Con sobrescritura, la extracción exige el archivo normal seguro esperado y lo reemplaza mediante renombrado en el mismo volumen sin truncarlo primero. Una promesa de archivo siempre se publica sin sobrescribir. Si el sistema de archivos carece de la operación necesaria o cambia el destino, falla la escritura afectada. No hay reversión global de archivos ya publicados ni directorios creados. |
| Crear un archivo comprimido único | Codificar en preparación, volver a abrir y verificar entradas y contenido esperados para el perfil y publicar el archivo completo una vez. Un conflicto de nombre puede elegir otro nombre o permitir reemplazo elegible. Una entrada protegida no puede ser destino de sobrescritura. |
| Crear volúmenes divididos | Verificar el contenido preparado, preparar cada parte y publicar el conjunto plano nuevo sin sobrescribir volúmenes existentes. Las partes se hacen visibles individualmente. Ante fallo parcial, la reversión intenta eliminar solo objetos recién publicados cuya identidad aún coincida. Una reversión fallida puede dejar resultado incierto. |
| Modificar un archivo existente | `ArchiveUpdate` vincula el directorio padre autorizado y la versión esperada del archivo original. Volver a abrir y verificar el archivo reescrito y reemplazar el original una vez mediante esa autoridad. La extracción normal no puede usar esta vía especial para sobrescribir su origen. |

La cancelación antes de la barrera de publicación impide la confirmación actual. Una publicación ya iniciada puede concluir y comunicar su resultado real. Los archivos de extracción confirmados anteriormente permanecen visibles. La instantánea terminal puede informar de datos confirmados junto con `cancelled`, `failed` o una advertencia de limpieza. Una publicación completa de archivo comprimido que tuvo éxito mantiene el éxito aunque llegue una cancelación tardía. [Cancelación de tareas](task-scheduling.md), [Modificación de archivos](../engineering/archive-transfers.md), [Extracción segura](../engineering/zip-safe-extraction.md)

<a name="xdzip-output-budgets-and-integrity-scope"></a>

## Presupuestos de salida y alcance de integridad

La extracción limita por separado la salida declarada y real: 8 GiB por archivo normal y 20 GiB por tarea. Una comprobación de relación elevada puede rechazar la solicitud al admitirla en formatos compatibles. Para ZIP, una relación declarada superior a 1 000 a uno en una entrada de más de 1 MiB requiere autorización explícita. Esta autorización mantiene los límites estrictos. Los decodificadores sólidos o secuenciales pueden trabajar en datos no seleccionados. Ese trabajo, las lecturas o intentos repetidos y el trabajo auxiliar se contabilizan por separado de los bytes publicados.

La navegación de metadatos, admisión de tareas, memoria del proceso y vista previa tienen presupuestos propios. Un índice ZIP grande en disco no aumenta límites de extracción ni escritura. Las comprobaciones de integridad dependen del formato. Por ejemplo, estructura y legibilidad ISO/UDF no aportan una suma por archivo inexistente. [Índices de archivos grandes](../engineering/large-index.md), [Comprobación de integridad](../engineering/archive-checking.md), [Contabilidad auxiliar de decodificación](../engineering/auxiliary-decoding-work.md)

<a name="xdzip-preview-temporary-opening-and-secrets"></a>

## Vista previa, apertura temporal y secretos

- **Vista previa.** Seleccionar un archivo compatible inicia una vista previa en la tarjeta de propiedades visible. Rust lee y verifica toda la entrada admitida dentro de sus presupuestos. El texto está limitado a 256 KiB. Las imágenes codificadas se limitan a 16 MiB, con límites separados de dimensiones y píxeles originales. El proceso privado de imagen recibe bytes y devuelve una miniatura acotada. No recibe nombre de archivo comprimido, contraseña ni identificador de destino. Los formatos y restricciones actuales se describen en el [contrato de vista previa](../engineering/content-preview.md).
- **Apertura temporal.** Una entrada normal se extrae a la caché privada con las reglas normales de verificación de la tarea. El escritorio entrega el archivo verificado a la asociación del sistema como URL local. Que se acepte la entrega no demuestra que el programa externo haya terminado de leer. En una operación posterior, tras el periodo de retención, la limpieza comprueba identidad de caché y bloqueos. Las ediciones de la copia son independientes del archivo original. [Apertura temporal](../engineering/archive-opening.md), [Transferencias de archivos](../engineering/archive-transfers.md)
- **Secretos.** Las contraseñas de apertura y extracción permanecen en contenedores específicos de la operación que se ponen a cero al liberarse durante la vida de la sesión o tarea. No entran en instantáneas de tarea ni en el diario de preparación. Las reglas de almacenamiento privado, retención y divulgación de datos del usuario se describen en el [contrato de privacidad](../engineering/privacy.md) y el [contrato de contraseñas](../engineering/password-extraction.md).

<a name="xdzip-journals-leftovers-and-verification"></a>

## Diarios, restos y verificación

Los diarios de preparación registran el contexto de tarea y nombres relativos de salida durante preparación y confirmación. La limpieza normal comprueba propiedad e identidades y elimina solo objetos de preparación conocidos. Los diarios privados y objetos restantes pueden aportar evidencia tras una terminación anormal. El entorno cuenta posibles restos al comprobar un destino. No usa un diario descubierto para repetir escrituras ni eliminar contenido desconocido. La caducidad de caché de salida temporal privada tiene sus propias comprobaciones de identidad y bloqueo.

Para cambios en este límite, verificar publicación y limpieza además de códigos de error. Las [pruebas de E/S](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-io/src/tests.rs), [pruebas de extracción](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/extraction/tests.rs) y el [contrato de extracción segura](../engineering/zip-safe-extraction.md) cubren capas distintas. Los cambios de comportamiento nativo requieren evidencia específica de cada plataforma afectada.
