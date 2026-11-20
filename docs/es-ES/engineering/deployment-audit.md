<a name="xdzip-deployment-dependency-verification-and-sbom"></a>

# Verificación de dependencias desplegadas y SBOM

[English](../../engineering/deployment-audit.md) | [简体中文](../../zh-CN/engineering/deployment-audit.md) | [繁體中文](../../zh-TW/engineering/deployment-audit.md) | [日本語](../../ja-JP/engineering/deployment-audit.md) | [한국어](../../ko-KR/engineering/deployment-audit.md) | [Français](../../fr-FR/engineering/deployment-audit.md) | [Deutsch](../../de-DE/engineering/deployment-audit.md) | Español | [Português (Brasil)](../../pt-BR/engineering/deployment-audit.md) | [Русский](../../ru-RU/engineering/deployment-audit.md)

La verificación del despliegue genera por defecto materiales de revisión de desarrollo marcados expresamente como `incomplete`; no demuestra automáticamente preparación para publicar. El Release oficial recopila por separado materiales del mismo candidato y los entrega con el paquete tras revisar el proceso de publicación. El formato usa el [esquema oficial CycloneDX 1.6](https://github.com/CycloneDX/specification/blob/55343ba19dee1785acf1ce9191540d5fd7b590db/schema/bom-1.6.schema.json). El commit original y el SHA-256 de tres archivos de esquema están fijados en la [configuración](../../../scripts/sbom-schemas.json). La validación solo descarga y analiza esos recursos, almacenándolos en build/; no descarga esquemas de URL arbitrarias incluidas en un SBOM.

| Artefacto | Fuente de datos y finalidad | Lo que no puede demostrar |
| --- | --- | --- |
| source-build.cdx.json | Grafo de dependencias del espacio de trabajo resuelto por Cargo para el anfitrión, incluidas herramientas de desarrollo y compilación; módulos Qt declarados; hashes de fuentes tipográficas y de marca | Que un paquete Rust esté necesariamente enlazado estáticamente al producto o que todos los plugins Qt y terceros integrados estén identificados |
| deployment-audit.json | SHA-256 de archivos bajo la raíz real indicada, importaciones binarias nativas, dependencias SDK, de sistema o sin resolver e identidad real de compilación CLI | Todas las cargas dinámicas de QML o plugins por nombre, instalación limpia, ABI del sistema, firmas o cumplimiento de licencias |
| artifact-files.cdx.json | Artefactos reales y archivos SDK externos observados, relaciones conocidas de importaciones nativas, vinculados al inventario fuente y a la versión, commit y estado dirty del ejecutable real | Identidad completa de componentes desplegados, atribución de enlace estático, licencias de bibliotecas integradas en Qt o un SBOM formal de producto terminado |
| Registros de escritorio opcionales | Comprobaciones de escritorio ejecutadas y limitaciones según el cambio y el entorno disponible | Acciones no realizadas, otras plataformas o aprobación de publicación |

Los registros de fuentes no incluyen ID opacos de Cargo basados en rutas ni inventan purl de crates.io para paquetes del espacio de trabajo. Las rutas de artefactos son relativas a la raíz indicada o al SDK Qt. Otras dependencias externas se identifican por hash de contenido y nombre, sin rutas absolutas privadas del SDK. Las fuentes tipográficas usan OFL-1.1; los originales de marca conservan Apache-2.0 de la biblioteca de recursos. Declarar módulos Qt no permite inferir las licencias de todos los componentes realmente desplegados.

<a name="xdzip-automated-verification"></a>

## Verificación automatizada

`deployment_audit.py` comprueba solo compilaciones confiables del proyecto indicadas explícitamente, no archivos comprimidos de usuarios. macOS usa otool para importaciones y RPATH; Windows usa dumpbin de MSVC para importaciones DLL ordinarias y diferidas; Linux usa ldd, que no debe usarse con ejecutables de origen desconocido. Un enlace simbólico de archivo que salga de la raíz de artefacto indicada hace fallar la verificación. Los informes deben escribirse fuera de la raíz para no incluir su propia salida en el inventario.

- `--mode sdk` permite resolver hacia un SDK Qt externo y muestra expresamente que la compilación no es independiente. Las importaciones sin resolver siguen fallando. La recursión cubre el artefacto y el SDK Qt declarado; otras bibliotecas externas solo se registran por hash y recuento no recorrido, sin considerarlas verificadas como cierre completo.
- `--mode deployed` prohíbe dependencias externas no pertenecientes al sistema e importaciones sin resolver. En Windows las importaciones VC++ como MSVCP y VCRUNTIME deben venir del paquete; el System32 de la máquina de desarrollo no puede ocultar archivos ausentes. UCRT y contratos API del sistema siguen siendo proporcionados por el sistema. Continúa con pruebas nativas básicas de arranque tras limpiar las rutas de búsqueda de desarrollo Qt.
- Las importaciones sin resolver, diferencias de versión, commit o estado dirty entre fuente y binario real, errores de hash o formato del esquema oficial y referencias de componentes duplicadas o colgantes bloquean la comprobación correspondiente.
- El cierre de importaciones nativas no sustituye la cobertura de plugins cargados por nombre ni las comprobaciones en una máquina limpia. Aprobar importaciones de un directorio Windows desplegado no equivale a aceptar el instalador.

El flujo Qt bajo demanda para tres plataformas conserva dos archivos cdx.json en la raíz y deployment-audit.json sin generar automáticamente formularios manuales vacíos. macOS lee solo el .app compilado localmente y el SDK; las comprobaciones ordinarias no ejecutan macdeployqt ni firman, comprimen o suben paquetes de aplicación. Linux proporciona Qt/QML/plugins mediante un [directorio de ejecución independiente](linux-deployment.md), con verificación deployed y un contenedor sin SDK; el tar.gz descargado conserva los bits de ejecución. Actualmente solo se declara la base Ubuntu 24.04, x86_64, X11, no un instalador universal de Linux.

Para artefactos Windows confiables del proyecto, extrae el instalador por separado y compara su directorio de aplicación con el ZIP portátil del mismo candidato mediante `python scripts/check-windows-package-payload.py --installer-app <extracted-app-directory> --portable <portable-package.zip> --output build/review/payload-comparison.json`. La comprobación compara hashes por archivo, manifiesto de instalación y DLL gráficas del SDK que no deben incluirse. La ruta de salida debe ser nueva. Esta comparación estática no ejecuta el instalador ni una actualización y no sustituye la verificación real de instalación.

Ejemplo local de macOS tras recompilar desde fuentes coherentes:

```sh
python3 scripts/collect-notices.py build/review/wp12/notices
python3 scripts/deployment_audit.py \
  --root build/ui/apps/desktop/xdzip.app \
  --app build/ui/apps/desktop/xdzip.app/Contents/MacOS/xdzip \
  --qt-prefix build/toolchain/Qt/6.8.2/macos --mode sdk \
  --output build/review/wp12/deployment-audit.json
python3 scripts/sbom.py \
  --audit build/review/wp12/deployment-audit.json \
  --source build/review/wp12/notices/source-build.cdx.json \
  --output build/review/wp12/artifact-files.cdx.json
```

Después de instalar jsonschema==4.23.0 en un entorno de verificación aislado, usa el Python de ese entorno para ejecutar `scripts/validate-sbom.py` con los dos cdx.json anteriores. Consulta la [documentación de jsonschema 4.23](https://python-jsonschema.readthedocs.io/en/v4.23.0/referencing/) para el uso del registro. Las herramientas de verificación no se despliegan en el producto.

<a name="xdzip-attribution-results-and-verification-limits"></a>

## Resultados de atribución y límites de verificación

La [atribución de componentes reales](runtime-attribution.md) está integrada: el SPDX del SDK relaciona archivos reales, módulos Qt y terceros integrados; se entregan textos oficiales de licencia con hash fijado; se registran diferencias de hash SDK/original y atribuciones no confirmadas. Los candidatos Rust normales de ejecución se separan de herramientas de compilación, desarrollo y proc-macro. Las nuevas relaciones de componentes se vinculan a la auditoría real y se incorporan al SBOM del artefacto, que sigue marcado expresamente como incompleto.

En el árbol instalado, relaciona cada DLL, framework y plugin entregado con su componente original, versión, licencia y código correspondiente. Revisa por separado los componentes integrados de Qt, la atribución del enlace Rust estático y la procedencia de la cadena de herramientas. Usa la [guía de revisión de publicación](../../../developers/qa/release-review.md) para decisiones sobre candidatos.
