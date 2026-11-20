<a name="xdzip-xdzip-product-requirements"></a>

# Requisitos del producto XDZip

[English](../../design/product-requirements.md) | [简体中文](../../zh-CN/design/product-requirements.md) | [繁體中文](../../zh-TW/design/product-requirements.md) | [日本語](../../ja-JP/design/product-requirements.md) | [한국어](../../ko-KR/design/product-requirements.md) | [Français](../../fr-FR/design/product-requirements.md) | [Deutsch](../../de-DE/design/product-requirements.md) | Español | [Português (Brasil)](../../pt-BR/design/product-requirements.md) | [Русский](../../ru-RU/design/product-requirements.md)

XDZip permite explorar archivos comprimidos como carpetas, trabajar con el contenido seleccionado y crear o editar archivos cuando el formato lo permite. El escritorio y la [CLI independiente](../engineering/command-line.md) comparten el entorno de ejecución. El [README del proyecto](../../../README.es-ES.md) describe las capacidades y distribuciones actuales. Los siguientes requisitos definen el comportamiento que hay que conservar y verificar.

<a name="xdzip-requirements-and-implementation"></a>

## Requisitos e implementación

| ID | Requisito | Comportamiento o alcance actual |
| --- | --- | --- |
| P01 | Explorar sin extraer | La apertura identifica la fuente y termina un análisis acotado antes de publicar la primera página. Decodificar el contenido de una entrada requiere otra acción. |
| P02 | Navegar, buscar, ordenar y seleccionar | Están implementados los identificadores físicos estables, identidades de navegación separadas, consultas paginadas y selección acotada. ZIP/ZIP64 normal de un solo archivo tiene un índice de metadatos mayor y límites de tarea independientes. |
| P03 | Extraer el ámbito previsto | El entorno de ejecución fija el alcance, valida los nombres y autoriza un destino. Gestiona conflictos y notifica los archivos guardados, omisiones, errores y cancelación. |
| P04 | Crear archivos comprimidos a partir de archivos y carpetas | Las opciones se validan frente al registro de creación. La salida se prepara y verifica antes de guardarse. Los formatos y opciones compatibles dependen del registro. |
| P05 | Respetar las capacidades de los formatos | La interfaz combina el descriptor de formato con el estado de la sesión. El entorno de ejecución valida cada solicitud. Lectura, vista previa, comprobación, creación, contraseñas y volúmenes tienen límites propios. |
| P06 | Limitar tareas y permitir su cancelación | Los metadatos y las tareas pesadas usan colas separadas. Las operaciones pesadas del escritorio se ejecutan de una en una. Una tarea aceptada puede continuar después de cerrar su pestaña. |
| P07 | Limitar las escrituras a los permisos y límites verificados | El host controla nombres, enlaces, identidad, preparación temporal, salida real y escritura definitiva. El formato comprueba la estructura e integridad correspondientes. |
| P08 | Ofrecer controles de escritorio coherentes y accesibles | QML y C++ proporcionan controles compartidos, navegación por teclado, foco y accesibilidad. El IME, los lectores de pantalla y la visualización nativos requieren verificaciones propias. |
| P09 | Limitar y medir recursos | El código aplica presupuestos independientes. Los objetivos de rendimiento y los resultados medidos se registran por separado. |
| P10 | Mantener Qt fuera de la lógica de formatos | Los traits de Rust, el enrutamiento explícito del registro y las interfaces de procesos dedicados permiten implementaciones integradas. El escritorio y la CLI usan el mismo núcleo. |
| P11 | Proteger secretos y datos | Las operaciones con archivos comprimidos funcionan sin conexión. Las contraseñas se guardan temporalmente en memoria borrable. Privacidad, diagnósticos y acceso explícito a actualizaciones siguen contratos propios. |
| P12 | Auditar la entrega | Usar canales de empaquetado definidos, entradas fijadas, manifiestos, sumas, licencias y comprobaciones de plataforma. Los resultados corresponden a la compilación y distribución realmente verificadas. |
| P13 | Permitir extensiones de terceros solo con confianza y permisos verificados | El ensayo XDP1 de solo lectura está implementado por separado. La instalación y activación de extensiones en el producto siguen en diseño. |
| P14 | Limitar vista previa y apertura | Con el panel de propiedades visible, seleccionar un archivo compatible inicia una vista previa acotada de texto o imagen. Abrir expresamente una entrada de archivo normal compatible usa una copia privada verificada y la asociación del sistema. |
| P15 | Informar con precisión los fallos y los resultados inciertos | Los errores estables y resultados acotados cubren cambios de origen, fallos de disco, entradas omitidas y limpieza. La recuperación automática tras reiniciar sigue siendo una opción de diseño. |

<a name="xdzip-workflows-and-scope"></a>

## Flujos y alcance

- **Abrir y examinar.** La solicitud llega al coordinador de sesiones. Un análisis correcto publica una sesión. La búsqueda y la ordenación crean consultas nuevas. Un doble clic abre un directorio o un archivo compatible mediante una copia temporal. Con el panel de propiedades visible, seleccionar un archivo compatible inicia su vista previa. [Exploración y vista previa](../user-guide/browse.md), [Apertura temporal](../engineering/archive-opening.md)
- **Extraer.** «Extraer todo» abarca el archivo comprimido completo. Para extraer solo coincidencias, selecciónalas en la consulta y revisa el alcance. La comprobación previa de metadatos informa antes de autorizar la escritura. La confirmación por archivo conserva los archivos completados ante fallos o cancelaciones posteriores. [Extracción](../../engineering/zip-safe-extraction.md), [Planificación de tareas](task-scheduling.md)
- **Crear, modificar o comprobar.** La creación y las ediciones admitidas verifican la salida temporal antes de guardarla. Comprobar revisa un archivo compatible según su formato sin obtener permiso de salida. Las operaciones dependen del formato, la entrada y el perfil de origen. [Registro de formatos](../engineering/format-registry.md), [Transferencias](../../engineering/archive-transfers.md), [Comprobación](../engineering/archive-checking.md)

<a name="xdzip-planned-scope-and-verification"></a>

## Alcance previsto y verificación

La gestión de extensiones externas, la recuperación persistente de tareas y nuevos perfiles de formato requieren diseño, implementación y verificación propios. El número de versión por sí solo no acredita cobertura de formatos o plataformas. La sincronización de archivos comprimidos en la nube, el montaje de imágenes de disco y un servicio de análisis de malware quedan fuera del producto. La [matriz de trazabilidad](requirements-traceability.md) relaciona requisitos y verificaciones. La [política de soporte](../engineering/support-policy.md) y las notas de versión describen las compilaciones distribuidas.
