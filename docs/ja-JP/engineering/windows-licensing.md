<a name="xdzip-windows-development-tool-and-runtime-licensing"></a>

# Windows 開発ツールとランタイムのライセンス

[English](../../engineering/windows-licensing.md) | [简体中文](../../zh-CN/engineering/windows-licensing.md) | [繁體中文](../../zh-TW/engineering/windows-licensing.md) | 日本語 | [한국어](../../ko-KR/engineering/windows-licensing.md) | [Français](../../fr-FR/engineering/windows-licensing.md) | [Deutsch](../../de-DE/engineering/windows-licensing.md) | [Español](../../es-ES/engineering/windows-licensing.md) | [Português (Brasil)](../../pt-BR/engineering/windows-licensing.md) | [Русский](../../ru-RU/engineering/windows-licensing.md)

このページは Windows のビルドと再配布に適用される経路を定義し、[ネイティブランタイム資料](native-runtime-materials.md)、[開発者向けビルド入口](../engineering/developer-build.md)、[インストール契約](installers-associations.md)と併せて使用します。具体的な環境、検証コミット、資料のハッシュ、受け入れ結論は issue とローカルまたは CI 成果物に記録し、このページに個々のマシンのライセンス証明は保存しません。

<a name="xdzip-free-community-path-and-local-choice"></a>

## 無料の Community とローカルでの選択

プロジェクトのオリジナルコンテンツは OSI 承認のオープンソースライセンス Apache-2.0 で公開されています。Visual Studio Community 2022 の個人向け条項は個人が自分のアプリを開発することを認め、組織向け条項も OSI 承認ライセンスで公開されるアプリの開発とテストをユーザーに認めています。組織のオープンソース利用とその他の商用内部利用は別々に評価し、このプロジェクトの適格性を他のプロジェクトへ拡張してはいけません。

