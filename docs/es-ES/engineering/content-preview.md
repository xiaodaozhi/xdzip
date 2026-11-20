<a name="xdzip-content-preview"></a>

# Vista previa del contenido

[English](../../engineering/content-preview.md) | [简体中文](../../zh-CN/engineering/content-preview.md) | [繁體中文（台灣）](../../zh-TW/engineering/content-preview.md) | [日本語](../../ja-JP/engineering/content-preview.md) | [한국어](../../ko-KR/engineering/content-preview.md) | [Français](../../fr-FR/engineering/content-preview.md) | [Deutsch](../../de-DE/engineering/content-preview.md) | Español | [Português (Brasil)](../../pt-BR/engineering/content-preview.md) | [Русский](../../ru-RU/engineering/content-preview.md)

Cuando la tarjeta de propiedades está visible, seleccionar un archivo compatible inicia una vista previa una vez estabilizada la selección. El icono de actualización aparece arriba a la derecha cuando el puntero está sobre la vista previa o el botón recibe foco de teclado; vuelve a cargar el archivo seleccionado. La tarjeta informa de errores de lectura o decodificación y muestra un mensaje para selecciones múltiples. La ventana principal y el visor independiente de propiedades comparten estos comportamientos y la ventana de contraseña existente. Los metadatos siguen disponibles por separado.

<a name="xdzip-capabilities-and-budgets"></a>

## Capacidades y presupuestos

Solo se admiten archivos regulares legibles, de tamaño conocido y sin nombres peligrosos. La lista de extensiones de texto permitidas es txt, md, log, json, csv, tsv, toml, yaml, yml, ini, xml, rs, c, cpp, h y py. Solo se acepta UTF-8 estricto o UTF-16 con BOM. La entrada y el UTF-8 convertido se limitan cada uno a 256 KiB; se prohíben bytes de control binarios como NUL. El contenido se muestra en un control PlainText; etiquetas, URL, texto de programas y macros no se ejecutan.

| Tipo de imagen | Alcance y semántica real en las tres plataformas |
| --- | --- |
| PNG, JPEG (jpg, jpeg, jfif, jpe) | Leer completamente una imagen estática y aplicar metadatos de transformación disponibles para la orientación |
| GIF, WebP | Imagen estática o primer fotograma de animación; las animaciones no se reproducen |
| BMP, ICO/CUR | Imagen ordinaria o predeterminada por el decodificador; CUR no activa un cursor del sistema |
| ICNS | Imágenes PNG o JPEG2000 incrustadas compatibles con un decodificador de sistema disponible, hasta 256 bloques; se rechazan explícitamente tipos incrustados desconocidos e iconos antiguos de mapa de bits sin procesar o RLE |
| TGA | Imágenes de color real sin comprimir de 16/24/32 bits, sin mapa de color ni ID de imagen y con origen a la izquierda; completar un pie antiguo ausente para la misma disposición; sin RLE ni color indexado |
| TIFF (tif, tiff) | Solo primera página; no se recorren páginas automáticamente |
| SVG | Vector estático UTF-8 rasterizado tras análisis restringido; ver subconjunto abajo |
| HEIC/HEIF | Solo macOS, mediante el plugin oficial de Qt con ImageIO del sistema; rechazado explícitamente en otras plataformas |

No se añaden AVIF, RAW, PSD, EXR, PDF, Office ni bibliotecas independientes de decodificadores de terceros. La extensión, comprobaciones Rust de firmas de formato y decodificadores Qt fijados determinan conjuntamente la admisión. Los plugins adicionales instalados en el sistema no se convierten en capacidades del producto ni se prueban automáticamente otros decodificadores.

El contenido de imagen codificado se limita a 16 MiB. Las colas de eventos Rust y el buzón Qt tienen cada uno límite de 32 MiB; el presupuesto de comandos sigue siendo 8 MiB. Ancho y alto originales son cada uno de hasta 16 384, con un máximo total de 33 554 432 píxeles. La admisión del origen y la miniatura de salida se contabilizan por separado: ancho y alto finales de hasta 512 cada uno, proporción conservada y RGBA8 de hasta 1 MiB. Fotos como imágenes de 12 MP pueden leerse como miniaturas dentro de los presupuestos. Algunos decodificadores leen el original antes de reducirlo, por lo que el tamaño de la miniatura no demuestra que internamente se use solo 1 MiB. Los perfiles de color disponibles se convierten a sRGB; los desconocidos conservan el resultado del decodificador. No se promete prueba profesional de color.

El proveedor asíncrono de imágenes Qt recibe solo tokens aleatorios, no rutas del sistema ni URL de red. Se conservan globalmente como máximo 32 MiB de contenido codificado y se ejecutan hasta dos tareas de decodificación en paralelo. Cada tarea cancelada aún en recuperación puede conservar como máximo una entrada de 16 MiB. Cada tarea inicia el proceso privado adyacente fijado `xdzip-image-preview`, sin pasar rutas del archivo comprimido, contraseñas, identificadores de destino ni credenciales de la aplicación. La salida solo permite una cabecera de tamaño fijo y RGBA8 cuyo ancho, alto y longitud coincidan estrictamente, sin salida adicional. Fallos, caídas, fotogramas inválidos, tiempos agotados y resultados tardíos no pueden mostrarse.

