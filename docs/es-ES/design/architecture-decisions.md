# xdzip: registros de decisiones de arquitectura

[English](../../design/architecture-decisions.md) | [简体中文](../../zh-CN/design/architecture-decisions.md) | [繁體中文](../../zh-TW/design/architecture-decisions.md) | [日本語](../../ja-JP/design/architecture-decisions.md) | [한국어](../../ko-KR/design/architecture-decisions.md) | [Français](../../fr-FR/design/architecture-decisions.md) | [Deutsch](../../de-DE/design/architecture-decisions.md) | Español | [Português (Brasil)](../../pt-BR/design/architecture-decisions.md) | [Русский](../../ru-RU/design/architecture-decisions.md)

Los primeros registros son del 11 de octubre de 2026. Los posteriores indican su propia fecha. Una decisión aceptada queda adoptada. Las notas de implementación actual relacionan cada decisión con los contratos mantenidos y la evidencia de capacidades. Para sustituir una decisión, conserva la original y añade un nuevo registro.

## ADR-001: Qt Quick como única interfaz del producto

**Estado: aceptada.**

**Implementación actual.** El escritorio utiliza Qt Quick/QML con adaptadores C++ delgados. El ejecutable SFX de Windows con diálogos del sistema es la excepción limitada de ADR-013. [Interfaz de escritorio](qt-quick-ui.md)

Contexto: el producto necesita una interfaz de escritorio moderna, una experiencia coherente y un consumo reducido de recursos. Se completaron prototipos de Slint, Tauri y Qt Quick con el mismo núcleo. Al considerar las capacidades de interfaz, la coherencia y las mediciones disponibles del renderizado por software en Linux, el proyecto eligió Qt Quick para la etapa de producto; aún deben verificarse las demás plataformas.

Decisión: Qt Quick/QML y Qt Quick Controls 2 con una adaptación C++ delgada. Slint y Tauri quedan como experimentos históricos y no se empaquetan en el producto.

Costes: entorno de ejecución Qt, módulos QML, compilaciones multilenguaje y gestión de licencias. La entrada, accesibilidad, GPU e instalación en Mac y Windows requieren más pruebas. El ejecutable del prototipo de 0,80 MiB no es un instalador Qt completo.

Condición de revisión: Qt no satisface la funcionalidad, licencias o mantenimiento de seguridad necesarios en las plataformas objetivo y sigue sin cumplir los presupuestos esenciales tras una optimización real. Las fluctuaciones normales de rendimiento no provocan una nueva selección de framework.

## ADR-002: Rust controla toda la lógica de negocio de archivos comprimidos

**Estado: aceptada.**

**Implementación actual.** El entorno Rust gestiona el estado de los archivos comprimidos y las tareas. La selección de archivos expresa una intención del escritorio y la capa de E/S del host autoriza el acceso al sistema de archivos. [Arquitectura del sistema](system-architecture.md)

Decisión: dominio, índices, planes, tareas, cuotas, políticas de rutas y registro de formatos residen en Rust. Qt representa el estado, autoriza la selección de archivos del sistema y transmite las decisiones del usuario. Las pruebas de negocio no requieren Qt.

Alternativas: conectar libarchive o 7zip directamente a C++/QML, o repetir la lógica de control en cada interfaz. Esto acopla el núcleo a las interfaces de plataforma e impide verificar coherentemente la seguridad y la cancelación. Rust no elimina los riesgos de las bibliotecas nativas; las bibliotecas pertinentes pueden aislarse en procesos de trabajo.

Condición de revisión: si una API de plataforma no puede implementarse de forma fiable en Rust, se permite una pequeña adaptación C++ específica sin cambiar la responsabilidad sobre la lógica de negocio.

## ADR-003: puente cxx generado durante la compilación

**Estado: aceptada.**

**Implementación actual.** El producto compila y utiliza el puente tipado con manejadores separados de comandos y eventos. El receptor de eventos y el ciclo de cierre se definen en el [contrato del puente](qt-rust-bridge.md).

Decisión: usar dentro del producto un puente seguro en tipos generado por cxx y compilado desde el mismo commit. No mantener a largo plazo una ABI C escrita a mano ni serializar JSON en rutas críticas. El puente no contiene lógica de negocio Qt.

Alternativas: conservar la ABI C experimental, generar QObject con CXX-Qt o conectar toda la interfaz y el núcleo mediante IPC local. cxx mantiene límites explícitos para los modelos y los hilos de Qt sin introducir inicialmente un framework generador de QObject. A cambio hay que mantener la capa QObject delgada y la coordinación entre CMake y Cargo.

