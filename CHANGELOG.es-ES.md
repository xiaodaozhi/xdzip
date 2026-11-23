# Registro de cambios

[English](CHANGELOG.md) | [简体中文](CHANGELOG.zh-CN.md) | [繁體中文](CHANGELOG.zh-TW.md) | [日本語](CHANGELOG.ja-JP.md) | [한국어](CHANGELOG.ko-KR.md) | [Français](CHANGELOG.fr-FR.md) | [Deutsch](CHANGELOG.de-DE.md) | Español | [Português (Brasil)](CHANGELOG.pt-BR.md) | [Русский](CHANGELOG.ru-RU.md)

<a id="unreleased"></a>

## Sin publicar

- Añadir un SDK experimental independiente de solo lectura con procesamiento de CBOR acotado, E/S controlada, dos implementaciones independientes del protocolo y una comprobación del aislamiento que rechaza las ejecuciones inseguras. Las extensiones de terceros siguen desactivadas en el producto.

## 0.3.1

- Colocar el chino simplificado y el chino tradicional al principio del selector manual de idioma, con las etiquetas «中文 (简体)» y «中文 (繁体)».
- Mejorar las categorías de formatos y la paginación de la página inicial, y corregir la disposición de la barra de herramientas y las ventanas de operación.
- Corregir el aviso prematuro de finalización de consultas paginadas y conservar la información de origen de los archivos extraídos en Windows.
- Actualizar las dependencias de vista previa de imágenes.

Consulta las [notas de la versión](release-notes/0.3.1.md) y la [versión de GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.3.1) para conocer las plataformas, las comprobaciones y las limitaciones.

## 0.3.0

- Admitir diez idiomas en la interfaz de escritorio, los menús del sistema y la documentación de usuario. El idioma del sistema se puede seleccionar automáticamente o cambiar de forma manual.
- Ampliar la edición a otros formatos de archivo comprimido. Se pueden añadir, sustituir, mover, renombrar y eliminar las entradas aptas.
- Cargar las vistas previas automáticamente y permitir actualizarlas cuando se solicite. Mejorar las acciones sobre archivos comprimidos, las asociaciones de archivos de macOS y la entrega de ventanas y archivos en Ubuntu.

Consulta las [notas de la versión](release-notes/0.3.0.md) y la [publicación de GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.3.0) para ver las plataformas compatibles, el alcance de la verificación y las limitaciones conocidas.

## 0.2.1

- Ampliar la exploración y extracción de formatos como LHA/LZH, ARJ, ARC, DEB, ASAR, Zstandard y LZ4. Añadir la creación de LZH, imágenes de datos ISO9660/Joliet y ZIP autoextraíbles para Windows x64.
- Ampliar la vista previa a imágenes de mapa de bits habituales y un perfil restringido de SVG estático. Usar la compatibilidad del sistema con HEIC/HEIF en macOS y generar miniaturas en un proceso privado con recursos limitados. La especificación de vista previa indica los límites de cada formato.
- Indexar hasta un millón de registros de un ZIP/ZIP64 ordinario de un solo archivo mediante un índice en disco construido en flujo. Cargar los resultados voluminosos cuando se necesiten y conservar la selección entre páginas.
- Añadir a la ficha de propiedades vistas previas bajo demanda de texto sin formato e imágenes PNG/JPEG, con límites. Compartir el tratamiento de las contraseñas, la integridad y las cancelaciones.

Consulta las [notas de la versión](release-notes/0.2.1.md) y la [publicación de GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.2.1) para ver las plataformas compatibles, el alcance de la verificación.

## 0.2.0

- Admitir actualizaciones nativas de XDZip en instalaciones de macOS, tanto por conexión directa como mediante proxies HTTP CONNECT o SOCKS5 personalizados.
- Proporcionar paquetes CLI independientes para Windows, macOS y Linux, con comandos de gestión de archivos, resultados JSON, registro explícito de comandos y actualizaciones verificadas con manifiestos firmados.
- Aplicar las correcciones oficiales de QtNetwork para CVE-2026-116337 y CVE-2026-105946. Conservar los parches completos, su procedencia y las instrucciones de sustitución.
- Trasladar los metadatos de la CLI incluida en macOS a los recursos ordinarios de la aplicación, cubiertos por la firma de integridad del paquete.