メンテナーが自分の Community 開発環境を必要とする場合は、[Microsoft の公式インストーラー](https://learn.microsoft.com/en-us/visualstudio/install/use-command-line-parameters-to-install-visual-studio?view=vs-2022)から Community 2022 を取得し、[そのバージョンの原文の条項](https://visualstudio.microsoft.com/license-terms/vs2022-ga-community/)を読み、承諾して、その範囲で使用します。C++ ビルドには `Microsoft.VisualStudio.Component.VC.Tools.x86.x64` と Windows SDK を選び、すべてのワークロードをインストールする必要はありません。Professional/Enterprise の購入や、プロジェクトごとのランタイム配布証明書の別途申請は要求されません。

ライセンス確認のために大きな IDE をローカルに常設する必要はありません。以下の標準 GitHub ホスト環境だけでビルドと公開を行う場合、適用根拠はホストされた Visual Studio の使用許諾とその REDIST 条項であり、ホスト CI のためにローカル Community や個人用 Enterprise を別途用意する必要はありません。Community は自分の環境を必要とするメンテナーの無料の選択肢であり、ホストビルドの適法性に対する追加のローカルインストール条件ではありません。

Community の第 3 節は条件を満たすビルドデバイスでのソフトウェアファイルと Build Tools の使用を認め、第 4 節は機能、下流の条項、制限を条件として一覧掲載の配布可能コードの配布権を与えます。VC_redist のダウンロード、Build Tools のインストール、またはライセンス文書の同梱だけでは開発ツールの使用資格は証明されません。Visual Studio ライセンスがない場合の Build Tools の例外は、自分のアプリのビルドに合理的に必要な、第三者が公開した OSI 承認オープンソース C++ コンポーネントのコンパイルとビルドを対象とします。その依存関係の開発やテストは、コンパイルとビルドに必要な軽微な変更を除き許可せず、自分のすべての C++ プログラムの開発に対する一般的な例外ではありません。VS Code の使用によって MSVC ライセンスが自動的に得られるわけではありません。[Microsoft のライセンス案内](https://www.microsoft.com/licensing/guidance/Visual-Studio)と [Build Tools の条項](https://visualstudio.microsoft.com/license-terms/vs2022-ga-diagnosticbuildtools/)を参照してください。

ローカルでは `vswhere` が報告する製品、完全なバージョン、プレビュー版でない状態、C++ コンポーネントを確認し、実際のインスタンスの開発環境に入って `cl`、`link`、SDK を検査します。インストール済みの 2019 Build Tools は Community 2022 ではなく、現在の MSVC 2022 ツールチェーンの検証を代替しません。汎用 `desktop.py prepare` はシステム IDE や SDK を黙ってインストールしません。初回インストールには Microsoft のインストーラーを使い、既存ツールを保持し、自動再起動を禁止します。

2022 の選択は現在のリリースツールチェーンの決定であり、無料の Community ライセンスが 2022 だけを認めるという主張ではありません。[Qt 6.8 のサポート表](https://doc.qt.io/qt-6.8/supported-platforms.html)は Windows MSVC 2022 を掲載し、このリポジトリーは対応する `msvc2022_64` SDK を使います。2019 ツールはローカル開発用に残せますが、現在の SDK と混在する場合、バージョン間 ABI 互換性だけでは完全なサポートを証明できません。[Microsoft の互換性規則](https://learn.microsoft.com/en-us/cpp/porting/binary-compat-2015-2017?view=msvc-170)は一般に最終リンカーが入力ライブラリーのツールより古くないことも要求し、ランタイムライブラリーと `/GL`/`/LTCG` を別途制限します。代替構成には独自のライセンスと検証が必要であり、ローカルへの完全な 2022 IDE のインストールを強制するものではありません。

<a name="xdzip-github-hosted-builds"></a>

## GitHub ホストのビルド

プロジェクトの Windows ワークフローは標準の GitHub ホスト `windows-2022` ランナーを使います。プリインストール製品は Community ではなく Visual Studio Enterprise 2022 です。runner-images のメンテナーは、GitHub と Microsoft の合意により、プリインストールされた Enterprise が GitHub Actions/Azure DevOps の公開および非公開のホストランナーで個人用 Enterprise ライセンスを別途取得せずに使用できると明確に[確認しています](https://github.com/actions/runner-images/issues/12945#issuecomment-3348002951)。このプロジェクトはそのホストビルド環境に依拠します。公開説明は完全な合意を代替せず、その内容全体を公開していると主張するものでもありません。

[GitHub のホストランナー文書](https://docs.github.com/en/actions/concepts/runners/github-hosted-runners)によれば、各 Windows ジョブには独立した VM が割り当てられ、`Set up job` はその実行のイメージとソフトウェア一覧を示します。ワークフローは [Actions の追加条項](https://docs.github.com/en/site-policy/github-terms/github-terms-for-additional-products-and-features#actions)の範囲内で、このリポジトリーだけをコンパイル、テスト、配置、リリースします。公開リポジトリーの無料計算枠と Microsoft コンポーネントの再配布権は別です。

CI は実際の VS 製品とバージョン、コンパイラーの相対位置、ツールと SDK のバージョンを `windows-toolchain.json` に記録し、製品キーや開発者の私的パスを含めず、同じコミットのパッケージと資料とともに保存します。これはツールの識別であり、ライセンス資格の自動的な証明ではありません。セルフホストランナー、イメージからローカルにコピーしたツール、他の CI サービス、一般的な共有サーバーはホストライセンスを引き継ぎません。Community/Build Tools の条項とデバイス条件を再確認します。

<a name="xdzip-microsoft-component-redistribution"></a>

## Microsoft コンポーネントの再配布

[VS 2022 REDIST 一覧](https://learn.microsoft.com/en-us/visualstudio/releases/2022/redistribution)には有効にライセンスされた Community 2022 が明示的に含まれます。現在のビルドは [windows-runtime.json](../../../scripts/windows-runtime.json) で固定したリリース版 VC_redist x64 を使い、変更せずに展開して八つの CRT DLL と元のランタイムライセンスを個別に確認します。コンパイラー、デバッガー、`debug_nonredist` はパッケージに含めません。無料の Community 開発経路もホスト CI の Enterprise 識別も Microsoft DLL を Apache-2.0 で再ライセンスするものではありません。

新しい開発パッケージには Windows SDK の `d3dcompiler_47.dll`、`dxcompiler.dll`、`dxil.dll` も Mesa のソフトウェア OpenGL ライブラリーも含めません。過去のパッケージではこれら三つは指定 SDK の `Redist/D3D/x64` から取得しました。[SDK REDIST 一覧](https://learn.microsoft.com/en-us/legal/windows-sdk/redist)は、有効な SDK ライセンスの下で Classic Windows アプリとともに変更せずに配布することを認め、UWP アプリとの配布を禁止します。xdzip は Classic Windows デスクトップアプリです。過去の SDK ライセンス、通知、配布条件は、将来のパッケージからファイルを削除しても別途適用され続けます。

同梱の `share/notices/Windows-runtime-NOTICE.txt` は各コンポーネントのライセンス境界を二言語で説明します。Microsoft CRT の原文条項は `Microsoft-VC-runtime/license.rtf` にあります。インストーラーは元の CRT 契約を表示して承諾を求め、その後コンポーネントの通知を表示します。ポータブル版の利用者も同梱の Microsoft 条項を読み、遵守する必要があります。両パッケージは元のライセンスと由来を保持します。下流の再配布者は資格と配布条件を自ら満たす必要があり、プロジェクトの Apache-2.0 ライセンスだけでは Microsoft コンポーネントの再配布を許可しません。

SDK 原文の表示や同梱だけでは、外部配布者とエンドユーザーが SDK の要求する保護条項に同意したことを証明せず、発行者による有効なライセンスと責任の評価を代替しません。現在のインストーラーの承諾画面は CRT のみを対象とします。将来 SDK のグラフィック DLL を再配布する場合は、インストーラーとポータブルパッケージの適用条項と承諾を別々に評価し、実際の証拠をリリース記録に残します。過去の [#135](https://github.com/xiaodaozhi/xdzip/issues/135) を参照してください。

インストーラーをコンパイルする前に CRT 契約と二言語通知の固定ハッシュを確認します。資料には同じ CRT 原文と出典記録を含めます。過去の SDK 条項は `Windows-SDK-graphics/sdk_license.rtf` にあります。ライセンスの修正は過去の公開資産やビルド識別情報を書き換えません。修正パッケージは[不変のリリース手順](release-protocol.md)に従って新しいバージョンで発行します。
