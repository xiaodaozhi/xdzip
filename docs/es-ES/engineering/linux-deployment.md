<a name="xdzip-linux-standalone-runtime-directory-and-verification"></a>

# Directorio autónomo de ejecución Linux y verificación

[English](../../engineering/linux-deployment.md) | [简体中文](../../zh-CN/engineering/linux-deployment.md) | [繁體中文（台灣）](../../zh-TW/engineering/linux-deployment.md) | [日本語](../../ja-JP/engineering/linux-deployment.md) | [한국어](../../ko-KR/engineering/linux-deployment.md) | [Français](../../fr-FR/engineering/linux-deployment.md) | [Deutsch](../../de-DE/engineering/linux-deployment.md) | Español | [Português (Brasil)](../../pt-BR/engineering/linux-deployment.md) | [Русский](../../ru-RU/engineering/linux-deployment.md)

La base actual de despliegue para desarrollo es Ubuntu 24.04, x86_64 y X11. El objetivo es ejecutar el producto actual sin el SDK Qt de desarrollo; esto no promete otras distribuciones, versiones anteriores de glibc, Wayland ni compatibilidad completa del instalador. La CI de macOS conserva compilación, pruebas y verificación de inicio nativas. Los DMG locales de desarrollo siguen la [guía de despliegue Mac](macos-deployment.md) y no se ven afectados por este proceso.

<a name="xdzip-layout-and-generation"></a>

## Estructura y generación

`scripts/deploy_linux.py` acepta solo archivos ELF fiables del proyecto y un SDK Qt 6.8.2 indicado explícitamente, y crea un nuevo directorio build/ o dist/:

- `xdzip` es el lanzador y utiliza bibliotecas, complementos y QML locales sin heredar las rutas de búsqueda del SDK de desarrollo.
- `bin/xdzip` es el producto; el `qt.conf` adyacente usa rutas Qt relativas.
- `lib/` contiene bibliotecas compartidas Qt/SDK realmente importadas; `qml/` conserva el estilo Basic Controls y los submódulos de implementación necesarios. Otros estilos de Controls, NativeStyle, Effects, variantes de diálogo específicas de estilos, archivos de diseño y metadatos de tipos QML se eliminan antes de recopilar dependencias nativas para no desplegar sus bibliotecas exclusivas.
- `plugins/` contiene xcb, verificación fuera de pantalla, imágenes, iconos, entrada de plataforma, integración gráfica xcb y el necesario `tls/libqopensslbackend.so`; el despliegue falla sin el backend OpenSSL. Los complementos de plataforma EGLFS, framebuffer y Wayland no se presentan como entrega verificada.
- `translations/` conserva traducciones de controles estándar Qt; `share/notices/` entrega avisos actuales y materiales de dependencias fuente.
- `share/deployment.json` registra QML, dependencias de ejecución del sistema, bibliotecas SDK copiadas y límites; README.md explica el uso.

