<a name="xdzip-split-volume-reading-and-temporary-opening"></a>

# Lectura de volúmenes y apertura temporal

[English](../../engineering/archive-opening.md) | [简体中文](../../zh-CN/engineering/archive-opening.md) | [繁體中文](../../zh-TW/engineering/archive-opening.md) | [日本語](../../ja-JP/engineering/archive-opening.md) | [한국어](../../ko-KR/engineering/archive-opening.md) | [Français](../../fr-FR/engineering/archive-opening.md) | [Deutsch](../../de-DE/engineering/archive-opening.md) | Español | [Português (Brasil)](../../pt-BR/engineering/archive-opening.md) | [Русский](../../ru-RU/engineering/archive-opening.md)

La selección en el escritorio, las solicitudes del gestor de archivos y la CLI usan las mismas reglas Rust para abrir fuentes. El anfitrión resuelve los conjuntos de volúmenes permitidos, registra su identidad y examina los bytes autorizados. El motor seleccionado proporciona los [metadatos de navegación](readonly-browser.md), a veces mediante el [proceso de formato aislado](native-codecs.md). Al abrir expresamente una entrada se crea una copia temporal verificada para la aplicación asociada. Usa una [comprobación de integridad](archive-checking.md) independiente para verificar todo el archivo comprimido.

Si la cabecera parece ser TAR pero falla la comprobación estructural completa, se usa el mismo orden de candidatos y el mismo límite de entrada compartido que para una fuente no identificada. Solo se prueba LZMA independiente al final, cuando no se ha identificado una cabecera TAR ni un ZIP autoextraíble válido.

<a name="xdzip-split-zip"></a>

## ZIP dividido

La capa de lectura admite flujos continuos numerados como `name.zip.001`, `.002` y volúmenes PKZIP como `name.z01`, `.z02` y el último `name.zip`. Puede abrirse desde cualquier volumen. Deben estar en la misma carpeta, con nombres consecutivos y un máximo de 1 024 volúmenes; se rechazan los volúmenes ausentes, enlaces y archivos especiales. Las extensiones solo se usan para descubrir conjuntos adyacentes. El contenedor real se identifica por sus bytes de origen y su motor lo examina por completo.

Cuando el número de disco clásico de ZIP64 es 0xFFFF, primero se descubre el conjunto real usando el número total acotado de volúmenes del locator adyacente. El marcador no significa que falten 65 535 volúmenes. Este descubrimiento no sustituye la validación completa del registro final y del directorio.

El anfitrión Rust conserva identidad, longitud y fecha de modificación de cada volumen. La entrada lógica lee entre volúmenes bajo demanda sin unir todo el archivo. Los desplazamientos PKZIP relativos al disco se convierten en una vista acotada del directorio central en un volumen. El códec existente sigue comprobando el directorio, las cabeceras locales, CRC, autenticación de contraseña y cuotas de extracción. Todos los volúmenes de origen se vuelven a verificar en cada uso y antes de confirmar; cualquier cambio detiene la tarea. Las identidades de todos están protegidas frente a la sobrescritura por la salida.

La normalización PKZIP solo procesa registros centrales admitidos y no admite el registro final de firma digital del directorio central. Este rechazo no significa que haya fallado la verificación de una firma. Un archivo normal de un solo volumen puede leer ese registro dentro de los límites, pero no verifica su firma criptográfica y rechaza modificaciones para evitar perder o dañar metadatos desconocidos al reubicarlos.

La aceptación cubre volúmenes numerados, PKZIP, ZIP64, cabeceras y contenidos que cruzan volúmenes, volúmenes ausentes o no consecutivos, sustitución de la fuente, números de disco inválidos, nombres chinos y regresiones en archivos normales de un volumen.

<a name="xdzip-7z-and-rar-volumes"></a>

## Volúmenes 7z y RAR

Los volúmenes 7z numerados forman flujos continuos como name.7z.001 y .002. RAR usa name.partN.rar o nombres antiguos como name.rar y name.r00; al entregar los volúmenes al lector RAR aislado de Rust se conservan sus límites físicos. Ambos localizan el conjunto en la misma carpeta desde cualquier parte existente. Los límites son 1 024 volúmenes y 100 000 entradas durante la búsqueda en el directorio. Se rechazan expresamente volúmenes ausentes, conflictos de numeración, enlaces y puntos de reanálisis.

Rust conserva identidad, longitud y marcas de tiempo del identificador de cada volumen y declara los límites autorizados al adaptador nativo mediante ReadSource::volume_lengths(). El proceso nativo solo puede solicitar al anfitrión lecturas de rangos autorizados; el cambio de volumen no acepta rutas arbitrarias del anfitrión proporcionadas por el archivo. Todos los volúmenes participan en la nueva verificación de la fuente y la protección contra sobrescritura. Un cambio en el primer volumen o en cualquiera posterior detiene la tarea.

El cifrado de nombres 7z y RAR puede requerir contraseña al abrir. Si Open devuelve password_required, no se publica ningún índice. Qt reutiliza la ventana de contraseña y vuelve a enviar el archivo originalmente seleccionado y el secreto acotado mediante OpenWithPassword; un valor de verificación incorrecto devuelve wrong_password. Tras el éxito, solo la sesión y sus tareas admitidas conservan la contraseña. Al cerrar la sesión se libera su copia; no se escribe en los ajustes ni en los registros de recuperación.

<a name="xdzip-cab-volumes"></a>

## Volúmenes CAB

