<a name="xdzip-ace-alz-and-egg-source-and-licensing-gates"></a>

# Fontes e condições de licença de ACE, ALZ e EGG

[English](../../engineering/ace-alz-egg-format-sources.md) | [简体中文](../../zh-CN/engineering/ace-alz-egg-format-sources.md) | [繁體中文](../../zh-TW/engineering/ace-alz-egg-format-sources.md) | [日本語](../../ja-JP/engineering/ace-alz-egg-format-sources.md) | [한국어](../../ko-KR/engineering/ace-alz-egg-format-sources.md) | [Français](../../fr-FR/engineering/ace-alz-egg-format-sources.md) | [Deutsch](../../de-DE/engineering/ace-alz-egg-format-sources.md) | [Español](../../es-ES/engineering/ace-alz-egg-format-sources.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/ace-alz-egg-format-sources.md)

Esta página descreve o suporte do xdzip a ACE, ALZ e EGG e as referências técnicas públicas e restrições de licença por trás das decisões. Os arquivos de referência ficam em `build/toolchain/references`, fora das dependências do produto.

<a name="xdzip-ace"></a>

## ACE

- Nota técnica de Marcel Lemke, *Technical information of the archiver ACE v1.2*, obtida de `wdlkmpx/unace`, SHA-256 `F645C91348D5DCD560A19CA21DEF849D25C304EFFAACF65B03EFBB0A893E51F9`.
- Implementação independente `droe/acefile`, commit `11ed076ae19e8228148731cfbc05f019a395b3b9`. SHA-256 da licença BSD-2-Clause `AD376BD9695F0E355F5C523443ABF822BB896999A3252AAA614D313CC4F1F5B5`. SHA-256 da implementação `BC701569AD8521964C62256472AB44976AC57FB46E52B147A21018BC96348F1F`.
- A implementação declara ler ACE 1.0 e 2.0, incluindo LZ77, DELTA, EXE, PIC, SOUND, senhas e volumes. A licença cobre apenas a implementação em Python e não reivindica direitos sobre a tecnologia ACE.

A documentação técnica basta para definir cabeçalhos de blocos, CRCs, entradas armazenadas sem compactação e a família antiga LZ77. Ela não concede autorização comercial para a tecnologia ACE. As referências da implementação a WinAce e unace não constituem um conjunto independente de interoperabilidade. O produto exclui ACE em definitivo. O xdzip não implementará nem registrará ACE e não adicionará extensões, detectores, tipos MIME ou associações para o formato.

<a name="xdzip-alz"></a>

## ALZ

- Especificação de formato `alkegi/docs`, commit `36230c34d3056604cc63519e9855a79e894496b7`. SHA-256 da licença CC0 `F4E7F373B9B996950337E8D41A4A2939C2D90B7725E9BAF3D5084A22717AD328`.
- SHA-256 de `alz.md` `CBD35E77AB53CFD08C0FF192FA2793438150E4935522608936B1A0942560E237`.
- O documento especifica assinaturas de ALZ e de arquivos locais, tamanhos de largura variável, CRC-32, nomes UTF-8/CP949, Store, Deflate bruto conforme RFC 1951 e um fluxo BZip2 modificado pelo ALZ. Também especifica a criptografia tradicional PKWARE e a estrutura dos volumes divididos.

O xdzip admite um escopo limitado de ALZ somente para leitura. Arquivos compactados comuns, de volume único, podem usar Store ou Deflate bruto sem criptografia, com verificação exata de CRC, tamanho e término. Os testes automatizados cobrem o codec, o ambiente de execução, a CLI e as associações. BZip2 modificado, ZipCrypto e volumes divididos exigem amostras completas independentes e testes separados de recursos e segurança. A licença CC0 do documento não comprova, por si só, que todos os detalhes de implementação estejam livres de obrigações de patentes ou marcas de terceiros.

<a name="xdzip-egg"></a>

## EGG

- Documento público do formato `alkegi/docs`, mesmo commit `36230c34d3056604cc63519e9855a79e894496b7`, sob a licença CC0 acima.
- SHA-256 de `egg.md` `4CA2CB6E8FA9A4F40C30EDE05A5E977BE113A96E1527E28A7B1ED0836D8A43A3`.
- Módulo de referência `dterracino/UnEgg`, commit `f21e653b02f7ab467bbc16af879e68a48dabcb2b`. SHA-256 do PDF de especificação do EGG `0BB7E6F174CAFB9F6726D70B7DEE8DBF33C034717058B1B0EB2B372D9A8FBB37`.
- SHA-256 da licença do código UnEGG `9B012928756D9243CC549FA0A7D7B5742ED8F69B00A2BE1C964F42EF21F1D8D6` para o pacote de código-fonte e `30F2E45148D43D7A77574403807C13AA359EE9ACE841A00B6A5348AD961423F0` para o pacote do módulo.

A licença UnEGG exige aprovação da ESTsoft para uso comercial, restringe a redistribuição a fins não comerciais e proíbe usar o módulo para desenvolver um compactador EGG. A ESTsoft mantém todos os direitos. O módulo serve apenas de referência para testes. Ele não pode ser copiado, vinculado ou usado como dependência do produto.

É possível implementar o formato independentemente do módulo restrito. O xdzip admite uma implementação própria de Store e Deflate bruto em volume único, sem criptografia ou modo sólido. Testes do codec, do ambiente de execução, da CLI e das associações verificam tamanhos exatos de blocos e CRC-32. BZip2 padrão e o enquadramento LZMA documentado ainda exigem amostras completas separadas e revisão. AZO, AES/LEA, ZipCrypto e arquivos divididos ou sólidos continuam fora do escopo atual.

ALZ e EGG validam separadamente os cabeçalhos do contêiner e conferem as entradas físicas com seus respectivos instantâneos de metadados. Após a admissão, ambos usam o mesmo leitor com limites para Store e Deflate bruto. Ele confere o tamanho declarado e o CRC-32 e contabiliza entrada, bytes decodificados e trabalho acumulado entre as entradas.

<a name="xdzip-required-implementation-gates"></a>

## Requisitos de admissão da implementação

1. Mantenha as referências fora da árvore do produto e verifique os hashes fixados antes do uso.
2. Prepare amostras completas e independentes para cada método admitido. Uma amostra apenas com cabeçalho ou só com Store não comprova interoperabilidade do método.
3. Adicione os pontos de entrada do contêiner, ambiente de execução, CLI e desktop juntos. Os métodos desconhecidos ficam visíveis sem autorização para processar dados.
4. Mantenha senhas, volumes, estado sólido e objetos especiais desabilitados até passarem pelas próprias verificações de fontes, segurança e cancelamento.
5. Registre a decisão jurídica e as identidades das amostras na issue antes de alterar o registro público de capacidades.
