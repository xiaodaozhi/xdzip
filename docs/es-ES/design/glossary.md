<a name="xdzip-xdzip-terminology-and-implementation-limits"></a>

# Terminología y límites de implementación de XDZip

[English](../../design/glossary.md) | [简体中文](../../zh-CN/design/glossary.md) | [繁體中文（台灣）](../../zh-TW/design/glossary.md) | [日本語](../../ja-JP/design/glossary.md) | [한국어](../../ko-KR/design/glossary.md) | [Français](../../fr-FR/design/glossary.md) | [Deutsch](../../de-DE/design/glossary.md) | Español | [Português (Brasil)](../../pt-BR/design/glossary.md) | [Русский](../../ru-RU/design/glossary.md)

<a name="xdzip-1-terminology"></a>

## 1. Terminología

| Término | Definición |
| --- | --- |
| Archivo comprimido | Contenedor o flujo comprimido compatible junto a su origen autorizado. |
| Códec | Implementación que detecta, escanea, lee o crea un formato. |
| Sesión | Origen abierto, motor, índice y contexto de navegación. Las tareas tienen su propio ciclo. |
| Entrada | Registro real del archivo. Una carpeta también puede tener un nodo de presentación sintético. |
| SourceVersion | Huella calculada en la medida de lo posible de un archivo de origen o volumen admitido. |
| QuerySnapshot | Orden de resultados actual e inmutable, identificado por sesión, consulta, generación y revisión. |
| Generación | Sustitución de solicitudes de la interfaz que impide que resultados obsoletos reemplacen a los nuevos. |
| Operación preparada | Entrada de ejecución acotada que incluye el origen, selección y opciones pertinentes. |
| Tarea | Ciclo de vida y resultado de una operación admitida en la cola. |
| Autorización | Permiso de acceso a un archivo o carpeta gestionado por el host, distinto de la ruta. |
| Receptor de salida | Interfaz de escritura con medición, cuotas y control transaccional. |
| Proceso o hilo de trabajo | Los comandos y tareas usan hilos del host. Los trabajadores de archivos comprimidos e imágenes son procesos separados. |
| Confirmación | Publicación de la salida temporal verificada mediante la operación autorizada de archivo o contenedor. |
| Instantánea de tarea | Último informe de estado, progreso, aviso o resultado. Las instantáneas intermedias se pueden agrupar. |
| Capacidades | Intersección de implementación, características del formato, plataforma y políticas. |
| Compresión sólida | Varios archivos comparten bloques comprimidos. Leer una selección puede exigir descodificar el contenido anterior. |
| Primer fotograma presentado | Imagen realmente renderizada y presentada. Un registro ui_ready no la sustituye. |

<a name="xdzip-2-units-and-values"></a>

## 2. Unidades y valores

Los tamaños de memoria y archivos usan KiB/MiB/GiB binarios, con 1 MiB = 1024² bytes. El transporte y el cómputo conservan bytes enteros. El tiempo se expresa en ms o s. Las mediciones de duración usan un reloj monótono. Las marcas de tiempo mantienen su representación de calendario y precisión de origen si existen. El rendimiento se expresa en MiB/s e indica si los bytes son entrada comprimida, datos descodificados o salida.

El 100% de CPU de un núcleo significa un núcleo lógico totalmente ocupado. Un árbol de procesos puede superar el 100%. PSS, RSS, USS y huella física son medidas distintas. El tamaño de descarga comprimida, el instalado y el del ejecutable se registran por separado.

<a name="xdzip-3-where-to-find-current-limits"></a>

## 3. Dónde consultar los límites actuales

| Tema | Referencia de implementación |
| --- | --- |
| Solicitudes, páginas y límites del buzón | [Puente Qt/Rust](qt-rust-bridge.md) |
| Admisión de tareas, progreso y códigos de estado | [Planificación](task-scheduling.md) |
| Límites de extracción, autorización y publicación | [Seguridad de archivos](file-security.md) |
| Límites de metadatos e índice ZIP en disco | [Indexación de archivos grandes](../engineering/large-index.md) |
| Límites de orígenes y volúmenes | [Apertura de archivos comprimidos](../engineering/archive-opening.md) |
| Límites de vista previa de texto e imágenes | [Vista previa del contenido](../engineering/content-preview.md) |
| Tramas, supervisión y límites de recursos | [Proceso de archivos comprimidos](../engineering/native-codecs.md) |
| Protocolo de extensión experimental | [Ensayo del SDK](../engineering/sdk-trial.md) |
| Objetivos de rendimiento | [Diseño de rendimiento](performance.md) |

<a name="xdzip-4-versioning"></a>

## 4. Versiones

La versión de la aplicación, la interfaz interna de compilación CXX, el protocolo externo y el esquema de datos persistentes se versionan por separado. La versión del producto no determina por sí sola la versión del protocolo del SDK. Consulta el [diseño del SDK de formatos](codec-extension-sdk.md).
