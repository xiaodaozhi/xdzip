<a name="xdzip-performance-design-and-measurement"></a>

# Diseño y medición de rendimiento

[English](../../design/performance.md) | [简体中文](../../zh-CN/design/performance.md) | [繁體中文（台灣）](../../zh-TW/design/performance.md) | [日本語](../../ja-JP/design/performance.md) | [한국어](../../ko-KR/design/performance.md) | [Français](../../fr-FR/design/performance.md) | [Deutsch](../../de-DE/design/performance.md) | Español | [Português (Brasil)](../../pt-BR/design/performance.md) | [Русский](../../ru-RU/design/performance.md)

XDZip limita por separado memoria y trabajo de indexación, transporte, ejecución de tareas y vista previa de imágenes. Los límites de código siguientes son reglas de implementación. Tiempos, memoria y rendimiento dependen de la compilación y la carga. Los objetivos de referencia posteriores son metas de ingeniería.

<a name="xdzip-limits-and-mechanisms-in-the-current-code"></a>

## Límites y mecanismos del código actual

| Área | Comportamiento y coste actuales |
| --- | --- |
| ZIP/ZIP64 ordinario de un solo archivo | El analizador en flujo admite hasta un millón de registros físicos, sujeto a presupuestos de nombres, directorios y lectura. Cuando los metadatos activos superan 64 MiB o un nuevo registro superaría 100 000 entradas residentes, se trasladan a almacenamiento privado en disco. El archivo de metadatos se limita a 512 MiB y tiene una tabla de posiciones independiente. La navegación analiza metadatos sin decodificar todos los contenidos. |
| Consultas en memoria y disco | El entorno construye una vez el orden inmutable del resultado actual. La ruta en disco vuelca secuencias de ordenación de hasta 8 MiB y las fusiona en un orden de tokens. El almacenamiento de nombres y ordenación es limitado. La caché contiene la consulta actual, no una vista completamente representada ni un historial indefinido. |
| Navegación y transferencia Qt | El árbol de directorios tiene presupuestos de 32 768 nodos, 64 niveles y 4 MiB de nombres. Si se superan, el entorno puede mantener navegación de raíz y búsqueda. El puente envía hasta 128 filas y 64 KiB de nombres por página. Las vistas grandes mantienen hasta 1 024 filas residentes y un recuento lógico de todo el resultado. |
| Comandos, tareas y escrituras | Los hilos de metadatos y tareas usan colas acotadas separadas. Se admite una tarea pesada de escritorio a la vez. El trabajo de decodificación, bytes de salida, confirmaciones de archivos y recursos de procesos se contabilizan de forma distinta. Los formatos sólidos o en flujo pueden leer o decodificar datos anteriores antes de alcanzar una entrada seleccionada. |
| Vista previa y procesos nativos | Texto e imágenes codificadas tienen límites de bytes separados. Dimensiones, píxeles, ejecución del proceso y tamaño de miniaturas se limitan independientemente. Los procesos de formato tienen sus propios límites de protocolo y proceso. |

[Índice en disco](../../../crates/xdzip-runtime/src/disk_index.rs), [Índices grandes](../engineering/large-index.md), [Límites del puente Qt/Rust](qt-rust-bridge.md), [Admisión de tareas](task-scheduling.md), [Presupuestos de salida](file-security.md), [Presupuestos de vista previa](../engineering/content-preview.md)

El coste de memoria incluye Qt, fuentes, representación y modelos, metadatos, tabla de posiciones y búferes de consulta, estado de decodificadores, procesos hijos y búferes de E/S. El disparador de migración de 64 MiB es un umbral de metadatos. La memoria del árbol de procesos incluye todo lo que queda fuera de ese índice. Admitir más metadatos no amplía cuotas de extracción ni habilita comprobación o edición del archivo completo en sesiones respaldadas por disco. La escritura del índice privado y la preparación de extracción generan costes de disco separados.

<a name="xdzip-measuring-the-current-implementation"></a>

## Medición de la implementación actual

| Entrada | Qué mide o verifica |
| --- | --- |
| `scripts/probe-large-index.py` y `xdzip-m2-probe` | Generan un ZIP64 propio, ejercitan el analizador real, ordenación y consultas paginadas y notifican por separado fases y almacenamiento. Incluir duplicados, nombres largos y estructura de directorios e indicar si se alcanzó la ruta de disco. |
| `scripts/measure-desktop.py` y `xdzip-process-metrics` | Miden el árbol de procesos de escritorio, inicio y navegación. Con la opción de extracción ejecutan una extracción real y verifican independientemente los bytes de salida. Registrar motor de representación, identidad de compilación y carga exacta. |
| `scripts/run-performance-suite.py` | Recopila observaciones Release de inicio, inactividad estable y navegación, y compara extracción con una sonda de núcleo fijada sobre datos propios. El informe registra identidades de compilación y datos. La comparación es observacional y la aprobación de máquina de referencia es una condición separada. |
| `scripts/check-runtime-resources.py` | Comprueba el registro de recursos y fuentes de ejecución de la sonda. Acredita identidad y cobertura de recursos para esa sonda sin medir latencia interactiva. |