El analizador utiliza [qmlimportscanner](https://doc.qt.io/qt-6.8/qtqml-tooling-qmlimportscanner.html) del Qt fijado. Xdzip.Settings y Xdzip.Native son módulos registrados integrados; las bibliotecas estáticas de desarrollo no se copian. Fallan las dependencias ordinarias ausentes, rutas QML fuera del SDK, conflictos entre bibliotecas del mismo nombre y bibliotecas externas desconocidas. La recopilación recursiva de importaciones nativas copia solo bibliotecas del SDK, no la glibc ni los controladores gráficos del anfitrión; las dependencias del sistema se enumeran según su paquete Debian propietario real.

La poda valida el árbol QML copiado, los complementos Basic necesarios, las importaciones fuertes conservadas y las estructuras de estilos conocidas antes de eliminar. Enlaces inesperados, estilos o selectores de diálogo desconocidos y dependencias fuertes de módulos omitidos provocan fallo. Diálogos predeterminados, métodos de entrada, accesibilidad, complementos de imagen, TLS y traducciones del sistema mantienen sus reglas. `share/deployment.json` registra los archivos QML retirados y el número de bytes. El `cli/` privado comparte `bin/xdzip-native-codec` mediante su ruta de contexto fija; los paquetes CLI independientes siguen siendo autónomos.

HTTPS requiere además `libssl3t64` y `ca-certificates` de Ubuntu. Verificar la pertenencia a paquetes del sistema de `libssl.so.3` y `libcrypto.so.3` y un almacén de confianza CA del sistema no vacío. El backend OpenSSL de Qt puede cargar estas bibliotecas en ejecución, por lo que ldd por sí solo no demuestra que las dependencias estén completas. El manifiesto de despliegue registra versiones de paquetes binarios instalados, paquetes fuente y sus versiones. Los instaladores declaran ambas dependencias; el SBOM de archivos reales las registra como paquetes de sistema necesarios sin incluir las bibliotecas del sistema en la aplicación.

patchelf establece un RPATH `$ORIGIN` relativo en cada ELF, calculando la distancia relativa de cada complemento a lib/. La redirección de rutas Qt sigue las [reglas oficiales de qt.conf](https://doc.qt.io/qt-6.8/qt-conf.html), permitiendo trasladar todo el directorio. Las bibliotecas siguen enlazadas dinámicamente y son reemplazables; no se restringe el reemplazo compatible de Qt. Véase [despliegue Qt en Linux](https://doc.qt.io/qt-6.8/linux-deployment.html) para los límites generales.

Tras reescribir el RPATH relativo, el despliegue elimina tablas de símbolos nativas innecesarias y comprueba que cada entrada DT_NEEDED y RPATH/RUNPATH no haya cambiado. Registra versión de herramienta, hashes y número de bytes antes y después para cada ELF en `symbol_stripping`; las entradas SDK y los binarios de compilación originales permanecen intactos. Para una copia de diagnóstico el script de despliegue subyacente admite `--keep-symbols`. La atribución SDK final y los hashes de la aplicación describen los archivos transformados. Esto reduce el tamaño del paquete, no demuestra una mejora medida de memoria o rendimiento.

Usar por etapas el [punto de entrada unificado de compilación](developer-build.md) para generación y verificación habituales. Completar primero compilación, pruebas y preparación del empaquetado con la misma configuración y ejecutar:

```sh
python3 scripts/desktop.py doctor --for-stage package
python3 scripts/desktop.py package --kind all
python3 scripts/desktop.py verify --kind all
```

El directorio de compilación predeterminado es build/desktop, la salida build/package y el SDK Qt `build/toolchain/Qt/6.8.2/gcc_64`; pasar las opciones compartidas en cada etapa para otras ubicaciones. `package --kind runtime` genera el directorio de ejecución y los materiales de auditoría; después `package --kind installer` con el mismo output-dir genera tar.gz y .deb. El valor predeterminado `all` realiza ambos en orden. Los scripts subyacentes [deploy_linux.py](../../../scripts/deploy_linux.py), [package_linux.py](../../../scripts/package_linux.py) y [verify_linux_runtime.py](../../../scripts/verify_linux_runtime.py) conservan la lógica de plataforma; consultar su `--help` para resolver problemas.

El despliegue solo se ejecuta en Linux x86_64 nativo y no sobrescribe directorios existentes. La comprobación previa del empaquetado verifica qmlimportscanner, ldd, patchelf, dpkg-query y dpkg-deb necesario para el instalador; estas herramientas de compilación no entran en el producto. La verificación de ejecución en contenedor requiere además Docker y un daemon accesible; las compilaciones se limitan a la imagen Ubuntu fijada y los paquetes de sistema registrados. La ausencia de Docker no bloquea compilación y pruebas ordinarias, pero verify falla explícitamente; una comprobación de aislamiento no ejecutada no se registra como superada. verify ordinario no instala el .deb en el anfitrión ni cambia asociaciones; el ciclo de instalación sigue siendo una comprobación CI desechable.

<a name="xdzip-automated-evidence-without-the-sdk"></a>

## Evidencia automatizada sin SDK

El contenedor usa Ubuntu 24.04 fijado por resumen e instala las dependencias de ejecución del sistema reales y soporte de prueba Xvfb, fuentes y configuración regional, pero no Qt, Rust ni compilador, y vuelve a comprobar su ausencia. No monta el SDK de desarrollo ni el árbol completo de fuentes, solo el directorio del producto de solo lectura, un ZIP independiente de 100.000 entradas propiedad del proyecto y scripts de verificación. La verificación de ejecución de archivos desactiva red y escalada de privilegios; solo se puede escribir en el directorio temporal de configuración.

Montar la aplicación en una ruta nueva que contenga espacios. Tras limpiar las variables de búsqueda de desarrollo, usar el lanzador para comprobar:

1. La ejecución real de `--version`.
2. El inicio y cierre seguro de una ventana QML real en X11.
3. La sonda nativa del producto completa la primera imagen, lectura de 100.000 entradas, cancelación de una solicitud activa y salida; recuentos y resultados de cancelación cumplen el contrato.
4. Tras limpiar las variables de desarrollo del anfitrión, verificar todos los ELF empaquetados en modo desplegado sin recurrir al SDK. El SBOM de archivos reales conserva la misma identidad de versión y commit y supera el esquema oficial.
5. Activar la red en un nuevo proceso de la misma imagen sin SDK, ejecutar `--update-probe` del producto con `XDZIP_SETTINGS_PATH` aislado y solicitar realmente GitHub por HTTPS. Debe informar de un backend OpenSSL disponible y completar la comprobación de actualización; conservar `.https.json` y `.https.log` por separado. Los fallos de red, certificado o TLS bloquean la verificación.

La sonda HTTPS registra por separado backend OpenSSL, archivo CA del sistema y resultados reales de la solicitud a GitHub. Una solicitud real correcta por sí sola no demuestra verificación de firma ni instalación nativa de actualización. El análisis de firmas y los flujos de actualización sin SDK se verifican mediante el [flujo independiente de regresión de actualizaciones](../../../.github/workflows/update-regression.yml); estas evidencias no se sustituyen entre sí.

Conservar registros brutos e informes estructurados de cada intento como `.https.attempt-N.log` y `.https.attempt-N.json`, además de `.https.log` para el intento final. Informes ausentes o inválidos, tiempos agotados y salidas del proceso mantienen el estado de fallo. Reintentar solo cuando el controlador del producto confirme la limitación de tasa de la lista anónima de publicaciones de GitHub y estén completos el estado, la fase de solicitud, las cabeceras de respuesta y los diagnósticos. Esperar hasta el plazo del servidor o usar espera progresiva limitada, con un máximo de tres intentos y 3.700 segundos de espera total, como la sonda de aceptación de macOS. Un plazo superior al presupuesto restante, una espera interrumpida o un error TLS, de red u otro falla inmediatamente. La sonda debe completar una solicitud real de actualización para superar la prueba. Registrar cada decisión y duración real de espera en `.https-waits.json`.

Antes de reutilizar una ruta de salida, mover el informe y registros anteriores a un directorio `.history` con el nombre del informe en el mismo directorio padre, con un subdirectorio por ejecución. Si una nueva ejecución falla a mitad de camino, el antiguo informe correcto no permanece en la ruta de salida actual, evitando confusión; los artefactos históricos siguen siendo inspeccionables por separado.

linux-runtime.json registra eventos reales y SHA-256 del ejecutable. La representación por software Xvfb/Qt Quick solo demuestra el flujo de ejecución, no GPU física, ventanas de candidatos chinos, lectores de pantalla reales ni instalación completa de escritorio.

<a name="xdzip-download-delivery-and-remaining-gates"></a>

## Entrega descargable y requisitos restantes

La CI genera xdzip-linux-x86_64.tar.gz y registra SHA-256 y hashes del ejecutable. tar.gz conserva permisos de ejecución y evita perderlos al extraer ZIP de Actions. Las rutas del archivo, UID/GID y marcas de tiempo se normalizan sin directorios privados. Tras descargar, extraer y ejecutar `./xdzip` dentro del directorio; no mover solo bin/xdzip. El directorio linux/ sin empaquetar no se vuelve a subir.

El directorio de ejecución `build/package`, tar.gz y .deb generados aquí son entregas experimentales de desarrollo y no instalan ni cambian asociaciones predeterminadas automáticamente. Los tar.gz y .deb oficiales de Ubuntu para Release se generan en el flujo Release dedicado y se entregan según el [protocolo de publicación](release-protocol.md). El alcance de plataformas sigue la [política de soporte](support-policy.md); otras distribuciones o escritorios, Wayland y ampliaciones requieren incidencias separadas. La aceptación existente de escritorio permanece en [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5). Para cambios posteriores volver a verificar las comprobaciones afectadas de IME y lector de pantalla reales y de instalación, actualización y desinstalación limpias; reconstruir un directorio no exige por sí solo repetir la aceptación existente. La revisión histórica de componentes de `v0.1.0` está en [#4](https://github.com/xiaodaozhi/xdzip/issues/4) y [#53](https://github.com/xiaodaozhi/xdzip/issues/53); ejecutar correctamente el directorio no completa automáticamente los requisitos de futuras publicaciones.

Véase [instalación y asociaciones](installers-associations.md) para instalación, actualización, reversión y desinstalación de Ubuntu .deb, bloqueos de ejecución y automatización MIME de escritorio. Superar el contenedor de ejecución no sustituye las comprobaciones reales de escritorio ni la revisión de licencias de publicación.
