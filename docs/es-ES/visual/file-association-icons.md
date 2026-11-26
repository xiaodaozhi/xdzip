<a name="xdzip-file-association-icons"></a>

# Iconos de asociación de archivos

[English](../../visual/file-association-icons.md) | [简体中文](../../zh-CN/visual/file-association-icons.md) | [繁體中文](../../zh-TW/visual/file-association-icons.md) | [日本語](../../ja-JP/visual/file-association-icons.md) | [한국어](../../ko-KR/visual/file-association-icons.md) | [Français](../../fr-FR/visual/file-association-icons.md) | [Deutsch](../../de-DE/visual/file-association-icons.md) | Español | [Português (Brasil)](../../pt-BR/visual/file-association-icons.md) | [Русский](../../ru-RU/visual/file-association-icons.md)

Los iconos de archivo del sistema reutilizan el documento con esquina doblada, la insignia de curvatura continua, las cuatro bandas entrelazadas y la marca abierta en U de xdzip. Los parámetros provienen de [file-association-icons.json](../../../specs/visual/file-association-icons.json) y las fuentes de iconos y contenedores de plataforma están en [file-associations](../../../design/assets/file-associations). Los usuarios siguen eligiendo sus aplicaciones predeterminadas.

<a name="xdzip-shape-and-identification"></a>

## Forma e identificación

La geometría de papel y marca reutiliza [brand_assets.py](../../../scripts/brand_assets.py). Los formatos comparten papel, pliegue, posición de insignia y centro óptico; las etiquetas identifican formatos y los colores ayudan a distinguirlos. TAR.GZ, TGZ y GZ forman una familia verde azulado. La alternativa genérica usa gris azulado neutro y líneas de documento sin etiqueta de formato.

Las etiquetas siguen estrictamente los glifos ZIP originales: altura 61, trazo 14, extremos cuadrados, uniones redondeadas, borde superior en Y=817 del original y color #39465D. ZIP reutiliza directamente el trazado completo del texto original; las demás letras siguen las mismas medidas y reglas de trazo. No se usan texto SVG, fuentes del sistema, iconos de terceros ni mapas de bits incrustados. El ancho máximo es 504. Conservar la etiqueta TAR.GZ completa sin estirar glifos; solo reducir uniformemente y centrar verticalmente cuando supere el ancho disponible. El color no es el único medio de identificación.

El original tiene 1024 unidades. PNG/ICO genéricos conservan el viewBox `32 32 960 960` con cuerpo de papel de aproximadamente 90% de altura; PNG/ICNS de macOS conservan `-20 -20 1064 1064` con cuerpo de aproximadamente 81,2%. Esto continúa la calibración óptica del proyecto y no se presenta como requisito del sistema. Los iconos usan lienzos transparentes sin papel específico del tema, fondo de escritorio ni máscaras redondeadas adicionales.

A 16 y 24 píxeles se amplía la insignia y se simplifica el papel; a 32 píxeles la insignia es ligeramente menor. Estos tres tamaños omiten etiquetas y sombras de imágenes grandes conservando la proporción original de la marca. Los iconos solos no distinguen de forma fiable todos los formatos en esos tamaños, especialmente la familia GZIP del mismo color; las columnas de nombre y tipo permiten identificarlos con precisión. Se usan originales etiquetados desde 48 píxeles; las etiquetas largas son más legibles desde 64 píxeles. Renderizar cada tamaño directamente desde vectores en vez de reducir repetidamente un PNG de 1024.

Los iconos de 24 unidades de la interfaz y la densidad de ventana principal y menús siguen [layout-scale.json](../../../specs/visual/layout-scale.json). Estos recursos no añaden ventanas QML ni cambian dimensiones lógicas de ventana principal o menús. Las dimensiones PNG de plataforma son dimensiones de muestreo y no deben trasladarse a la geometría lógica de componentes QML.

<a name="xdzip-format-and-platform-mapping"></a>

## Correspondencia de formatos y plataformas

| ID de icono | Etiqueta | Extensión / uso | Color |
| --- | --- | --- | --- |
| `zip` | ZIP | `.zip` | Azul de marca |
| `7z` | 7Z | `.7z` | Índigo |
| `rar` | RAR | `.rar` | Morado |
| `tar` | TAR | `.tar` | Ámbar |
| `tar-gz` | TAR.GZ | Sufijo compuesto `.tar.gz` confirmado | Verde azulado |
| `tgz` | TGZ | `.tgz` | Verde azulado |
| `gz` | GZ | Candidato del sistema `.gz` | Verde azulado |
| `cab` | CAB | `.cab` | Azul acero |
| `iso` | ISO | `.iso` | Azul lago |
| `udf` | UDF | `.udf` | Azul cian |
| `archive` | Ninguna | Alternativa para archivos reconocidos sin icono específico | Gris azulado neutro |

