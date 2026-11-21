<a name="xdzip-simplified-chinese-ui-terminology"></a>

# Terminología de la interfaz en chino simplificado

[English](../../ux/chinese-terminology.md) | [简体中文](../../zh-CN/ux/chinese-terminology.md) | [繁體中文](../../zh-TW/ux/chinese-terminology.md) | [日本語](../../ja-JP/ux/chinese-terminology.md) | [한국어](../../ko-KR/ux/chinese-terminology.md) | [Français](../../fr-FR/ux/chinese-terminology.md) | [Deutsch](../../de-DE/ux/chinese-terminology.md) | Español | [Português (Brasil)](../../pt-BR/ux/chinese-terminology.md) | [Русский](../../ru-RU/ux/chinese-terminology.md)

Los textos para usuarios en chino simplificado siguen la [configuración terminológica](../../../specs/ux/chinese-terminology.json). Los textos estáticos de Qt conservan sus identificadores estables en resources/i18n/zh-CN.json. El HTML, los diseños actuales de alta fidelidad y la documentación de usuario usan los términos de la tabla. Los documentos técnicos y de formatos pueden usar sus propios términos. Se conservan las dimensiones de los componentes, la densidad de 0.8 en la ventana principal y 1 en el menú, el DPR y los parámetros de escala.

| Significado | Término normalizado en chino simplificado | Uso |
| --- | --- | --- |
| Contenedor como ZIP o TAR | 压缩包 | 压缩包名称, 空压缩包, 压缩包信息 |
| Nombres completos de las acciones | 打开压缩包, 创建压缩包, 解压 | Abreviar como «打开» y «新建» cuando esté claro el contexto de la barra de herramientas |
| Elementos en el archivo comprimido | 文件, 文件夹, 内容 | 所选文件和文件夹, 全部内容 |
| Propiedades de archivo | 原始大小, 压缩大小 | Usar los mismos nombres en cabeceras, propiedades de la vista previa y menús de ordenación |
| Restaurar el orden físico de las entradas | 按原始顺序排列 | Distinguir de la ordenación por nombre o tamaño |

«档案文件» queda fuera de la terminología del producto. «归档», «归档文件» y «归档格式» se reservan para especificaciones técnicas, descripciones de formatos o API públicas. La interfaz usa «压缩包» incluso cuando TAR solo empaqueta los archivos. El formato y el nivel indican la capacidad de compresión.

«压缩文件» solo describe una acción, como en «正在压缩文件». La propiedad se llama «压缩大小» y se retira la etiqueta «压缩后». La ventana de creación propone «新建压缩包.zip» y conserva los nombres de archivos existentes.

Conserva los nombres y rutas aportados por el usuario, los nombres internos de los archivos comprimidos, los campos de protocolos, los identificadores de código y las imágenes históricas de los diseños. El inglés usa «archive» según su gramática y «All contents» para «全部内容». En cabeceras estrechas, «Compressed» abrevia «compressed size». Mantén los identificadores del código en inglés.

Consulta el diseño de estados de la terminología. Las imágenes originales, los hashes y las mediciones de la superficie visible del texto quedan como evidencia histórica. El script de medición del contenido valida los textos nuevos mediante correspondencias explícitas con las etiquetas anteriores y conserva los objetivos geométricos y las tolerancias. Comprueba que los textos largos se vean completos en la ventana principal, las cabeceras, la vista previa y los menús.

Ejecuta `python3 scripts/check-ui-i18n.py` para comprobar todos los recursos de idioma del producto y la terminología en chino simplificado. La comprobación terminológica se aplica a los textos estáticos. Admite el uso verbal autorizado de «压缩文件» y deja fuera el contenido del usuario y los términos técnicos.

Los gestores de archivos conservan «添加到压缩文件...» y «解压每个压缩文件到单独的文件夹», confirmados en [#270](https://github.com/xiaodaozhi/xdzip/issues/270). La configuración solo aplica la excepción al texto completo de esos dos identificadores estables. En el resto de la interfaz se mantienen los términos establecidos para los contenedores.
