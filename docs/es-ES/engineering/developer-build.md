# Compilaciones de desarrollo y verificación por plataforma

[English](../../engineering/developer-build.md) | [简体中文](../../zh-CN/engineering/developer-build.md) | [繁體中文](../../zh-TW/engineering/developer-build.md) | [日本語](../../ja-JP/engineering/developer-build.md) | [한국어](../../ko-KR/engineering/developer-build.md) | [Français](../../fr-FR/engineering/developer-build.md) | [Deutsch](../../de-DE/engineering/developer-build.md) | Español | [Português (Brasil)](../../pt-BR/engineering/developer-build.md)

Esta página es el punto de entrada común para preparar, compilar, probar, empaquetar y verificar entornos en tres plataformas. La coordinación compartida se implementa en [scripts/desktop.py](../../../scripts/desktop.py). Los documentos de cada plataforma complementan las estructuras de despliegue, el comportamiento de las herramientas y los límites del sistema, sin mantener comandos rutinarios separados.

La CLI independiente usa `python3 scripts/cli.py build`, `test` y `package` sin Qt, X11 ni sesión gráfica. La compilación en Linux también requiere `libssl-dev pkg-config`; la ejecución usa OpenSSL y certificados del sistema. Las compilaciones de escritorio crean automáticamente un directorio privado de CLI. Véase el [contrato de CLI](command-line.md) para comandos, instalación y límites de actualización confiable.

