<a name="xdzip-external-read-only-sdk-trial"></a>

# Prueba externa de SDK de solo lectura

[English](../../engineering/sdk-trial.md) | [简体中文](../../zh-CN/engineering/sdk-trial.md) | [繁體中文](../../zh-TW/engineering/sdk-trial.md) | [日本語](../../ja-JP/engineering/sdk-trial.md) | [한국어](../../ko-KR/engineering/sdk-trial.md) | [Français](../../fr-FR/engineering/sdk-trial.md) | [Deutsch](../../de-DE/engineering/sdk-trial.md) | Español | [Português (Brasil)](../../pt-BR/engineering/sdk-trial.md) | [Русский](../../ru-RU/engineering/sdk-trial.md)

`xdzip-plugin-protocol` y `xdzip-plugin-host` implementan la prueba independiente de solo lectura descrita en el [diseño de extensión de formatos](../design/codec-extension-sdk.md). Está excluida de la detección de formatos en escritorio y CLI, instalación, empaquetado e inicio automático. El usuario ejecuta explícitamente el comando de prueba y selecciona paquete y origen; las extensiones de terceros siguen desactivadas en el producto. Solo se negocia `xdp1`, sin creación, contraseñas, volúmenes, enlaces, interfaz gráfica ni permisos arbitrarios sobre rutas del anfitrión. No es un ecosistema completo de complementos ni una publicación del SDK.

<a name="xdzip-reproducible-entry-points"></a>

## Puntos de entrada reproducibles

```sh
python3 scripts/sdk-trial.py test
python3 scripts/sdk-trial.py example --output-dir build/sdk-example
target/debug/xdzip-plugin-host scan build/sdk-example/owned-package/manifest.json build/sdk-example/owned.xdp
mkdir build/sdk-example/output
target/debug/xdzip-plugin-host read build/sdk-example/owned-package/manifest.json build/sdk-example/owned.xdp 0 build/sdk-example/output
```

En Windows usar `python` y `.exe`. `test` compila el programa nativo de referencia y ejecuta regresiones Rust de entramado, interoperabilidad Python independiente, protocolo malicioso, cancelación, cambio de origen, conflicto y permisos del núcleo. Python solo lo inicia el punto de conformidad de confianza compilado para pruebas; el anfitrión distribuido no acepta intérprete, argumentos, shell ni opción de ejecución sin aislamiento. El ejemplo genera un paquete propio, manifiesto y XDP1 solo bajo `build/`; no activa publicación ni descarga de terceros. Los registros omiten rutas del espacio de trabajo y del directorio personal; el JSON vincula commit, estado de modificaciones y hashes binarios. Ambos son artefactos locales o de CI. La misma matriz nativa Foundation llama a este punto; el [flujo independiente de prueba SDK](../../../.github/workflows/sdk-trial.yml) resuelve primero la compatibilidad de aislamiento por plataforma sin repetir los procesos ajenos de formatos y Qt. El flujo Foundation también puede llamar a la prueba reutilizable con `sdk_only=true`; esto omite el control Foundation completo y no demuestra un resultado Foundation completo. Windows verifica el rechazo de ejecución.

<a name="xdzip-package-admission"></a>

## Admisión del paquete

El manifiesto JSON estricto usa `deny_unknown_fields`. Los campos son `manifest_version=1`, `plugin_id`, `version`, `author`, `protocol_majors=[1]`, `targets`, un `entrypoint` dentro del paquete, `sha256` hexadecimal en minúsculas, `formats=["xdp1"]`, `permissions`, `resources`, `license_files` y una `source_url` HTTPS. El manifiesto de referencia generado por `example` arriba es una entrada ejecutable del esquema actual.

Las `permissions` son exactamente `broker_source_read=true`, `broker_output_write=true`, `network=false`; los `resources` son exactamente `memory_bytes=536870912`, `threads=1`. El manifiesto está limitado a 32 KiB, el ejecutable nativo a 64 MiB y se admiten hasta cuatro archivos de licencia de 128 KiB cada uno. Los puntos de entrada y archivos de licencia no pueden ser absolutos, contener recorrido al directorio padre ni escapar mediante enlaces o puntos de reanálisis. Cada miembro se abre componente a componente sin seguir enlaces mediante un directorio cap-std mantenido y no se reabre mediante su cadena original tras la verificación. El anfitrión copia desde el archivo ordinario mantenido, comprueba el número real de bytes, SHA-256 y versión del archivo antes y después de copiar, y ejecuta solo la copia temporal privada, que conserva hasta finalizar el proceso y la IPC. El hash declarado establece coherencia del contenido del paquete seleccionado; no autentica al editor ni demuestra que el código sea seguro.

<a name="xdzip-wire-protocol-and-state"></a>

## Protocolo y estado

