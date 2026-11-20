<a name="xdzip-xdzip-design-guide"></a>

# Guía de diseño de XDZip

[English](../../design/index.md) | [简体中文](../../zh-CN/design/index.md) | [繁體中文](../../zh-TW/design/index.md) | [日本語](../../ja-JP/design/index.md) | [한국어](../../ko-KR/design/index.md) | [Français](../../fr-FR/design/index.md) | [Deutsch](../../de-DE/design/index.md) | Español | [Português (Brasil)](../../pt-BR/design/index.md) | [Русский](../../ru-RU/design/index.md)

Empieza por la [arquitectura del sistema](system-architecture.md) para ver cómo el escritorio Qt Quick y la CLI independiente en Rust acceden al motor de archivos comprimidos. Las páginas de diseño explican los contratos actuales, las decisiones y el alcance de las propuestas. El [manual de usuario](../user-guide/README.md) cubre la instalación y las operaciones habituales. Para compilar o modificar el proyecto, consulta [Compilar y contribuir](../contributing/README.md).

<a name="xdzip-find-a-design-topic"></a>

## Encontrar un tema de diseño

| Tema | Lecturas |
| --- | --- |
| Comportamiento del producto | Los [requisitos](product-requirements.md) definen las operaciones previstas y su alcance. La [trazabilidad de requisitos](requirements-traceability.md) identifica las comprobaciones correspondientes. |
| Datos de archivos comprimidos | El [modelo de dominio](domain-data.md) describe sesiones, versiones del origen, entradas físicas, consultas y datos de entrada de operaciones. |
| Escritorio | La [interfaz Qt Quick](qt-quick-ui.md) describe vistas, propiedad de controladores e interacción. El [puente Qt/Rust](qt-rust-bridge.md) detalla comandos, identificadores, eventos y ciclos de vida reales. |
| Formatos y extensiones | Los [adaptadores de formatos y ensayo de extensiones](codec-extension-sdk.md) explican el registro integrado, el proceso incluido y el ensayo independiente XDP1. |
| Salida y tareas en segundo plano | La [autorización y publicación segura](file-security.md) cubre la preparación, la verificación y la confirmación. La [planificación de tareas](task-scheduling.md) describe admisión, avisos, progreso y cancelación. |
| Rendimiento y calidad | El [diseño de rendimiento](performance.md) explica los presupuestos, las mediciones y los objetivos. La [ingeniería de calidad](quality-engineering.md) relaciona cambios y comprobaciones. |
| Entrega y decisiones | La [compilación, distribución y publicación](build-release.md) cubre plataformas, paquetes y actualizaciones. Las [decisiones](architecture-decisions.md) registran los motivos, y la [evolución de la arquitectura](architecture-evolution.md) explica cómo se llegó desde el prototipo al producto. |
| Terminología | La [terminología y los límites](glossary.md) explica el vocabulario y enlaza los límites de cada subsistema. |

<a name="xdzip-interpreting-the-references"></a>

## Interpretar las referencias

El comportamiento actual se documenta con enlaces a la implementación. Las opciones de diseño y las herramientas experimentales indican su propio alcance. La [referencia técnica](../reference/README.md) lleva a los contratos. El [README del proyecto](../../../README.es-ES.md) y las notas de la versión describen las distribuciones compatibles y las funciones entregadas. Los resultados de tiempo, recursos y comprobaciones nativas se aplican a la compilación, plataforma y carga que se indican.
