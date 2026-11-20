# Contrato de ejecución de línea de comandos

[English](../../engineering/command-line.md) | [简体中文](../../zh-CN/engineering/command-line.md) | [繁體中文](../../zh-TW/engineering/command-line.md) | [Français](../../fr-FR/engineering/command-line.md) | Español | [Português (Brasil)](../../pt-BR/engineering/command-line.md) | [Русский](../../ru-RU/engineering/command-line.md)

El CLI es un anfitrión Rust independiente que llama a los servicios de formatos y E/S segura mediante el mismo `xdzip-runtime`; no depende de Qt, QML ni de una sesión gráfica. 7z, RAR, CAB, ISO y UDF siguen utilizando procesos aislados incluidos y no buscan herramientas alternativas de extracción en PATH. Las plataformas y arquitecturas siguen la [política de soporte](support-policy.md). El CLI independiente de Linux no requiere X11; para ejecutarlo se necesitan OpenSSL 3 y certificados del sistema, y para compilarlo también libssl-dev y pkg-config.

Desde `v0.2.0`, las versiones oficiales ofrecen paquetes CLI independientes para tres plataformas y los paquetes de escritorio incluyen un CLI privado. Los archivos de versiones anteriores no cambian retroactivamente. Consulte las [versiones](https://github.com/xiaodaozhi/xdzip/releases) para descargar y las secciones siguientes y el [protocolo de publicación](release-protocol.md) para los límites de compilación, instalación y actualización. El progreso de implementación y aceptación se sigue en [#254](https://github.com/xiaodaozhi/xdzip/issues/254).

## Uso y salida

```sh
xdzip --help
xdzip version
xdzip list archive.zip --json
xdzip extract archive.zip docs/ -C output
xdzip extract archive.zip --entry-id 3 -C output
xdzip create backup.7z docs/ --level normal
xdzip create data.iso docs/
xdzip create portable.sfx.exe docs/ --level normal
xdzip create private.zip docs/ --encrypt --password-stdin
xdzip add backup.zip notes.txt --into docs
xdzip test backup.zip
xdzip update --check
xdzip update --download --output downloads
xdzip update --yes
xdzip update --status
xdzip gui archive.zip
```

Sin argumentos o con `--help`, xdzip muestra la ayuda. `version`, `--version` y `--version-json` informan de la identidad de `VERSION.json` y del estado de Git de la compilación. `--version-json` emite un objeto independiente con product, version, build, revision, dirty, component, channel, configuration, distribution, platform y architecture. Para ayuda y mensajes use `--language zh-CN|en-US`. Por defecto se consultan `LC_ALL`, `LC_MESSAGES` y `LANG` en ese orden, y se usa chino si el valor elegido empieza por `zh`. Los identificadores públicos y códigos de error están en inglés. `l`, `x`, `c`, `a` y `t` son alias de `list`, `extract`, `create`, `add` y `test`. Use `--` para pasar argumentos de archivo literales.

Todos los comandos de negocio aceptan `--json`. stdout emite exactamente un objeto JSON con los campos fijos schema_version (1), command, status, error_code, data, summary y warnings. Los tamaños desconocidos son null, incluidos los elementos que comparten un flujo TAR comprimido; los miembros ar sin procesar de DEB conservan su tamaño comprimido exacto. Los datos de lista incluyen formato y entradas, cada una con id, path, display_name, size, packed_size, method, directory, link, payload_supported, encrypted y warning. path es el nombre efectivo fijado; los nombres inválidos o no resueltos son null. display_name solo sirve para presentación escapada. Los resúmenes incluyen los recuentos reales de lectura, decodificación, verificación, confirmación, omisión, directorios y volúmenes, conservando advertencias de limpieza y resultados inciertos.

El progreso, los errores y las preguntas interactivas van a stderr. Las llamadas JSON, sin terminal o con `--non-interactive` no preguntan. Si falta una contraseña o decisión de conflicto, se cancela la tarea en espera, se espera la limpieza y se devuelve interaction_required. Los códigos de salida son 0 para éxito o entrega explícita, 1 para fallo, finalización parcial o advertencia de limpieza, 2 para argumentos inválidos, 3 para interacción, autorización o condiciones de instalación ausentes, y 130 para cancelación confirmada. La omisión explícita es una política ejecutada; su recuento se informa por separado. Si la cancelación no se acepta y la confirmación ya terminó, se conserva el resultado real.

<a name="xdzip-archive-behavior"></a>

## Comportamiento de archivos comprimidos

`list` devuelve registros físicos y conserva duplicados. `extract` acepta rutas efectivas completas y exactas de miembros, sensibles a mayúsculas y con barras diagonales. Los directorios se expanden recursivamente y conservan la jerarquía bajo la raíz. Una ruta duplicada devuelve `ambiguous_member`. Pase el ID numérico de `list`, que comienza en 1, con `--entry-id`; repita la opción para varios ID. Las rutas o ID explícitos que no se resuelvan fallan antes de crear el directorio de destino. No se admiten comodines, leer un archivo comprimido de stdin ni escribir el contenido extraído en stdout.

`--limit` o `--offset` habilitan la paginación. El desplazamiento empieza en cero y no puede superar 1 000 000. Con solo `--limit`, el desplazamiento es cero; con solo `--offset`, el límite es 128. Los límites válidos son 1–4 096. `data` de JSON contiene `offset`, `total` y `next_offset`, que puede ser null. `summary.entries` cuenta filas realmente devueltas. Un ZIP/ZIP64 ordinario de un solo archivo puede indexar un millón de entradas, pero la lista sin páginas rechaza más de 100 000. Ambos modos tienen un presupuesto de 16 MiB para filas JSON serializadas y texto renderizado. Si se supera, se devuelve `resource_limit` sin filas parciales. Las páginas no cambian los ID físicos. Véase [indexación de archivos grandes](large-index.md).

extract usa por defecto el directorio actual. Si `-C` indica un directorio inexistente, obtiene autorización componente por componente desde el padre seguro existente más cercano, en vez de crear directorios recursivamente sin condiciones. Destino, identidad de origen, enlaces, colisiones Unicode o de mayúsculas, preparación, verificación y confirmación siguen la [extracción segura](../../engineering/zip-safe-extraction.md) y la [codificación de nombres](filename-encoding.md). El conflicto usa Ask por defecto; `--conflict ask|skip|rename|overwrite` selecciona la política explícitamente. `--allow-high-ratio` solo confirma una relación de compresión elevada y no elimina límites estrictos de recursos.

`create` elige el perfil según el [sufijo](format-registry.md) de salida y exige que el directorio padre exista. `--level store|fast|normal|best` se valida frente al perfil seleccionado. Para TAR simple e ISO omita el nivel o use `store`. LHA (`.lzh` o `.lha`) usa niveles LH5 1, 6 y 9 para `fast`, `normal` y `best`. ZIP y 7z admiten `--encrypt` y `--volume-size`; 7z también admite `--encrypt-names` y `--solid` según su perfil. El tamaño de volumen acepta bytes enteros positivos o sufijos `KiB`, `MiB` y `GiB`. Los argumentos no admitidos por el perfil fallan antes de escribir.

Las contraseñas se pasan solo mediante entrada oculta del terminal o una línea UTF-8 de `--password-stdin`; se eliminan finales de línea sin recortar la contraseña. La creación cifrada interactiva requiere una segunda confirmación. Se rechazan argumentos con contraseñas en claro; no se leen variables de entorno con contraseñas ni se escriben contraseñas en ajustes o registros. Longitud y puesta a cero siguen el [contrato de contraseñas](password-extraction.md) y el [contrato de creación](../../engineering/zip-creation.md).

`add` envía una actualización al anfitrión y usa la raíz del archivo comprimido por defecto. `--into` selecciona un directorio seguro existente mediante su nombre relativo al archivo. Las entradas admisibles pueden añadir o reemplazar elementos. Los movimientos internos son una operación de escritorio. El CLI comparte las comprobaciones de [modificación de archivos](../../engineering/archive-transfers.md) para formatos y registros admitidos, protección de entradas, reconstrucción temporal y sustitución final. `test` verifica todo el archivo sin producir archivos de miembros. `data.check_kind` en JSON informa de la intensidad de una comprobación terminada según [verificación de archivos](archive-checking.md).

Cada operación CLI tiene su propio entorno de ejecución y no se une a las sesiones de navegación del escritorio. Ctrl+C y las señales de terminación solicitan cancelación cooperativa y esperan el estado final, la limpieza y la unión del entorno; los archivos ya confirmados permanecen. `submit_wait` del servicio compartido obtiene secuencialmente solo un breve bloqueo de estado y no espera espacio en la cola ni cambia la admisión no bloqueante de la GUI. Las páginas de catálogo contienen como máximo 128 entradas físicas. Se mantienen los límites de cantidad y nombres de SelectMembers; todas las solicitudes se vinculan a una identidad de sesión inmutable.

<a name="xdzip-build-and-command-installation"></a>

## Compilación e instalación del comando

```sh
python3 scripts/cli.py build
python3 scripts/cli.py test
python3 scripts/cli.py package --output build/package/cli
build/cli/bin/xdzip --help
```

Esta entrada usa Rust, CMake y el compilador C++ de plataforma fijados por el repositorio y el proceso aislado existente, sin requerir Qt. El empaquetado de Windows también requiere el `prepare-windows-runtime.py` existente para preparar el entorno VC++ local fijado con su licencia original. La entrada unificada de Qt también compila el CLI en un directorio cli privado sin cambiar el nombre del ejecutable GUI, el icono de aplicación ni las entradas de asociación.

El ejecutable CLI incluido en macOS está en `Contents/Helpers/cli/`. Su contexto JSON normal está en `Contents/Resources/xdzip-cli-context.json`, protegido por la firma de recursos de la aplicación. El CLI solo encuentra este recurso en la estructura fija Contents/Helpers/cli y verifica el esquema, tipo de escritorio y ruta relativa fija de GUI. Los marcadores ausentes se tratan como identidad portable; los inválidos se rechazan. Las compilaciones repetidas eliminan marcadores generados obsoletos de Helpers; no debilite la comprobación de firma para solucionar un error de estructura.

Los nuevos paquetes de escritorio también declaran `worker_relative` en este contexto y comparten el proceso de formatos del escritorio. Solo se aceptan rutas relativas fijas de la plataforma: el proceso de escritorio adyacente en Windows, `Contents/MacOS/xdzip-native-codec` en macOS y el proceso junto a la compilación o desplegado en `bin/` en Linux. Un proceso ausente, enlazado o inesperado rechaza el contexto sin buscar en PATH ni otra copia. Los contextos antiguos sin este campo y los paquetes CLI independientes o gestionados conservan la búsqueda del proceso vecino. Las sustituciones explícitas del desarrollador mantienen prioridad. Verifique la búsqueda desplegada con `scripts/check-cli.py --default-worker`; las sustituciones de prueba normales no la demuestran.

Tras descomprimir un paquete independiente ejecute bin/xdzip, añadiendo .exe en Windows. `bin/xdzip setup` instala explícitamente una entrada estable de usuario y un directorio de ejecución versionado. En Unix la entrada predeterminada es ~/.local/bin; en Windows está en xdzip/cli/bin de LocalAppData del usuario. Los directorios de gestión usan ubicaciones de datos de usuario de la plataforma. `--bin-dir` y `--install-dir` sustituyen explícitamente las ubicaciones. El usuario añade el directorio a PATH; la herramienta no modifica archivos de inicio del shell. Nunca sobrescribe entradas o archivos de registro existentes.

La identidad gestionada requiere un installation.json válido. Un directorio llamado versions sin archivo de registro sigue siendo portable.

Al retirar un lanzador Windows en ejecución, la copia propia cuyo hash aún coincide se mueve al subdirectorio privado .xdzip-retired del mismo directorio de comandos. El registro se elimina inmediatamente y el resultado registra retained_launcher. Un setup explícito posterior limpia solo copias en recibos limitados cuyos hashes aún coinciden y cuyos procesos terminaron. Los archivos modificados permanecen y no se sobrescriben. El CLI del directorio versionado puede volver a ejecutar setup para restaurar el registro.

El campo distribution distingue la identidad de origen y de versión publicada. Una compilación ordinaria desde código fuente, aunque esté limpia y en configuración Release, solo admite comprobación y descarga. La sustitución automática requiere a la vez una identidad distribution marcada por el pipeline Release dedicado, una instalación gestionada válida y una firma de destino autenticada.

La opción PATH del instalador de escritorio Windows está desmarcada por defecto y rechaza conflictos con otros comandos xdzip. En Mac/Linux puede ejecutarse setup desde el CLI privado del paquete de escritorio; solo registra el CLI dentro de esa instalación y no copia un núcleo que pueda actualizarse por separado. `setup --remove` elimina únicamente entradas y archivos de registro propios de la herramienta verificados por hash. Permanecen los directorios versionados y materiales de recuperación; no se eliminan entradas del usuario ni directorios de ejecución que aún puedan estar en uso.

El lanzador estable no usa escape del shell. En Unix ejecuta el CLI real mediante exec; en Windows espera al proceso de consola hijo y transmite cancelación y código de salida. Un puntero atómico current.json selecciona el directorio de ejecución actual. Un fallo de instalación inicial, autocomprobación de la versión nueva o confirmación de archivos no debe dar un resultado de instalación satisfactoria.

## Actualizaciones, entrega al escritorio y recuperación

update actúa sobre la instalación actual. Un CLI independiente solo actualiza el CLI; el incluido en el paquete de escritorio actualiza toda la edición de escritorio. `--check` consulta, `--download` descarga y verifica, la operación por defecto instala tras confirmar y `--status` comprueba recibos e identidad real. `--yes` confirma solo esa actualización y no sustituye la autorización del sistema. Las compilaciones portables, modificadas, de origen o con identidad desconocida solo permiten comprobar y descargar. La sustitución automática independiente requiere una instalación Release gestionada y limpia.

El CLI usa GitHub Releases públicos fijos, canal y reglas de número de compilación monotónico. Los archivos CLI de las tres plataformas utilizan cli-release.json y cli-release.json.sig separados; la clave pública integrada verifica los bytes originales antes de comprobar tamaño y SHA-256. Si falta la firma o la clave no es de confianza solo se ofrece información para obtenerlos manualmente; una firma inválida falla explícitamente. El release.json de los seis paquetes de escritorio no incluye entradas CLI y los clientes antiguos siguen usando el protocolo original.

La red usa por defecto certificados del sistema y variables de entorno estándar de proxy. `--proxy` selecciona un proxy HTTP/SOCKS5 sin credenciales; `--no-proxy` conecta directamente; `--mirror` selecciona un prefijo HTTPS sin credenciales, consulta ni fragmento. Las URL originales de GitHub, los hosts de redirección, tamaños de respuesta, tiempos y presupuestos de descarga siguen limitados. Se informan por separado los resultados de red, TLS, falta de conexión, límite de tasa y ausencia de nueva versión compatible; no hay sondeo ni reintento automáticos.

Las actualizaciones independientes se preparan en un directorio privado propiedad del usuario, se extraen de forma segura con el mismo entorno de archivos, verifican el inventario fijo completo y el hash de cada archivo, producto, componente, plataforma y arquitectura locales y canal del manifiesto firmado, y ejecutan la autocomprobación `--version-json` de la versión nueva antes de cambiar el puntero. Las transacciones se excluyen mutuamente y los recibos se vinculan a la identidad de destino. Un fallo conserva el puntero anterior o inicia recuperación explícita; no se inventa una reversión automática.

La comprobación y descarga de escritorio consultan solo la entrada no gráfica de identidad de instalación del programa principal; no crean QApplication ni consumen recibos de instalación. La intención de instalar pasa por el canal existente de instancia única del mismo usuario, que solo acepta un directorio limitado de solicitud y una versión candidata, no comandos, paquetes o raíces de confianza arbitrarios. Los archivos de solicitud y resultado reutilizan la autorización nativa de directorios de Rust y confirmaciones seguras. El controlador de escritorio vuelve a comprobar y cotejar independientemente el candidato, reutilizando Acerca de, instalador, Sparkle y la barrera de unión de todas las tareas. Las tareas CLI de escritorio también mantienen protección de ocupación de instalación; la instalación no puede destruirlas.

El terminal que recibe una entrega aceptada solo informa handed_off. La instalación y recuperación reales dependen de los recibos del escritorio y de la identidad en ejecución; descargar o entregar no significa que se haya instalado. El modo no interactivo rechaza la instalación de escritorio que requiera GUI o autorización del sistema. Véase el [contrato de actualización](../../engineering/software-update.md) para los límites de seguridad y aceptación nativa. La aceptación automatizada, nativa y manual siguen siendo distintas.

El perfil `.iso` crea imágenes de datos ISO9660/Joliet sin comprimir al nivel 0. Omita `--level` o use `--level store`. El perfil Windows x64 `.sfx.exe` encapsula ZIP Store/Deflate con el `xdzip-sfx.exe` vecino incluido. Rechaza cifrado, modo sólido y volúmenes. Ninguno admite editar una salida existente. Véanse [creación ISO](iso-creation.md) y [creación SFX propia](sfx-creation.md).