[Sonda de índice](../../../scripts/probe-large-index.py), [Medición de escritorio](../../../scripts/measure-desktop.py), [Suite de rendimiento](../../../scripts/run-performance-suite.py), [Sonda de recursos](../../../scripts/check-runtime-resources.py), [Recopilación de evidencia](../../../developers/engineering/candidate-verification.md#performance-and-resource-evidence)

Para comparar, registrar commit y configuración, sistema y arquitectura, motor de representación, hardware, número de ejecuciones, datos de prueba y condiciones del dispositivo y caché. Separar inicios en frío y repetidos. Cuando la medición lo permita, informar por separado de primera interacción, análisis del archivo comprimido, ordenación de consulta, transferencia de página y representación visible. Distinguir respuesta de cancelación de parada y limpieza reales. Separar bytes de entrada, decodificados y confirmados al informar del rendimiento.

La herramienta del árbol de procesos distingue PSS de Linux, huella física de macOS y compromiso privado de Windows, e informa también de RSS o conjunto de trabajo cuando estén disponibles. Comparar la misma métrica en condiciones comparables. Informar por separado de los tamaños de descarga e instalación de paquetes reales. Los tiempos fuera de pantalla y del prototipo describen solo sus propias condiciones.

<a name="xdzip-workloads-for-comparative-tests"></a>

## Cargas para pruebas comparativas

| Carga | Mediciones |
| --- | --- |
| B01/B08 Ventana vacía, inactividad y cambio de tema | Tiempos de inicio y fotograma, CPU inactiva, memoria retenida y motor de representación. |
| B02 100 000 y un millón de registros ZIP | Análisis de metadatos, migración, ordenación, obtención de páginas y costes separados de memoria y disco. |
| B03/B04 Un archivo ordinario grande o muchos pequeños | Rendimiento, cancelación, trabajo de destino, sincronización y limpieza. Las operaciones de archivos pueden dominar en archivos pequeños. |
| B05/B06 Entradas muy comprimidas o profundas en un flujo | Gestión de cuotas y trabajo adicional de decodificación antes de alcanzar la entrada seleccionada. |
| B07 Trabajo pesado pendiente durante la navegación | Latencia de búsqueda y páginas mientras una operación pesada se ejecuta y otra espera. |
| B09 Procesos incluidos y paquete completo | Uso del árbol de procesos, coste del protocolo, tamaño descargado e instalado. |

Son escenarios comparativos. Elegir la carga correspondiente al cambio y usar datos propios con tamaños y hashes registrados.

<a name="xdzip-reference-targets-and-planned-research"></a>

## Objetivos de referencia e investigación prevista

El diseño existente usa una referencia Apple Silicon con 16 GiB de memoria, SSD local, pantalla de 60 Hz, fuente del sistema, GPU real y compilación Release. Los objetivos son candidatos para mediciones comparables. No afirman nada sobre la última entrega u otra plataforma.

| Escenario | Objetivo de diseño |
| --- | --- |
| Inicio repetido de ventana vacía | P95 de como máximo 500 ms hasta un fotograma interactivo, con al menos 30 ejecuciones |
| Abrir y consultar 100 000 registros ZIP en caché | P95 de como máximo 1 s para preparar metadatos y 80 ms desde consulta indexada hasta visibilidad |
| Salto de página en caliente y planificación GUI | P95 de como máximo 50 ms para un salto visible y 8 ms de bloqueo continuo del hilo GUI |
| Memoria | Objetivo del árbol de procesos de hasta 150 MiB vacío y 256 MiB al navegar por 100 000 entradas sin vista previa ni diccionario de decodificación |
| CPU inactiva | Media de como máximo 0,5% de un núcleo lógico tras estabilización |
| Cancelación | P95 de como máximo 50 ms para respuesta y 250 ms para parada cooperativa local en flujo, medidos por separado |
| Extracción y distribución | Comparar interfaz y núcleo con carga y E/S idénticas, con objetivo de pérdida de rendimiento de hasta 10%. Los objetivos Mac de una arquitectura son como máximo 100 MiB descargados y 250 MiB instalados. |

La extracción paralela por bloques, reutilización persistente de índices y planificador compartido de tokens CPU siguen siendo opciones de diseño. Analizar la ruta actual de tareas en serie y las rutas de memoria y disco antes de cambiar concurrencia o almacenamiento. Mantener verificación equivalente de bytes, cancelación y publicación segura en cualquier comparación. [Evolución de arquitectura](architecture-evolution.md), [Publicación de archivos](file-security.md)