El anfitrión impone un plazo total de cinco segundos desde el inicio y comprueba la validez de la petición cada 50 ms. Cerrar, cancelar o cambiar de entrada termina y recoge el proceso. El proceso limita las asignaciones QImage a 128 MiB. Windows usa un Job con 512 MiB de memoria comprometida del proceso y tres segundos de CPU de usuario. Unix usa tres segundos de CPU, desactiva volcados de memoria y limita descriptores de archivo. Linux limita el espacio de direcciones a 512 MiB y el tamaño de archivo regular a cero. macOS añade 512 MiB de espacio de direcciones a las reservas virtuales dyld existentes; no es un límite estricto de RSS. ImageIO del sistema puede usar recursos de ejecución respaldados por archivos, por lo que cada archivo regular se limita a 16 MiB y los excesos siguen terminando la decodificación. La aplicación no exporta copias de entradas del archivo comprimido; los recursos del sistema y la salida de entradas se gestionan por separado. Esto aísla fallos y recursos de un decodificador privado, no es una caja de permisos del núcleo para programas arbitrarios de terceros.

Cada lectura de contenido del archivo comprimido se limita acumulativamente a 256 MiB de lecturas de origen, 64 MiB de trabajo de decodificación y omisión y 256 MiB de trabajo algorítmico auxiliar. Los diccionarios, procesos y grafos de métodos siguen sujetos a presupuestos separados del [contrato del motor de formatos](native-codecs.md). Un archivo sólido o flujo comprimido secuencial puede rechazar la vista previa de un archivo pequeño si la decodificación anterior supera el presupuesto de trabajo.

<a name="xdzip-static-svg-subset"></a>

## Subconjunto SVG estático

SVG usa la misma ruta de representación Qt Svg en las tres plataformas de escritorio. Antes de representar, un analizador XML en flujo reconstruye gráficos estáticos, texto, estilos, definiciones internas, degradados, recortes, máscaras y filtros restringidos admitidos. Los límites son 8192 elementos, 64 niveles, 64 atributos por elemento, 1 MiB por atributo y 16 MiB de salida total. Los metadatos no participan en la representación. Los elementos de estilo solo aceptan texto; combinar segmentos XML y CDATA antes de comprobar todo el CSS para que la segmentación no eluda restricciones de referencias externas.

Rechazar DTD, entidades, instrucciones de procesamiento, scripts, eventos, animación, foreignObject, elementos image, archivos de fuentes, estilos externos y referencias a recursos externos. href y CSS url solo aceptan identidades de fragmento simples del documento actual. xml:base, src, importaciones CSS y formas escapadas están prohibidas. Los mapas de bits incrustados y SVGZ quedan fuera del subconjunto. Los elementos de dibujo desconocidos o contenidos prohibidos hacen fallar toda la vista previa en vez de descartarse silenciosamente y presentarse como completos. Esto admite SVG estático habitual sin afirmar todas las capacidades SVG o del navegador.

<a name="xdzip-integrity-passwords-and-lifecycle"></a>

## Integridad, contraseñas y ciclo de vida

La vista previa reutiliza identidad física congelada, versión de origen, ubicación de datos y decodificadores existentes. El contenido debe decodificarse por completo hasta EOF y superar las comprobaciones de integridad y autenticación disponibles del formato antes de que texto o imágenes codificadas lleguen a la vista previa. Los formatos como TAR sin sumas de contenido por archivo conservan solo sus garantías existentes de estructura y legibilidad. Los índices grandes en disco usan lecturas acotadas de datos seleccionados sin cargar todos los metadatos en Qt.

Las contraseñas se vinculan solo a la petición actual y usan el contenedor existente que se borra con ceros; errores, ambigüedad de autenticación y cancelación siguen las categorías existentes, con un máximo de diez intentos. Cerrar la ventana de contraseña solo cancela la vista previa y conserva el directorio utilizable. Abrir, navegar y otros comandos de lectura en primer plano tienen prioridad sobre una lectura de vista previa pendiente. Cerrar la vista previa, cambiar de entrada, ocultar una página del archivo comprimido o cerrar la sesión borra el contenido; peticiones invalidadas y resultados de decodificación no pueden restaurar contenido antiguo. Fallo y cancelación descartan contenido parcial no verificado, no escriben copia de la entrada ni inician una aplicación del sistema. La apertura temporal por doble clic sigue su [flujo separado](archive-opening.md).

<a name="xdzip-toolchain-and-acceptance"></a>

## Herramientas y aceptación

`desktop.py prepare --install-qt` añade qtimageformats de Qt 6.8.2 fijado a SDK nuevos o existentes; doctor comprueba Svg y los plugins necesarios antes de compilar. Las fuentes oficiales, hashes, avisos y correspondencia del SDK están en la configuración compartida. libwebp/libtiff incluidos por Qt y bibliotecas de formato añadidas por separado se gestionan independientemente. `check-image-preview.py` verifica proceso real, plugins y decodificación mínima en pruebas predeterminadas, verificación de despliegue Windows/Linux y empaquetado Mac autorizado. Mac incluye explícitamente offscreen y elimina las anulaciones de búsqueda del SDK antes de la autocomprobación; los directorios de plugins del SDK de desarrollo no sustituyen las pruebas del paquete de ejecución real.

Las regresiones usan imágenes propias que cubren cada tipo, primeros fotogramas de animación, primeras páginas TIFF, transparencia, fotos de 12 MP, HEIC en macOS y rechazo en otras plataformas, definiciones internas SVG y rechazo de recursos externos, entrada dañada, exceso de píxeles, fotogramas inválidos, tiempos agotados y cancelación. Las rutas reales de archivos comprimidos siguen cubriendo contraseñas, integridad, identidad del origen y limpieza tras invalidación de interfaz. La disposición sigue la [norma visual](../visual/design-standard.md#xdzip-content-preview); reutilizar la evidencia de disposición sin cambios según la [verificación](../../../developers/qa/README.md) y observar la visualización real de las imágenes nuevas.
