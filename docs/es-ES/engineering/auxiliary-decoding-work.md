<a name="xdzip-auxiliary-algorithm-work-accounting"></a>

# Contabilización del trabajo auxiliar de los algoritmos

[English](../../engineering/auxiliary-decoding-work.md) | [简体中文](../../zh-CN/engineering/auxiliary-decoding-work.md) | [繁體中文（台灣）](../../zh-TW/engineering/auxiliary-decoding-work.md) | [日本語](../../ja-JP/engineering/auxiliary-decoding-work.md) | [한국어](../../ko-KR/engineering/auxiliary-decoding-work.md) | [Français](../../fr-FR/engineering/auxiliary-decoding-work.md) | [Deutsch](../../de-DE/engineering/auxiliary-decoding-work.md) | Español | [Português (Brasil)](../../pt-BR/engineering/auxiliary-decoding-work.md) | [Русский](../../ru-RU/engineering/auxiliary-decoding-work.md)

La extracción y la comprobación acumulan por separado los bytes de archivo decodificados, la salida real y el trabajo auxiliar. Inicializar el modelo PPM, reiniciarlo por capacidad y decodificar controles o parámetros quedan fuera del contenido de los archivos. Esas operaciones no deben aumentar `decoded_bytes`, `written_bytes`, el progreso de archivo ni la relación de expansión, ni crear crédito de entrega anticipada. Las instantáneas de tareas no añaden un contador auxiliar visible.

`DecodedSink::charge_auxiliary(bytes)` informa solo del trabajo auxiliar ya realizado. La implementación predeterminada deja intactos los contadores de archivo. Los algoritmos admiten primero el trabajo previsto en su presupuesto restante y después contabilizan el trabajo real, incluido el completado antes de un fallo o una cancelación. Las reservas sin ejecutar no cuentan como trabajo real. Un error, el reinicio del modelo o un nuevo intento de contraseña no devuelve trabajo ya realizado. El presupuesto privado de RAR sigue limitando la suma de decodificación de archivos y trabajo auxiliar.

<a name="xdzip-isolation-boundary"></a>

## Límite de aislamiento

- `AUX_WORK=24` transporta un contador `u64` little-endian y ningún dato de contenido. Cada paquete contiene un valor de 1 a 65 536 y la trama debe terminar justo allí. El host responde con un `ACK` vacío tras contabilizarlo. Rechaza ceros, truncamientos, campos finales, valores excesivos y fases incorrectas.
- Este mensaje solo se permite al decodificar datos de RAR. No se usa al examinar metadatos, crear, modificar ni con otros formatos. CXX pasa los contadores de forma síncrona y no guarda punteros Rust. Las porciones de entrada y salida se toman prestadas solo para la llamada. Las excepciones de C++ se traducen a errores estables dentro de las devoluciones de llamada. El desenrollado de pila de Rust no atraviesa CXX y un aborto o salida del proceso hace fallar la salida incompleta.
- Una instancia nativa de decodificación acumula hasta 20 GiB de trabajo auxiliar y mantiene el total entre entradas, reinicios y cambios de contraseña. Registra el trabajo terminado antes de llamar a la salida del host. El rechazo del receptor no devuelve el consumo en un reintento. `AUX_WORK` no aumenta `WorkerCredit`.
- Las tareas de extracción y comprobación del entorno de ejecución acumulan de forma independiente hasta 20 GiB de trabajo auxiliar. El objeto contable queda fuera de los bucles de entradas y reintentos de contraseña. El límite es independiente de los bytes reales de archivo y de las cuotas de salida. Superarlo o cancelar devuelve un fallo y evita publicar archivos incompletos. Cambiar contadores públicos no permite eludir los límites.
- `WORK=17` sigue representando bytes decodificados de archivo. Solo la admisión correcta da crédito de entrega de un solo uso para `PRECHARGED_DATA=23` al mismo proceso válido. El crédito se deduce antes de la entrega y se borra al reiniciar. RAR, 7z y CAB siguen este camino. Ni el examen ni la creación lo usan. El trabajo auxiliar no otorga crédito para la salida de archivos.

La admisión de ratios altos usa los tamaños declarados y los tamaños comprimidos o del archivo comprimido completo que quedaron fijados. Los límites de decodificación y salida se aplican de forma independiente. Si un archivo de 87 bytes requiere varios MiB de inicialización del modelo, los contadores de archivo siguen reflejando solo sus 87 bytes reales.

<a name="xdzip-required-acceptance-conditions"></a>

## Condiciones de aceptación

- Un archivo pequeño normal puede completar la extracción y la comprobación tras más de 64 KiB de trabajo auxiliar e inicialización del modelo. Los recuentos son exactos y la verificación final del contenido tiene éxito.
- La capa nativa y el entorno de ejecución verifican el rechazo cuando los paquetes auxiliares reales repetidos alcanzan 20 GiB. Los reintentos de contraseña y la sustitución del proceso no reinician el coste acumulado. Los archivos incompletos no se guardan de forma definitiva.
- Se rechazan las solicitudes de un proceso simulado para otros formatos, fases de metadatos, paquetes vacíos o demasiado grandes, campos finales y salidas cuyo único crédito provenga del trabajo auxiliar.
- `WORK` normal puede dar crédito para salida RAR precontabilizada del mismo proceso. Se consume una vez y no se reutiliza después de un error o reinicio.
- Las pruebas del protocolo comprueban la máquina de estados y la contabilidad. El proceso RAR completo en Rust, las tareas reales del entorno de ejecución, las tres plataformas y la aceptación de escritorio requieren pruebas propias.

La prueba de protocolo solo compila el `tools/native-codec/main.cpp` del proyecto y su simulador, sin algoritmo de compresión.

```sh
cmake -S crates/xdzip-codec-native/tests -B build/auxiliary-protocol
cmake --build build/auxiliary-protocol --config Release
```

Apunta `XDZIP_NATIVE_CODEC_WORKER` a la prueba, establece `XDZIP_AUXILIARY_PROTOCOL_FIXTURE=1` y ejecuta explícitamente `cargo test -p xdzip-codec-native auxiliary_protocol -- --ignored`. La regresión normal dirigida usa `cargo test -p xdzip-codec-native -p xdzip-runtime auxiliary`. Las pruebas de la API de códecs comprueban la compatibilidad del receptor predeterminado. Las evidencias generadas quedan en build o CI. Los resultados y puntos pendientes se registran en la [tarea de migración #51](https://github.com/xiaodaozhi/xdzip/issues/51).