La tabla expresa intención de diseño, no un registro uniforme entre plataformas. Cuando es posible se prioriza el sufijo completo `.tar.gz` antes de `.gz`; los sistemas que solo reconocen la última extensión pueden mostrar GZ para ambos. No prometer que todas las plataformas distinguen automáticamente sufijos compuestos. El icono GZ indica una extensión compartida por GZIP independiente y la capa externa TAR.GZ; la lectura real sigue la validación del contenido y los contratos de formato.

El cifrado y la división en volúmenes son estados del archivo comprimido, no se deducen del icono de extensión; no dibujar candados ni marcas de éxito. Los volúmenes ZIP, 7z y RAR reconocidos pueden reutilizar su familia dentro de la aplicación sin registrar `.001`, `.rNN`, `.zNN` o `.img` genéricos. Los iconos ISO/UDF no implican montaje ni ejecución. Una extensión coincidente no demuestra por sí sola contenido válido.

El instalador ordinario de Windows usa ProgID e ICO por extensión. MSIX declara candidatos de «Abrir con» con PNG derivados de los fotogramas ICO correspondientes. Ambos siguen la asignación de asociaciones generada, que puede reutilizar imágenes entre sufijos. macOS usa ICNS por tipo de documento, incluidos TAR comprimido y GZIP independiente. Linux usa iconos MIME del sistema. La aplicación predeterminada y las cachés del sistema afectan a la visualización real. Consulta [instalación y asociaciones](../engineering/installers-associations.md).

MSIX declara un `uap:Logo` independiente para cada asociación de formato. Sus PNG de tamaño objetivo reutilizan byte a byte los fotogramas ICO canónicos de Windows, conservando simplificación de tamaños pequeños, etiquetas grandes, límites visibles y centrado. Los logotipos de aplicación son recursos separados con formas predeterminada y sin placa para ambos temas. [msix-icons.json](../../../specs/visual/msix-icons.json) define tamaños y variantes del paquete; MakePRI los indexa antes del empaquetado MakeAppx. La imagen prevista del documento no incluye placa del sistema ni alternativa con logotipo de aplicación.

La entrada de aplicación en aplicaciones predeterminadas y «Abrir con» de Windows usa el [ICO de marca](../../../design/assets/brand/xdzip.ico), incorporado en el ejecutable principal y el lanzador de asociaciones. Los iconos de documento por formato no sustituyen la identidad de la aplicación. El sistema debe mostrar una única entrada xdzip opcional con el icono de marca; el antiguo comando compartido mantiene compatibilidad con elecciones existentes. Esto reutiliza recursos existentes sin cambiar geometría lógica de ventana principal, menús o QML. Los iconos nativos de aplicación, de documento y el número de candidatos se aceptan por separado; un cuadro genérico de aplicación del sistema no debe clasificarse como problema del icono de documento.

<a name="xdzip-generation-and-delivery"></a>

## Generación y entrega

Ejecutar desde la raíz del repositorio:

```sh
python scripts/build-association-icons.py
python scripts/build-design-assets.py --metadata-only
python scripts/build-association-icons.py --check --check-runtime
# Exportar artefactos locales con Pillow, Playwright y Chromium existentes.
python scripts/build-association-icons.py --export build/file-association-icons
python scripts/build-association-icons.py --check --export build/file-association-icons
# Tras copiar las exportaciones ICO/ICNS deseadas a la carpeta canónica:
python scripts/build-design-assets.py --metadata-only --refresh-association-icons
```

`XDZIP_CHROMIUM` puede indicar una ruta existente de Chromium/Edge. Los originales SVG, fuentes pequeñas, y especificación están versionados; las fuentes están registradas en el [manifest.json](../../../design/assets/manifest.json) principal. La entrada de generación es [build-association-icons.py](../../../scripts/build-association-icons.py), con composiciones en [association_icon_assets.py](../../../scripts/association_icon_assets.py).

El registro asigna una imagen a cada formato y extensión admitida. Los alias pueden compartir un icono con etiqueta. La superficie del papel es lisa debajo de la etiqueta. La exportación admite PNG de 16 a 1024 píxeles, ICO e ICNS. La compilación lee directamente los contenedores de plataforma versionados. Linux usa los iconos MIME del sistema.

<a name="xdzip-verification-and-acceptance"></a>

## Verificación y aceptación

Las comprobaciones de generación verifican que los SVG correspondan a la especificación y no dependan de fuentes o imágenes externas. Las comprobaciones de plataforma cubren lienzo y RGBA, esquinas transparentes, altura visible y centrado, resúmenes vectoriales y concordancia de píxeles decodificados en cada representación ICO/ICNS. Los límites del cuerpo se miden con alfa ≥ 128, usando la tolerancia de marca de 1,25 píxeles físicos.

La revisión visual cubre fondos claros, oscuros y seleccionados, tamaños de 16 a 256 píxeles y representación real con DPR normal y alto. La aceptación nativa requiere verificación en la plataforma real. La integración posterior debe comprobar nombres reales, sufijos compuestos, cachés de iconos, antes y después de elegir el predeterminado y visualización tras instalar o desinstalar. Registrar estado y alcance real en incidencias de GitHub en vez de acumular informes aquí.
