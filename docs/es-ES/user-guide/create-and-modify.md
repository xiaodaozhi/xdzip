# Crear y modificar archivos comprimidos

[English](../../user-guide/create-and-modify.md) | [简体中文](../../zh-CN/user-guide/create-and-modify.md) | [繁體中文（台灣）](../../zh-TW/user-guide/create-and-modify.md) | [日本語](../../ja-JP/user-guide/create-and-modify.md) | [한국어](../../ko-KR/user-guide/create-and-modify.md) | [Français](../../fr-FR/user-guide/create-and-modify.md) | [Deutsch](../../de-DE/user-guide/create-and-modify.md) | Español | [Português (Brasil)](../../pt-BR/user-guide/create-and-modify.md) | [Русский](../../ru-RU/user-guide/create-and-modify.md)

<a name="xdzip-create-an-archive"></a>

## Crear un archivo comprimido

1. Abre la ventana de creación desde la página de bienvenida o con **Nuevo**. Añade archivos o carpetas y revisa la lista de orígenes.
2. Indica el nombre y el destino. Elige un formato y una de sus opciones de compresión. ZIP es habitual para compartir. Consulta la [tabla de formatos](../../../README.es-ES.md#xdzip-format-support) para ver las opciones de creación. Un flujo de archivo único como GZIP solo admite un archivo normal.
3. ZIP y 7z permiten usar cifrado y volúmenes divididos cuando las opciones seleccionadas lo admiten. Elige ISO para una imagen de disco de datos o el perfil SFX de Windows x64 para un ZIP autoextraíble. Inicia la operación y consulta el resultado para saber qué archivos se guardaron. [Contraseñas y volúmenes](passwords-and-volumes.md)
4. Si tienes que cancelar o repetir la operación, revisa primero el resultado. El archivo comprimido se prepara en un área temporal antes de publicar el resultado terminado.

<a name="xdzip-add-replace-or-move-files"></a>

## Añadir, sustituir o mover archivos

Abre un archivo ZIP, 7z, TAR o TAR.GZ compatible y ve a la carpeta de destino dentro de él. Arrastra archivos o carpetas locales a la lista o cuadrícula, revisa la selección y confirma la adición. Entre las opciones para nombres en conflicto está sustituir los archivos normales que lo permitan. Para mover una entrada existente, arrástrala a otra carpeta del mismo archivo comprimido y confirma. Si la arrastras fuera de la ventana, exportas una copia. [Arrastrar y soltar según la plataforma](../engineering/archive-transfers.md)

Al crear y añadir se omiten los archivos `.DS_Store` y los directorios `__MACOSX` y `_MACOSX`. Las posibilidades de modificación dependen del formato, contenido y origen. Los archivos comprimidos cifrados, divididos o con índice en disco no se pueden modificar por esta vía. Si no se puede editar, extrae el contenido en una carpeta nueva y crea otro archivo comprimido. [Reglas de modificación y arrastre](../engineering/archive-transfers.md)

<a name="xdzip-rename-or-delete-entries"></a>

## Cambiar el nombre de las entradas o eliminarlas

En una carpeta normal de un archivo comprimido editable, haz clic con el botón derecho en un archivo o carpeta de la lista o cuadrícula. Si el elemento ya estaba seleccionado, se conserva la selección. Si no lo estaba, solo se selecciona ese elemento.

- **Cambiar nombre** está disponible para un solo elemento. Escribe el nombre nuevo en la misma carpeta. Al cambiar el nombre de una carpeta, se actualizan todas las rutas de su contenido. Si ya existe el nombre de destino, elige otro.
- **Eliminar** admite uno o más elementos seleccionados. Antes de confirmar, comprueba el nombre o el número de elementos. Al eliminar una carpeta, también se elimina todo su contenido. Si eliminas todos los elementos de la raíz, el archivo comprimido sigue disponible como archivo vacío.

Para editar un resultado de la búsqueda, abre primero la carpeta que lo contiene. [Reglas de modificación de archivos comprimidos](../engineering/archive-transfers.md)