Límites: cxx no garantiza compatibilidad binaria dinámica entre versiones arbitrarias de Rust y Qt ni genera un protocolo de formatos de terceros.

Condición de revisión: falla la verificación de la compilación de escritorio o del ciclo de vida admitidos, o la capa de adaptación crece considerablemente. Evaluar CXX-Qt o una ABI C con versión explícita mediante una ADR, sin cambiar silenciosamente.

## ADR-004: separar el SDK de código integrado del protocolo de procesos externos

**Estado: límite de diseño aceptado. El protocolo externo es experimental.**

**Implementación actual.** Los formatos del producto se registran explícitamente y están integrados. El proceso de formatos incluido usa su propio protocolo. La implementación CBOR externa es una prueba independiente de solo lectura. [Adaptadores de formato y prueba](codec-extension-sdk.md)

Decisión: usar interfaces de código Rust para formatos integrados y un protocolo de procesos separado y acotado para extensiones externas. El host conserva la autoridad sobre la E/S.

Motivo: la ABI de bibliotecas dinámicas de Rust es inestable y los complementos nativos arbitrarios dentro del proceso perjudican el aislamiento de fallos y el control de recursos. Wasm no es prioritario como entorno general de formatos por los costes de migración, hilos, acceso mediante búsqueda y verificación de rendimiento de los codecs nativos existentes; no se descarta su uso futuro.

Costes: protocolo, intermediario, sandbox y sobrecarga de IPC. El protocolo solo se congela después de que dos implementaciones independientes hayan demostrado su interoperabilidad.

## ADR-005: el host controla centralmente las escrituras reales

**Estado: aceptada.**

**Implementación actual.** El entorno prepara las operaciones y `xdzip-io` controla autorizaciones, preparación temporal y confirmación. Los adaptadores de formato trabajan con lectores y destinos de salida. [Autorización de archivos](file-security.md)

Decisión: los motores solo escriben en `OutputSink` autorizados. El host controla las rutas reales, conflictos, preparación temporal y publicación. No llamar directamente a una función general de extracción de una biblioteca hacia directorios arbitrarios, salvo que sus adaptaciones se hayan verificado íntegramente como restringidas por el host.

Motivo: implementar por separado la protección de rutas para cada formato produce fácilmente incoherencias; comprobar cadenas no basta para impedir enlaces o condiciones de carrera. El coste es implementar y probar ante fallos capas seguras de archivos en tres plataformas.

Límites: el host no puede demostrar la corrección semántica de la salida de un complemento malicioso ni detener un proceso malicioso que ya controla al mismo usuario. La confianza y los permisos de los complementos deben seguir siendo explícitos.

## ADR-006: publicar cada archivo extraído y cada nueva agrupación completa

**Estado: aceptada.**

**Implementación actual.** La extracción publica los archivos individualmente. La creación y la edición verifican los archivos comprimidos temporales antes de reemplazarlos o publicarlos. La creación de volúmenes publica un conjunto verificado parte por parte e intenta una reversión limitada si se produce un fallo parcial. [Semántica de publicación](file-security.md)

Decisión: cada archivo extraído se prepara y verifica antes de su publicación; una tarea fallida puede conservar resultados parciales. Los nuevos archivos comprimidos se escriben por completo antes de publicarse. Por defecto no se sobrescribe.

Motivo: fusionar resultados en un directorio existente no ofrece fácilmente una transacción verdaderamente atómica para todo el directorio en todas las plataformas. Fingir la reversión de toda la tarea podría borrar cambios posteriores del usuario. Se necesitan resultados y registros de recuperación detallados y una explicación clara de la cancelación.

Condición de revisión: un nuevo modo de publicación de un directorio completamente nuevo puede implementarse por separado bajo condiciones limitadas, sin cambiar la semántica de fusión en directorios existentes.

## ADR-007: actores y grupos de trabajo acotados con eventos enviados

**Estado: aceptada.**

**Implementación actual.** Cada conexión usa hilos separados para comandos y tareas; el escritorio admite en serie las tareas pesadas, y las colas Rust y el buzón Qt son acotados. Un grupo generalizado de tokens de recursos sigue siendo una opción de diseño. [Planificación de tareas](task-scheduling.md), [puente Qt/Rust](qt-rust-bridge.md)

Decisión: un planificador ligero determina el estado, grupos acotados ejecutan el trabajo y Qt tiene un hilo receptor de eventos independiente. Agrupar las actualizaciones de progreso, entregar los estados finales y reservar una admisión de control independiente para cancelación y cierre.

