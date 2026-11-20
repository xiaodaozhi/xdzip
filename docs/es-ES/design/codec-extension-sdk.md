<a name="xdzip-format-adapters-and-the-extension-trial"></a>

# Adaptadores de formato y prueba de extensiones

[English](../../design/codec-extension-sdk.md) | [简体中文](../../zh-CN/design/codec-extension-sdk.md) | [繁體中文](../../zh-TW/design/codec-extension-sdk.md) | [日本語](../../ja-JP/design/codec-extension-sdk.md) | [한국어](../../ko-KR/design/codec-extension-sdk.md) | [Français](../../fr-FR/design/codec-extension-sdk.md) | [Deutsch](../../de-DE/design/codec-extension-sdk.md) | Español | [Português (Brasil)](../../pt-BR/design/codec-extension-sdk.md) | [Русский](../../ru-RU/design/codec-extension-sdk.md)

<a name="xdzip-three-execution-paths"></a>

## Tres rutas de ejecución

| Ruta | Implementación y disponibilidad |
| --- | --- |
| Adaptadores integrados en el anfitrión | Escritorio y CLI utilizan el mismo registro fijo de formatos y despacho de ejecución. Las implementaciones Rust de ZIP, TAR, filtros de flujo y contenedores admitidos como DEB y ASAR se llaman en el proceso anfitrión. |
| Proceso de archivos comprimidos incluido | `xdzip-codec-native` en el anfitrión inicia `xdzip-native-codec`. Su capa de protocolo C++ llama a los backends Rust de 7z, RAR, CAB y medios ópticos mediante `xdzip-codec-bridge`. La misma ruta gestiona la creación y reescritura 7z aplicables. La creación ISO usa su escritor óptico dentro del proceso. |
| Prueba de SDK externo | `xdzip-plugin-host` y `xdzip-plugin-protocol` forman un experimento XDP1 independiente y explícito. Las compilaciones de producto de escritorio y CLI no descubren, instalan ni ejecutan extensiones de terceros. |

Las dos primeras rutas son componentes integrados del producto. El proceso incluido es independiente, con protocolo privado y supervisión de recursos. La prueba externa tiene validación de paquete, negociación de versión y requisitos de permisos por plataforma propios. [Arquitectura del sistema](system-architecture.md), [Contrato del proceso de archivos comprimidos](../engineering/native-codecs.md), [Contrato de prueba del SDK](../engineering/sdk-trial.md)

<a name="xdzip-product-routing-and-operation-capabilities"></a>

## Enrutamiento del producto y capacidades de operación

