<a name="xdzip-cxx-bridge-dependency-rationale"></a>

# Justificativa para a ponte cxx

[English](../../engineering/cxx-admission.md) | [简体中文](../../zh-CN/engineering/cxx-admission.md) | [繁體中文（台灣）](../../zh-TW/engineering/cxx-admission.md) | [日本語](../../ja-JP/engineering/cxx-admission.md) | [한국어](../../ko-KR/engineering/cxx-admission.md) | [Français](../../fr-FR/engineering/cxx-admission.md) | [Deutsch](../../de-DE/engineering/cxx-admission.md) | [Español](../../es-ES/engineering/cxx-admission.md) | Português (Brasil) | [Русский](../../ru-RU/engineering/cxx-admission.md)

Qt e Rust usam uma ponte de propriedade tipada gerada por `cxx`. C++ mantém os objetos Qt, e Rust mantém a lógica de domínio e execução. `cxx` e `cxx-build` precisam ter exatamente a mesma versão, conforme o [Cargo.lock](../../../Cargo.lock) e a [política de ferramentas](toolchain.md). cxx não fornece ABI estável para extensões de formato de terceiros. Uma ABI C manual reduziria dependências e aumentaria a manutenção de strings, funções livres e regras de propriedade.

<a name="xdzip-source-and-licensing"></a>

## Origem e licenças

A origem é crates.io, com [dtolnay/cxx](https://github.com/dtolnay/cxx) como projeto original. O arquivo de bloqueio e o [inventário de dependências da ponte](../../../specs/engineering/bridge-dependencies.json) registram hashes, versões, recursos, scripts de compilação e licenças. Antes da publicação, confira e reúna os avisos realmente aplicáveis. O arquivo de bloqueio pode incluir dependências cujas condições de destino não se cumprem. O inventário de binários vinculados, compiladores, Qt e bibliotecas do sistema é verificado separadamente.

<a name="xdzip-build-and-ownership-boundaries"></a>

## Compilação e limites de propriedade

cxx-build gera fontes e cabeçalhos e chama as ferramentas C++. cxx compila a ponte fornecida e link-cplusplus vincula a biblioteca padrão C++. Cargo verifica as dependências obtidas com o arquivo de bloqueio. Scripts novos ou recursos opcionais exigem rever o comportamento de rede e execução. Verificações anteriores não garantem a segurança de uma versão nova.

Mantenha std/alloc e os demais recursos necessários. Ao reduzi-los, verifique novamente UTF-8, propagação de erros, identificadores exclusivos, contrapressão, encerramento e esvaziamento das filas. O [contrato de execução](product-runtime.md) e o [projeto da ponte](../design/qt-rust-bridge.md) definem as interfaces efetivas. A análise de licenças segue o [processo de revisão para publicação](../../../developers/qa/release-review.md).

Testes, execuções de CI e conclusões são registrados na [issue WP-01](https://github.com/xiaodaozhi/xdzip/issues/1) e em seus artefatos de Actions.