Alternativas: llamadas síncronas desde la interfaz, un hilo por archivo, Tokio o `spawn_blocking` sin límite, o sondeo continuo. El trabajo con archivos comprimidos es principalmente síncrono y utiliza CPU y E/S; no crear tareas ilimitadas solo porque una operación se denomine asíncrona. Las futuras E/S de red reales pueden usar localmente un entorno asíncrono.

Costes: control de presión, cierre y reservas de recursos explícitos. Ventajas: baja carga en reposo y una interfaz que las tareas no pueden saturar.

## ADR-008: formatos limitados en la primera versión según sus capacidades

**Estado: aceptada.**

**Implementación actual.** Los formatos iniciales son contexto histórico. El registro mantenido y la admisión en ejecución definen las capacidades actuales del código, incluidos los perfiles de lectura y creación ampliados y las restricciones específicas de cada operación. [Registro de formatos](../engineering/format-registry.md)

Decisión: v0.1 usa ZIP Store/Deflate y TAR/GZIP; el cifrado, los volúmenes, RAR, 7z y los algoritmos adicionales se verifican por separado. La interfaz no deduce una compatibilidad completa de la extensión.

Motivo: el nombre de un formato no describe todas sus funciones reales. La tabla inicial es más corta, pero honesta y verificable para los usuarios.

## ADR-009: separar los pequeños datos del producto del contenido del usuario

**Estado: aceptada.**

**Implementación actual.** Ajustes, índices y registros tienen ciclos de vida distintos. Los índices en disco son privados y se limitan a la sesión. La reutilización persistente de índices y la reanudación de trabajos incompletos siguen siendo opciones de diseño. [Modelo de dominio](domain-data.md), [publicación de archivos](file-security.md)

Decisión: ajustes, índices temporales, registros de transacciones y manifiestos de extensiones tienen ciclos de vida separados. Sin telemetría por defecto, contraseñas persistentes ni caché de índices de archivos comprimidos entre arranques.

Costes: varios esquemas y reglas de limpieza. Reconsiderar índices entre arranques exige diseñar la identidad e invalidación de las fuentes, la privacidad y la capacidad.

## ADR-010: la distribución completa y las licencias son requisitos de arquitectura

**Estado: aceptada.**

**Implementación actual.** Los binarios de GitHub y los MSIX de Microsoft Store siguen canales de empaquetado documentados y distintos. Se revisan las dependencias realmente instaladas, la integridad de los artefactos y el material de licencias de cada canal. [Compilación y publicación](build-release.md)

Decisión: desplegar Qt dinámicamente por defecto y revisar las licencias de los módulos realmente utilizados. El tamaño de la instalación incluye los entornos de ejecución. La firma, la SBOM y las obligaciones de sustitución o código fuente forman parte de las comprobaciones de publicación.

Alternativas: comparar solo los ejecutables o añadir `LICENSE` justo antes de publicar. Ninguna demuestra que el producto sea pequeño o distribuible legalmente.

Costes: mantenimiento continuo de despliegue y licencias. Una compilación estática de Qt o la distribución en una tienda requiere una ADR y una revisión de licencias independientes; una PR de reducción de tamaño no puede cambiar estas decisiones incidentalmente.

<a name="xdzip-adr-011-initial-support-scope-and-personal-project-maintenance-policy"></a>

## ADR-011: alcance inicial y política de mantenimiento de un proyecto personal

**Estado: aceptada el 19 de octubre de 2026 con aprobación explícita del propietario del proyecto.**

**Implementación actual.** La [política de compatibilidad](../engineering/support-policy.md) y el [contrato de actualización](../../engineering/software-update.md) definen el soporte y las actualizaciones actuales. La cláusula de actualización manual describe v0.1.

Decisión: admitir inicialmente macOS 14 o posterior en Apple Silicon arm64, versiones mantenidas de Windows 11 x86_64 y Ubuntu 24.04 LTS x86_64 con X11 y glibc 2.39 o posterior. Priorizar únicamente la última versión pública, atender los comentarios según los medios disponibles sin un plazo fijo de respuesta, usar actualizaciones manuales en v0.1 y mantener las versiones preliminares de desarrollo como experimentales. Véase la [política de compatibilidad](../engineering/support-policy.md).

Motivo: las compilaciones nativas, instaladores y dependencias disponibles delimitan el alcance y reducen las promesas no verificadas de sistemas y arquitecturas de diseños anteriores a la capacidad de mantenimiento de un proyecto personal de código abierto. No se promete soporte para Mac Intel, Windows 10, Windows/Linux ARM64, otras distribuciones ni Wayland. Ampliar el alcance exige primero implementación y verificación reales.

