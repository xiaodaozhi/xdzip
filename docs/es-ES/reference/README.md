<a name="xdzip-product-design-and-technical-reference"></a>

# Diseño del producto y referencia técnica

[English](../../reference/README.md) | [简体中文](../../zh-CN/reference/README.md) | [繁體中文](../../zh-TW/reference/README.md) | [日本語](../../ja-JP/reference/README.md) | [한국어](../../ko-KR/reference/README.md) | [Français](../../fr-FR/reference/README.md) | [Deutsch](../../de-DE/reference/README.md) | Español | [Português (Brasil)](../../pt-BR/reference/README.md) | [Русский](../../ru-RU/reference/README.md)

Empieza por la [arquitectura del sistema](../design/system-architecture.md) para conocer las rutas de ejecución del escritorio y la CLI. Las secciones siguientes enlazan el contrato que define cada comportamiento. La [tabla de formatos](../../../README.es-ES.md#xdzip-format-support) resume las operaciones compatibles. Para pasos de uso o preparación del entorno, consulta el [manual de usuario](../user-guide/README.md), los [ejemplos de CLI](../integration/README.md) y la [guía de contribución](../contributing/README.md).

<a name="xdzip-architecture-and-interfaces"></a>

## Arquitectura e interfaces

- [Guía de diseño](../design/index.md)
- [Requisitos y alcance de las operaciones](../design/product-requirements.md)
- [Modelo de dominio, identidad de entradas y versiones del origen](../design/domain-data.md)
- [Puente de comandos y eventos Qt/Rust](../design/qt-rust-bridge.md)
- [Planificación, progreso, cancelación y cierre](../design/task-scheduling.md)
- [Presupuestos de rendimiento y comprobación](../design/performance.md)
- [Términos y límites de implementación](../design/glossary.md)
- [Trazabilidad de requisitos y comprobaciones](../design/requirements-traceability.md)
- [Decisiones de arquitectura](../design/architecture-decisions.md)
- [Evolución de los prototipos al escritorio y a Rust](../design/architecture-evolution.md)
- [Estructura del repositorio y dependencias](../engineering/repository-layout.md)
- [Contratos de interfaces y comprobaciones](../engineering/contracts.md)
- [Entorno de ejecución y ciclo de vida de Qt](../engineering/product-runtime.md)
- [Sesiones y cierre de la aplicación](../engineering/application-sessions.md)
- [Motivos para usar el puente cxx](../engineering/cxx-admission.md)
- [Servicios de calendario y codificación del host](../engineering/host-platform-services.md)

<a name="xdzip-reading-and-writing-archives"></a>

## Lectura y escritura de archivos comprimidos

- [Autorización, preparación y publicación segura](../design/file-security.md)
- [Navegación de solo lectura](../engineering/readonly-browser.md)
- [Navegación ZIP de solo lectura](../engineering/zip-readonly-browsing.md)
- [Índices grandes y navegación bajo demanda](../engineering/large-index.md)
- [Lectura de volúmenes y apertura temporal](../engineering/archive-opening.md)
- [Codificación adaptativa de nombres de archivo](../engineering/filename-encoding.md)
- [Vista previa del contenido](../engineering/content-preview.md)
- [Comprobación previa de metadatos para la extracción](../engineering/extraction-preflight.md)
- [Extracción segura de ZIP](../engineering/zip-safe-extraction.md)
- [Comprobación de integridad](../engineering/archive-checking.md)
- [Apertura de entradas, arrastre y modificación](../engineering/archive-transfers.md)
- [Contraseñas y apertura de directorios cifrados](../engineering/password-extraction.md)
- [Contrato de creación ZIP](../engineering/zip-creation.md)
- [Creación ISO9660 y Joliet](../engineering/iso-creation.md)
- [Archivos ZIP autoextraíbles propios para Windows](../engineering/sfx-creation.md)

<a name="xdzip-format-support-and-implementation"></a>

## Compatibilidad e implementación de formatos

- [Capacidades integradas, perfiles de creación y alcance de comprobación](../engineering/format-registry.md)
- [Adaptadores de formato y diseño del ensayo SDK externo](../design/codec-extension-sdk.md)
- [Ensayo de SDK externo de solo lectura y alcance de ejecución](../engineering/sdk-trial.md)
- [Contrato del proceso de formato aislado](../engineering/native-codecs.md)
- [Formatos y algoritmos propios en Rust](../../engineering/independent-rust-codecs.md)
- [Contabilidad del trabajo auxiliar de descodificación](../engineering/auxiliary-decoding-work.md)
- [Admisión de la dependencia para metadatos ZIP](../engineering/zip-metadata-admission.md)
- [Admisión de dependencias de extracción y escritura ZIP](../engineering/zip-extraction-admission.md)
- [Fuentes del formato 7z e implementación independiente](../engineering/sevenzip-format-sources.md)
- [Bases de la implementación independiente RAR y límites de compatibilidad](../../engineering/rar-format-sources.md)
- [Fuentes del formato CAB y bases de implementación](../engineering/cab-format-sources.md)
- [Contrato de implementación TAR/GZIP](../engineering/tar-gzip.md)
- [Flujos de un solo archivo y paquetes derivados de ZIP](../engineering/single-file-streams.md)
- [Contrato de los flujos LZ4](../engineering/lz4-streams.md)
- [Contrato de los flujos Zstandard](../engineering/zstandard-streams.md)
- [Contrato de Unix compress Z](../engineering/compress-streams.md)
- [Contrato de flujos Brotli ordinarios](../engineering/brotli-streams.md)
- [Estructura y algoritmos LHA](../engineering/lha-containers.md)
- [Contenedores PMA y métodos PMarc](../engineering/pma-streams.md)
- [Flujos comprimidos y admisión de contenedores ARJ](../engineering/arj-streams.md)
- [Contenedores ARC y perfiles de flujos comprimidos](../engineering/arc-streams.md)
- [Contenedores BH y perfil BlakHole admitido](../engineering/bh-containers.md)
- [Fuentes y requisitos de licencia de ACE, ALZ y EGG](../engineering/ace-alz-egg-format-sources.md)
- [Envoltorios AES Crypt de solo lectura](../engineering/aescrypt-envelopes.md)
- [Contrato de contenedores DEB](../engineering/deb-containers.md)
- [Contrato de contenedores ASAR](../engineering/asar-containers.md)
- [Envoltorios de texto UU, UUE y XXE](../engineering/text-envelopes.md)
- [Fuentes y licencias de WIM, MSI e instaladores](../engineering/wim-msi-installer-format-sources.md)
- [Fuentes y licencias de imágenes de disco y otros formatos](../engineering/disk-image-archive-format-sources.md)
- [Inventario de formatos Bandizip y viabilidad en xdzip](../engineering/bandizip-format-assessment.md)

<a name="xdzip-desktop-interaction-and-visual-design"></a>

## Interacción y diseño de escritorio

- [Vistas Qt Quick y responsabilidades de los controladores](../design/qt-quick-ui.md)
- [Especificaciones visuales de escritorio](../../visual/README.md)
- [Norma de diseño de interfaz](../../visual/design-standard.md)
- [Proceso de diseño y corrección de interfaz](../visual/design-workflow.md)
- [Iconos de asociaciones de archivos](../visual/file-association-icons.md)
- [Revisión de fidelidad visual HTML](../visual/html-fidelity.md)
- [Especificación adicional de interacción y textos](../ux/interaction-spec.md)
- [Funciones, componentes e interacción de la ventana principal](../ux/main-window-components.md)
- [Diseño e integración del árbol de directorios](../ux/directory-tree.md)
- [Página de bienvenida de la ventana principal](../ux/welcome.md)
- [Capa de carga de bienvenida y fin de creación](../ux/welcome-opening.md)
- [Interacción con la contraseña de extracción](../ux/password-extraction.md)
- [Ventanas de tareas y comprobación de integridad](../ux/task-windows.md)
- [Ventana de configuración e idioma de la interfaz](../ux/settings-localization.md)
- [Terminología de la interfaz en chino simplificado](../ux/chinese-terminology.md)
- [Menús del sistema y área de notificación](../ux/desktop-integration.md)
- [Ventanas y barras de título por plataforma](../ux/platform-windows.md)
- [Estados de alta fidelidad y contrato de implementación](../ux/hifi-state-spec.md)
- [Prototipo de interacción v0.1 y correspondencia de funciones](../ux/prototype-spec.md)

<a name="xdzip-platform-integration-releases-and-trust"></a>

## Integración, publicaciones y confianza

- [Instaladores y asociaciones de archivos](../../engineering/installers-associations.md)
- [Compatibilidad y política de mantenimiento](../engineering/support-policy.md)
- [Acciones del gestor de archivos y host de tareas independiente](../engineering/shell-actions.md)
- [Actualización iniciada por el usuario](../../engineering/software-update.md)
- [Ciclo de vida de datos y diagnóstico](../engineering/privacy.md)
- [Versión del producto e identidad de compilación](../engineering/versioning.md)
- [Protocolo GitHub Release v1](../engineering/release-protocol.md)
- [DMG de desarrollo local para macOS](../engineering/macos-deployment.md)
- [Directorio de ejecución Linux independiente y comprobación](../engineering/linux-deployment.md)
- [Comprobación de dependencias de despliegue y SBOM](../engineering/deployment-audit.md)
- [Atribución y licencias de los componentes utilizados](../engineering/runtime-attribution.md)
- [Procedencia y avisos del entorno nativo](../engineering/native-runtime-materials.md)
- [Correspondencia entre Qt SDK y código fuente](../engineering/qt-sdk-correspondence.md)
- [Licencias de herramientas y entorno de ejecución de Windows](../engineering/windows-licensing.md)
- [Decisión de licencia y materiales de terceros](../engineering/licensing.md)
- [Admisión de dependencias y motores de formato](../../engineering/dependencies.md)
- [Código fuente correspondiente y recursos de recompilación](../../engineering/source-delivery.md)
- [Reconstruir datos ICU 73.2 a partir de CLDR](../engineering/icu-cldr-rebuild.md)

<a name="xdzip-build-and-cli-reference"></a>

## Referencia de compilación y CLI

- [Diseño de compilación, despliegue y publicación](../design/build-release.md)
- [Convenciones de código y niveles de comprobación](../design/quality-engineering.md)
- [Preparación del entorno de desarrollo](../engineering/development.md)
- [Compilaciones de desarrollo y comprobación por plataforma](../../engineering/developer-build.md)
- [Versiones fijadas de herramientas y plataformas](../engineering/toolchain.md)
- [Comandos CLI, JSON, instalación y recibos de actualización](../engineering/command-line.md)