Las compilaciones de desarrollo siguen siendo experimentales y no son instaladores Release oficiales. Véanse el [README](../../../README.es-ES.md) raíz para capacidades y límites y la [política de soporte](support-policy.md) para el ámbito admitido. La revisión histórica de `v0.1.0` está en [#4](https://github.com/xiaodaozhi/xdzip/issues/4) y [#53](https://github.com/xiaodaozhi/xdzip/issues/53); las versiones futuras requieren nuevas revisiones de sus candidatos específicos.

<a id="environment-and-entry-points"></a>

## Entorno y puntos de entrada

Instale primero Python 3.10 o posterior, Git, CMake 3.24 o posterior, Ninja, rustup y el compilador/SDK de la plataforma. La versión de Rust procede de rust-toolchain.toml en la raíz; Qt está fijado en 6.8.2. Véase la [política de herramientas](toolchain.md) para reglas de dependencias y actualización. Ejecute los comandos desde la raíz del repositorio. En Windows sustituya `python3` de los ejemplos por un `python` disponible y use un entorno de desarrollo MSVC x64.

## Directorios locales de herramientas

Al cargar el entorno PowerShell de Visual Studio, conserve los valores Unicode de `USERPROFILE`, `LOCALAPPDATA` y `APPDATA` y restáurelos si el cargador altera su codificación. Qt usa esas ubicaciones para las rutas predeterminadas de caché y configuración; unos valores dañados pueden causar fallos del sistema de archivos aunque el compilador funcione.

`prepare` compila y verifica en el SDK existente un QtNetwork con parches de seguridad para CVE-2026-116337 y CVE-2026-105946; la biblioteca reconstruida no sustituye otros módulos de ejecución. También se necesitan Perl, make (nmake en Windows) y la herramienta de línea de comandos OpenSSL. Ubuntu requiere `zlib1g-dev libpcre2-dev libicu-dev libglib2.0-dev libzstd-dev libkrb5-dev`. El código, declaraciones generadas y cachés de compilación permanecen bajo `build/`, reutilizando por defecto `build/toolchain/qt-network-security/<platform>`. Si no coinciden los digests de entrada y de la biblioteca real, la compilación y el empaquetado fallan inmediatamente; vuelva a ejecutar `prepare`. Las [notas de mantenimiento del parche](../../../patches/qt/README.md) describen procedencia, ABI, funcionalidad TLS y recuperación.

Las dependencias locales del punto de entrada común usan únicamente `build/toolchain/`, ignorado por Git y no incluido en commits:

| Contenido | Directorio fijo | Notas |
| --- | --- | --- |
| Qt SDK 6.8.2 | `build/toolchain/Qt/6.8.2/<platform-subdirectory>` | `<platform-subdirectory>` es `macos`, `msvc2022_64` o `gcc_64`; qmllint, qmake, configuración CMake y runtimes Qt proceden de aquí |
| Entorno de herramientas Python | `build/toolchain/desktop-tools` | Entorno virtual fijo para aqtinstall y scripts de empaquetado y verificación |
| Preparación de descargas Qt | `build/toolchain/qt-install-tool` | Solo descargas e entradas temporales del instalador Qt |

`scripts/desktop.py prepare --install-qt`, `doctor`, `build` y `test` comparten estas ubicaciones por defecto. No cree copias separadas `.cache/Qt`, `.cache/desktop-tools` u otras similares para cada canalización de desarrollo. Especifique `--qt-prefix`, `--qt-root` y `--tools-dir` solo para aislar versiones Qt, plataformas o compilaciones experimentales distintas. CI puede apuntarlos explícitamente a directorios temporales del runner, pero sus rutas alternativas no cambian los valores locales. CMake, Ninja, rustup, Rust y el compilador C++ de la plataforma pertenecen a las herramientas del host y se instalan o ejecutan según los requisitos de comprobación previa, no se copian mediante el directorio Qt.

Después de un ciclo de desarrollo y de limpiar ramas y worktrees, elimine los directorios y archivos temporales creados por ese ciclo bajo `build/`. Conserve herramientas compartidas reutilizables como `build/toolchain/`; elimine compilaciones, diagnósticos y preparación de descargas propios del ciclo. Véase [gestión del contenido e historial del repositorio](../../../developers/engineering/repository-hygiene.md#研发周期工件收尾) para reglas y procedimiento manual.

| Plataforma | Entorno del sistema que debe prepararse primero | Subdirectorio Qt SDK |
| --- | --- | --- |
| macOS | Terminal nativo Apple Silicon, Xcode Command Line Tools y SDK macOS válido | macos |
| Windows | Herramientas C++ MSVC 2022 x64 y Windows SDK, Developer PowerShell o línea de comandos x64 | msvc2022_64 |
| Ubuntu 24.04 | Compilador C++ x86_64, paquetes de desarrollo OpenGL/EGL/xkbcommon y bibliotecas de ejecución X11 | gcc_64 |

En macOS, `xcode-select --install` prepara las herramientas de línea de comandos. Los paquetes básicos de desarrollo de Ubuntu incluyen `libgl1-mesa-dev libegl1-mesa-dev libxkbcommon-dev libx11-dev libxext-dev libnautilus-extension-dev pkg-config`; el empaquetado también necesita patchelf, binutils (readelf y strip) y herramientas de paquetes Debian, y la verificación aislada de ejecución necesita Docker accesible. Instalar Visual Studio en Windows sin entrar en un entorno de desarrollo puede dejar cl, link, rc o dumpbin inaccesibles; la comprobación previa identifica las herramientas realmente ausentes. El punto de entrada común no instala automáticamente compiladores de sistema, SDK ni paquetes que requieran privilegios de administrador.

Los desarrolladores Windows pueden preparar sus herramientas siguiendo la [vía gratuita de licencia Community](windows-licensing.md), tras leer y aceptar las condiciones de esa versión. Tener solo Build Tools o VC_redist no acredita elegibilidad. El uso empresarial y la redistribución asociada en runners estándar alojados por GitHub tienen autorización pública separada; los mantenedores no necesitan comprar Enterprise para compilaciones alojadas ni instalar localmente un IDE Community, y esa autorización no se aplica a equipos autoalojados. No hace falta desinstalar herramientas 2019 existentes para revisar licencias; la compatibilidad de alternativas locales con el SDK de publicación 2022 actual debe comprobarse por separado.

Todos los comandos comparten estas opciones:

| Opción | Valor predeterminado y propósito |
| --- | --- |
| `--qt-prefix` | Primero `XDZIP_QT_PREFIX`; si no existe, `build/toolchain/Qt/6.8.2/<platform-subdirectory>`. Apunta al prefijo del SDK, no a bin |
| `--build-dir` | `build/desktop`, directorio de compilación y prueba CMake |
| `--output-dir` | `build/package`, directorio de entregables y resultados de verificación |
| `--tools-dir` | `build/toolchain/desktop-tools`, herramientas Python aisladas y materiales de preparación |
| `--configuration` | `Release`; puede elegirse explícitamente otra configuración de desarrollo |

build-dir y output-dir deben ser subdirectorios independientes dentro de build/ del repositorio para respetar los límites de salida de empaquetado y verificación. Mantenga las mismas opciones en las etapas de una ejecución. `--qt-prefix` explícito prevalece sobre la variable de entorno; no hace falta cambiar CMAKE_PREFIX_PATH global. El punto de entrada prepara los entornos de búsqueda de Qt y herramientas para los subprocesos. Los inicios normales de escritorio no deben heredar variables de pruebas fuera de pantalla. Los resultados generados se guardan solo en build/, target/, dist/ o cachés ignorados y no se incluyen en commits.

## Ejecución por etapas

Las etapas pueden ejecutarse independientemente y no repiten implícitamente la preparación, compilación o prueba anteriores. Si falta un requisito se produce un error; ejecute la etapa correspondiente y vuelva a intentarlo. Cada etapa comprueba su entorno antes de trabajar; `doctor` muestra problemas sin compilar ni empaquetar.

Las compilaciones protegidas de Windows coordinan procesos dentro de su consola existente. Consolas distintas pueden ejecutarse en paralelo; las compilaciones que comparten una consola existente se ejecutan secuencialmente. Durante la compilación, otro código no debe desconectar ni volver a conectar procesos protegidos de esa consola. Una consola temporal creada por un proceso que no tiene una es para ese proceso y sus hijos; procesos externos no deben conectarse. Los scripts antiguos usan un bloqueo alternativo común para consolas sin identificador de ventana y no pueden coordinarse con scripts nuevos; no inicie una compilación nueva en la misma consola sin ventana hasta que termine la antigua. Si no se pueden determinar las listas de procesos, deténgase sin cambiar la página de códigos.

Para compilaciones no interactivas puede usarse explícitamente un wrapper de consola independiente con stdin, stdout y stderr totalmente redirigidos. Ejemplo en PowerShell:

```powershell
New-Item -ItemType Directory -Force build\isolated-logs | Out-Null
cmd /c "python scripts\windows_detached_console.py --timeout 7200 -- python scripts\desktop.py build <NUL >build\isolated-logs\desktop.log 2>&1"
$LASTEXITCODE
```

El wrapper crea una consola oculta independiente por comando y termina los hijos al agotarse el tiempo o salir el supervisor; la ausencia de señales de finalización produce un fallo. Las compilaciones paralelas siguen necesitando valores `--build-dir` separados; el wrapper no bloquea directorios de compilación. Las comprobaciones existentes de dependencias de desktop-build y `windows_ninja_safety.py` siguen ejecutándose, pero envolver otros comandos no las añade automáticamente. Este punto de entrada es solo para comandos no interactivos; no admite prompts, entrada de contraseñas ni interacción con pseudoconsola.

`prepare` verifica el inventario actual de fuentes de formatos del producto. Los contenedores ZIP, TAR/GZIP, 7z, RAR, CAB y ópticos usan Rust independiente; la lista oficial de fuentes del producto está vacía y no se descargan ni compilan los motores oficiales UnRAR/7-Zip. Las referencias de prueba independientes se preparan desde sus propias fuentes fijadas y se separan de las dependencias de ejecución del producto. Los avisos de fuente originales de mapeos RAR limitados permanecen en el empaquetado y los materiales de código correspondiente; véase el [contrato de licencias](licensing.md). `build` solo usa entradas preparadas y no descarga versiones flotantes. Para pruebas solo de Rust, compile primero `xdzip-native-codec`, apunte `XDZIP_NATIVE_CODEC` a ese proceso y ejecute las pruebas del workspace. El flujo del backend nativo también ejecuta `cargo test -p xdzip-runtime --locked native_backend_tests -- --ignored`; tanto `test` común como Foundation CI lo requieren. Un estado ignored no equivale a una comprobación de formato aprobada. El [contrato de backend nativo](native-codecs.md) describe fuentes, autorización de flujos y límites de licencia.

| Comando | Propósito y requisitos |
| --- | --- |
| `doctor --for-stage build` | Comprueba PATH, Rust fijado, Qt, compilador, SDK y versiones; por defecto comprueba el entorno de compilación |
| `prepare` | Prepara herramientas y dependencias compartidas; descargar o instalar Qt, Rust y materiales de empaquetado requiere las opciones explícitas indicadas abajo |
| `build` | Configura y compila producto real, pruebas y diagnósticos; necesita entorno preparado y admite cambios sin commit |
| `test` | Ejecuta por defecto pruebas del workspace Rust, qmllint y CTest; necesita compilación terminada con la misma configuración y admite cambios sin commit |
| `package` | Despliega el runtime, recopila materiales de licencias y procedencia, audita el despliegue y produce los artefactos seleccionados; requiere código limpio, recibos de compilación y prueba del mismo commit y herramientas de empaquetado |
| `verify` | Verifica un runtime o paquete existente sin recompilar, instalar el producto ni cambiar asociaciones predeterminadas; requiere el artefacto y el entorno de verificación de la plataforma |

La comprobación previa distingue herramientas necesarias ausentes, versiones incompatibles y herramientas innecesarias en la etapa actual; la ausencia de Docker o Inno Setup no bloquea compilaciones ordinarias. La comprobación de empaquetado verifica realmente la invocación de Rust/Cargo y los metadatos de dependencias bloqueadas para detectar problemas de PATH o caché antes de recopilar licencias. Los comandos fallidos y límites de verificación permanecen en la salida local o CI; una ejecución parcial no se informa como aprobada.

Prepare primero las herramientas Qt y Rust fijadas:

```sh
python3 scripts/desktop.py prepare --install-rust --install-qt --qt-root build/toolchain/Qt
python3 scripts/desktop.py doctor --for-stage build
python3 scripts/desktop.py build
```

`--install-rust` usa rustup instalado para preparar la versión fijada por el repositorio; no instala rustup. `--install-qt` descarga explícitamente el SDK Qt fijado a `build/toolchain/Qt` por defecto. No hace falta descargar de nuevo un SDK existente; pase su ruta a cada etapa:

```sh
python3 scripts/desktop.py doctor --qt-prefix build/toolchain/Qt/6.8.2/gcc_64
python3 scripts/desktop.py build --qt-prefix build/toolchain/Qt/6.8.2/gcc_64
```

Tras compilar, prepare explícitamente las entradas públicas para el lote completo `native_backend_tests`. Las fixtures CAB, ISO/UDF y RAR básicas se conservan en el código del proyecto bajo `tests/fixtures/cab-iso/` y `tests/fixtures/native-archives/`. Las pruebas 7z crean sus propios archivos con el proceso Rust recién compilado. Este lote de ejecución no requiere herramientas oficiales 7-Zip/UnRAR ni imágenes montadas; otras referencias de interoperabilidad Foundation siguen el [contrato de backend nativo](native-codecs.md).

Para obtener por primera vez las entradas públicas fijadas de RAR PPM, contraseñas antiguas y otros casos, ejecute explícitamente:

```sh
python3 scripts/prepare-rar-rust-fixtures.py --output build/native-test-inputs
```

Si existe una caché fijada verificada por tamaño y SHA-256, use explícitamente el modo sin conexión, sustituyendo la ruta y manteniendo la estructura del subdirectorio `junrar/` del manifiesto:

```sh
python3 scripts/prepare-rar-rust-fixtures.py --output build/native-test-inputs --archives /path/to/fixed-rar-cache --offline
```

Las fuentes y avisos fijados están en `scripts/rar-rust-fixtures.json` y su inventario público de compatibilidad. La salida `sources.json.environment` usa rutas relativas a la raíz del código. Los materiales actuales de código correspondiente no contienen estos siete archivos públicos de caché; `--offline` de Cargo no los proporciona. El preparador no exporta variables de entorno a la shell. Antes del `test` completo inicie un proceso hijo con la siguiente entrega explícita:

```sh
python3 -c "
import json, os, pathlib, subprocess, sys
root = pathlib.Path.cwd()
worker = pathlib.Path(sys.argv[1]).resolve(strict=True)
prepared = json.loads((root/'build/native-test-inputs/sources.json').read_text(encoding='utf-8'))
environment = os.environ.copy()
environment.update({name: str(root/value) for name, value in prepared['environment'].items()})
environment.update(XDZIP_NATIVE_CODEC=str(worker), XDZIP_NATIVE_CODEC_WORKER=str(worker), XDZIP_CAB_BACKEND='rust')
environment['XDZIP_RAR_FIXTURE_PASSWORD'] = 'password'
subprocess.run([sys.executable, 'scripts/desktop.py', 'test', *sys.argv[2:]], env=environment, check=True)
" build/desktop/tools/native-codec/xdzip-native-codec
```

En Windows añada `.exe` a la ruta del proceso. Con SDK, build-dir o tools-dir personalizados, añada las mismas opciones después de la ruta del proceso, por ejemplo `--qt-prefix /path/to/Qt/6.8.2/gcc_64`, y ajuste la ruta al directorio real. `XDZIP_CAB_BACKEND=rust` selecciona la comprobación Rust CAB fijada; `password` es un valor público existente de fixture que solo se pasa al hijo de prueba. Esta entrega mantiene las pruebas predeterminadas del workspace y el lote ignored completo; no descarga entradas ni compila implícitamente el proceso.

`test` habilita el modo fuera de pantalla/software solo para los hijos y no contamina comandos posteriores de escritorio. Para cambios rutinarios, recompile y verifique según el impacto. La verificación exclusiva del núcleo Rust puede ejecutar `cargo test --locked` en el crate afectado sin Qt. `test --skip-rust` ejecuta qmllint y Qt CTest, pero omite el workspace Rust y las pruebas completas de backend nativo; los recibos marcan Rust como `not-run`. Si se necesita una conclusión sobre Rust, ejecútelo por separado y registre el resultado real.

Las pruebas del navegador pueden ejecutarse directamente o en grupos mediante el punto de entrada existente. Cada proceso de la suite crea una raíz temporal corta y única bajo `build/test-tmp/` del proyecto. Al salir, después de destruir los objetos de prueba y antes de destruir la aplicación Qt, restaura TEMP, TMP y TMPDIR y limpia la raíz. También se aíslan los archivos de configuración predeterminados. Unix comprueba la longitud en bytes de la raíz temporal más el nombre IPC existente frente a la capacidad real de sun_path. Una ruta demasiado larga hace fallar explícitamente la precondición de la fixture; acorte la ruta del checkout en lugar de salir del proyecto o cambiar la IPC del producto. Los procesos auxiliares de un grupo heredan la raíz temporal del padre para conservar la identidad del directorio IPC de instancia única, usan sondas privadas previas y terminan antes de la limpieza del padre. Los grupos no comparten raíces. Los hijos de reinicio de idioma y modo de apertura a los que se pasa explícitamente una configuración de prueba siguen compartiéndola en vez de usar una nueva vacía. Antes de ejecutar, verifique realmente creación de archivos hijos, bloqueos, preparación, commits atómicos y lectura de configuración. Un fallo informa `Fixture precondition` y salida distinta de cero; no omita pruebas ni debilite las aserciones de ocultación de ventanas. La existencia del directorio y QTemporaryDir::isValid por sí solos no prueban que se pueda escribir, y no se modifican permisos del sistema para aprobar fixtures.

Las rutas de configuración de prueba usan guardado y restauración limitados mediante `tests/test_environment`. Windows conserva valores de caracteres anchos y distingue entradas no definidas de entradas explícitamente vacías en el bloque de entorno del sistema; Unix conserva los bytes originales. El setter del CRT MSVC elimina valores vacíos, por lo que el bloque nativo se restaura por separado. Compruebe la presencia de valores vacíos mediante processValue del auxiliar o QProcessEnvironment, sin prometer la distinción vacío/nulo del CRT de Qt. Este auxiliar se limita al hilo principal de pruebas secuenciales: configúrelo antes de crear los motores o procesos pertinentes y restáurelo después de destruirlos o unirlos. No sirve para código de producto ni cambios concurrentes del entorno. Con caracteres ASCII, chinos y no representables en ANSI hay que comprobar archivos reales en la ruta original, hijos de restauración del idioma y conservación de ventana y borrador ante fallos. Los resultados automáticos se registran aparte de las operaciones de configuración realizadas por personas.

El producto real usa el nombre `xdzip` en todas las plataformas. Puntos de entrada predeterminados de compilación:

| Plataforma | Punto de entrada |
| --- | --- |
| macOS | `build/desktop/apps/desktop/xdzip.app/Contents/MacOS/xdzip` |
| Windows | `build/desktop/apps/desktop/xdzip.exe` |
| Linux | `build/desktop/apps/desktop/xdzip` |

El ejemplo de componentes `xdzip-ui-preview` no sirve para aceptación con archivos reales. Ajuste las rutas si cambia build-dir. La identidad de compilación real registra versión, canal, commit y estado con cambios. Véase la [aplicación Qt de desarrollo](../../../apps/desktop/README.md) para módulos y depuración CMake de bajo nivel.

## Generación de fuentes de interfaz

`prepare` instala FontTools/Brotli fijados en el entorno compartido y guarda en caché las fuentes de interfaz upstream verificadas por hash. Cada compilación de escritorio recopila automáticamente los catálogos de idiomas admitidos y el texto fijo QML/C++ y genera los subconjuntos de fuentes bajo `<build-directory>/generated/fonts/`. Añadir caracteres no exige incluir binarios de fuentes en commits. Tras la preparación se puede compilar sin conexión; una caché ausente o dañada falla explícitamente. Las compilaciones CMake directas deben seleccionar ese entorno mediante `Python3_EXECUTABLE`.

Para vista previa solo HTML, exportación sin conexión, importación manual de fuentes y comprobaciones de tipografía, consulte la [guía de fuentes compartidas](../../../design/assets/fonts/README.en.md). El empaquetado valida el manifiesto de fuentes de la compilación de producto correspondiente sin regenerarlo. El texto fijo de interfaz y la selección de fuentes del sistema para datos de usuario conservan sus funciones existentes.

## Empaquetado y verificación

`test` y `package` comprueban recibos de compilación, huellas del código, configuración, identidad del SDK y archivos reales del producto. Recompile primero tras cambiar código o configuración. Antes de empaquetar, haga commit del código entregable y compile y pruebe desde ese commit limpio. package no debe eludir comprobaciones ignorando cambios, alterando la identidad del artefacto o reutilizando informes antiguos. Materiales de licencia, metadatos Rust y versión real de la CLI deben coincidir con ese código. Las herramientas de plataforma comprueban restricciones de uso y sobrescritura de directorios de paquete existentes. Prefiera un output-dir nuevo para candidatos recompilados y evitar mezcla de archivos antiguos.

En Windows/Linux prepare las herramientas de empaquetado y ejecute las etapas comunes:

```sh
python3 scripts/desktop.py prepare --packaging
python3 scripts/desktop.py doctor --for-stage package
python3 scripts/desktop.py build
python3 scripts/desktop.py test
python3 scripts/desktop.py package --kind all
python3 scripts/desktop.py verify --kind all
```

`prepare --packaging` prepara dependencias Python aisladas de validación, textos de licencia fijados, cachés de esquemas y materiales de empaquetado de plataforma; Windows incluye CRT e Inno Setup fijados. Los desarrolladores siguen instalando primero las herramientas de sistema. `package --kind runtime` prepara solo el directorio de ejecución desplegado y materiales de auditoría. Después puede ejecutarse `package --kind installer` con el mismo output-dir para crear un instalador desde el runtime verificado; Linux también crea un tar.gz que conserva permisos de ejecución. `--kind all` es el valor predeterminado y realiza ambas operaciones. El empaquetado Windows usa Release o RelWithDebInfo, sin desplegar bibliotecas Qt Debug. Windows proporciona un directorio desplegado y un instalador por usuario; Linux proporciona un directorio de ejecución, tar.gz y .deb. La verificación Windows necesita el ejecutable de prueba del navegador de la misma configuración y entradas de Qt Test/fuera de pantalla. La verificación Linux necesita m2-flat.zip generado por las pruebas de compilación. Los archivos de prueba no entran en el directorio del producto. verify `--kind runtime` verifica el runtime y el valor predeterminado `--kind all` verifica los entregables seleccionados. La verificación ordinaria no instala, actualiza, revierte ni modifica asociaciones de archivos del usuario.

Un DMG macOS necesita una compilación principal seleccionada por separado que pueda registrarse como candidata para documentos, evitando que copias normales de desarrollo o prueba aparezcan en «Abrir con»:

```sh
python3 scripts/desktop.py prepare --packaging
python3 scripts/desktop.py build --build-dir build/desktop-dmg --macos-handler-rank Alternate
python3 scripts/desktop.py test --build-dir build/desktop-dmg
python3 scripts/desktop.py doctor --for-stage package --build-dir build/desktop-dmg
python3 scripts/desktop.py package --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
python3 scripts/desktop.py verify --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
```

El rango predeterminado del manejador en una compilación macOS ordinaria es `None` y no cambia valores predeterminados del usuario; seleccione `Alternate` solo para una aplicación principal duradera o entrada de DMG. El empaquetado local macOS solo admite `all` y produce un DMG de desarrollo arm64 con la firma ad-hoc de integridad necesaria. verify comprueba Cocoa/Metal tras la reubicación y una extracción real. La CI ordinaria no debe ejecutar el empaquetado Mac ni desplegar, firmar, comprimir o subir un paquete de aplicación; el [protocolo de publicación](release-protocol.md) describe la excepción del Release dedicado. Véase la [guía de despliegue Mac](macos-deployment.md) para límites completos y mantenimiento del registro de copias.

Véanse [despliegue Linux](linux-deployment.md), [DMG Mac](macos-deployment.md) e [instalación y asociaciones](installers-associations.md) para estructuras y herramientas subyacentes. Los instaladores Windows/Linux no tienen firma formal; macOS no tiene firma Developer ID ni notarización. Diagnostique problemas de instalación o ejecución con el producto real y la verificación de plataforma correspondiente; ejecutar bajo el SDK no demuestra la verificación de un paquete independiente.

## Relación con CI

Antes de hacer commit o push, ejecute compilaciones o verificaciones locales adecuadas al contenido propuesto. Los cambios de documentación ejecutan `python3 scripts/check-docs.py` y comprueban los resultados generados afectados. Los cambios Rust comprueban formato y al menos compilan o prueban los crates afectados. Las pruebas con backends de formato aislados preparan el proceso real y las fixtures necesarias como se describe arriba. Los cambios C++, de puente y QML compilan el producto afectado con `python3 scripts/desktop.py build` y ejecutan las pruebas pertinentes. No todos los cambios requieren toda la interoperabilidad de formatos ni diez repeticiones completas de la suite del navegador.

Al añadir o cambiar una ventana QML, ejecute `python3 scripts/check-window-contracts.py --self-test`, `python3 scripts/check-operation-components.py` y las regresiones de diseño afectadas con dos idiomas y varios tamaños de fuente. Después de compilar, `python3 scripts/desktop.py test --skip-rust` ejecuta todas las comprobaciones de componentes Qt y CTest. Como alternativa, tras configurar rutas del SDK Qt y entorno fuera de pantalla, ejecute casos pertinentes mediante puntos existentes como `ctest --test-dir build/desktop -R qt_task_components --output-on-failure`. Los cambios de botones de opción o casillas también ejecutan las regresiones QAccessibleActionInterface apropiadas, como `accessibleRadioActionsCommitSelection` y `accessibleCheckboxActionsCommitIntent`.

La [CI ordinaria](../../../.github/workflows/docs.yml) ejecuta solo comprobaciones estáticas ligeras y no sustituye compilaciones locales del producto. [Foundation](../../../.github/workflows/foundation.yml) y el [workflow Qt](../../../.github/workflows/qt-components.yml) se ejecutan manualmente cuando se necesitan. Qt usa el mismo desktop.py para compilación, pruebas Qt y empaquetado y verificación de ejecución Windows/Linux. [ci-desktop.ps1](../../../scripts/ci-desktop.ps1) conserva comprobaciones de repetición, visuales, interoperabilidad y compatibilidad de instalación bajo demanda y no es obligatorio para la verificación local rutinaria.

Los ciclos históricos completos de producto, instalación real con actualización o reversión y asociaciones se ejecutan solo en hosts CI Windows/Linux desechables o contenedores aislados; el punto de entrada común no los convierte en comportamiento predeterminado de verify local. El trabajo macOS del workflow Qt bajo demanda usa auditoría de despliegue en modo SDK y comprobaciones Cocoa de inicio y apertura de archivos, conservando solo logs y JSON. La [política CI](../../../developers/engineering/ci-policy.md) describe todas las comprobaciones y límites de evidencia.

## Límites de archivos y tareas

Abrir solo permite navegar; no extrae automáticamente. Los archivos normales con el mismo nombre admiten preguntar, omitir, renombrar y reemplazar explícitamente. Publique un archivo completo solo después de verificar y sincronizar su contenido. Cancelar antes del commit o un fallo de escritura conserva el archivo anterior; las entradas extraídas ya confirmadas permanecen. Cerrar la ventana de tarea solo oculta los detalles. Al salir se usan la decisión sobre tareas activas y la barrera de cierre asíncrono.

ZIP puede restaurar enlaces simbólicos relativos y bits ejecutables bajo condiciones POSIX restringidas, pero no bits de privilegio, atributos extendidos ni fechas originales de modificación. Esto no acredita instalación de un paquete de aplicación ni verificación de firma. Origen y destino deben cumplir el [contrato de extracción segura](zip-safe-extraction.md). El staging tras un fallo solo se informa y conserva para inspección; el contenido desconocido no se recupera ni elimina automáticamente.

## Microsoft Store MSIX

Para enviar a Store, siga el [procedimiento para mantenedores](../../../developers/market/windows/build.md).

<a id="windows-msix-实验构建"></a>

## Compilación experimental de MSIX Windows

MSIX reutiliza la compilación de escritorio Windows Release, pruebas y directorio de ejecución auditado del mismo commit. Complete `prepare --packaging`, `build` y `test` como se indicó y ejecute con las mismas opciones:

```powershell
python scripts/desktop.py package --kind runtime
python scripts/desktop.py package --kind msix --msix-name xdzip.Development --msix-publisher 'CN=xdzip development' --msix-publisher-display-name xdzip
python scripts/desktop.py verify --kind msix
```

La identidad de ejemplo es solo para desarrollo local. El nombre del paquete, publicador y nombre visible de una identidad Store formal deben coincidir con Partner Center. La salida `build/package/xdzip-windows-x64.msix` es un paquete experimental sin firmar, no automáticamente distribuible por instalación lateral ni certificado por Store. La instalación lateral normal necesita una firma confiable para el dispositivo que coincida con el publicador. El modo restringido de desarrollo sin firma de Windows 11 exige además un OID de publicador concreto y, para contenido ejecutable, normalmente derechos de administrador; la identidad de ejemplo no cumple ese modo. Los scripts no crean certificados, firman, instalan ni envían a Store. El mínimo técnico es Windows 10 22H2 x64 y no amplía los compromisos de plataformas admitidas. La versión `1.0.<product build>.0` es distinta de la versión visible; la política de versiones Store debe revisarse antes de agotar los números de compilación.

La compilación requiere código limpio, recibos de compilación y prueba coincidentes, un directorio de ejecución totalmente auditado y MakeAppx y MakePRI del Windows SDK. Los scripts vuelven a comprobar el runtime y sus marcadores de capacidades de actualización, asociación y menú, generan el manifiesto y recursos de iconos en staging separado, comparan cada archivo empaquetado y registran hashes. No sobrescriben paquetes ni staging existentes. El manifiesto declara los tipos de extensión existentes del registro de formatos, un punto de entrada de escritorio de confianza completa y clases de menú específicas del canal.

Los iconos de aplicación incluyen variantes de escala y tamaños exactos de destino en formas predeterminada, sin placa para fondo oscuro y sin placa para fondo claro. MakePRI indexa estos recursos en `resources.pri`, permitiendo que Windows seleccione el tamaño previsto sin añadir una placa del sistema. Las asociaciones declaran su propio logotipo por formato y reutilizan los fotogramas PNG de 16–256 píxeles de cada ICO Windows existente sin cambiar diseño, tamaño visible, márgenes ni posición. La política está en [msix-icons.json](../../../specs/visual/msix-icons.json). Estos son recursos del paquete instalado; los logotipos de ficha de Partner Center son independientes. `verify --kind msix` compara los nuevos recibos de iconos con el paquete real, incluidas dimensiones, variantes completas, imágenes canónicas de formato y presencia del índice de recursos. Los cambios llegan a usuarios instalados solo al compilar y entregar un paquete actualizado.

El ejemplo usa `--msix-channel sideload` por defecto, que registra solo el menú clásico de Windows 11. Para subir a Store use `package --kind msix --msix-channel store` con la identidad exacta de Partner Center. Esto declara ambos menús, moderno y clásico; la entrada moderna permanece oculta hasta que el paquete tenga realmente una firma de sistema Store. Los marcadores de canal se añaden al empaquetar y no requieren recompilar el runtime auditado. Mantenga las salidas en directorios separados. Véase la [política de canales de menú](shell-actions.md#límite-del-paquete-windows). Licencias, instalación real, asociaciones, desinstalación, firma Store y aceptación real en el sistema de destino son independientes del éxito del empaquetado.

El manifiesto MSIX declara los diez idiomas del registro compartido, correspondientes a los catálogos integrados; los recibos registran y verifican la lista. Este flujo MakeAppx no convierte automáticamente los recursos de traducción Qt en declaraciones de idioma del paquete. «Idiomas admitidos por el paquete» de Partner Center refleja el paquete subido; tras corregir el código hay que recompilar y sustituir el paquete del borrador para actualizarlo. Los «idiomas adicionales de la ficha Store» se configuran por separado y no sustituyen las declaraciones del manifiesto. Véase la [documentación de idiomas del manifiesto de Microsoft](https://learn.microsoft.com/windows/apps/design/globalizing/manage-language-and-region).

## Evidencia y nueva verificación

Seleccione las comprobaciones afectadas según la [verificación general](../../../developers/qa/README.md) y la [verificación de escritorio bajo demanda](../../../developers/qa/private-test-manual.md); el trabajo ordinario no exige un formulario de aceptación manual. Los recibos de compilación y prueba son `desktop-build.json` y `desktop-test.json` bajo build-dir. Los recibos de preparación, empaquetado y verificación son `<stage>.json` bajo output-dir y enlazan la evidencia subyacente. Los recibos automáticos registran etapas y fuentes reales. El renderizado fuera de pantalla/software, las comprobaciones Cocoa nativas y las comprobaciones manuales realizadas se registran por separado y no se sustituyen entre sí. La aceptación existente permanece en [#5](https://github.com/xiaodaozhi/xdzip/issues/5) y no se repite solo por regenerar el inventario de candidatos.

La entrega de artefactos Windows/Linux incluye código del mismo commit y avisos de licencia; Actions conserva artefactos durante 14 días. Tras su caducidad, reconstruya con las identidades y entradas fijadas. deployment-audit.json y los inventarios CycloneDX por fuente o archivo ayudan a verificar la procedencia, pero pueden ser incompletos y no acreditan aprobación de licencias. Véanse [entrega de fuentes](source-delivery.md) y el [contrato de verificación automatizada](../../../developers/engineering/candidate-verification.md) para reconstrucción del código y sustitución de Qt.

Compilaciones, capturas, mediciones, informes y paquetes permanecen solo como artefactos locales ignorados o de CI. No describa formularios generados pendientes, comprobaciones fuera de pantalla ni SBOM parciales como aprobaciones de entorno limpio y de todas las condiciones de publicación.

## Framework de actualización macOS

Las compilaciones nativas usan `prepare-sparkle.py` para preparar `build/toolchain/Sparkle/2.10.0-xdzip-proxy.2`. La biblioteca principal del framework se compila desde código upstream fijado y el parche de proxy; asistentes de instalación, recursos y herramientas de firma conservan la procedencia del SDK oficial. Véase el [contrato de dependencias](dependencies.md#macos-更新提供者). Se pueden reutilizar recibos de compilación válidos; un SDK oficial antiguo no puede sustituir el framework derivado con interfaces de proxy. CI puede seleccionar explícitamente un directorio temporal de SDK.

`python3 scripts/prepare-sparkle.py --output build/toolchain/Sparkle/2.10.0-xdzip-proxy.2` usa Clang nativo de Xcode Command Line Tools o Xcode existentes y no necesita otro SDK Qt. Los archivos originales de SDK y fuentes se guardan en `build/source-cache/`; en modo sin conexión pueden pasarse `--archive` y `--source-archive` por separado. La compilación no ejecuta scripts del código descargado, modifica proxies del sistema, accede a claves de firma de producción ni recopila archivos de entrega de código del proyecto.

`check-macos-proxy.py` comprueba por defecto solo el descargador propio mediante loopback. `--system-conflicts` está limitado a runners Apple Silicon alojados y desechables en un workflow dedicado. Un auxiliar restringido prepara, restaura y verifica el proxy del sistema y las entradas hosts de dominios de prueba fijados. No añada esta opción localmente.
