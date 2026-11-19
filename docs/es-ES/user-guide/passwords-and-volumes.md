# Contraseñas y volúmenes divididos

[English](../../user-guide/passwords-and-volumes.md) | [简体中文](../../zh-CN/user-guide/passwords-and-volumes.md) | [繁體中文（台灣）](../../zh-TW/user-guide/passwords-and-volumes.md) | [日本語](../../ja-JP/user-guide/passwords-and-volumes.md) | [한국어](../../ko-KR/user-guide/passwords-and-volumes.md) | [Français](../../fr-FR/user-guide/passwords-and-volumes.md) | [Deutsch](../../de-DE/user-guide/passwords-and-volumes.md) | Español | [Português (Brasil)](../../pt-BR/user-guide/passwords-and-volumes.md) | [Русский](../../ru-RU/user-guide/passwords-and-volumes.md)

<a name="xdzip-passwords"></a>

## Contraseñas

AES Crypt y los archivos comprimidos que cifran los nombres, como 7z con cifrado de nombres, requieren una contraseña para mostrar la lista de archivos. Otros pueden pedirla al extraer, verificar o previsualizar. Respeta los espacios y las mayúsculas y minúsculas al introducirla. Se admiten los perfiles compatibles de ZIP AES-256. ZIP AES-128/192 y ZipCrypto tradicional no se admiten. Si la contraseña es correcta y sigue fallando, comprueba si se admite el cifrado y si el archivo está dañado. [Cifrado compatible](../engineering/password-extraction.md)

Para crear un archivo comprimido protegido por contraseña, elige un formato con cifrado. ZIP AES-256 protege el contenido y deja visibles los nombres. Si necesitas proteger también la lista, elige 7z y activa el cifrado de nombres. Guarda la contraseña en un lugar distinto del archivo.

<a name="xdzip-split-volumes"></a>

## Volúmenes divididos

1. Coloca todos los volúmenes del mismo archivo comprimido en una carpeta y conserva sus nombres originales.
2. Abre un volumen compatible. XDZip busca los demás en la misma carpeta. Los volúmenes numerados de ZIP y 7z, PKZIP y los conjuntos compatibles de RAR y CAB siguen convenciones de nombres diferentes.
3. Si el conjunto está incompleto, consigue las partes que faltan y abre de nuevo el archivo comprimido. Unir o renombrar las partes a mano puede hacer que ya no se puedan leer.

Para crear un archivo dividido, elige ZIP o 7z y establece el tamaño del volumen en la ventana de creación. Comparte el conjunto completo. Dividir un archivo comprimido no permite editarlo en XDZip. [Nombres de volúmenes y perfiles compatibles](../engineering/archive-opening.md)