Los nombres de volúmenes CAB proceden de las cabeceras y pueden ser cualquier nombre de archivo válido; no se aplican las reglas de concatenación de bytes de volúmenes numerados. La referencia siguiente avanza un volumen cada vez. Una referencia lógica al anterior puede apuntar a un volumen más antiguo donde comenzó un archivo continuado, en vez del volumen inmediatamente anterior; algunas muestras existentes también usan referencias anteriores adyacentes. Desde el volumen seleccionado, Rust sigue hacia atrás las referencias anteriores y después descubre hacia delante el conjunto completo en la misma carpeta. Tras validar la identidad del conjunto, el orden consecutivo y las identidades, fija la longitud y el nombre original de cada volumen. El decodificador verifica todos los fragmentos; un volumen anterior no adyacente debe ser el volumen donde realmente comenzó un archivo continuado. Los límites son 1 024 volúmenes y 100 000 entradas de búsqueda en la misma carpeta. Se rechazan volúmenes ausentes o incorrectos, ciclos, identidades duplicadas, rutas absolutas, separadores de directorio, escapes al directorio padre, enlaces y puntos de reanálisis. Un subconjunto abierto no se publica como archivo completo.

Las llamadas de cambio de volumen del proceso solo pueden seleccionar nombres de este conjunto autorizado; cada volumen ofrece su propio flujo acotado. Todos participan en la nueva verificación de la fuente antes de las tareas y de confirmar archivos, y en la protección contra sobrescritura. Las advertencias del controlador nativo o los resultados de volumen ausente no pueden convertirse en un índice correcto. Los archivos CAB de uno o varios volúmenes son de solo lectura.

<a name="xdzip-double-clicking-regular-files"></a>

## Doble clic en archivos normales

Un doble clic en un directorio sigue entrando en él. En un archivo normal, envía una tarea de extracción en segundo plano con esa única entrada física a un directorio de nombre aleatorio en la caché privada del usuario. Los enlaces y objetos especiales no se pueden abrir por esta vía. Siguen aplicándose la contraseña, CRC, autenticación, cancelación y admisión secuencial de tareas pesadas de la aplicación; no hace falta elegir un destino.

Tras obtener la salida, se invoca la asociación del sistema con una URL de archivo local, sin intérprete de comandos. La ventana de progreso correspondiente solo se oculta automáticamente si el sistema acepta la solicitud de apertura. Si falla la extracción, se cancela o el sistema la rechaza, permanece visible con un error legible. Que el sistema acepte no implica que la aplicación externa haya terminado de leer; el archivo temporal no puede eliminarse al enviar la solicitud.

Los directorios temporales usan nombres aleatorios impredecibles de 128 bits y separan contenido y marcador de identidad. El directorio de salida solo permite acceso al usuario actual y a las entidades de sistema de confianza. En Windows se establece una DACL protegida mediante el identificador retenido únicamente en un directorio vacío creado exclusivamente para esta operación, y se comprueba de nuevo antes de escribir texto sin cifrar; no se modifican los permisos compartidos de una carpeta elegida por el usuario. POSIX usa 0700 y comprueba las ACL heredadas de macOS. Si no pueden garantizarse los permisos, se rechaza la salida; consulta la [API DACL de Win32](https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-setsecurityinfo).

No se limpia mientras exista un bloqueo. La salida terminada se conserva al menos siete días; después, los directorios caducados y desbloqueados se limpian durante una apertura temporal posterior. La caché conserva como máximo 256 entradas y rechaza expresamente al llegar al límite; cada exploración se limita a 4 096 entradas. La limpieza valida permisos privados, marcadores y nombres y no sigue enlaces. Los cambios de una copia temporal no se guardan en el archivo comprimido.

La limpieza se limita a la caché privada creada por la aplicación; no examina ni elimina directorios de extracción del usuario. La aceptación cubre ocultación automática al terminar, ventanas visibles ante errores, cancelación, reintentos de contraseña, nombres duplicados, Unicode y caracteres especiales, enrutamiento de tareas tras cerrar la página, seguridad del directorio temporal y política de limpieza al salir.

La admisión de solo lectura de [Brotli/TAR.BR](brotli-streams.md) valida por completo los candidatos sin cabecera antes de publicar filas, comparte el registro de lecturas de candidatos Brotli/LZMA, conserva la prioridad de fuentes fuertes y de la detección TAR/ZIP válida y nunca trata un TAR decodificado dañado como flujo normal.

Las propuestas [LHA/LZH](lha-containers.md) requieren validar por completo y dentro de límites las cabeceras y nombres antes de publicarlos, también para un archivo vacío de un solo cero. Las propuestas débiles LHA, Brotli y LZMA comparten un registro de lecturas reales. La detección de contenedores fuertes válidos conserva la prioridad; una admisión LHA completa puede resolver la ambigüedad de un marcador TAR sin procesar inválido. No se admiten perfiles de contraseña, volúmenes ni reparación.

[PMA](pma-streams.md) usa el mismo registro compartido de sondeos débiles y la admisión completa de metadatos, con un terminador cero exacto y relleno CP/M opcional y acotado. Se admiten contenidos PM0/PM1/PM2 de nivel 0; los métodos desconocidos siguen visibles pero no son compatibles. Un contenedor CP/M vacío con relleno identifica PMA; un terminador cero de un byte indistinguible conserva la elección LZH anterior. No se admiten perfiles de creación, contraseña, volúmenes, modificación ni reparación.
