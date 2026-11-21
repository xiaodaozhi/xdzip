# Configuración y actualizaciones

[English](../../user-guide/settings-and-updates.md) | [简体中文](../../zh-CN/user-guide/settings-and-updates.md) | [繁體中文](../../zh-TW/user-guide/settings-and-updates.md) | [日本語](../../ja-JP/user-guide/settings-and-updates.md) | [한국어](../../ko-KR/user-guide/settings-and-updates.md) | [Français](../../fr-FR/user-guide/settings-and-updates.md) | [Deutsch](../../de-DE/user-guide/settings-and-updates.md) | Español | [Português (Brasil)](../../pt-BR/user-guide/settings-and-updates.md) | [Русский](../../ru-RU/user-guide/settings-and-updates.md)

<a name="xdzip-set-up-your-workspace"></a>

## Configurar el espacio de trabajo

Abre **Configuración** desde el menú de la aplicación o la barra de herramientas. Elige el idioma, el tema, el tamaño del texto y la densidad de la interfaz en los ajustes correspondientes. La opción automática usa el idioma del sistema si es compatible. Elige si los archivos comprimidos se abren en pestañas o ventanas. Puedes ajustar los archivos recientes y el icono de notificación si tu plataforma lo permite. Los cambios se aplican al momento. Si no se puede guardar un ajuste, consulta el error que aparece en la página. [Funcionamiento de la configuración](../ux/settings-localization.md)

Cambia las asociaciones de archivos en el sistema operativo. Configura las acciones del menú contextual por separado. En macOS, la activación de la extensión de Finder se gestiona desde el sistema. Los servicios de Finder están disponibles si están registrados. [Acciones del gestor de archivos](../engineering/shell-actions.md)

<a name="xdzip-update-the-app"></a>

## Actualizar la aplicación

- **Edición de Microsoft Store.** Actualiza desde Microsoft Store. Si la aplicación se ejecuta con identidad de paquete MSIX, su página de información no tiene controles de actualización de GitHub.
- **Instalaciones de GitHub.** Abre **Acerca de xdzip** y busca actualizaciones. La instalación desde la aplicación requiere una instalación de Windows compatible por usuario, una copia compatible de macOS en Aplicaciones o una instalación DEB en Ubuntu. También requiere un paquete de versión verificado. Windows puede pedir confirmación del instalador y Ubuntu utiliza la autorización del sistema. Si hay tareas en curso que retrasan la instalación, espera a que terminen o cancela la espera.
- **Versiones portátiles o de desarrollo.** Descarga el paquete correspondiente. En Windows y Linux, extrae el ZIP o TAR.GZ nuevo completo en una carpeta separada. En macOS, cierra la aplicación anterior antes de sustituirla en Aplicaciones. Consulta los [pasos de instalación](install.md).

La búsqueda de actualizaciones de GitHub se inicia manualmente y usa la configuración de proxy compatible. Si solo se ofrece la descarga manual, usa el enlace a la versión y verifica el paquete. Una vez descargado, todavía hay que instalarlo. Tras actualizar, abre de nuevo **Acerca de xdzip** para comprobar la versión en ejecución. Microsoft Store y GitHub publican de forma independiente. [Funcionamiento y recuperación de actualizaciones](../engineering/software-update.md)
