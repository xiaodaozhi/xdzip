<a name="xdzip-host-calendar-and-encoding-services-for-format-workers"></a>
# Serviços de calendário e codificação do sistema hospedeiro

[English](../../engineering/host-platform-services.md) | [简体中文](../../zh-CN/engineering/host-platform-services.md) | [繁體中文](../../zh-TW/engineering/host-platform-services.md) | [日本語](../../ja-JP/engineering/host-platform-services.md) | [한국어](../../ko-KR/engineering/host-platform-services.md) | [Français](../../fr-FR/engineering/host-platform-services.md) | [Deutsch](../../de-DE/engineering/host-platform-services.md) | [Español](../../es-ES/engineering/host-platform-services.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/host-platform-services.md)

O Rust processa campos do formato, regras de compatibilidade de senhas e validação. Regras históricas de fuso horário, páginas de código OEM e ANSI e a codificação C local pertencem ao sistema hospedeiro e são expostas por serviços opcionais de `ReadSource`. A camada de formato não deve deduzi-las do nome do arquivo, de HostOS ou do deslocamento UTC atual. Os serviços não dependem do Qt nem concedem acesso a caminhos ou autorização para publicar arquivos.

<a name="xdzip-calendar-projection"></a>

## Conversão em data e hora

`Timestamp` distingue valores FILETIME em unidades de 100 ns desde 1601 de segundos inteiros Unix sem sinal desde 1970. `ReadSource::local_calendar` retorna um `Calendar` opcional. Arquivos comuns e fontes em memória usam UTC de forma determinística por padrão. O processo de trabalho do produto usa as regras locais de fuso horário do sistema, por chamadas CXX síncronas. O Windows usa APIs do sistema e o Unix usa `localtime_r`. Ambos verificam o resultado. Se a conversão estiver indisponível, não há calendário e os campos não inicializados não podem ser lidos.

O Rust valida ano, mês, dia, hora, minuto e segundo antes de codificar o valor de exibição DOS. O 7z mantém UTC e o intervalo de exibição 1980..2107. O RAR aplica as [regras fixadas de compatibilidade](rar-format-sources.md) para aritmética modular interna, substituição de horários e diferenças entre plataformas. Depois usa o calendário do sistema e o ajuste circular histórico do ano. A precisão original, os marcadores de ausência e os cabeçalhos físicos completos ficam no índice privado do formato. A precisão da exibição não os sobrescreve. Quando não há como converter a data, a regra segura do RAR exibe 0. A implementação de referência pode copiar dados indefinidos após uma falha não verificada do sistema.

<a name="xdzip-controlled-encoding-mappings"></a>

## Conversões de codificação controladas

| Serviço | Entrada e saída | Limite do sistema |
| --- | --- | --- |
| `oem_to_ansi_wide` | 512 unidades UTF-16 para 512 unidades UTF-16, incluindo NUL | Conversão completa do Windows por OEM e ANSI, preservando o mapeamento e as substituições reais |
| `wide_to_ansi` | Até 512 unidades UTF-16, incluindo NUL, para 512 bytes | Página de código ANSI atual do Windows. Não se presume um caractere universal de substituição |
| `wide_to_local_prefix` | Até 128 valores escalares largos, incluindo NUL, para 512 bytes | Codificação C atual do Unix. Em caso de falha, limpa e repete a conversão segundo a política fixa, preservando o prefixo convertido |

Os comprimentos retornados excluem NUL. As implementações sem serviços do sistema retornam `None`. A camada de formato pode aplicar somente alternativas portáveis definidas ou rejeitar a conversão explicitamente. Não deve adivinhar a página de código. As chamadas do produto validam dimensões, NUL e intervalos retornados. Um mapeamento indisponível retorna a categoria de erro estável Unsupported.

O parâmetro de comprimento de [CharToOemBuffW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-chartooembuffw) conta caracteres largos de entrada, inclusive o preenchimento convertido após NUL. Ele não mede a capacidade do destino em bytes. Primeiro o sistema consulta `GetCPInfo(CP_OEMCP)` para admitir páginas de no máximo quatro bytes por unidade. Então usa um buffer temporário de 2048 bytes para converter todas as 512 unidades. Só depois grava NUL no byte 511 e converte para ANSI. Se não for possível garantir a capacidade necessária, rejeita a conversão. Não se deve receber 512 caracteres largos em um destino de 512 bytes nem cortar a entrada para contornar a exigência. Os testes reais cobrem todo o limite DBCS de 511 unidades e fazem uma comparação independente com a WinAPI direta.

A admissão de 512 unidades, o primeiro NUL, as condições de compensação OEM, o truncamento de 127 unidades e por bytes, o protocolo de marcadores legados e UTF8 explícito no macOS seguem regras de compatibilidade em Rust. Não se deve truncar a senha antes de uma conversão do sistema que originalmente é feita primeiro, nem deslocar arbitrariamente um truncamento interno de uma sequência DBCS para o limite de um caractere.

<a name="xdzip-ownership-and-failure"></a>

## Propriedade e falhas

Os segmentos de memória são emprestados só durante chamadas síncronas. O C++ não retém ponteiros. O Rust controla quando limpar entradas e saídas secretas. Os buffers temporários limitados de C++, largos ou estreitos, são apagados em todos os caminhos de retorno. Isso não garante o apagamento dentro do sistema operacional ou de todas as cópias anteriores em memória. As primitivas não registram senhas nem as inserem em argumentos de processo ou variáveis de ambiente.

Os leitores verificam o cancelamento antes de usar os serviços. Os resultados não contam como saída da extração nem alteram os contadores de bytes lidos, decodificados ou gravados. As falhas preservam as categorias de origem, cancelamento e cota e não devem ser confundidas com erros de senha. Só a camada de formato pode determinar se há ambiguidade entre senha e integridade, conforme o estado real do verificador.
