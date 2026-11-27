<a name="xdzip-data-lifecycle-and-diagnostics"></a>
# Ciclo de vida dos dados e diagnósticos

[English](../../engineering/privacy.md) | [简体中文](../../zh-CN/engineering/privacy.md) | [繁體中文](../../zh-TW/engineering/privacy.md) | [日本語](../../ja-JP/engineering/privacy.md) | [한국어](../../ko-KR/engineering/privacy.md) | [Français](../../fr-FR/engineering/privacy.md) | [Deutsch](../../de-DE/engineering/privacy.md) | [Español](../../es-ES/engineering/privacy.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/privacy.md)

A [política de privacidade](../../../PRIVACY.md) explica os dados tratados pelo xdzip e as opções disponíveis. A tabela mostra onde os dados ficam e quando são liberados. Navegação, compactação e extração funcionam sem conexão. O xdzip não envia automaticamente telemetria nem conteúdo de arquivos compactados aos responsáveis pela manutenção.

| Dados | Tempo de vida | Armazenamento e limpeza |
| --- | --- | --- |
| Configurações e temas | Entre execuções, em um esquema versionado | As alterações usam substituição atômica. Se houver falha, preservam-se as configurações anteriores e a interface informa o erro. |
| Aplicativos padrão anteriores no macOS | Localmente, entre execuções | Ao definir o xdzip como padrão, um arquivo separado registra o identificador do pacote do aplicativo anterior para aquele tipo. Restaurar ou escolher outro aplicativo apaga o registro do tipo. |
| Arquivos recentes | Opcional, apenas local | O arquivo privado `recent.json` guarda caminhos distintos do mais recente ao mais antigo, com um limite de 64 KiB para o arquivo. Desativar impede novas entradas. Limpar exclui o histórico sem tocar nos arquivos originais. |
| Índices de arquivos compactados | Durante a sessão aberta | Índices pequenos ficam na memória. Índices ZIP grandes usam arquivos privados em disco para metadados e ordem da consulta. Encerramento normal, troca de consulta, falha e cancelamento limpam arquivos conhecidos. Uma criação posterior pode recuperar sobras sem uso após um travamento segundo as regras de identidade e vigência. Os índices não são reutilizados entre execuções. Consulte [índices de arquivos grandes](large-index.md). |
| Saídas temporárias e estado de recuperação | Durante as tarefas e a recuperação | A recuperação atua apenas sobre metadados controlados e limpa itens de origem comprovada. Ela não executa conteúdo de arquivos compactados nem restaura gravações em caminhos arbitrários. |
| Prévias no aplicativo | Iniciadas pelo usuário e limitadas | Mudar o contexto da prévia ou encerrar a sessão remove o resultado atual. Resultados atrasados não recuperam conteúdo invalidado. |
| Senhas | Durante a operação que precisa delas | O aplicativo não grava senhas nas configurações nem nos registros. O sistema operacional pode manter cópias adicionais na memória fora do controle do aplicativo. |
| Verificações e downloads de atualizações independentes | Iniciados pelo usuário nas edições aplicáveis | As solicitações usam os serviços públicos de lançamentos e downloads do GitHub. Um proxy configurado pode tratar as solicitações, e um espelho HTTPS opcional recebe o URL de download original. Os provedores podem ver os dados de conexão e a versão incluída nas solicitações. O Windows MSIX desativa o atualizador independente. |
| Comprovantes de transações de atualização | Localmente, inclusive após reinícios durante a instalação | O arquivo separado `update-state.ini` ao lado das configurações guarda a identidade da transação e do destino. Downloads com falha podem ficar no cache privado de atualização. O atualizador lê apenas os arquivos de comprovante previstos e não executa seu conteúdo. |
| Estado e diagnóstico | Interface, terminal ou registros locais, conforme o recurso | O aplicativo não envia diagnósticos automaticamente. Copiar dados de versão ou de arquivo os grava na área de transferência. Confira nomes, caminhos e outras informações privadas antes de compartilhar um erro, registro ou captura de tela. |

Consulte a [especificação de atualizações](software-update.md) para rede, assinaturas, encaminhamento da instalação e recuperação.
