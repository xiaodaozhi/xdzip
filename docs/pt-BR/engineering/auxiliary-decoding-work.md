<a name="xdzip-auxiliary-algorithm-work-accounting"></a>

# Contabilização do trabalho auxiliar de algoritmos

[English](../../engineering/auxiliary-decoding-work.md) | [简体中文](../../zh-CN/engineering/auxiliary-decoding-work.md) | [繁體中文](../../zh-TW/engineering/auxiliary-decoding-work.md) | [日本語](../../ja-JP/engineering/auxiliary-decoding-work.md) | [한국어](../../ko-KR/engineering/auxiliary-decoding-work.md) | [Français](../../fr-FR/engineering/auxiliary-decoding-work.md) | [Deutsch](../../de-DE/engineering/auxiliary-decoding-work.md) | [Español](../../es-ES/engineering/auxiliary-decoding-work.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/auxiliary-decoding-work.md)

A extração e a verificação acumulam separadamente os bytes de arquivos decodificados, a saída real e o trabalho auxiliar de algoritmos. A inicialização do modelo PPM, as reinicializações por capacidade e a decodificação de controles e parâmetros ficam fora do conteúdo dos arquivos. Elas não podem aumentar `decoded_bytes`, `written_bytes`, o progresso de arquivos ou a taxa de expansão, nem gerar crédito de entrega pré-contabilizada. Os instantâneos das tarefas não incluem um contador auxiliar visível ao usuário.

`DecodedSink::charge_auxiliary(bytes)` informa apenas o trabalho auxiliar já executado. A implementação padrão mantém os contadores de arquivos. Antes de executar, os algoritmos verificam se o trabalho previsto cabe no orçamento restante. Depois, contabilizam o trabalho real, inclusive o que terminou antes de falha ou cancelamento. Reservas não usadas não são informadas como trabalho real. Erro, reinicialização do modelo e novas tentativas de senha não devolvem trabalho já realizado. O orçamento privado de trabalho total do RAR continua limitando a soma da decodificação de arquivos com o trabalho auxiliar.

<a name="xdzip-isolation-boundary"></a>

## Limite de isolamento

- `AUX_WORK=24` transporta uma contagem `u64` little-endian sem conteúdo. Cada pacote contém um valor de 1 a 65.536, e o quadro precisa terminar exatamente ali. Após a contabilização bem-sucedida, o host responde com um `ACK` vazio. Rejeite zero, truncamento, campos no fim, contagens excessivas e fases inválidas.
- A mensagem é permitida apenas na decodificação de dados RAR. Não é usada na análise de metadados, criação, modificação ou em outros formatos. O CXX transmite contagens de forma síncrona e não retém ponteiros Rust. As fatias de origem e saída são emprestadas apenas durante a chamada. Exceções de C++ viram erros estáveis nos retornos de chamada. O desenrolamento da pilha Rust não atravessa o CXX, e interrupções abruptas ou a saída do processo fazem a saída incompleta falhar.
- Uma instância nativa de decodificação acumula até 20 GiB de trabalho auxiliar e mantém a soma entre entradas, reinicializações do processo e mudanças de senha. Registra o trabalho concluído antes de chamar o receptor do host. A recusa do receptor não devolve o custo em uma nova tentativa. `AUX_WORK` não aumenta `WorkerCredit`.
- As tarefas de extração e verificação do ambiente de execução acumulam de forma independente até 20 GiB de trabalho auxiliar. O objeto de contabilização fica fora dos laços de entradas e novas tentativas de senha. A cota é verificada separadamente dos bytes reais dos arquivos e dos limites de saída. Excesso ou cancelamento resulta em falha sem publicação de arquivos incompletos. Alterar contadores públicos não contorna os limites.
- `WORK=17` continua representando bytes decodificados de arquivos. Só a admissão bem-sucedida concede crédito de uso único para entregar `PRECHARGED_DATA=23` ao mesmo processo válido. O crédito é descontado antes da entrega e zerado no reinício. RAR, 7z e CAB usam esse caminho. A análise e a criação não o usam. O trabalho auxiliar não financia a saída de arquivos.

A admissão de altas taxas de compactação usa os tamanhos declarados e os tamanhos compactados ou do arquivo inteiro que foram fixados. Os limites de decodificação e saída de arquivos valem de forma independente. Se um arquivo de 87 bytes exige vários MiB de inicialização do modelo, os contadores de arquivos ainda refletem apenas os 87 bytes reais.

<a name="xdzip-required-acceptance-conditions"></a>

## Condições de aceitação

- Um arquivo pequeno comum pode concluir a extração e a verificação depois de mais de 64 KiB de trabalho auxiliar e inicialização do modelo. As contagens são exatas e a verificação final do conteúdo é bem-sucedida.
- A camada nativa e o ambiente de execução verificam a recusa quando pacotes auxiliares realmente processados atingem 20 GiB. Novas tentativas de senha e substituição do processo não zeram o custo acumulado. Arquivos incompletos não têm a gravação confirmada.
- Falham as solicitações de processos simulados para outros formatos, fases de metadados, pacotes vazios ou grandes demais, campos no fim e saídas pré-contabilizadas financiadas apenas com trabalho auxiliar.
- `WORK` comum pode financiar a saída RAR pré-contabilizada do mesmo processo. O crédito é consumido uma vez e não pode ser reutilizado após erro ou reinício.
- As amostras de protocolo verificam a máquina de estados e a contabilização. O processo RAR completo em Rust, o fluxo real do ambiente de execução, as três plataformas e a aceitação de desktop exigem evidências próprias.

A amostra de protocolo compila apenas o `tools/native-codec/main.cpp` do projeto e seu simulador, sem um algoritmo de compactação.

```sh
cmake -S crates/xdzip-codec-native/tests -B build/auxiliary-protocol
cmake --build build/auxiliary-protocol --config Release
```

Aponte `XDZIP_NATIVE_CODEC_WORKER` para a amostra, defina `XDZIP_AUXILIARY_PROTOCOL_FIXTURE=1` e execute explicitamente `cargo test -p xdzip-codec-native auxiliary_protocol -- --ignored`. A regressão direcionada comum usa `cargo test -p xdzip-codec-native -p xdzip-runtime auxiliary`. Os testes da API de codecs verificam a compatibilidade do receptor padrão. As evidências geradas ficam em build ou CI. Resultados e pendências ficam na [tarefa de migração #51](https://github.com/xiaodaozhi/xdzip/issues/51).
