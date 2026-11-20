<a name="xdzip-deployment-dependency-verification-and-sbom"></a>

# デプロイ依存関係の検証と SBOM

[English](../../engineering/deployment-audit.md) | [简体中文](../../zh-CN/engineering/deployment-audit.md) | [繁體中文（台灣）](../../zh-TW/engineering/deployment-audit.md) | 日本語 | [한국어](../../ko-KR/engineering/deployment-audit.md) | [Français](../../fr-FR/engineering/deployment-audit.md) | [Deutsch](../../de-DE/engineering/deployment-audit.md) | [Español](../../es-ES/engineering/deployment-audit.md) | [Português (Brasil)](../../pt-BR/engineering/deployment-audit.md) | [Русский](../../ru-RU/engineering/deployment-audit.md)

デプロイ検証は既定で開発レビュー用資料を生成し、明示的に `incomplete` と記録します。リリース準備完了を自動的に証明するものではありません。正式 Release は同一候補の資料を別途収集し、リリース手順のレビュー後にパッケージとともに提供します。形式は[公式 CycloneDX 1.6 スキーマ](https://github.com/CycloneDX/specification/blob/55343ba19dee1785acf1ce9191540d5fd7b590db/schema/bom-1.6.schema.json)を使用します。上流コミットと 3 つのスキーマファイルの SHA-256 は[設定](../../../scripts/sbom-schemas.json)に固定されています。検証はこれらのリソースだけをダウンロードして解析し build/ にキャッシュします。SBOM 内の任意 URL からスキーマをダウンロードすることはありません。

| 成果物 | データソースと目的 | 証明できないこと |
| --- | --- | --- |
| source-build.cdx.json | 開発、ビルドツールを含む Cargo のホスト向けワークスペース依存グラフ、宣言された Qt モジュール、フォントとブランドのソースハッシュ | Rust パッケージが必ず製品に静的リンクされること、すべての Qt プラグインと組み込み第三者が特定されたこと |
| deployment-audit.json | 指定した実際の成果物ルート以下のファイル SHA-256、ネイティブバイナリーのインポート、SDK、システム、未解決の依存関係、実際の CLI ビルド ID | 名前で動的に読み込むすべての QML とプラグイン、クリーンインストール、システム ABI、署名、ライセンス遵守 |
| artifact-files.cdx.json | 実際の成果物と観察された外部 SDK ファイル、ネイティブインポートからの既知の関係。ソース一覧と実際の実行ファイルのバージョン、コミット、dirty 状態に結び付ける | デプロイ済みコンポーネントの完全な特定、静的リンクの帰属、Qt 組み込みライブラリーのライセンス、正式な製品 SBOM の完成 |
| 任意のデスクトップ記録 | 変更と利用可能な環境に応じて実施したデスクトップチェックと制限 | 未実施の操作、他のプラットフォーム、リリース承認 |

ソース記録には Cargo のパスに基づく不透明 ID を含めず、ワークスペースパッケージに crates.io purl を作りません。成果物のパスは指定されたルートまたは Qt SDK からの相対パスです。その他の外部依存関係は内容ハッシュとファイル名で識別し、非公開の SDK 絶対パスは出しません。フォントは OFL-1.1、独自のブランドソースはアセットライブラリーの Apache-2.0 を保持します。Qt モジュールの宣言から実際にデプロイされた全コンポーネントのライセンスを推測しません。

<a name="xdzip-automated-verification"></a>

## 自動検証

`deployment_audit.py` は明示的に指定された信頼できるプロジェクトビルドのみを確認し、ユーザーのアーカイブは対象にしません。macOS はインポートと RPATH に otool、Windows は通常および遅延 DLL インポートに MSVC dumpbin、Linux は ldd を使用します。ldd を出所不明の実行ファイルに使用してはいけません。指定された成果物ルートの外へ出るファイルのシンボリックリンクは検証を失敗させます。レポートは自分の出力を一覧に含めないようルート外に書く必要があります。

- `--mode sdk` は外部 Qt SDK への解決を許可し、ビルドが独立していないことを明示します。未解決のインポートは引き続き失敗します。再帰は成果物と宣言された Qt SDK を対象とし、その他の外部ライブラリーはハッシュと未走査数だけを記録し、依存関係の閉包を検証済みとは扱いません。
- `--mode deployed` はシステム以外の外部依存関係と未解決インポートを許可しません。Windows の MSVCP や VCRUNTIME など VC++ インポートはパッケージから取得する必要があり、開発マシンの System32 で欠落を隠せません。システム UCRT と API 契約は引き続きシステムから提供されます。Qt 開発用検索パスを消去してネイティブ起動スモークテストも続けます。
- 未解決インポート、ソースと実バイナリーのバージョン、コミット、dirty 状態の不一致、公式スキーマのハッシュや形式の不一致、重複または参照先のないコンポーネント参照は、該当するチェックを阻止します。
- ネイティブインポートの閉包は、名前で読み込むプラグインのカバレッジやクリーンマシンでのチェックの代わりにはなりません。デプロイ済み Windows ディレクトリーのインポートが通過してもインストーラーの受け入れにはなりません。

必要時の 3 プラットフォーム Qt ワークフローはルートの 2 つの cdx.json と deployment-audit.json を保存し、空の手動フォームを自動生成しません。macOS はローカルでビルドした .app と SDK だけを読み取り、通常チェックでは macdeployqt の実行やアプリパッケージの署名、圧縮、アップロードを行いません。Linux は[独立したランタイムディレクトリー](linux-deployment.md)で Qt/QML/プラグインを提供し、deployed 検証と SDK のないコンテナーを使用します。ダウンロードした tar.gz は実行ビットを保持します。現在宣言されているのは Ubuntu 24.04、x86_64、X11 の基準だけで、汎用 Linux インストーラーではありません。

信頼できるプロジェクト所有の Windows 成果物では、インストーラーを別途展開し、`python scripts/check-windows-package-payload.py --installer-app <extracted-app-directory> --portable <portable-package.zip> --output build/review/payload-comparison.json` でそのアプリディレクトリーを同一候補のポータブル ZIP と比較します。ファイルごとのハッシュ、インストールマニフェスト、含めてはいけない SDK グラフィックス DLL を比較します。出力パスは新規でなければなりません。この静的比較はインストーラーや更新を実行せず、実際のインストール検証の代わりにはなりません。

整合したソースから再ビルドした後のローカル macOS 例：

```sh
python3 scripts/collect-notices.py build/review/wp12/notices
python3 scripts/deployment_audit.py \
  --root build/ui/apps/desktop/xdzip.app \
  --app build/ui/apps/desktop/xdzip.app/Contents/MacOS/xdzip \
  --qt-prefix build/toolchain/Qt/6.8.2/macos --mode sdk \
  --output build/review/wp12/deployment-audit.json
python3 scripts/sbom.py \
  --audit build/review/wp12/deployment-audit.json \
  --source build/review/wp12/notices/source-build.cdx.json \
  --output build/review/wp12/artifact-files.cdx.json
```

分離された検証環境に jsonschema==4.23.0 をインストール後、その環境の Python で上記 2 つの cdx.json を指定して `scripts/validate-sbom.py` を実行します。レジストリーの使用法は [jsonschema 4.23 文書](https://python-jsonschema.readthedocs.io/en/v4.23.0/referencing/)を参照してください。検証ツールは製品にデプロイしません。

<a name="xdzip-attribution-results-and-verification-limits"></a>

## 帰属結果と検証の限界

[実際のコンポーネント帰属](runtime-attribution.md)を統合しています。SDK SPDX は実際のファイル、Qt モジュール、組み込み第三者を関連付け、固定ハッシュの公式ライセンス文を配布し、SDK と上流のハッシュ差異や未確認の帰属を記録します。通常の Rust ランタイム候補はビルド、開発、proc-macro ツールから分離します。新しいコンポーネント関係は実際の監査に結び付き、成果物 SBOM に統合されますが、SBOM は明示的に未完成のままです。

インストールされたツリーで、配布された各 DLL、フレームワーク、プラグインを上流コンポーネント、バージョン、ライセンス、対応ソースに関連付けます。Qt の組み込みコンポーネント、静的 Rust リンクの帰属、ツールチェーンの出所は別々にレビューします。候補の判断には[リリースレビューガイド](../../../developers/qa/release-review.md)を使用してください。