La [fuente del registro de formatos](https://github.com/xiaodaozhi/xdzip/blob/main/specs/engineering/archive-formats.json) declara extensiones, tipos de contenedor y filtro, operaciones admitidas, métodos y opciones de creación. El generador sincroniza el registro Rust, las listas de escritorio y los artefactos de diseño. Al leer, el entorno de ejecución examina los bytes de origen y valida el formato antes de elegir un backend. Al crear, el nombre de salida solicitado selecciona un perfil de creación y el entorno comprueba las opciones frente a él. La lista de extensiones ayuda a descubrir formatos y mostrar indicaciones. La detección y el análisis satisfactorios establecen el formato abierto.

Lectura, vista previa, extracción, comprobación del archivo completo, creación y reescritura tienen restricciones separadas. Una entrada listada puede ser visible sin tener un método de contenido compatible. Contraseñas, volúmenes y registros especiales afectan a la admisión; el entorno gestiona diferencias específicas, como comprobaciones estructurales ISO/UDF y perfiles de reescritura no admitidos. [Enrutamiento de ejecución](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/formats.rs), [Contrato del registro de formatos](../engineering/format-registry.md), [Comprobación de archivos comprimidos](../engineering/archive-checking.md)

<a name="xdzip-internal-rust-adapter-interfaces"></a>

## Interfaces internas de adaptadores Rust

| Interfaz | Responsabilidad |
| --- | --- |
| `ReadSource` y `MetadataCodec` | Leer o posicionar una fuente autorizada por el anfitrión, incluidos los metadatos de volúmenes aplicables, y producir registros de formato acotados. |
| `PayloadCodec`, `PayloadReader` y `DecodedSink` | Resolver contenido y decodificar un registro físico apto hacia un receptor del anfitrión. Distinguir bytes normales, trabajo de decodificación omitido y trabajo auxiliar cuando el backend los informe. |
| `CreationCodec`, `EncodeInput` y `EncodeOutput` | Codificar desde metadatos de entrada fijados y flujos indexados. Escribir en un receptor del anfitrión, con escrituras aleatorias acotadas cuando el escritor lo requiera. |
| `RewriteCodec` | Reescribir archivos compatibles a partir de entradas originales conservadas, cambios y adiciones autorizadas hacia un nuevo archivo preparado temporalmente. |

El anfitrión controla rutas, nueva validación del origen, cancelación, contraseñas, cuotas y toda publicación final. Un códec recibe capacidades mediante las interfaces necesarias para la operación. El proceso de archivos comprimidos solicita intervalos de origen permitidos y entrega registros o bloques de salida acotados. El anfitrión valida el estado del protocolo y las cuotas de bytes antes de actuar. Un fallo del proceso o una respuesta inválida no puede autorizar la publicación. La frontera del proceso aísla fallos y recursos. Las restricciones de permisos del núcleo para código externo arbitrario son un requisito separado de la prueba. [Contratos de metadatos](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/lib.rs), [Contratos de contenido](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/payload.rs), [Contratos de codificación y reescritura](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/encode.rs), [Autorización de archivos](file-security.md)

Estos traits de Rust son interfaces internas de código fuente. Un adaptador de producto se integra en el espacio de trabajo y en el enrutamiento explícito de ejecución. Sus operaciones permitidas necesitan pruebas separadas de archivos válidos, funciones no compatibles, límites de recursos, corrupción y comportamiento de salida. Usa los [contratos de códecs Rust independientes](../../engineering/independent-rust-codecs.md) y la [guía de desarrollo](../contributing/README.md).

<a name="xdzip-implemented-external-sdk-trial"></a>

## Prueba implementada del SDK externo

La prueba acepta un manifiesto de paquete nativo seleccionado expresamente y una fuente elegida por el anfitrión. Permite analizar una muestra XDP1 propia y leer una entrada normal. La admisión valida el manifiesto versión 1, el destino ejecutable y su hash, rutas internas del paquete, perfil fijo de permisos y límites. El anfitrión ejecuta una copia privada verificada. El hash declarado demuestra coherencia del paquete; la autenticación del editor requeriría una política de confianza independiente.

El protocolo de prueba versión 1 usa CBOR acotado con prefijo de longitud y comprueba identidad y secuencia de operaciones. El anfitrión concede lecturas de la fuente y un receptor de salida mediante tokens opacos. El par recibe intervalos de bytes y crédito de salida de un bloque. El anfitrión mide y verifica contenido, comprueba finalización del protocolo y estado de la fuente y publica sin sobrescribir solo después de que el par termine correctamente. La muestra no expone al par contraseñas del archivo, volúmenes divididos, creación, actualizaciones, enlaces ni rutas del anfitrión.

La prueba tiene requisitos nativos explícitos de permisos en macOS y Linux, con requisitos por plataforma y rechazo ante fallos definidos en su contrato. Windows rechaza la ejecución. Las pruebas nativas de interoperabilidad y denegación ejercitan estos límites independientemente del escritorio. Consulta el [contrato de ejecución y protocolo de la prueba](../engineering/sdk-trial.md) para el esquema, presupuestos y entradas de prueba reales. [Anfitrión de prueba](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-plugin-host/src/lib.rs), [Implementación del encuadre](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-plugin-protocol/src/lib.rs)

<a name="xdzip-design-options-outside-the-current-implementation"></a>

## Opciones de diseño fuera de la implementación actual

Un gestor de extensiones del producto, confianza del editor y política de actualización, compatibilidad más allá del protocolo de prueba y capacidades externas de escritura o contraseña siguen siendo trabajo de diseño. La admisión al producto requeriría verificar restricciones de permisos, interoperabilidad y comportamiento ante fallos para cada plataforma y capacidad admitida. El proceso incluido y el experimento XDP1 no activan esta función. [Decisiones de arquitectura](architecture-decisions.md)
