<a name="xdzip-actual-component-attribution-and-license-materials"></a>

# 実際のコンポーネントの帰属とライセンス資料

[English](../../engineering/runtime-attribution.md) | [简体中文](../../zh-CN/engineering/runtime-attribution.md) | [繁體中文](../../zh-TW/engineering/runtime-attribution.md) | 日本語 | [한국어](../../ko-KR/engineering/runtime-attribution.md) | [Français](../../fr-FR/engineering/runtime-attribution.md) | [Deutsch](../../de-DE/engineering/runtime-attribution.md) | [Español](../../es-ES/engineering/runtime-attribution.md) | [Português (Brasil)](../../pt-BR/engineering/runtime-attribution.md) | [Русский](../../ru-RU/engineering/runtime-attribution.md)

このフローは、実際の Qt ライブラリーとプラグインを SDK の SPDX 宣言に関連付け、ファイル単位の SBOM を補完します。出力はレビュー可能な証拠であり、ライセンスの自動選択やコンプライアンス承認ではありません。正式リリースに必要な完全性は未達成です。

<a name="xdzip-inputs-and-sources"></a>

## 入力と情報源

Qt 6.8 以降はビルドに対応する [SPDX インベントリー](https://doc.qt.io/qt-6.8/licenses-used-in-qt.html)を提供します。ツールは固定された Qt 6.8.2 SDK の sbom/ から qtbase、qtdeclarative、qtsvg、qttranslations などの実行時宣言のみを読み取り、qttools のビルド・テスト用ツールは除外します。SPDX バージョン、モジュール名、Qt バージョンを検証し、元文書の SHA-256、名前空間、安定したパッケージ参照を保持します。出力には必要なフィールドだけを残し、開発者のマシンのパスを含む upstream のビルドコメントはコピーしません。

`scripts/qt-license-sources.json` は公式 Qt ソースと SPDX license-list-data のコミット、ライセンステキスト URL、SHA-256 を固定します。ダウンロードは build/ にキャッシュし、キャッシュまたはダウンロードのハッシュ不一致は失敗します。資料の取得に任意の SPDX URL はたどりません。LICENSES/ の標準テキストに加え、Qt GUI の平滑拡大縮小アルゴリズムと SVG の元の通知、およびシステム依存関係の宣言が参照する LGPL-2.1 テキストを含めます。upstream の LicenseRef テキストと著作権は宣言 JSON に保持します。

Qt の実行時通知には、Qt Network が使う固定された編集可能な Public Suffix List ソース、MPL-2.0 テキスト、取得方法も含めます。収集器は upstream blob、SHA-256、ライセンスのバイト列を検証します。ソース配布では、この入力が Qt アーカイブ内の 2 種類の生成データを再現することを別途検証します。[対応するソース資料](../../engineering/source-delivery.md)を参照してください。これは他の Qt モジュールのソースおよび置換確認の代わりにはなりません。

Qt Core の Tika MIME 通知は、Apache Software Foundation と英国国立公文書館 PRONOM の関連する帰属表示、Apache-2.0 と OGL-UK-3.0 のライセンステキスト、固定された upstream の完全な NOTICE を保持します。完全な NOTICE は他の upstream Tika コンポーネントも説明していますが、それらが本製品に同梱されることを意味しません。元の MIME データと Qt 変換の検証は[対応するソース資料](../../engineering/source-delivery.md)を参照してください。

<a name="xdzip-file-and-component-association"></a>

## ファイルとコンポーネントの関連付け

1. 検証ルート内のファイルハッシュを再計算し、deployment-audit.json 後に変更されたファイルを拒否します。
2. macOS SDK モードは実際に解決された Qt SDK 相対パスとバイト一致を使用し、単独のデプロイを主張しません。
3. Linux のデプロイは元の SDK パス/SHA-256 と patchelf 後の成果物パス/SHA-256 を記録します。後続の検査は両方向で記録を検証し、古いまたは変更されたコピー証拠を拒否します。
4. Windows は SDK ファイルのバイト一致を優先します。ファイル名が一意でもバイトが異なる候補は filename-only-unverified-rewrite とだけ記し、検証済みソースではなく不足として登録します。
5. SPDX CONTAINS でファイルの所有者を探し、DEPENDS_ON を再帰的に解決します。Qt リポジトリ間の参照や組み込みの第三者も含みます。欠けたパッケージ参照は不足として残します。リポジトリ全体の CONTAINS によってすべてのツールを実行時に誤って含めてはなりません。
6. ファイルごとの宣言がない非ネイティブの QML と翻訳は、関連するソースリポジトリの範囲にのみ関連付け、個別ライセンスを推測しません。SPDX ファイル記録がない SDK ネイティブライブラリー、Qt の範囲外のネイティブファイル、NOASSERTION ライセンス、欠けたテキストは別々に記録します。

SDK の実際のファイルバイトは、後のインストール処理や署名などにより upstream ビルド SPDX の SHA-1 と異なることがあります。ツールは期待値、比較結果、不足を保持し、原因を推測しません。SDK のバイト関連付け、デプロイ時の書き換え記録、upstream の元のハッシュは別々の 3 種類の証拠です。

専用の macOS パッケージングは最終署名済みアプリが存在する間にデプロイ済み関連付けを生成し、リソースの完全なバイト一致、または arm64 セクション、UUID、インポートバージョンで SDK 入力と照合し、その報告を実ファイル SBOM に追加します。比較では署名、ロードコマンドのレイアウト、LINKEDIT を除外します。範囲と専用通知は[ネイティブ実行時の出所と通知](native-runtime-materials.md)を参照してください。通常の SDK モードからデプロイの結論は得られません。

QML ディレクトリから `Contents/PlugIns/quick/` に移動されたプラグインについては、パッケージ内の監査済み QML シンボリックリンクから一意の SDK 入力を特定できます。記録の生成時と利用時の両方で実際のリンク、監査済みの対象、ファイルハッシュを再確認し、同じバイト比較または Mach-O 比較を行います。出所が欠ければ未検証のままとし、異なる複数のソースや変更されたリンクは拒否します。ファイル名の一致だけでは帰属を強化できず、upstream SDK のハッシュ差異やライセンスレビューの不足も解消しません。

Rust はホストの完全なビルドグラフを保持し、デスクトップの `xdzip-qt-bridge`、形式ワーカーの `xdzip-codec-bridge`、Windows/Linux の実際の `xdzip-update-helper` から通常の依存関係をたどって実行時候補を生成します。macOS には独立した更新ヘルパーのルートはありません。ビルド、開発、proc-macro のツールとその依存関係は除外します。これらは実行時入力の候補であり、各パッケージの機械コードがリンク後も残った証拠ではありません。リンカーによる削除、ネイティブ C/C++、コンパイラーの実行時ライブラリーは追加レビューが必要です。

スラッシュで区切られた単純な旧式 Cargo ライセンスの選択肢は、ソース CycloneDX では SPDX `OR` に変換し、upstream の原文を `xdzip:cargo_license_raw` に保持します。優先順位が曖昧なスラッシュ式は拒否します。元のビルドインベントリーや通知は書き換えず、標準の `AND`、`OR`、`WITH` はそのままです。これは形式の正規化であり、ライセンス分岐の選択やリリース承認ではありません。

<a name="xdzip-execution-and-delivery"></a>

## 実行と配布

既知のパッチ適用済み QtNetwork 派生物はファイル記録に `network_security_variant` を追加します。報告には完全で再検証済みのビルド記録も保持します。upstream SDK のダイジェスト差異は残し、リリースレビューでは固定ソース、完全なパッチ、ビルド記録、デプロイ変換の証拠で説明します。リリース準備では最終 Network の帰属をこれらのバイトに結び付ける必要があり、ファイル名の一致では通過しません。ソースと成果物の SBOM は派生物の識別情報を別々に記し、全体の未完了状態をライセンス承認に変えません。

```sh
python3 scripts/prepare-runtime-notices.py --qt-prefix /path/to/Qt/6.8.2/gcc_64 \
  --output build/review/notices/Qt-runtime
python3 scripts/runtime_attribution.py --audit build/review/deployment-audit.json \
  --root build/review/linux --qt-prefix /path/to/Qt/6.8.2/gcc_64 \
  --deployment build/review/linux-deployment.json --output build/review/runtime-attribution.json
python3 scripts/sbom.py --audit build/review/deployment-audit.json \
  --source build/review/source-build.cdx.json --attribution build/review/runtime-attribution.json \
  --output build/review/artifact-files.cdx.json
```

Linux はデプロイ書き換え記録を渡し、他のプラットフォームでは --deployment を省略します。自己インベントリーの再帰を避けるため、報告は検証ルート外に書き込みます。オンデマンドの Qt ワークフローと Release ワークフローは、デプロイと最終ハッシュの前にライセンス資料を準備します。Windows と Linux は通知を実際の実行環境に配置し、Linux tar.gz にも含めます。macOS のオンデマンド Qt は SDK を読み、ログと JSON のみを記録します。明示的に承認された専用 Release ワークフローは、[リリースプロトコル](release-protocol.md)に従い、実際のリリース資料を別途デプロイ、ad-hoc 署名、パッケージ化、確認し、デプロイ範囲とハッシュを別々に保持します。SDK の記録は最終アプリバンドルの帰属の代わりになりません。

runtime-attribution.json はファイル、コンポーネント、関係、upstream 宣言、著作権、テキストの出所、不足を含みます。artifact-files.cdx.json は同じコンポーネントと既知の所有関係を追加し、報告を同一の実監査ハッシュとビルド識別情報に結び付けます。公式 CycloneDX スキーマと参照の検証は継続し、compositions は未完了のままです。

既定の検査は明示的に記録された開発レビュー上の不足を許容しますが、黙って合格にはしません。識別情報、パス、バイト、バージョン、コピー記録、ライセンステキストのハッシュに誤りがあれば直ちに失敗します。--require-complete はリリースを阻止する検査であり、現在は未完了の帰属やリリース資料があれば失敗します。テストは書き換えられたファイル、古いコピー記録、パス逸脱、誤ったバージョン、文書間の欠落、部分的な SBOM、Rust ツールの除外を対象とします。

<a name="xdzip-unresolved-release-gates"></a>

## 未解決のリリース条件

[ソース配布の収集器](../../engineering/source-delivery.md)は実際のプロジェクトソース、Rust vendor、4 つの実行時 Qt モジュールアーカイブ、2 つのビルドモジュールアーカイブ、再ビルドと動的置換の手順を保持します。SDK と upstream の各項目のビルド同等性を証明するものではありません。各リリースでは、正確な静的リンクの帰属、OS と再配布可能な実行時ライブラリー、組み込みリソースごとのライセンス、オープンソースライセンス分岐の選択と義務の履行を引き続き確認します。[ローカル Mac デプロイ検証](../../engineering/macos-deployment.md)におけるメタデータの変更・置換と、[Qt ソースの実際のビルド・読み込み・置換](../../../developers/engineering/candidate-verification.md#qt-source-and-compatible-replacement)は別のフローであり、証拠も別々です。どちらも任意の SDK のバイト単位のビルド同等性を証明しません。ソースインベントリーと SPDX フィールドはライセンス義務の代わりにはなりません。過去の `v0.1.0` 候補の出所とライセンスは [#4](https://github.com/xiaodaozhi/xdzip/issues/4) と [#53](https://github.com/xiaodaozhi/xdzip/issues/53) を参照し、後の候補は個別にレビューします。有料の署名 ID は条件ではありません。既存のデスクトップ受け入れは [#5](https://github.com/xiaodaozhi/xdzip/issues/5) にあります。

[Qt SDK 対応確認](qt-sdk-correspondence.md)は、固定アーカイブのコミット、SDK ソース宣言内のファイルごとのハッシュ、インストール済み設定、既知の AGL 利用側パッチも記録します。宣言範囲内の一致は SDK ビルド全体の同等性を証明せず、既存のリリース阻止条件は残ります。

自動化された完了作業は、実際のリンカーマップと静的ライブラリーのメンバー、実行時リソースのハッシュ、信頼できるネイティブ依存関係のバージョン、正確に可逆なバイナリ差分分析を追加します。[完全な契約](../../../developers/engineering/candidate-verification.md)を参照してください。再構築したバイトのハッシュが upstream 宣言と完全に一致するときのみ変換を説明済みとします。元の SDK 差異を削除したり、文書化されていない発行元の履歴を推測したりしてはなりません。
