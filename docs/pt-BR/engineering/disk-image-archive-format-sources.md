<a name="xdzip-sources-and-licenses-for-disk-images-and-other-archive-formats"></a>

# Fontes e licenças de imagens de disco e outros formatos de arquivo

[English](../../engineering/disk-image-archive-format-sources.md) | [简体中文](../../zh-CN/engineering/disk-image-archive-format-sources.md) | [繁體中文（台灣）](../../zh-TW/engineering/disk-image-archive-format-sources.md) | [日本語](../../ja-JP/engineering/disk-image-archive-format-sources.md) | [한국어](../../ko-KR/engineering/disk-image-archive-format-sources.md) | [Français](../../fr-FR/engineering/disk-image-archive-format-sources.md) | [Deutsch](../../de-DE/engineering/disk-image-archive-format-sources.md) | [Español](../../es-ES/engineering/disk-image-archive-format-sources.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/disk-image-archive-format-sources.md)

Esta página descreve referências técnicas públicas, restrições de licença e requisitos de suporte para DAA, ISZ, ZPAQ, PEA, BOND, J2J e formatos relacionados. Uma descrição pública de formato não concede permissão para distribuir um decodificador, e uma implementação de referência não é necessariamente uma dependência de produto redistribuível. Nenhuma das referências abaixo é vinculada ao xdzip. Antes de um formato entrar no registro do produto devem ser estabelecidos revisão da fonte, texto de licença, amostras completas, limites de recursos e comportamento em falhas.

<a name="xdzip-daa-10"></a>

## DAA 1.0

- **Especificação pública:** Não foi encontrada especificação DAA 1.0 publicada pelo fornecedor. Descrições públicas identificam DAA como imagem proprietária de disco/arquivo da PowerISO e descrevem cabeçalho, tabela de comprimentos de blocos e blocos ISO compactados independentemente. Os registros públicos abrangem variantes zlib e LZMA e versões internas posteriores como 110, mas não definem um contrato 1.0 estável.
- **Implementação de referência:** `daa2iso` 0.1.7e, pacote Debian/Fedora GPL-2.0-or-later. A fonte serve apenas à pesquisa histórica. Nenhum commit Git upstream ou especificação do fornecedor pode atualmente ser fixado a partir de repositório público oficial.
- **Licença e redistribuição:** A implementação é GPL-2.0-or-later. Sua licença não concede direitos à tecnologia proprietária DAA e o projeto não possui registro de permissão independente para distribuir um decodificador no xdzip.
- **Amostras completas:** Amostras públicas são frequentemente imagens de disco ou arquivos multipartes sem origem e versão fixadas. Nenhum corpus redistribuível cobrindo cabeçalhos antigos, versão 110, ordem multipartes, zlib, LZMA, ofuscação, somas de verificação e blocos malformados está atualmente admitido.
- **Viabilidade em sala limpa:** Uma reconstrução ISO limitada pode ser tecnicamente possível após registro estável do formato e decisão jurídica. Engenharia reversa apenas de um decodificador GPL não é evidência suficiente para implementação no produto.
- **Limite de capacidade:** Permanentemente sem suporte por decisão de produto. Não adicionar detector, codec, registro, tipo MIME, associação, leitor ou gravador DAA. Pesquisa histórica não autoriza admissão futura.
- **Conclusão:** **Excluído permanentemente por decisão de produto.** xdzip não seguirá com DAA. Os resultados de fontes, licenças e amostras permanecem pesquisa histórica, não condições pendentes de entrega.

<a name="xdzip-isz"></a>

## ISZ

