# Usar XDZip en scripts

[English](../../integration/README.md) | [简体中文](../../zh-CN/integration/README.md) | [繁體中文](../../zh-TW/integration/README.md) | [日本語](../../ja-JP/integration/README.md) | [한국어](../../ko-KR/integration/README.md) | [Français](../../fr-FR/integration/README.md) | [Deutsch](../../de-DE/integration/README.md) | Español | [Português (Brasil)](../../pt-BR/integration/README.md) | [Русский](../../ru-RU/integration/README.md)

La CLI autónoma funciona sin Qt ni sesión de escritorio. Elige un paquete `xdzip-cli-` para tu sistema entre las [versiones de GitHub](https://github.com/xiaodaozhi/xdzip/releases). Descomprime todo el paquete y deja el proceso auxiliar al lado del ejecutable. Desde el directorio extraído, ejecuta `./bin/xdzip --help` en macOS o Linux, o `.\bin\xdzip.exe --help` en PowerShell.

Los ejemplos usan `xdzip` en PATH. Ejecuta `setup` con la CLI extraída para registrar un lanzador para tu usuario y añade su directorio a PATH si hace falta. Usa `xdzip --version-json` para confirmar que el programa encontrado indica `"component":"cli"`. El lanzador de escritorio y la CLI pueden tener el mismo nombre. Si hay un conflicto, usa la ruta explícita de la CLI extraída. [Instalar y quitar el comando](../engineering/command-line.md#xdzip-build-and-command-installation)

<a name="xdzip-create-and-check"></a>

## Crear y verificar

En un directorio de trabajo nuevo, privado y con permiso de escritura, crea una carpeta `docs` que contenga `hello.txt`. Coloca un segundo archivo, `notes.txt`, junto a la carpeta. Los siguientes comandos crean un ZIP, añaden el segundo archivo en la carpeta `docs/` que ya existe dentro del ZIP y verifican el resultado.

```sh
xdzip create sample.zip docs --level normal --json
xdzip add sample.zip notes.txt --into docs --json
xdzip test sample.zip --json
```

Elige la extensión y las opciones de creación en la [tabla de formatos](../../../README.es-ES.md#xdzip-format-support). Guarda la salida fuera del árbol de origen y usa un directorio padre ya existente. `add` requiere un archivo existente y compatible. Usa `--into` para añadir en una carpeta existente del archivo comprimido u omítelo para añadir en la raíz. `test` no escribe archivos extraídos. Si la verificación tiene éxito, `data.check_kind` indica su alcance. Consulta el [registro de formatos](../engineering/format-registry.md) y las [verificaciones de integridad](../engineering/archive-checking.md).

<a name="xdzip-list-select-and-extract"></a>

## Listar, seleccionar y extraer

Continúa con el archivo anterior en el mismo directorio de trabajo. Usa un destino nuevo o elige explícitamente cómo tratar los archivos existentes.

```sh
xdzip list sample.zip --limit 2 --json
xdzip extract sample.zip docs/hello.txt -C output --conflict skip --json
```

Las rutas de los miembros se comparan exactamente, distinguen mayúsculas y minúsculas y usan barras inclinadas. Si varios registros tienen la misma ruta, selecciona por el `id` que devuelve `list`. Los identificadores empiezan en 1. Por ejemplo, sustituye `docs/hello.txt` por `--entry-id 2` en el segundo comando si la lista asigna el identificador 2 a ese archivo. Repite `--entry-id` para más miembros. Usa `path` para localizar los miembros y `display_name` para mostrarlos. Vuelve a listar tras modificar el archivo comprimido.

La primera página empieza en el desplazamiento 0. Pasa `data.next_offset` a `--offset` con el mismo `--limit` para leer la siguiente página. Detente cuando `next_offset` sea null. El desplazamiento es una posición de fila que empieza en cero. Un identificador representa un registro físico. Cada comando vuelve a abrir el archivo comprimido. Mantén el origen sin cambios entre las llamadas paginadas. [Límites de selección y listado](../engineering/command-line.md#xdzip-archive-behavior)

<a name="xdzip-run-without-prompts"></a>

## Ejecutar sin preguntas

`--json` desactiva las preguntas interactivas. En un trabajo desatendido, elige explícitamente una política de conflictos. En los comandos que admiten contraseña, como `extract`, `test` o `create` con cifrado, usa `--password-stdin` y envía una línea UTF-8 con la contraseña por una tubería privada de entrada estándar. Para crear con cifrado también hace falta `--encrypt`. Si falta una decisión o contraseña obligatoria, la CLI devuelve `interaction_required`. [Opciones de contraseña y formato](../engineering/command-line.md#xdzip-archive-behavior)

<a name="xdzip-interpret-results"></a>

## Interpretar los resultados

Para operaciones con `--json`, procesa el objeto UTF-8 de stdout. Lee `schema_version`, `status`, `error_code`, `data`, `summary` y `warnings`. Conserva stderr para el diagnóstico. `--version-json` devuelve un objeto separado de identificación de compilación.

| Código de salida | Qué hacer |
| --- | --- |
| 0 | Lee el resultado y el recuento de archivos omitidos. Si el comando permite transferir la operación al escritorio, `handed_off` confirma la entrega. |
| 1 | Revisa el error, los avisos y el resumen para detectar fallos, resultados parciales o avisos de limpieza. |
| 2 | Corrige el comando o los argumentos. |
| 3 | Comprueba el requisito de interacción, autorización, plataforma o instalación indicado. |
| 130 | Se ha confirmado la cancelación. Comprueba qué archivos se han guardado definitivamente. |

Los valores desconocidos pueden ser null. Un error o una cancelación puede dejar archivos ya guardados. Antes de repetir la operación, comprueba `summary.committed_files` si está disponible. Consulta la [referencia de la CLI](../engineering/command-line.md) para ver la transferencia al escritorio, los comprobantes de actualización y la especificación completa de salida.