Un subconjunto CBOR propio y acotado sigue [RFC 8949](https://www.rfc-editor.org/rfc/rfc8949.html) sin implementación CBOR de terceros. El par Rust de referencia reutiliza el crate de entramado; el [par Python](../../../examples/sdk/reference.py) implementa codificación, descodificación y análisis XDP1 independientemente, sin importar Rust ni código de protocolo generado. La interoperabilidad cubre varios bloques de contenido real y el resumen del anfitrión.

Las tramas son `u32 LE length + CBOR`; la longitud se comprueba antes de asignar memoria. Límites: trama 1 MiB, bytes/text individuales 256 KiB, profundidad 16, arrays 128, mapas 32 y nodos descodificados 8192. Solo se aceptan UInt, bytes, texto UTF-8 estricto, arrays, mapas con claves UInt y bool. Se rechazan enteros no mínimos, claves duplicadas, longitudes indefinidas, enteros negativos, etiquetas, flotantes y datos posteriores. Las claves del mapa de envoltura `1..5` son kind, request, operation, sequence y payload. La secuencia comienza en 1 y aumenta consecutivamente en cada dirección; request aumenta en cada invocación y operation es una identidad aleatoria no nula del anfitrión. Cada respuesta debe coincidir exactamente con la identidad actual. Las claves payload desconocidas `<128` se rechazan; `>=128` son campos opcionales ignorables.

| kind | Mensaje | Claves numéricas del payload |
| --- | --- | --- |
| 1 | Hello | 1 major mínimo, 2 major máximo, 3 ID de paquete, 4 `xdp1` |
| 2 | Welcome | 1 major, 2 límite de bloque, 3 crédito de un bloque, 4 presupuesto total de lectura, 5 presupuesto de salida |
| 10 | Probe | 1 token de origen, 2 longitud de origen |
| 11 | Open | 1 token de origen |
| 12 | Scan | 1 token de sesión |
| 13 | ReadEntry | 1 sesión, 2 entrada, 3 destino, 4 cantidad de créditos 1, 5 presupuesto de salida, 6 token de crédito del primer bloque |
| 14 | Close | 1 sesión |
| 20, 21 | ReadAt, ReadData | 1 token de origen, 2 desplazamiento, 3 longitud o bytes |
| 30 | EntryBatch | 1 sesión, 2 entradas; por registro: 1 token, 2 ruta original, 3 tipo 0 archivo / 1 directorio, 4 tamaño conocido |
| 31 | ScanFinished | 1 número real de registros |
| 40 | WriteData | 1 destino, 2 secuencia de bloque desde 0, 3 bytes, 4 token de crédito actual |
| 41 | FinishOutput | 1 destino, 2 cantidad de salida declarada, 3 bytes SHA-256 |
| 42 | WriteAck | 1 destino, 2 secuencia del bloque siguiente, 3 nuevo token de crédito aleatorio |
| 50, 51, 52 | Cancel, CancelAck, Shutdown | Mapa vacío |
| 61 | OperationFinished | 1 resultado, 0 éxito / 1 cancelado; Probe añade 2 formato, Open añade 2 sesión |

La máquina de estados de esta prueba rechaza otros kinds aunque el entramado los reconozca. Hello debe llegar primero y ambas partes seleccionan major 1; la identidad del saludo es cero y las llamadas posteriores son distintas de cero. ScanFinished aún debe ir seguido de un único estado terminal exitoso. Solo se ejecuta una operación a la vez y no hay reintentos de extensiones en segundo plano. El crédito actual de un bloque es más estricto que el máximo de diseño de cuatro; un token impredecible se renueva tras la aceptación real de cada bloque de salida y no puede adelantarse ni reutilizarse.

<a name="xdzip-broker-budgets-and-publication"></a>

## Intermediario, presupuestos y publicación

Los tokens de origen, destinos y créditos por bloque son valores aleatorios de 128 bits que no pueden reutilizarse entre operaciones. El par no recibe rutas del archivo ni del destino. El anfitrión devuelve como máximo 256 KiB por lectura dentro del intervalo de origen autorizado; fallan desbordamiento del desplazamiento, lectura después de EOF, longitud cero y lecturas acumuladas superiores a 64 MiB. Comprueba el `FileVersion` del objeto mantenido y la ruta correspondiente antes y después de cada lectura y antes de publicar. Esto reutiliza la huella existente de identidad, tiempo y longitud y no afirma detectar modificaciones concurrentes arbitrarias de un atacante de la misma cuenta que restaure todos los metadatos.

Los metadatos se limitan a 1000 registros físicos y 256 KiB en total de rutas y tokens, con tokens únicos por registro. El anfitrión valida estrictamente las rutas originales según PortablePath; los nombres duplicados conservan identidades separadas. Se selecciona explícitamente una entrada ordinaria cada vez, con tamaño declarado de hasta 4 MiB; los directorios no pueden ser salida de archivo. El anfitrión mantiene el Grant de destino, el Workspace privado y el Stage, mide los bytes reales y calcula SHA-256 sin dar al par un identificador de archivo ni autoridad de publicación.

La E/S segura existente solo puede publicar sin sobrescribir cuando se hayan completado todos los bloques, el recuento y resumen de FinishOutput, el único estado terminal, Close, salida normal tras Shutdown y EOF limpio, identidad del origen y comprobación final de cancelación. Cualquier anomalía, estado terminal duplicado, fallo o tiempo agotado revoca primero la salida; la destrucción elimina el stage antes de limpiar el workspace. Los destinos existentes no cambian. El análisis y los resúmenes esperados siguen procediendo de la extensión; el cálculo del anfitrión no puede demostrar que una extensión maliciosa produzca una salida fiel al archivo.

El saludo está limitado a 2 segundos y cada invocación de prueba a 5 segundos en total, sin que el progreso reinicie el plazo. La CPU por proceso se limita a 3 segundos, los descriptores de archivo a 32; se prohíbe la salida directa a archivos y se desactivan los volcados de memoria. Las colas IPC reciben hasta cuatro tramas y envían una, y las escrituras en tubería tienen un límite de 2 segundos. stderr se descarta directamente con presupuesto cero; no se reenvía contenido no fiable. La cancelación revoca inmediatamente el destino, concede 2 segundos para cooperar y después termina y recoge el proceso y las tuberías; Ack no autoriza la publicación. Estos presupuestos limitan una prueba pequeña, no prometen rendimiento para archivos grandes ordinarios.

<a name="xdzip-platform-gates"></a>

## Condiciones por plataforma

| Plataforma | Política de prueba y comportamiento ante fallos |
| --- | --- |
| macOS | La política deny-default de `/usr/bin/sandbox-exec` fijado permite exactamente la copia ejecutable privada, bibliotecas del sistema y caché dyld, lecturas del cargador en la raíz y metadatos de `/var`. Se prohíben archivos del usuario, escritura, red y fork; se limpia el entorno y solo se hereda stdio. Los rlimits nativos y la supervisión RSS cada 20 ms terminan por encima de 512 MiB o si falla la medición. RSS se muestrea, por lo que picos breves entre muestras pueden superar el umbral; esta interfaz no se promete para una futura distribución oficial. |
| Linux x86_64 / aarch64 | Landlock ABI ≥3 controla todos los derechos de archivo v3, sin reglas de escritura y con lectura solo del ejecutable y bibliotecas del cargador del sistema. `no_new_privs` y una lista de llamadas seccomp comprobada por arquitectura incluyen readv/writev necesarios para las tuberías estándar de Rust y deniegan socket, fork/clone, ptrace, io_uring y otros. Se aplican un límite de CPU de 3 segundos, 512 MiB de espacio de direcciones y otros rlimits. La falta de reglas, ABI, seccomp o llamadas necesarias falla antes de exec. Solo la CI nativa demuestra resultados reales de esta plataforma. |
| Windows y otras plataformas | La ejecución se rechaza incondicionalmente. AppContainer, Job y lista permitida de identificadores no se han implementado ni verificado; no se sustituye el aislamiento por un subproceso ordinario. |

Las decisiones de API Linux siguen la [documentación Landlock del núcleo](https://docs.kernel.org/userspace-api/landlock.html) y UAPI; los futuros trabajos Windows siguen el [aislamiento AppContainer](https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation). El código unsafe nativo del anfitrión se limita al módulo de aislamiento y documenta estructuras, condiciones previas de llamada y fallos; pre-exec no asigna memoria ni registra. El par de referencia añade sondas controladas fork/_exit/waitpid solo en modo de auditoría explícito; el hijo no ejecuta limpieza Rust ni lógica de formato. Las regresiones de plataforma usan primero marcadores propios legibles y escribibles, un listener localhost propio y self-exec como controles positivos; después verifican la denegación del núcleo de lectura, escritura, red, fork directo y self-exec según el estado real de marcadores, archivos y conexiones. Las plataformas no probadas no se marcan como superadas y la ejecución de terceros en escritorio sigue desactivada.

<a name="xdzip-owned-xdp1-example-format"></a>

## Formato de ejemplo XDP1 propio

XDP1 es solo una muestra de interoperabilidad y no sustituye ningún formato de archivo real. Los cuatro bytes `XDP1` van seguidos de un recuento de registros `u32 LE`. Cada registro contiene un número de bytes del nombre UTF-8 `u16 LE`, un número de bytes del contenido `u32 LE`, el nombre y el contenido sin comprimir. Se permiten hasta 1000 registros sin datos posteriores; nombres y contenidos se leen dentro de los presupuestos del anfitrión. Los tokens de entrada son ordinales físicos LE de cuatro bytes, sin deduplicación por ruta. Los modos de fallo de Python se seleccionan explícitamente solo en pruebas de conformidad para verificar acceso no autorizado, secuencia, repetición, identidad duplicada, rutas, créditos, resúmenes, fallos, cancelación y estados terminales duplicados.
