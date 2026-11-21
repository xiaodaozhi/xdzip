<a name="xdzip-developer-preparation-guide"></a>

# Preparación del entorno de desarrollo

[English](../../engineering/development.md) | [简体中文](../../zh-CN/engineering/development.md) | [繁體中文](../../zh-TW/engineering/development.md) | [日本語](../../ja-JP/engineering/development.md) | [한국어](../../ko-KR/engineering/development.md) | [Français](../../fr-FR/engineering/development.md) | [Deutsch](../../de-DE/engineering/development.md) | Español | [Português (Brasil)](../../pt-BR/engineering/development.md) | [Русский](../../ru-RU/engineering/development.md)

Usa [Compilar y contribuir](../contributing/README.md) para elegir la compilación y las comprobaciones adecuadas. La [arquitectura](../design/system-architecture.md) y la [estructura del repositorio](repository-layout.md) explican los límites de Qt, el entorno de ejecución en Rust y los códecs.

<a name="xdzip-obtain-and-modify"></a>

## Obtener y modificar el código

Parte de `main` actualizado y conserva otros cambios del directorio de trabajo. Sigue la [política de contribución](../../../CONTRIBUTING.md) para el alcance y la revisión, y la [guía de entrega](../../../developers/engineering/repository-hygiene.md) para la integración autorizada. Cuando cambie un contrato, actualiza sus versiones lingüísticas y las pruebas afectadas. Los colores y las dimensiones provienen de `specs/visual`.

<a name="xdzip-documentation-builds"></a>

## Generar y comprobar la documentación

Mantén un README raíz y páginas de docs para cada idioma de interfaz compatible. El inglés predeterminado usa `README.md` y `docs/`. Las traducciones usan `README.<locale>.md` y `docs/<locale>/`. El manual de mantenimiento usa inglés de EE. UU. y chino simplificado. Coloca la selección de idioma debajo del título. Muestra el idioma actual como texto y enlaza las demás versiones del mismo tema. Desde una página en chino simplificado, enlaza el manual correspondiente en chino simplificado. Desde las demás, enlaza la página inglesa predeterminada. Al añadir o mover documentos, actualiza índices, enlaces entrantes y anclas.

Las herramientas de documentación necesitan Python 3.10+, lxml, Pandoc y Graphviz. Se recomiendan fuentes CJK del sistema. Ejecuta `python3 scripts/build-design-html.py` para generar 16 documentos detallados de diseño y una edición de un solo archivo bajo build/docs-html, a partir del chino de `docs/zh-CN/design/`. El paquete descargable se escribe bajo dist/. El resto de las guías técnicas y de colaboración se encuentra en docs/README.md. Esa salida HTML es documentación. La interfaz del producto usa Qt Quick.

Ejecuta `python3 scripts/check-docs.py` para comprobar cobertura de traducciones, enlaces recíprocos entre idiomas, idioma de destino de enlaces, anclas Markdown, JSON y recursos de diseño. La comprobación funciona sin conexión. El generador HTML comprueba la estructura y los enlaces de las páginas que produce. Si modificas el comprobador, ejecuta `python3 scripts/test-check-docs.py`.

<a name="xdzip-product-toolchain-preparation"></a>

## Preparar las herramientas del producto

macOS requiere Xcode Command Line Tools y Qt. Windows necesita MSVC Build Tools, Windows SDK y Qt compatible. Linux requiere un compilador, el entorno glibc de destino y las dependencias Qt Wayland/X11. Los archivos de herramientas de la raíz y la CI fijan las versiones. Registra orígenes y licencias en la [tabla de herramientas](toolchain.md). La [política de soporte](support-policy.md) recoge sistemas y mantenimiento. Usa la incidencia de revisión del lanzamiento para comprobar licencias o procedencia concretas.

La [aplicación Qt de desarrollo](../../../apps/desktop/README.md) documenta los comandos para compilar y ejecutar desde una copia limpia, opciones de CMake, arquitectura y destinos de macOS y límites de las pruebas sin pantalla. Vuelve a comprobar rutas con espacios o caracteres chinos después de cambiar la compilación o instalación. El desarrollo local no requiere una identidad de firma de pago. La [guía de compilación](../engineering/developer-build.md) explica la firma ad hoc de integridad para el DMG local y el límite de CI que excluye el despliegue de la aplicación Mac.

<a name="xdzip-independent-engineering-probes"></a>

## Pruebas técnicas independientes

La [prueba de herramientas](../../../tools/toolchain-probe/README.md) ofrece una compilación mínima y comandos CTest para una ventana Qt Quick independiente. Consulta su README para conocer las versiones y el alcance. Los resultados están en la [incidencia de validación inicial](https://github.com/xiaodaozhi/xdzip/issues/1).

La [prueba unificada del puente CMake/Cargo](../../../tools/bridge-probe/README.md) ofrece una entrada de compilación en la raíz, dependencias fijadas y comprobaciones de ciclo de vida. Consulta la [selección de cxx](cxx-admission.md) para las dependencias.
