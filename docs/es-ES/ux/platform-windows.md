<a name="xdzip-platform-windows-and-title-bars"></a>

# Ventanas de plataforma y barras de título

[English](../../ux/platform-windows.md) | [简体中文](../../zh-CN/ux/platform-windows.md) | [繁體中文](../../zh-TW/ux/platform-windows.md) | [日本語](../../ja-JP/ux/platform-windows.md) | [한국어](../../ko-KR/ux/platform-windows.md) | [Français](../../fr-FR/ux/platform-windows.md) | [Deutsch](../../de-DE/ux/platform-windows.md) | Español | [Português (Brasil)](../../pt-BR/ux/platform-windows.md) | [Русский](../../ru-RU/ux/platform-windows.md)

Las interfaces de operación de xdzip usan ventanas independientes de primer nivel, no hojas adjuntas a la ventana principal ni paneles superpuestos dibujados dentro de ella. Una ventana independiente no implica un proceso separado ni una entrada en la barra de tareas para cada ventana auxiliar. Los menús desplegables y las ayudas emergentes siguen siendo Popups ligeros y no son ventanas de operación.

<a name="xdzip-window-contract"></a>

## Contrato de ventanas

| Ventana | Relación con la ventana principal | Cerrar / Escape | Minimizar / maximizar |
| --- | --- | --- | --- |
| Formulario de extracción o creación | `OperationWindow` no modal con propietario transitorio si se abre desde la navegación | Cerrar el borrador o la confirmación sin enviar una tarea nueva | Desactivado / desactivado |
| Aviso de conflicto de tarea | Aviso no modal propiedad del anfitrión de la ventana de operación. La tarea espera una decisión | Cerrar el aviso. Reabrirlo desde la tarea o cancelarla explícitamente allí | Desactivado / desactivado |
| Aviso de contraseña, salida o reinicio | Modal para la aplicación. El selector nativo de archivos también es modal para la aplicación y tiene un llamador transitorio | Contraseña, salida y reinicio tienen acciones explícitas de cierre o confirmación | Según las capacidades del diálogo nativo |
| Progreso y resultado de tarea | `TaskWindow` no modal. Se puede seguir navegando si no hay otra ventana modal activa | Cerrar los detalles y reabrirlos desde la entrada de la tarea. Una operación independiente conserva acceso a las tareas activas | Minimizar / desactivado |
| Configuración y Acerca de | No modal. Una entrada de la bandeja puede abrirla sin ventana de navegación propietaria | Cerrar la vista. La configuración guardada sigue activa | Desactivado / desactivado |

Cancelar una tarea es una orden explícita independiente del cierre de su ventana de progreso. Al terminar en segundo plano no roba el foco de primer plano; se actualizan los detalles abiertos o la entrada de la tarea. La salida de la aplicación sigue la política de salida de tareas; cerrar la ventana principal no puede traducirse directamente en cerrar las ventanas auxiliares.

<a name="xdzip-platform-appearance"></a>

## Apariencia por plataforma

- macOS: usar una barra de título con decoración del sistema y controles rojo, amarillo y verde arriba a la izquierda; no sustituir los controles nativos por puntos de colores personalizados. Las ventanas de operación de tamaño fijo conservan el cierre rojo y desactivan minimizar o ampliar si no están disponibles; las ventanas de progreso pueden minimizarse. La disponibilidad real depende de las capacidades de ventana AppKit/Qt y se acepta en un dispositivo real. La ventana principal conserva el cambio de tamaño, minimización y pantalla completa normales.
- Windows usa una barra de título principal extendida al área cliente con botones QML, un marco nativo redimensionable y sombra DWM. El arrastre, el ajuste de tamaño por bordes y el acoplamiento del sistema usan funciones de la plataforma. Las ventanas de operación independientes usan decoración nativa. Consulta los diseños principales de Windows y Ubuntu.
- Ubuntu X11 usa controles integrados a la derecha y contorno redondeado en estado normal. Las sesiones con composición y conformación del área de entrada también muestran una sombra suave que deja pasar los clics. Las ventanas maximizadas y a pantalla completa tienen bordes rectos y ocupan el área disponible. El sistema de ventanas gestiona el movimiento y el cambio de tamaño. Las ventanas de operación independientes conservan la decoración del sistema. Otros backends fuera de la ruta X11 personalizada usan decoración normal. Wayland queda fuera del alcance compatible. Consulta el diseño principal de Ubuntu.

