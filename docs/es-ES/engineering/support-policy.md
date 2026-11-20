<a name="xdzip-system-support-and-maintenance-policy"></a>
# Política de compatibilidad y mantenimiento

[English](../../engineering/support-policy.md) | [简体中文](../../zh-CN/engineering/support-policy.md) | [繁體中文](../../zh-TW/engineering/support-policy.md) | [日本語](../../ja-JP/engineering/support-policy.md) | [한국어](../../ko-KR/engineering/support-policy.md) | [Français](../../fr-FR/engineering/support-policy.md) | [Deutsch](../../de-DE/engineering/support-policy.md) | Español | [Português (Brasil)](../../pt-BR/engineering/support-policy.md) | [Русский](../../ru-RU/engineering/support-policy.md)

El propietario del proyecto aprobó la política inicial el 2026-10-19. La versión `v0.1.0` se publicó el 2026-11-06. Consulta la [página de versiones](https://github.com/xiaodaozhi/xdzip/releases) para encontrar las versiones y descargas actuales. Esta política define el alcance de la compatibilidad. Se mantienen las comprobaciones de versión mínima. Los resultados de CI y las pruebas de aceptación del usuario se refieren a los entornos probados. El usuario dio por concluida la revisión manual.

<a name="xdzip-initial-support-scope"></a>
## Alcance inicial

| Plataforma | Alcance inicial | Comprobaciones y límites |
| --- | --- | --- |
| macOS | macOS 14 o posterior, Apple Silicon arm64 | Objetivo de despliegue 14.0, CI nativa y DMG local de desarrollo. Sin paquete Intel ni universal. No se garantiza la compatibilidad con todas las versiones futuras. |
| Windows | Versiones x86_64 de Windows 11 que Microsoft sigue manteniendo | Compilación con MSVC, instalador por usuario y pruebas reales de instalación, actualización, reversión y desinstalación en CI. Las pruebas en Windows Server no cubren todas las versiones de Windows 11. Sin paquete ARM64. |
| Linux | Ubuntu 24.04 LTS x86_64, X11, glibc 2.39 o posterior | Verificación del .deb actual y de la ejecución en un entorno aislado sin el SDK de Qt. Quedan fuera otras distribuciones, versiones anteriores de glibc, Wayland y ARM64. |

La versión mínima de macOS sigue el objetivo de despliegue actual. La versión 13 mencionada en diseños anteriores no se verificó. El soporte estándar de Windows 10 terminó el 2025-10-14. Windows 10 queda fuera del compromiso inicial. LTSC y ESU no se incluyen automáticamente. Consulta el [anuncio de Microsoft sobre el ciclo de vida](https://learn.microsoft.com/en-us/lifecycle/announcements/windows-10-end-of-support). La lista de [plataformas compatibles con Qt 6.8](https://doc.qt.io/qt-6.8/supported-platforms.html) indica el alcance de esa dependencia. xdzip realiza sus propias comprobaciones.

<a name="xdzip-maintenance"></a>
## Mantenimiento

Este proyecto personal se mantiene según los recursos disponibles. No ofrece un servicio de pago, un plazo de respuesta fijo ni compromisos de seguridad más allá del trabajo realizado. Tiene prioridad la última versión estable publicada. Las versiones de desarrollo pueden introducir cambios de compatibilidad. Las notas de versión indican problemas conocidos y opciones para volver a una versión anterior. Si aparece un problema de seguridad, pueden retirarse las descargas afectadas. Desde «Acerca de xdzip», el usuario puede buscar actualizaciones e iniciar su instalación. La aplicación no comprueba actualizaciones en segundo plano ni las instala de forma silenciosa. El código actual también incluye un comando de actualización para la CLI. La [especificación de la línea de comandos](../../engineering/command-line.md) indica qué instalaciones pueden usarlo y las diferencias con los archivos ya publicados.

Las versiones candidatas siguen fijadas en Qt 6.8.2 y en las versiones de Rust y Cargo ya bloqueadas para que las comprobaciones sean reproducibles. Esto no garantiza que los parches antiguos estén libres de vulnerabilidades ni que haya actualizaciones LTS gratuitas de forma indefinida. Antes de cada publicación, la persona responsable revisa los avisos de seguridad oficiales de Qt y las actualizaciones de código abierto disponibles. Se evalúan primero las versiones públicas con mantenimiento. Si una dependencia tiene un problema de seguridad sin parche que afecta a xdzip, se suspende la publicación correspondiente. Actualizar Qt exige volver a comprobar el código fuente, las licencias de los módulos, las interfaces privadas de Windows, la sustitución dinámica, la compilación en las tres plataformas y el funcionamiento de los menús y las ventanas. Contratar mantenimiento comercial es opcional.

<a name="xdzip-approved-scope-and-changes"></a>
## Alcance aprobado y cambios

El alcance aprobado abarca las plataformas iniciales, el mantenimiento de la última versión pública según los recursos disponibles, la ausencia de un plazo de respuesta fijo y el etiquetado de las compilaciones de desarrollo como experimentales. Los resultados de `v0.1.0` figuran en la publicación y en sus incidencias de aceptación. Esos resultados no constituyen una garantía de rendimiento para todas las versiones de los sistemas. Cualquier arquitectura, sistema o canal de distribución adicional debe implementarse y verificarse antes de actualizar esta política. Consulta [ADR-011](../design/architecture-decisions.md#xdzip-adr-011-initial-support-scope-and-personal-project-maintenance-policy).

Ya no se exigen la firma oficial, la notarización ni las cuentas de pago. Para comunicar una vulnerabilidad, consulta la [política de seguridad](../../../SECURITY.md).
