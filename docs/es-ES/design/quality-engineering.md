<a name="xdzip-coding-conventions-and-quality-engineering"></a>

# Convenciones de código e ingeniería de calidad

[English](../../design/quality-engineering.md) | [简体中文](../../zh-CN/design/quality-engineering.md) | [繁體中文（台灣）](../../zh-TW/design/quality-engineering.md) | [日本語](../../ja-JP/design/quality-engineering.md) | [한국어](../../ko-KR/design/quality-engineering.md) | [Français](../../fr-FR/design/quality-engineering.md) | [Deutsch](../../de-DE/design/quality-engineering.md) | Español | [Português (Brasil)](../../pt-BR/design/quality-engineering.md) | [Русский](../../ru-RU/design/quality-engineering.md)

<a name="xdzip-code-and-ownership"></a>

## Código y propiedad

- **Rust.** Sigue las herramientas, lints y archivo de bloqueo del espacio de trabajo. Valida longitudes, desplazamientos y conversiones de enteros externos. Conserva tamaños desconocidos y errores tipados. Documenta precondiciones de unsafe y FFI, propiedad, vida útil y comportamiento de fallo. Mantén breves las secciones críticas y realiza fuera de ellas la E/S de archivos comprimidos.
- **C++ y Qt.** Usa C++17, RAII y propiedad clara de QObject. Modifica modelos en su hilo gráfico, transfiere valores con propiedad entre hilos y vincula notificaciones en cola a un receptor vivo. Mantén el cierre asíncrono cuando el receptor aún necesita la interfaz gráfica para vaciar eventos. Rust conserva la responsabilidad sobre archivos comprimidos y archivos del sistema. [Puente Qt/Rust](qt-rust-bridge.md)
- **QML.** Expresa el estado y la intención mediante controladores. Reutiliza controles compartidos y recursos visuales y de texto generados. Mantén la E/S de archivos y la interpretación del formato fuera de los delegados. Los cambios de ventanas siguen el registro, el comportamiento nativo y los estándares visuales. [Interfaz Qt Quick](qt-quick-ui.md), [Proceso visual](../visual/design-workflow.md)

Documenta las invariantes y qué queda tras un fallo. En el código de archivos, explica por ejemplo qué protege el permiso de confirmación, qué identidades se comprueban y qué partes pueden estar visibles tras un error. [Publicación de archivos](file-security.md)

<a name="xdzip-verification-layers"></a>

## Capas de comprobación

| Cambio | Evidencia pertinente |
| --- | --- |
| Dominio, consultas y servicio | Pruebas Rust de identidad, límites, nombres duplicados, orden, respuestas obsoletas, entrega acotada y cancelación. |
| Operaciones de formato | Datos de prueba propios, ciclos independientes cuando corresponda y casos de rechazo, corrupción, perfiles incompatibles y cuotas. |
| Archivos nativos y tareas | Identidad de archivos y bytes escritos, carreras entre cancelación y confirmación, protección de salidas existentes y limpieza en la plataforma afectada. |
| Escritorio, Qt y accesibilidad | Pruebas de modelos y controladores, comprobaciones QML y de controles compartidos, interacción nativa si procede y evidencia visual o de tecnología de asistencia del comportamiento. |
| Rendimiento y entrega | Cargas vinculadas al commit, métricas de renderizado y recursos, o identidad de compilación por canal, resultados de instalación y actualización y recibos de dependencias. |

La [guía de comprobación](../../../developers/qa/README.md) determina los controles necesarios. La conformidad de protocolo, los datos de prueba o la comprobación sin pantalla, las ejecuciones nativas automatizadas y la observación manual establecen hechos distintos. Consulta las entradas existentes y límites en la [guía de compilación](../engineering/developer-build.md) y la [matriz de trazabilidad](requirements-traceability.md).

<a name="xdzip-existing-ci-and-additional-techniques"></a>

## CI y técnicas adicionales

La [CI habitual](https://github.com/xiaodaozhi/xdzip/blob/main/.github/workflows/docs.yml) comprueba documentación, recursos generados, contratos de diseño, formato y estructura de flujos. Las comprobaciones nativas más costosas de Rust, formatos y Qt se ejecutan bajo demanda en Foundation y Qt. Los trabajos dedicados preparan los artefactos de publicación. La [política de CI](../../../developers/engineering/ci-policy.md) describe activaciones y condiciones. Las rutas y plataformas afectadas determinan las comprobaciones locales y remotas.

Usa fuzzing, sanitizadores, comprobación de modelos y pruebas de rendimiento cuando cubran el cambio. Elige datos acotados que se puedan redistribuir y conserva las evidencias aparte de esta guía. Para informes sensibles, sigue la [política de seguridad](../../../SECURITY.md). Los planes y ejemplos no equivalen a ejecuciones programadas ni a resultados de aceptación.

<a name="xdzip-maintaining-the-design"></a>

## Mantener el diseño

Si la implementación cambia una identidad, un comando, una capacidad, una tarea o un límite de escritura, actualiza el diseño, la referencia técnica y sus enlaces de aceptación. Mantén las ediciones en todos los idiomas alineadas. Las [decisiones de arquitectura](architecture-decisions.md) recogen los motivos y el [diseño de rendimiento](performance.md) fija las reglas de medición. Registra los resultados y límites reales con el cambio revisado.