Límite: la distribución personal de código abierto en GitHub no requiere cuentas de desarrollador de pago, certificados de firma, Developer ID ni notarización. macOS conserva la firma ad hoc necesaria para la integridad. Esta política no sustituye la revisión del origen de las licencias ni de los paquetes y no implica compatibilidad con todas las versiones futuras de los sistemas.

## ADR-012: migrar cada formato a algoritmos Rust independientes

**Estado: aceptada el 25 de octubre de 2026 y aprobada explícitamente por el propietario para su implementación. Sustituye la elección a largo plazo de motores de formato de terceros; no significa que la migración haya terminado.**

**Implementación actual.** El producto utiliza implementaciones de formato Rust. Las lecturas correspondientes de 7z, RAR, CAB y medios ópticos usan el proceso de trabajo incluido. La matriz actual también incluye un escritor ISO dentro del proceso. Comprueba cada operación en el [registro de formatos](../engineering/format-registry.md) y el [diseño de adaptadores](codec-extension-sdk.md).

Decisión: usar implementaciones Rust independientes para el análisis de formatos y los algoritmos de compresión y descompresión; se permiten bibliotecas generales de criptografía, codificación de caracteres y bases del sistema. Definir primero las especificaciones de comportamiento a partir de las especificaciones oficiales y después implementar de forma independiente; no traducir línea por línea ni copiar el código de referencia cambiando nombres. Los desarrolladores que hayan visto ese código no afirman haber trabajado en una sala limpia estricta. Las fuentes insuficientes y las restricciones de licencia requieren una revisión separada, en particular no se desarrolla un codificador RAR.

El plan contemplaba aceptar y sustituir ZIP, TAR, TAR.GZ, 7z, RAR, CAB, ISO y UDF individualmente según las capacidades del momento. Conservar la implementación actual de los formatos que no hayan superado la aceptación; no recurrir silenciosamente a otra implementación ni ampliar el alcance de solo lectura de RAR, CAB, ISO o UDF. Reutilizar los algoritmos de compresión compartidos. Los nombres, identidades, E/S autorizadas, recursos, cancelación y publicaciones siguen bajo el host Rust; el aislamiento de procesos no desaparece por migrar a Rust.

Costes: mantener de forma independiente y a largo plazo compatibilidad, seguridad, rendimiento, fuzzing y algoritmos. Pueden usarse referencias de terceros en pruebas, pero los formatos migrados deben retirar los motores correspondientes de las dependencias y archivos entregados. El [contrato de motores propios](../../engineering/independent-rust-codecs.md) recoge las fuentes y reglas de interfaz; el progreso y la aceptación real se siguen en [#45](https://github.com/xiaodaozhi/xdzip/issues/45).

<a name="xdzip-adr-013-standalone-windows-sfx-host"></a>

## ADR-013: ejecutable SFX independiente para Windows

**Estado: aceptada el 16 de noviembre de 2026 para el primer perfil SFX autorizado por el propietario.**

**Implementación actual.** El [contrato SFX](../engineering/sfx-creation.md) define el perfil Windows y el punto de entrada independiente en Rust con diálogos del sistema.

Contexto: un archivo autoextraíble generado para Windows x64 debe funcionar sin xdzip ni Qt instalados. Incluir otra aplicación Qt completa en cada archivo añadiría un entorno de despliegue ajeno a esa necesidad.

Decisión: el escritorio conserva Qt Quick y reutiliza su ventana de creación. El ejecutable SFX independiente del proyecto utiliza únicamente diálogos de carpeta y tarea del sistema Windows, con fuentes, DPI, tamaño y accesibilidad controlados por el sistema. ZIP, verificación, autorización de salida, conflictos, progreso, cancelación y limpieza utilizan el núcleo Rust existente. Es una excepción limitada para el ejecutable de distribución respecto a ADR-001, no otro framework de escritorio ni una nueva biblioteca de controles personalizados.

Costes: mantener y probar automáticamente un ejecutable Windows adicional, una envoltura versionada, enlace CRT estático, auditoría de dependencias sin Qt y un host de diálogos nativos. Nunca ejecuta el contenido extraído. Firmar el ejecutable generado es independiente de firmar un programa fijo.

Condición de revisión: cualquier solicitud de instaladores, ejecución automática de contenido, otras plataformas ejecutables o interfaz SFX personalizada necesita su propio alcance y revisión de arquitectura.