- **Especificação pública:** EZB Systems, *ISZ File Format Specification*, versão 1.00, revisada em 2006-07-03, documenta assinatura `IsZ!`, cabeçalho fixo, tabela opcional de definição de segmentos, tabela de definição de blocos e blocos de dados. Os campos incluem tamanho e total de setores, modo de senha, tamanho do segmento, número e tamanho de blocos, comprimento de ponteiro, número de segmento e deslocamentos de tabelas e dados. Sinalizadores identificam blocos zero, armazenados, zlib e BZip2. A especificação é reproduzida na [cópia do fórum ImgBurn](https://forum.imgburn.com/topic/14260-isz-images-supporting/#comment-115553).
- **Implementações de referência:** Filtro ISZ do CDEmu `libmirage`, commit `39fcffe09739e1dc40bef4781646f9ffb8b7dc4e`, e `IszImageStream` do Play-, commit `5bb010f27d8e2309f37b1f8eba0f74ecd546b29b`. Ambos são apenas material de análise de referência. `Helias/isz-tool` é um leitor Python, mas não foi possível obter neste ambiente um commit imutável oficial e registro de licença.
- **Licença e redistribuição:** A seção 8 da especificação EZB informa que incorporar tecnologia ISZ em um produto exige contatar a EZB para licença gratuita sujeita a condições. Texto público ou leitor independente com licença permissiva não elimina esse requisito de licença do produto. CDEmu e Play- não são dependências do xdzip.
- **Amostras completas:** Ferramentas públicas podem gerar blocos Store, zlib e BZip2 válidos, mas uma matriz redistribuível completa ainda precisa de imagens de volume único e segmentadas, blocos zero, todos os comprimentos de ponteiro, blocos finais exatos, modos de senha, ofuscação de tabelas, truncamento, deslocamentos e alteração de origem. Nenhum corpus próprio desse tipo está registrado no repositório.
- **Viabilidade em sala limpa:** O subconjunto não criptografado, de volume único e somente leitura é tecnicamente separável da navegação ISO. Uma primeira implementação poderia decodificar os quatro tipos de blocos documentados para uma saída virtual `image.iso`, sem montar ou executar inicialização. Porém, liberação jurídica, referência imutável fixada e conjunto completo de amostras devem preceder o código.
- **Limite de capacidade:** Até cumprir esses requisitos, não registrar `.isz`, não inferir suporte de `IsZ!` e não oferecer senha, AES, segmentação, gravação ou navegação óptica direta. Se admitido depois, o decodificador deve validar cabeçalho/versão, tabelas de ponteiros e segmentos, regras XOR/desofuscação, limites de blocos e dados, aritmética de tamanho lógico, término completo dos blocos, cancelamento e limites de entrada/trabalho/saída.
- **Conclusão:** **Excluído por decisão de produto.** xdzip não seguirá com ISZ nem contatará a EZB para licença de produto neste pacote de trabalho. Nenhum detector `.isz` ou registro pode ser adicionado sem rever a decisão.

<a name="xdzip-zpaq"></a>

## ZPAQ

- **Especificação pública:** Matt Mahoney, *The ZPAQ Open Standard Format for Highly Compressed Data*, versões 2.03 e 2.06. O formato armazena um programa de descompressão em bytecode ZPAQL e suporta blocos sólidos, diário, deduplicação, registros SHA-1, criptografia em níveis posteriores e arquivos multipartes.
- **Implementação de referência:** `zpaq/zpaq`, tag `6.22`, e fontes `libzpaq` correspondentes. A tag é referência de pesquisa fixada para esta avaliação; hash do commit e do texto de licença devem ser registrados antes de qualquer implementação.
- **Licença e redistribuição:** Implementação de referência e arquivos de configuração são GPLv3. Não podem ser vinculados ao produto atual conforme a política sem runtime de arquivos compactados de terceiros. O padrão público permite análise em sala limpa, mas a máquina virtual de decodificação e o corpus de configurações exigem implementação e revisão separadas.
- **Amostras completas:** Um corpus útil deve cobrir blocos de níveis 1 e 2, programas ZPAQL arbitrários, atualizações de diário, deduplicação, registros criptografados e multipartes, verificação SHA-1 e limites malformados de máquina virtual ou blocos. O repositório não tem esse corpus próprio.
- **Viabilidade em sala limpa:** Possível em teoria, mas é um projeto de máquina virtual e contabilização de recursos, não um pequeno analisador de contêiner. Limites seguros devem cobrar interpretação de bytecode, memória do modelo, saída descompactada, histórico e estado de deduplicação.
- **Limite de capacidade:** Uma primeira etapa futura seria somente leitura, não criptografada, sem multipartes nem diário, com subconjunto de máquina virtual declarado. Não implica criação, atualização, senha ou extração sólida ilimitada.
- **Conclusão:** **Excluído por decisão de produto.** xdzip não implementará a máquina virtual ZPAQL nem aceitará perfil ZPAQ dependente dela. Nenhum detector `.zpaq` ou registro pode ser adicionado sem rever a decisão.

<a name="xdzip-pea"></a>

## PEA

- **Especificação pública:** PeaZip, especificação *Pack, Encrypt, Authenticate*, cobrindo PEA 1.x e PEA 2.0 ainda proposto, fluxos, controles de volumes e objetos, marcadores finais e níveis PCOMPRESS. É publicada como [`pea_help.pdf`](https://peazip.github.io/pea_help.pdf) e resumida em [pea-file-format.html](https://peazip.github.io/pea-file-format.html). A página oficial atual libera explicitamente as especificações PEA e PCOMPRESS ao domínio público.
- **Implementação de referência:** `peazip/PeaZip`, commit `81b31f7e487b462980a52ac0f2e0b3cf700fe799`, incluindo `peazip-sources/dev/pea_utils.pas` e `unit_pea.pas`. A implementação é LGPLv3 e as unidades criptográficas de Wolfgang Ehrhardt incluídas usam licença zlib/libpng. A URL de publicação das fontes é [peazip-11.4.0.src.zip](https://github.com/peazip/PeaZip/releases/download/11.4.0/peazip-11.4.0.src.zip). A fonte fixada é apenas material de análise de referência.
- **Licença e redistribuição:** A especificação do formato não bloqueia por si só a licença do produto, mas o mecanismo LGPLv3 e dependências criptográficas não podem ser copiados ou vinculados por padrão ao produto atual. Uma implementação Rust em sala limpa deve usar a descrição do formato em domínio público e algoritmos selecionados independentemente; o binário de referência permanece apenas para testes.
- **Amostras completas:** O perfil admitido possui amostras Store completas próprias para arquivos, diretórios, nomes UTF-8 válidos, dados vazios e não vazios, truncamento, bytes finais, alteração de origem e limites. Um corpus mais amplo ainda deve incluir PEA 1.x e 2.0, vários fluxos, limites de volumes, níveis PCOMP, falhas de autenticação, cabeçalhos criptografados, marcadores finais malformados e tamanhos declarados grandes; esses perfis permanecem fora do produto.
- **Viabilidade em sala limpa:** Um leitor Store/Deflate não criptografado e de fluxo único é tecnicamente plausível, mas autenticação e criptografia devem ser tratadas como trabalho de segurança separado. O projeto de integridade do formato impede anunciar análise estrutural isolada como verificação de conteúdo.
- **Limite de capacidade:** Uma futura admissão começaria somente leitura, sem senha, criptografia, vários volumes, criação ou alteração. Cada fluxo extraído exigiria verificações do digest declarado e de autenticação final antes da publicação.
- **Conclusão:** **Admitido como perfil conservador somente leitura.** xdzip agora admite PEA 1.x, um volume físico, nomes UTF-8, arquivos/diretórios armazenados PCOMPRESS0 e nenhuma verificação de objeto, fluxo ou volume. Senhas, fluxos autenticados ou criptografados, algoritmos de soma, PCOMPRESS1–3, volumes divididos, PEA 2.0, criação e alteração continuam sem suporte. A referência LGPL permanece apenas para testes e não é vinculada ao produto.

<a name="xdzip-bond"></a>

## BOND

- **Especificação pública:** Não foi encontrada especificação, assinatura, versão ou contrato de compressão do arquivo Bandizip BOND. Notas recentes do Bandizip mencionam vulnerabilidades em “alguns arquivos BOND”, mas não definem o formato. Bancos públicos da extensão `.bond` descrevem principalmente dados com esquema Microsoft Bond, um formato de serialização diferente.
- **Implementação de referência:** Nenhuma com identidade oficial de formato de arquivo, revisão fixa e licença redistribuível foi encontrada.
- **Licença e redistribuição:** Desconhecida. Nenhum produto ou decodificador pode ser admitido enquanto proprietário do formato, direitos tecnológicos e termos de referência forem desconhecidos.
- **Amostras completas:** Nenhuma de fonte confiável e versionada. Uma extensão de arquivo sozinha não identifica uma amostra aceitável.
- **Viabilidade em sala limpa:** Não pode ser avaliada sem assinatura e amostras completas suficientes para distinguir um arquivo compactado de dados Microsoft Bond ou outra carga binária.
- **Limite de capacidade:** Sem detecção, registro de sufixo, navegação, extração, verificação, criação, senha ou volumes.
- **Conclusão:** **Excluído por decisão de produto e não confirmado como formato de arquivo Bandizip.** Nenhum detector `.bond` ou registro será adicionado neste pacote de trabalho.

<a name="xdzip-j2j"></a>

## J2J

- **Especificação pública:** Não foi encontrada especificação, assinatura, versão ou registro de desenvolvedor do formato J2J. Listas históricas de recursos do Bandizip mencionam J2J apenas para extração, mas as páginas oficiais atuais não fornecem definição técnica. Outros resultados J2J descrevem conjuntos de dados job-to-job ou ferramentas JPEG-to-JPEG sem relação.
- **Implementação de referência:** Nenhuma com revisão fixa e licença vinculável a leitor de arquivos Bandizip.
- **Licença e redistribuição:** Desconhecida. Páginas de extensões de terceiros e listas antigas de software não concedem licença.
- **Amostras completas:** Nenhuma matriz confiável e redistribuível existe no repositório. Um nome `.j2j` não estabelece identidade de contêiner.
- **Viabilidade em sala limpa:** Não avaliável até documentar assinatura, modelo de registros e comportamento de compressão/criptografia e disponibilizar corpus completo.
- **Limite de capacidade:** Sem detecção, registro de sufixo, navegação, extração, verificação, criação, senha ou volumes.
- **Conclusão:** **Excluído por decisão de produto e não confirmado como formato de arquivo Bandizip.** Nenhum detector `.j2j` ou registro será adicionado neste pacote de trabalho.

<a name="xdzip-requirements-for-format-support"></a>

## Requisitos de suporte a formatos

1. Registrar uma especificação oficial ou protocolo de sala limpa revisado juridicamente. Blog, banco de extensões ou apenas fonte de um decodificador não são suficientes.
2. Fixar cada fonte de pesquisa em commit imutável ou hash de conteúdo e registrar o texto de licença aplicável.
3. Criar matriz de amostras completa e redistribuível para cada método, versão, modo de volume, estado de criptografia e condição terminal declarados. Amostras apenas de cabeçalho ou Store não comprovam interoperabilidade.
4. Definir modelo limitado de recursos para leituras de entrada, metadados, trabalho de descompressão, memória, saída lógica, histórico, estado criptográfico e cancelamento. Rejeitar deslocamentos malformados, ciclos, estouro e alteração de origem antes da publicação.
5. Manter cada decodificador independente de Qt e das APIs de montagem, inicialização, instalador, scripts, registro e execução arbitrária de processos.
6. Conectar runtime, CLI, associações e documentação de usuário somente depois de codec, identidade de instantâneo, comportamento exato dos dados e classificação de falhas passarem pela automação específica.
7. Um candidato bloqueado permanece visível como bloqueado na avaliação. Não deve receber sufixo, entrada MIME ou capacidade do produto apenas porque Bandizip ou lista de terceiros o menciona.

A verificação do repositório `python scripts/test-m10-source-gates.py` confere que as seis seções bilíngues preservam os campos de evidência necessários, que cada conclusão permanece explicitamente bloqueada, excluída ou admitida e que nenhum candidato bloqueado ou excluído entrou em `specs/engineering/archive-formats.json`.
