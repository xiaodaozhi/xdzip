<a name="xdzip-architecture-evolution-and-current-boundaries"></a>

# Evolución de la arquitectura y alcance actual

[English](../../design/architecture-evolution.md) | [简体中文](../../zh-CN/design/architecture-evolution.md) | [繁體中文（台灣）](../../zh-TW/design/architecture-evolution.md) | [日本語](../../ja-JP/design/architecture-evolution.md) | [한국어](../../ko-KR/design/architecture-evolution.md) | [Français](../../fr-FR/design/architecture-evolution.md) | [Deutsch](../../de-DE/design/architecture-evolution.md) | Español | [Português (Brasil)](../../pt-BR/design/architecture-evolution.md) | [Русский](../../ru-RU/design/architecture-evolution.md)

<a name="xdzip-from-framework-experiments-to-the-product"></a>

## De los experimentos con frameworks al producto

Los primeros experimentos con frameworks usaban una lista de archivos comprimidos ficticia y tareas simuladas. El producto adoptó Qt Quick y un puente CXX tipado. Después se implementaron en Rust las operaciones reales sobre archivos comprimidos, tareas y archivos del sistema. El JSON, la ABI C y los tiempos del prototipo pertenecen a esos experimentos. Las [decisiones de arquitectura](architecture-decisions.md) conservan los motivos y las condiciones para reconsiderarlas.

El producto acota los modelos de filas y comprueba la identidad de las consultas para que los resultados antiguos no sustituyan a una vista nueva. Añade identificadores físicos de entradas, capacidades de formato explícitas, autorizaciones de archivos, preparación temporal y publicación verificada. La CLI independiente llama al mismo núcleo sin Qt. Consulta la [arquitectura del sistema](system-architecture.md) y el [modelo de dominio](domain-data.md).

<a name="xdzip-current-experimental-and-proposed-work"></a>

## Funciones actuales, experimentos y propuestas

| Área | Estado actual |
| --- | --- |
| Entorno de ejecución e indexación | Implementados en el host. Un ZIP/ZIP64 normal de un solo archivo puede pasar a un índice privado en disco. Los límites de tareas y metadatos son independientes. [Diseño de rendimiento](performance.md) |
| Aislamiento de formatos | El enrutamiento integrado elige Rust en el mismo proceso o el proceso de formato incluido. La operación debe estar permitida por el registro y el perfil y superar los controles del entorno de ejecución. [Adaptadores de formatos](codec-extension-sdk.md) |
| Extensiones externas | El ensayo independiente XDP1 de solo lectura usa un protocolo acotado y requisitos por plataforma. Se ejecuta en un host separado. La detección, instalación y activación de extensiones dentro del producto siguen siendo propuestas. [Ensayo del SDK](../engineering/sdk-trial.md) |
| Estado persistente y concurrencia | El planificador admite una sola operación de escritorio pesada a la vez. Los índices persistentes entre ejecuciones, la recuperación de tareas y la planificación compartida general siguen en diseño. [Planificación de tareas](task-scheduling.md), [Publicación de archivos](file-security.md) |

Consulta la [tabla de formatos](../../../README.es-ES.md#xdzip-format-support) para las capacidades del código actual. Para una versión publicada, usa sus notas y artefactos. Las implementaciones nativas, las herramientas experimentales y las mediciones tienen alcances propios.