Cada ventana de operación tiene título, borde, sombra y barra de título arrastrable propios; la ventana principal detrás no se oscurece ni se desenfoca como una página web. Al abrirse por primera vez se centra en el área disponible de la pantalla de su propietario. Las posiciones restauradas deben ajustarse a las pantallas visibles. Las barras de herramientas y el contenido no deben cubrir la barra de título.

<a name="xdzip-qt-quick-implementation-constraints"></a>

## Restricciones de implementación de Qt Quick

`OperationWindow` usa una `Window` independiente con `transientParent` opcional y `Qt.NonModal` por defecto. `PasswordWindow` y `OperationConfirmationWindow` usan explícitamente `Qt.ApplicationModal`. El selector nativo toma la ventana que llama realmente como propietario transitorio y solo restaura su activación mientras siga visible. Consulta el [registro de ventanas](../../../specs/visual/window-registry.json) y las [ventanas de tareas](task-windows.md).

Las barras de título personalizadas de Windows/Linux necesitan movimiento, cambio de tamaño y menú del sistema reales de la plataforma; no se debe simular el arrastre cambiando manualmente posiciones QML. `WindowController` gestiona el ciclo de vida, la propiedad, las solicitudes de cierre y la reapertura de vistas de tareas. El núcleo posee las tareas; destruir una vista no puede destruir una tarea activa. Las actualizaciones de estado entre hilos solo usan eventos de puente definidos.

<a name="xdzip-prototype-evidence-and-limits"></a>

## Evidencia y límites del prototipo

El prototipo HTML simula el aspecto de ventanas independientes, el arrastre de la barra de título, las interacciones bloqueantes o no bloqueantes y la semántica de cierre. El selector superior «Platform» compara macOS, Windows y Linux y no es un ajuste del producto. El `dialog` del navegador permanece en una página y no ofrece ventanas nativas reales, botones del sistema, minimización del sistema, barra de tareas ni comportamiento multipantalla. La minimización de una ventana de tarea se simula ocultando los detalles.

Los controles de plataforma de la ventana principal del prototipo ilustran su aspecto. Al volver de la confirmación de sobrescritura a la creación aún se reinicia el formulario de ejemplo; la implementación del producto debe conservar el borrador. Estas limitaciones no son prueba de aceptación nativa completada.

<a name="xdzip-real-device-acceptance-criteria"></a>

## Criterios de aceptación en dispositivo real

1. Cada ventana de operación es una ventana de plataforma independiente correctamente asociada con su ventana principal, sin hoja ni superposición de página web.
2. El orden de los controles de colores o botones de la derecha, su estado desactivado y las solicitudes de cierre corresponden a las capacidades de plataforma y ventana; títulos y botones del sistema no se solapan con distintos tamaños de letra y escalas.
3. Funcionan correctamente el arrastre, los límites de pantalla, varias pantallas, las escalas 125%/150%/200% y los temas claro y oscuro.
4. Cerrar los detalles deja la tarea en marcha y su entrada permite reabrirlos. Cerrar una ventana de conflicto deja la tarea esperando hasta reabrir el aviso o cancelarla explícitamente.
5. Funcionan la modalidad del propietario, restauración del foco, navegación por teclado, títulos para lectores de pantalla y accesos directos del sistema; otras ventanas de archivos comprimidos no quedan bloqueadas innecesariamente.
6. Verificar por separado el menú y acoplamiento de Windows y Linux X11; Wayland queda fuera del soporte inicial. Las conclusiones Qt existentes en dispositivo real están en [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5). Esta lista sirve para volver a comprobar los puntos afectados y no reinicia la aceptación ya completada.
