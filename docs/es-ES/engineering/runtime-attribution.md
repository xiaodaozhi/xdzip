<a name="xdzip-actual-component-attribution-and-license-materials"></a>

# Atribución de componentes reales y materiales de licencia

[English](../../engineering/runtime-attribution.md) | [简体中文](../../zh-CN/engineering/runtime-attribution.md) | [繁體中文](../../zh-TW/engineering/runtime-attribution.md) | [日本語](../../ja-JP/engineering/runtime-attribution.md) | [한국어](../../ko-KR/engineering/runtime-attribution.md) | [Français](../../fr-FR/engineering/runtime-attribution.md) | [Deutsch](../../de-DE/engineering/runtime-attribution.md) | Español | [Português (Brasil)](../../pt-BR/engineering/runtime-attribution.md) | [Русский](../../ru-RU/engineering/runtime-attribution.md)

Este proceso complementa los SBOM por archivo al asociar las bibliotecas y los complementos Qt reales con las declaraciones SPDX del SDK. Su salida es evidencia revisable, no una selección automática de licencias ni una aprobación de cumplimiento. La documentación necesaria para una publicación oficial sigue incompleta.

<a name="xdzip-inputs-and-sources"></a>

## Entradas y fuentes

Desde Qt 6.8 se proporcionan [inventarios SPDX](https://doc.qt.io/qt-6.8/licenses-used-in-qt.html) correspondientes a la compilación. Las herramientas leen solo declaraciones de ejecución como qtbase, qtdeclarative, qtsvg y qttranslations del directorio sbom/ del SDK Qt 6.8.2 fijado, excluyendo las herramientas de compilación y prueba qttools. Se validan la versión SPDX, el nombre del módulo y la versión Qt; se conservan el SHA-256, el espacio de nombres y las referencias estables a paquetes del documento original. La salida conserva solo los campos necesarios, sin copiar comentarios de compilación upstream con rutas de equipos de desarrollo.

`scripts/qt-license-sources.json` fija los commits oficiales de las fuentes Qt y SPDX license-list-data, las URL de los textos de licencia y los SHA-256. Las descargas se guardan en caché bajo build/; cualquier discrepancia de hash de caché o descarga produce un fallo. No se siguen URL SPDX arbitrarias para descargar materiales. Además de los textos estándar de LICENSES/, se incluyen los avisos originales de los algoritmos de escalado suave de Qt GUI y SVG, y los textos LGPL-2.1 referenciados por las declaraciones de dependencias del sistema. Los textos LicenseRef y derechos de autor upstream se conservan en el JSON de declaraciones.

Los avisos de ejecución de Qt incluyen también la fuente editable fijada de Public Suffix List utilizada por Qt Network, el texto MPL-2.0 y las instrucciones de obtención. El recopilador verifica el blob upstream, el SHA-256 y los bytes de la licencia. La entrega de fuentes verifica por separado que esta entrada reproduce dos formas de datos generados en el archivo Qt; véanse los [materiales fuente correspondientes](source-delivery.md). Esto no sustituye las comprobaciones de fuente y reemplazo de otros módulos Qt.

Los avisos MIME de Tika en Qt Core conservan la atribución pertinente de Apache Software Foundation y PRONOM de los Archivos Nacionales británicos, los textos Apache-2.0 y OGL-UK-3.0 y el NOTICE upstream completo fijado. El NOTICE completo también describe otros componentes upstream de Tika; ello no implica que se distribuyan con este producto. Véanse los [materiales fuente correspondientes](source-delivery.md) para los datos MIME originales y la verificación de la conversión de Qt.

<a name="xdzip-file-and-component-association"></a>

## Asociación de archivos y componentes

1. Recalcular los hashes de archivos dentro de la raíz de verificación y rechazar los modificados después de deployment-audit.json.
2. El modo SDK de macOS usa las rutas relativas del SDK Qt realmente resueltas y la identidad de bytes, sin afirmar un despliegue independiente.
3. El despliegue Linux registra la ruta y SHA-256 originales del SDK y la ruta y SHA-256 del artefacto tras patchelf. Las comprobaciones posteriores verifican los registros en ambos sentidos y rechazan pruebas de copia obsoletas o alteradas.
4. Windows prefiere la identidad de bytes de archivos del SDK. Los candidatos con nombre único pero bytes distintos se marcan solo como filename-only-unverified-rewrite y se registran como lagunas, no como fuentes verificadas.
5. Usar SPDX CONTAINS para localizar los propietarios de archivos y resolver recursivamente DEPENDS_ON, incluidas referencias entre repositorios Qt y terceros integrados. Las referencias de paquetes ausentes siguen siendo lagunas. No incluir por error todas las herramientas en la ejecución mediante un CONTAINS de todo el repositorio.
6. Los QML y traducciones no nativos sin declaración por archivo se asocian solo al ámbito del repositorio fuente correspondiente; no se infieren sus licencias individuales. Se registran por separado bibliotecas nativas del SDK sin entradas de archivo SPDX, archivos nativos fuera de Qt, licencias NOASSERTION y textos ausentes.

Los bytes reales del SDK pueden diferir del SHA-1 del SPDX de compilación upstream, por ejemplo por procesamiento posterior de instalación o firma. Las herramientas conservan valores esperados, comparaciones y lagunas sin inferir causas. La asociación de bytes del SDK, los registros de reescritura del despliegue y los hashes originales upstream son tres tipos de evidencia distintos.

El empaquetado dedicado de macOS genera asociaciones del despliegue mientras existe la aplicación final firmada, relacionando entradas SDK por bytes exactos de recursos o por secciones arm64, UUID y versiones de importación, y añade el informe al SBOM de archivos reales. La comparación excluye firmas, disposición de comandos de carga y LINKEDIT. Véanse [procedencia y avisos del entorno nativo de ejecución](native-runtime-materials.md) para el ámbito y los avisos específicos. El modo SDK ordinario no demuestra por ello conclusiones sobre el despliegue.

Para complementos trasladados desde directorios QML a `Contents/PlugIns/quick/`, la asociación del despliegue puede usar enlaces simbólicos QML auditados dentro del paquete para localizar una entrada SDK única. Volver a comprobar enlaces reales, destinos auditados y hashes de archivos tanto al generar como al utilizar registros, y efectuar la misma comparación de bytes o Mach-O. Una procedencia ausente sigue sin verificar; se rechazan varias fuentes distintas o enlaces modificados. Coincidir solo por nombre no refuerza la atribución ni elimina diferencias de hash del SDK upstream o lagunas de revisión de licencias.

Rust conserva el grafo completo de compilación del anfitrión y genera candidatos de ejecución siguiendo dependencias ordinarias desde `xdzip-qt-bridge` de escritorio, `xdzip-codec-bridge` del trabajador de formatos y el `xdzip-update-helper` real de Windows/Linux; macOS no tiene una raíz separada para el asistente de actualización. Se excluyen herramientas de compilación, desarrollo y proc-macro y sus dependencias. Son posibles entradas de ejecución, no una prueba de que el código máquina de cada paquete haya sobrevivido al enlazado; el recorte del enlazador, C/C++ nativo y los entornos de ejecución del compilador necesitan más revisión.

Las alternativas sencillas antiguas de licencia Cargo separadas por barras se convierten en SPDX `OR` en el CycloneDX de fuentes, conservando el original upstream en `xdzip:cargo_license_raw`. Se rechazan expresiones con barras y precedencia ambigua. No se reescriben los inventarios ni avisos de compilación originales; `AND`, `OR` y `WITH` estándar permanecen sin cambios. Es una normalización de formato, no una selección de rama de licencia ni una aprobación de publicación.

<a name="xdzip-execution-and-delivery"></a>

## Ejecución y entrega

La variante derivada conocida de QtNetwork con parches añade `network_security_variant` a los registros de archivos; los informes también conservan comprobantes completos de compilación revisados de nuevo. Las diferencias de resumen del SDK upstream siguen visibles; la revisión de publicación utiliza fuentes fijadas, parches completos, comprobantes de compilación y evidencia de transformación del despliegue para explicarlas. La preparación de publicación exige vincular la atribución final de Network a esos bytes; una coincidencia de nombre no basta. Los SBOM de fuentes y artefactos identifican por separado la variante sin convertir el estado global incompleto en aprobación de licencias.

```sh
python3 scripts/prepare-runtime-notices.py --qt-prefix /path/to/Qt/6.8.2/gcc_64 \
  --output build/review/notices/Qt-runtime
python3 scripts/runtime_attribution.py --audit build/review/deployment-audit.json \
  --root build/review/linux --qt-prefix /path/to/Qt/6.8.2/gcc_64 \
  --deployment build/review/linux-deployment.json --output build/review/runtime-attribution.json
python3 scripts/sbom.py --audit build/review/deployment-audit.json \
  --source build/review/source-build.cdx.json --attribution build/review/runtime-attribution.json \
  --output build/review/artifact-files.cdx.json
```

Linux pasa registros de reescritura del despliegue; las demás plataformas omiten --deployment. Escribir los informes fuera de la raíz de verificación para evitar recursión de autoinventario. Los flujos Qt bajo demanda y Release preparan materiales de licencia antes del despliegue y el hash final. Windows y Linux colocan los avisos en los entornos de ejecución reales; el tar.gz de Linux también los entrega. Qt bajo demanda en macOS lee el SDK y conserva solo registros y JSON. Un flujo Release dedicado y autorizado explícitamente despliega, firma ad hoc, empaqueta y comprueba por separado los materiales reales conforme al [protocolo de publicación](release-protocol.md), conservando un ámbito desplegado y hashes independientes. Los registros del SDK no sustituyen la atribución del paquete final de la aplicación.

runtime-attribution.json incluye archivos, componentes, relaciones, declaraciones upstream, derechos de autor, fuentes de textos y lagunas. artifact-files.cdx.json añade los mismos componentes y relaciones conocidas de propiedad, vinculando el informe con los mismos hashes de auditoría real e identidad de compilación. Continúan las comprobaciones del esquema oficial CycloneDX y de referencias; compositions sigue incompleto.

Las comprobaciones predeterminadas permiten lagunas de revisión de desarrollo registradas explícitamente, pero nunca las marcan silenciosamente como superadas. Los errores de identidad, ruta, bytes, versión, registro de copia o hash de texto de licencia fallan de inmediato. --require-complete bloquea la publicación y actualmente falla si la atribución o los materiales de publicación están incompletos. Las pruebas cubren archivos reescritos, registros de copia obsoletos, escape de ruta, versiones incorrectas, omisiones entre documentos, SBOM parciales y exclusión de herramientas Rust.

<a name="xdzip-unresolved-release-gates"></a>

## Requisitos de publicación sin resolver

Un [recopilador de entrega de fuentes](source-delivery.md) conserva el código real del proyecto, Rust vendor, cuatro archivos de módulos Qt de ejecución, dos de módulos de compilación e instrucciones de reconstrucción y reemplazo dinámico. No demuestra equivalencia de compilación SDK/upstream por elemento. Cada publicación aún debe revisar la atribución exacta del enlazado estático, los entornos de ejecución del sistema y redistribuibles, las licencias individuales de todos los recursos integrados, las elecciones de ramas de licencias libres y el cumplimiento. La modificación o sustitución de metadatos en la [verificación local del despliegue Mac](macos-deployment.md) y la [compilación, carga y sustitución reales de Qt desde fuentes](../../../developers/engineering/candidate-verification.md#qt-source-and-compatible-replacement) son procesos distintos con evidencia separada; ninguno demuestra equivalencia de compilación byte a byte para SDK arbitrarios. Los inventarios fuente y los campos SPDX no sustituyen las obligaciones de licencia. La procedencia y licencias del candidato histórico `v0.1.0` están en [#4](https://github.com/xiaodaozhi/xdzip/issues/4) y [#53](https://github.com/xiaodaozhi/xdzip/issues/53); los candidatos posteriores requieren revisión separada, sin exigir una identidad de firma de pago. La aceptación existente de escritorio permanece en [#5](https://github.com/xiaodaozhi/xdzip/issues/5).

La [verificación de correspondencia del SDK Qt](qt-sdk-correspondence.md) registra además commits de archivos fijados, hashes por archivo en declaraciones fuente del SDK, configuración instalada y el parche conocido del consumidor AGL. Las coincidencias dentro del ámbito declarado no demuestran equivalencia completa de compilación del SDK; siguen vigentes los bloqueos de publicación existentes.

El cierre automatizado añade mapas reales del enlazador y miembros de bibliotecas estáticas, hashes de recursos de ejecución, versiones fiables de dependencias nativas y análisis de diferencias binarias exactamente reversible; véase el [contrato completo](../../../developers/engineering/candidate-verification.md). Marcar una transformación como explicada solo cuando los hashes de los bytes reconstruidos coincidan exactamente con las declaraciones upstream. No eliminar las diferencias originales del SDK ni adivinar el historial no documentado del editor.