Consulta las [notas de la versión](release-notes/0.2.0.md) y la [publicación de GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.2.0) para ver las plataformas compatibles, el alcance de la verificación.

## 0.1.3

- Evitar que Finder trate como aplicación dañada la carpeta de extracción predeterminada que hereda el nombre de un archivo terminado en `.app`.

Consulta las [notas de la versión](release-notes/0.1.3.md) y la [publicación de GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.1.3) para ver las plataformas compatibles, el alcance de la verificación.

## 0.1.2

- Mantener la lista de archivos al cambiar de directorio, buscar u ordenar para reducir el parpadeo.
- Mejorar la explicación y los pasos de instalación manual cuando la instalación automática no está disponible en macOS.
- Corregir en macOS la extracción de enlaces simbólicos ZIP seguros con rutas relativas al directorio superior y las notificaciones del entorno de ejecución de Qt.

Consulta las [notas de la versión](release-notes/0.1.2.md) y la [publicación de GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.1.2) para ver las plataformas compatibles, el alcance de la verificación.

## 0.1.0

- Publicar la primera versión estable, con instaladores nativos o paquetes portátiles para Windows, macOS y Linux. Las instalaciones aptas pueden actualizarse a petición del usuario.
- Mejorar el desplazamiento en Configuración, la accesibilidad del estado de las actualizaciones y la compatibilidad con el proxy del sistema. Corregir el comportamiento nativo al pasar el puntero.
- Corregir la recopilación de avisos de terceros y normalizar las expresiones de licencia en la SBOM del código fuente y el grafo de dependencias del entorno de ejecución Rust candidato.

