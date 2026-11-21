<a name="xdzip-build-deployment-and-release-architecture"></a>

# Arquitectura de compilación, despliegue y publicación

[English](../../design/build-release.md) | [简体中文](../../zh-CN/design/build-release.md) | [繁體中文](../../zh-TW/design/build-release.md) | [日本語](../../ja-JP/design/build-release.md) | [한국어](../../ko-KR/design/build-release.md) | [Français](../../fr-FR/design/build-release.md) | [Deutsch](../../de-DE/design/build-release.md) | Español | [Português (Brasil)](../../pt-BR/design/build-release.md) | [Русский](../../ru-RU/design/build-release.md)

<a name="xdzip-build-and-package-inputs"></a>

## Elementos de compilación y empaquetado

CMake compila la aplicación de escritorio Qt, los adaptadores C++ y los auxiliares nativos. Cargo compila el entorno de ejecución Rust y genera el puente CXX que se enlaza con la aplicación. La CLI independiente tiene su propio punto de entrada de compilación y funciona sin Qt. `scripts/desktop.py` y `scripts/cli.py` coordinan la compilación y las pruebas. El código fija Rust 1.99.0 y el SDK Qt 6.8.2. Para comparar resultados, conserva juntos la identidad de la compilación, la arquitectura de destino, los archivos de bloqueo y la procedencia de la cadena de herramientas. [Cadena de herramientas](../engineering/toolchain.md), [Guía de compilación](../../engineering/developer-build.md)

El paquete de escritorio incluye las bibliotecas Qt, los módulos QML, los recursos de la plataforma y los auxiliares que necesita. El proceso interno de imágenes usa Qt para decodificar imágenes y mostrar SVG estáticos. La distribución de la CLI incluye su proceso auxiliar de archivos comprimidos. Los scripts y contratos de plataforma mantienen los perfiles de compilación, las listas de archivos admitidos en los paquetes y las comprobaciones de despliegue. Comprueba el paquete entero para conocer las dependencias instaladas y su tamaño total. [Auditoría de despliegue y dependencias](../engineering/deployment-audit.md), [Distribución de la CLI](../engineering/command-line.md)

<a name="xdzip-current-distribution-channels"></a>

## Canales de distribución actuales

| Destino | Paquete y ámbito de confianza |
| --- | --- |
| macOS Apple Silicon | GitHub proporciona un DMG y un ZIP con el paquete de aplicación. La versión mínima es macOS 14. El paquete usa una firma ad hoc para verificar la integridad. Consulta las instrucciones de la plataforma para instalar y actualizar. |
| Windows 11 x86_64 | GitHub proporciona un instalador por usuario sin firma y un ZIP portátil. Microsoft Store gestiona la identidad MSIX, la firma y el canal de actualización de su edición. |
| Ubuntu 24.04 LTS x86_64 | GitHub proporciona un archivo comprimido con el directorio de ejecución y un DEB. La base compatible incluye X11 y glibc 2.39 o posterior. |
| CLI independiente | Se publica con sus propias identidades de paquete y actualización. La CLI que se instala con la aplicación de escritorio sigue las reglas de integración y transferencia de su paquete. |

La [política de soporte](../engineering/support-policy.md) define los sistemas operativos y las arquitecturas compatibles. Consulta el [README del proyecto](../../../README.es-ES.md) y las notas de la versión para ver los archivos publicados. La [guía del instalador](../../engineering/installers-associations.md) detalla los límites de asociación e integración.

<a name="xdzip-routine-checks-native-builds-and-release"></a>

## Comprobaciones habituales, compilaciones nativas y publicación

El [flujo de CI habitual](https://github.com/xiaodaozhi/xdzip/blob/main/.github/workflows/docs.yml) ejecuta comprobaciones ligeras. Los trabajos nativos Foundation y Qt se ejecutan en flujos manuales separados. La CI habitual de Apple Silicon compila, prueba y verifica el inicio sin desplegar aplicaciones descargables. El flujo específico de publicación tiene sus propios pasos autorizados para desplegar, firmar, empaquetar y publicar. El [protocolo de publicación](../engineering/release-protocol.md) y la [política de CI](../../../developers/engineering/ci-policy.md) definen los requisitos y las pruebas de cumplimiento.

Los manifiestos de publicación, las sumas de comprobación, los inventarios, la SBOM y los documentos de licencia describen los artefactos entregados. El flujo de publicación no genera ni sube ZIP del código fuente del proyecto. GitHub genera sus propias instantáneas de etiquetas mediante otro mecanismo. Las obligaciones dependen de los componentes entregados, el [registro de licencias](../engineering/licensing.md) y la [política de entrega del código fuente](../../engineering/source-delivery.md).

<a name="xdzip-update-and-future-distribution"></a>

## Actualizaciones y futuras distribuciones

La selección y la verificación de actualizaciones son independientes de las tareas de archivos comprimidos. Las instalaciones elegibles siguen el contrato de actualización para los metadatos de confianza, los archivos verificados, la identidad de instalación y la transferencia cuando hay tareas activas. Windows y Linux usan el auxiliar correspondiente, macOS usa Sparkle y Microsoft Store mantiene su edición. Las configuraciones portátiles y de desarrollo tienen sus propios límites. [Actualizaciones del programa](../engineering/software-update.md)

Cada ampliación a otra arquitectura, versión mínima de un sistema, formato de paquete o tienda requiere sus propias pruebas de compilación y aceptación. Los [registros de decisiones de arquitectura](architecture-decisions.md) recogen las decisiones anteriores sobre distribución y los motivos para reconsiderarlas.
