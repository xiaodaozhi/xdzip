<a name="xdzip-host-calendar-and-encoding-services-for-format-workers"></a>
# Servicios de calendario y codificación del sistema anfitrión

[English](../../engineering/host-platform-services.md) | [简体中文](../../zh-CN/engineering/host-platform-services.md) | [繁體中文（台灣）](../../zh-TW/engineering/host-platform-services.md) | [日本語](../../ja-JP/engineering/host-platform-services.md) | [한국어](../../ko-KR/engineering/host-platform-services.md) | [Français](../../fr-FR/engineering/host-platform-services.md) | [Deutsch](../../de-DE/engineering/host-platform-services.md) | Español | [Português (Brasil)](../../pt-BR/engineering/host-platform-services.md) | [Русский](../../ru-RU/engineering/host-platform-services.md)

Rust procesa los campos del formato, las reglas de compatibilidad de contraseñas y la validación. Las reglas históricas de zona horaria del sistema, las páginas de códigos OEM y ANSI y las codificaciones locales de C pertenecen al anfitrión. Se exponen mediante servicios opcionales de `ReadSource`. La capa de formato no debe deducirlas del nombre de archivo, HostOS ni del desfase UTC actual. Estos servicios no dependen de Qt ni conceden acceso a rutas ni autorización para publicar archivos.

<a name="xdzip-calendar-projection"></a>

## Conversión a fecha y hora

`Timestamp` distingue los valores FILETIME en unidades de 100 ns desde 1601 de los segundos enteros Unix sin signo desde 1970. `ReadSource::local_calendar` devuelve un `Calendar` opcional. Los archivos normales y las fuentes en memoria usan UTC de forma determinista por defecto. El proceso de trabajo del producto usa las reglas locales del sistema mediante llamadas CXX síncronas. Windows usa las API de zona horaria del sistema y Unix usa `localtime_r`. Ambos comprueban el resultado. Si no es posible convertir el valor, no se devuelve calendario y los campos sin inicializar no deben leerse.

Rust valida año, mes, día, hora, minuto y segundo antes de codificar un valor de visualización DOS. 7z conserva UTC y el intervalo de visualización 1980..2107. RAR aplica sus [reglas fijas de compatibilidad](rar-format-sources.md) para la aritmética modular interna, las sustituciones de tiempos y las diferencias entre plataformas. Después usa el calendario del anfitrión y el ajuste circular histórico del año. La precisión original, las marcas de ausencia y las cabeceras físicas completas se conservan en el índice privado del formato. La visualización no los sobrescribe. Si un tiempo no se puede convertir, la regla segura de RAR muestra 0. La implementación de referencia puede copiar datos indefinidos si el sistema falla y no se comprueba ese fallo.

<a name="xdzip-controlled-encoding-mappings"></a>

## Conversiones de codificación controladas

| Servicio | Entrada y salida | Límite del anfitrión |
| --- | --- | --- |
| `oem_to_ansi_wide` | 512 unidades UTF-16 a 512 unidades UTF-16, incluido NUL | Conversión completa de Windows a través de OEM y ANSI con las correspondencias y sustituciones reales del sistema |
| `wide_to_ansi` | Hasta 512 unidades UTF-16, incluido NUL, a 512 bytes | Página de códigos ANSI actual de Windows. No se presupone un carácter universal de sustitución |
| `wide_to_local_prefix` | Hasta 128 valores escalares anchos, incluido NUL, a 512 bytes | Codificación C actual de Unix. Si hay un error, se borra y se reintenta según la política fija y se conserva el prefijo convertido |

Las longitudes devueltas excluyen NUL. Las implementaciones sin servicios del anfitrión devuelven `None`. Los llamadores de la capa de formato solo pueden aplicar una alternativa portable definida o rechazar la conversión explícitamente. No deben adivinar la página de códigos. Las llamadas del producto validan dimensiones, NUL e intervalos devueltos. Una conversión no disponible devuelve la categoría estable Unsupported.

El parámetro de longitud de [CharToOemBuffW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-chartooembuffw) cuenta caracteres anchos de entrada y sigue convirtiendo relleno después de NUL. No indica la capacidad del destino en bytes. El anfitrión usa `GetCPInfo(CP_OEMCP)` para admitir únicamente páginas de hasta cuatro bytes por unidad. A continuación usa un búfer de 2048 bytes para convertir las 512 unidades. Solo después fija el byte 511 a NUL y convierte a ANSI. Si no se puede garantizar la capacidad, se rechaza la conversión. No se deben convertir 512 caracteres anchos en un destino de 512 bytes ni recortar antes la entrada para eludir ese requisito. Las comprobaciones con el sistema cubren todo el límite DBCS de 511 unidades y lo comparan independientemente con una llamada directa a WinAPI.

La admisión de 512 unidades, el primer NUL, las condiciones de compensación OEM, el truncamiento de 127 unidades y por bytes, el protocolo de marcadores antiguos y el uso explícito de UTF8 en macOS son reglas de compatibilidad en Rust. En particular, no se debe truncar una contraseña antes de una conversión del sistema que originalmente se aplica primero, ni cambiar arbitrariamente un corte dentro de una secuencia DBCS a un corte entre caracteres.

<a name="xdzip-ownership-and-failure"></a>

## Propiedad y fallos

Las porciones de memoria se toman prestadas solo durante la llamada síncrona. C++ no guarda punteros. Rust controla cuándo se borran las entradas y salidas secretas. Los búferes temporales acotados de C++, anchos o estrechos, se borran en todas las salidas. No se garantiza el borrado dentro del sistema operativo ni de todas las copias históricas en memoria. Las primitivas no registran contraseñas ni las añaden a los argumentos del proceso ni a las variables de entorno.

Los lectores comprueban si hay una cancelación antes de usar los servicios. Sus resultados no cuentan como salida de extracción ni alteran los contadores de bytes leídos, decodificados o escritos. Los fallos conservan las categorías de origen, cancelación y cuota y no deben confundirse con errores de contraseña. Solo la capa de formato puede determinar la ambigüedad entre contraseña e integridad, a partir del estado real del verificador.