Consulta las [notas de la versión](release-notes/0.1.0.md) y la [publicación de GitHub](https://github.com/xiaodaozhi/xdzip/releases/tag/v0.1.0) para ver las plataformas compatibles, las limitaciones conocidas y los materiales de publicación.

## 0.1.0-alpha.9

- Retirar tres DLL del compilador gráfico del SDK de Windows del instalador y del paquete portátil. Las actualizaciones eliminan las antiguas copias distribuidas.
- Mejorar el acceso a Configuración y a las actualizaciones. Las descripciones de tipos de archivo en Windows siguen el idioma de la aplicación.
- Corregir la minimización durante los cambios a pantalla completa en macOS y varias asociaciones de archivos. Mejorar el arranque y la carga de fuentes.
- Corregir la limpieza al cerrar un archivo comprimido, las comprobaciones del destino de extracción y los avisos de las tareas.
- Simplificar la CI habitual y ejecutar las compilaciones completas y las verificaciones de publicación bajo demanda. Ampliar los materiales de licencia del código fuente y de terceros.

Las [notas de la versión](release-notes/0.1.0-alpha.9.md) describen las limitaciones de plataforma y licencia de esta versión preliminar experimental.

## 0.1.0-alpha.8

- Añadir a Acerca de la comprobación de actualizaciones iniciada por el usuario. Las instalaciones aptas se pueden actualizar después de verificar la firma, la versión y la plataforma. La versión pública anterior alpha.6 requiere una actualización manual.
- Usar las fuentes del sistema y sus alternativas para los nombres de archivos comprimidos, los nombres de archivo y las rutas. Los textos fijos de la interfaz conservan la fuente del producto. Mostrar mensajes distintos para carpetas vacías, archivos comprimidos vacíos y búsquedas sin resultados.
- Retirar la biblioteca de OpenGL por software de los paquetes de desarrollo de Windows. Mantener Direct3D 11 como opción predeterminada y WARP disponible para el renderizado por software.
- Corregir el doble recuento de tareas activas cuando una instantánea llega antes de aceptar la tarea. Mantener la aceptación, la cancelación y la limpieza al salir.
- Detectar las codificaciones antiguas habituales de nombres de archivo de forma uniforme al explorar, navegar, extraer, abrir copias temporales, exportar y editar. Priorizar las declaraciones Unicode, conservar la validación de nombres originales y escribir los nombres modificados en UTF-8. Los nombres cortos y las codificaciones mezcladas pueden seguir siendo ambiguos.
- Retirar los antiguos requisitos de publicación y las plantillas de informes. Armonizar las reglas de contribución, el resumen de preparación y los generadores de materiales. Registrar la procedencia y las licencias en la lista de publicación.
- Usar notas revisadas para cada versión y comprobar la versión y el número de compilación. Detener el proceso si el borrador difiere del documento revisado para que una repetición no sobrescriba los límites de aceptación registrados.
- Actualizar la documentación sobre la publicación específica de macOS, la lectura de volúmenes divididos y el registro de cambios.

## 0.1.0-alpha.6

- Limpiar los estados antiguos del puntero al ocultar o minimizar ventanas, incluido el destello rojo del botón de cierre al restaurar desde la bandeja de Windows. Compartir la limpieza del puntero entre las ventanas de exploración y de operaciones.
- Explorar y extraer volúmenes ZIP numerados y PKZIP. Usar la ruta de extracción común para las lecturas entre volúmenes, los desplazamientos ZIP64, las contraseñas y la integridad. Proteger todos los volúmenes frente a cambios en el origen y sobrescrituras.
- Abrir las entradas ordinarias con el sistema desde una copia temporal privada. Ocultar el progreso cuando el sistema acepte la solicitud y conservar los resultados fallidos para reintentar. Las copias siguen una política de conservación de siete días y las ediciones quedan fuera del archivo comprimido.
- Corregir el guardado del modo de apertura. Añadir desplazamiento de pestañas cuando no caben, una lista de pestañas y reordenación mediante arrastre. Mostrar las pestañas inactivas con grosor normal.
- Admitir nombres de archivo largos de GNU en TAR de código fuente de Git. Fijar los errores de la pantalla inicial en su parte superior. Al cerrar una ventana, quitar sus pestañas y conservar las tareas en segundo plano. Actualizar el texto del contador de tareas en la bandeja.

## 0.1.0-alpha.1

- Introducir el epoch 1 de Debian para que el sistema ordene los paquetes alpha como actualizaciones de los paquetes dev anteriores. Conservar la identidad de los archivos de prueba históricos.
- Mantener los estados normal, maximizado y de pantalla completa durante los diálogos, la restauración desde la bandeja y la apertura de archivos externos.
- Compartir un proceso y una bandeja por sesión de usuario. Reenviar solicitudes de activación y apertura por lotes durante el arranque, rechazar las que expiren y recuperar el estado residual tras un fallo.
- Añadir pestañas o ventanas separadas con controladores de archivo y estados de exploración independientes. Conservar las tareas y sus resultados al cerrar la página, dirigirlos por identidad de sesión y de tarea y serializar la admisión de tareas costosas.
- Separar el número de versión preliminar del de compilación. Preparar seis tipos de paquetes multiplataforma, manifiestos inmutables, sumas de verificación, materiales de código fuente y licencias, y procedimientos de publicación. Las actualizaciones automáticas estaban pendientes y las licencias de publicación se seguían por separado.
- Mantener todas las entradas ordinarias seleccionadas explícitamente para la creación, incluido un destino existente o un volumen anterior. Preparar la salida en privado y resolver los conflictos en la escritura final. Conservar el destino si también es una entrada y ofrecer otro nombre o la cancelación.
- Actualizar las entradas del menú de tareas en el mismo sitio según su ID para evitar el parpadeo al cambiar el progreso. Conservar el estado del puntero, el resaltado y el foco de teclado. Cambiar solo las entradas que se añaden al historial o salen de él.
- Reducir las DMG de desarrollo de macOS a los binarios Qt arm64, el estilo Basic de Qt Quick y las dependencias compartidas necesarias. Registrar arquitectura, estilo y tamaño y verificar el paquete final. Conservar la aplicación y el SDK originales y usar una firma de integridad ad hoc local.
- Incluir en Windows solo el estilo fijado de Qt Quick Basic, retirar los complementos de depuración QML y los metadatos de tipos de desarrollo, y conservar el estilo nativo y las alternativas gráficas. Corregir la carga de traducciones del sistema de Qt e incluir el mismo entorno de ejecución VC++ fijado en el paquete portátil y el instalador por usuario. Las auditorías rechazan el uso del entorno de ejecución del anfitrión.
- Unificar el nombre del ejecutable principal como xdzip en Windows, macOS y Linux. Armonizar los lanzadores, recursos, CI, recompilaciones y documentación, y mantener la compatibilidad de las pruebas de actualización con compilaciones anteriores.
- Extraer ZIP AES-256 de un solo volumen con AE-1/AE-2 y Store/Deflate. Solicitar la contraseña dentro de la tarea y permitir reintentos y cancelaciones. Escribir los archivos tras la autenticación. Actualizar el diseño, los textos, la cobertura de protocolo y las pruebas de interoperabilidad.
- Añadir limpieza selectiva de las asociaciones de «Abrir con» de copias de desarrollo de macOS. Eliminar los registros nuevos tras una comprobación o un fallo y conservar la aplicación principal, los archivos y los valores predeterminados del sistema. Añadir pruebas de límites y limpieza tras fallos.
- Cerrar el trabajo de desarrollo y aceptación con la aprobación del propietario. Registrar diez comprobaciones automáticas, ocho manuales y la política de soporte del candidato congelado. Rastrear la procedencia y las licencias en la lista de publicación y las incidencias. La publicación formal seguía pendiente.
- Añadir la barra de menús nativa de macOS y los menús de bandeja en las tres plataformas. Compartir comandos, foco de edición y estado de las tareas. Por defecto la aplicación sigue en la bandeja al cerrar la última ventana, con opción de desactivarlo. Salir y reiniciar conservan la confirmación y la limpieza de tareas. Añadir textos bilingües, iconos pequeños de plantilla, diseños y pruebas de ciclo de vida.
- Unificar la versión en VERSION.json, los metadatos nativos y Acerca de. Añadir textos bilingües y la copia de información de compilación. Mantener las pruebas de rendimiento de todo el proceso y las evidencias de licencia, con comprobaciones separadas de escritorio, equipo limpio y licencias.
- Ajustar las dimensiones de la ventana principal Qt, paneles, listas, textos, iconos, pestañas y vistas previas a las medidas del HTML y el diseño fijados. Compartir orígenes de fuentes y añadir mediciones independientes de CI y capturas de ZIP reales. Permitir cerrar pestañas y seleccionar todo en la página actual. Conservar los estados desactivados y las fechas desconocidas.
- Completar los menús comunes, las ayudas, los estados de botones y la ruta de navegación. Añadir al HTML menús contextuales, desplegables y submenús, pestañas de muestra, cuadrícula, ordenación, selección, entrada táctil y tareas de verificación de demostración. Qt incorpora menús reales de selección, copia, vista y directorios de solo lectura. Verificar 22 interacciones nuevas, 20 flujos existentes y cinco grupos Qt locales. Mantener desactivadas las acciones no disponibles e identificadas las demostraciones.
- Añadir al árbol de directorios en HTML y Qt líneas conectoras, contadores fijos, indicadores de antecesores contraídos y espacios de iconos compartidos. Conservar el diseño principal anterior. Seguir por separado los avisos de límites, los marcadores de carga y la accesibilidad manual.
- Entregar tres mesas de trabajo para el árbol de directorios, 26 recortes de píxeles originales, indicadores claros y oscuros, especificaciones de diseño e interacción y HTML en un único archivo con sus recursos. La integración en HTML y Qt estaba pendiente en ese momento. Conservar diseños y fotografías originales.
- Trasladar los diseños de alta fidelidad a design/hifi con vistas generales de las ventanas principales y operativas, propuestas anteriores y hashes. Armonizar las fuentes de recursos y los enlaces. La variante oscura del diseño principal seguía pendiente.
- Añadir un árbol de directorios acotado con carpetas implícitas, ordenación de todos los resultados y encabezados de columna utilizables con el teclado. Paginar después de ordenar y conservar los ID de registros duplicados. Añadir límites de nodos, profundidad y bytes, mensajes alternativos, pruebas de interacción y 54 comprobaciones visuales.
- Añadir la base del entorno de ejecución con dos manejadores cxx, comandos y eventos acotados, controles independientes de cancelación y cierre, buzón Qt y barreras de cierre. Verificar la CI del producto en tres plataformas, repetir diez ciclos de vida por plataforma y comprobar el arranque y cierre nativos tras el despliegue en macOS y Windows. Proporcionar una compilación de desarrollo Apple Silicon y una guía de verificación puntual.
- Reintentar de forma asíncrona y acotada los conflictos breves en el bloqueo de admisión. Desactivar comandos cuando se pierde la conexión, informar de argumentos inválidos y proteger la destrucción durante el tratamiento de eventos.
- Añadir una comprobación previa de extracción acotada y limitada a metadatos. Expandir directorios y verificar rutas, conflictos y tamaños, y mostrar el resultado en una ventana nativa. La extracción conserva la autoridad sobre la escritura.
- Mejorar el explorador de tres paneles con temas claro y oscuro, detalles, ámbito por directorio, búsqueda y selección múltiple por ID físico. Añadir pruebas de ratón y filtrado.
- Añadir exploración real de ZIP de solo lectura con un adaptador Rust independiente, escaneo acotado, nombres duplicados y chinos, páginas de 128 registros y cancelación y cierre asíncronos. Registrar la incorporación de rawzip y las capturas de Qt.
- Incorporar a las tareas de Qt tarjetas de rutas, métricas en tres columnas, progreso con degradado y pie compartido según el diseño. Mantener la recolocación con texto grande y las etiquetas de demostración. Capturar los estados claro, oscuro y con texto al 200 %.
- Definir invariantes de identidad y paginación del dominio. Añadir vistas previas independientes de ventanas de tareas Qt, ayudas compartidas y parámetros de tema generados. Incluir comprobaciones de diseño, cierre y CI sin pantalla.
- Añadir un diagnóstico independiente del puente Rust/cxx, compilaciones unificadas de CMake y Cargo, dependencias fijadas y CI nativa. Verificar la propiedad de los datos, los errores, UTF-8 y el cierre en Linux.
- Añadir un diagnóstico independiente de la cadena de herramientas Qt Quick y registrar la ejecución sin pantalla en Linux y las compilaciones en rutas con caracteres chinos y espacios. Las comprobaciones en tres plataformas y de cxx seguían pendientes.
- Añadir ocho mesas de trabajo de alta fidelidad y recursos para creación, conflictos, progreso, resultados, configuración y comparaciones de plataformas. Definir flujos para guardar, cancelar y recuperar.
- Entregar 24 recortes de píxeles originales, 27 tipos de iconos vectoriales claros y oscuros y PNG transparentes a varias densidades, con notas de procedencia y precisión.
- Armonizar los iconos de tipo de archivo, la geometría de las listas y la columna de tipos. Definir ventanas independientes, barras de título y cierre en las tres plataformas.
- Añadir un prototipo HTML sin conexión para exploración, selección, extracción, creación, tareas y errores. Vincularlo con los requisitos y la verificación.
- Adoptar Apache-2.0, añadir la licencia completa y actualizar los documentos de contribución y publicación.

<a id="pre-development-baseline-2026-10-11"></a>

### Base anterior al desarrollo (2026-10-11)

- Registrar el diseño de software, la trazabilidad de los requisitos y las decisiones de arquitectura.
- Elegir Qt Quick con una capa fina de C++ y Rust.
- Establecer la referencia visual y los parámetros de tema para las vistas clara, oscura y de extracción.
- Añadir documentación de contribución, gobernanza, seguridad, ingeniería, aceptación, riesgos y paquetes de trabajo.
- El código del producto y las publicaciones de binarios seguían pendientes.
