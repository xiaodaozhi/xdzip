<a name="xdzip-qt-sdk-and-source-correspondence-verification"></a>

# Qt SDK とソースの対応関係の検証

[English](../../engineering/qt-sdk-correspondence.md) | [简体中文](../../zh-CN/engineering/qt-sdk-correspondence.md) | [繁體中文](../../zh-TW/engineering/qt-sdk-correspondence.md) | 日本語 | [한국어](../../ko-KR/engineering/qt-sdk-correspondence.md) | [Français](../../fr-FR/engineering/qt-sdk-correspondence.md) | [Deutsch](../../de-DE/engineering/qt-sdk-correspondence.md) | [Español](../../es-ES/engineering/qt-sdk-correspondence.md) | [Português (Brasil)](../../pt-BR/engineering/qt-sdk-correspondence.md) | [Русский](../../ru-RU/engineering/qt-sdk-correspondence.md)

ソースの対応関係は公式アーカイブの実際のバイト列を使って検証します。バージョン番号が一致しても SDK のビルドが等価だとは言えません。`scripts/qt_sdk_correspondence.py` は固定されたソースアーカイブ、SDK が提供するソース SPDX、ビルド宣言、インストールされた設定、利用側の既知のパッチを関連付け、結果を build/ に出力します。SDK の変更やライセンス分岐の承認は行いません。

```sh
python3 scripts/qt_sdk_correspondence.py \
  --qt-prefix build/toolchain/Qt/6.8.2/macos \
  --output build/review/qt-sdk-correspondence.json
```

既定では build/source-cache をオフラインで読み取り、なければ失敗します。--fetch を明示した場合に限り、[固定マニフェスト](../../../scripts/corresponding-sources.json)から公式アーカイブをダウンロードして検証します。必要に応じて実行する 3 プラットフォームの Qt ワークフローは、固定 SDK のインストール後に --fetch を実行し、JSON を他の検証記録とともに保存します。Mac では引き続き記録のみをアップロードし、アプリの配置、署名、パッケージ化は行いません。ソース配布収集ツールに --qt-prefix を渡すと、この記録、パッチ、固定マニフェストが配布ハッシュに含まれます。[ソース配布](source-delivery.md)を参照してください。

<a name="xdzip-verification-layers"></a>

## 検証の各層

1. アーカイブの SHA-256 が固定値に一致し、アーカイブのルート、メンバーパス、固定された `.tag` マーカーも一致する必要があります。マーカーの Git オブジェクト型は別途記録します。ツリーマーカーの場合は、まず固定された元のコミットオブジェクトのハッシュを再計算し、tree フィールドを検証します。上流ソースを展開せず、ファイルを個別に読み取ります。
2. SDK がモジュールの .source.spdx を提供する場合、宣言された各 SHA-1 をアーカイブ内容と比較します。一致数、差異の両方のハッシュ、SDK が宣言したがアーカイブにないファイル、未カバーのアーカイブファイル数を記録します。SHA-1 は上流の元の宣言だけを確認し、アーカイブ全体は別途固定 SHA-256 で結び付けます。宣言の欠落、差異、未カバーのファイルを通過扱いにしません。
3. SDK モジュールのビルド SPDX ハッシュ、コンパイラー宣言、configure 引数を保持します。インストールされた modules/*.json、qconfig*.pri、QT_FEATURE 設定ヘッダーも読み取り、内容とハッシュを保持します。出力中の上流ビルダーの絶対ディレクトリーはプレースホルダーに置き換え、ファイルハッシュで元の証拠を結び付けます。この設定は完全な CMake キャッシュ、ビルド環境、ツールチェーン資料ではありません。SDK の既定アーキテクチャーフィールドは実ライブラリーのすべてのアーキテクチャーを表さず、ユニバーサルライブラリーを x86_64 と誤分類する根拠にはできません。
4. [AGL パッチ](../../../scripts/qt-patches/macos-no-agl.patch)は、このプロジェクトが Qt を利用する際に古い AGL へのリンクを避けるため、SDK の lib/cmake/Qt6/FindWrapOpenGL.cmake だけを変更します。ツールは固定された QtBase アーカイブから元の内容を読み、統一 diff を再適用し、パッチ、元の内容、結果の SHA-256 を検証してから実際の SDK ファイルを確認します。元の版または既知のパッチ版だけを受け入れます。Windows SDK の CRLF は改行を正規化した後にのみ比較し、実バイトの SHA-256、LF の SHA-256、変換マーカーを保持します。その他の未知の変更は失敗します。これは Qt ランタイムの機械語を変更せず、Qt が再ビルドされた証拠にもなりません。SDK の完全な変更履歴は引き続き検証が必要です。

ソース対応記録はスキーマ 2 を使用し、固定 archive_commit、アーカイブの実際の archive_marker、archive_marker_type、commit_tree_verified を別々に保持します。Qt Declarative 6.8.2 の `.tag` は Git ツリー `0a8139c574fe150aba92d5f2484a8f3fdb774977` であり、リリースタグはコミット `75534f3e7fff24ed7ccb364e2ed9950a73da879f` に対応します。リポジトリーで固定された base64 の元のコミットオブジェクトにより、コミットハッシュをオフラインで再計算し tree フィールドを検証できます。現在固定されている他のモジュールのマーカーはコミットです。ツリー関係が示すのはこの 2 つの Git オブジェクトの関係だけであり、アーカイブに完全なツリーが含まれることや SDK のビルドが等価であることは証明しません。

qt-license-sources.json の source_commits はライセンス文の取得に固定したコミットのみを示し、アーカイブマーカーやリリースコミットとは別に保存します。SDK のビルドコミットとして表すことはできません。過去のスキーマ 1 資料では Qt Declarative のツリーマーカーが archive_commit に置かれていました。公開済みの成果物は変更しません。後の説明ではオブジェクト型を訂正し、ライセンス文は引き続き元の SHA-256 で検証します。

<a name="xdzip-interpreting-results-and-release-boundaries"></a>

## 結果の解釈とリリース境界

QtNetwork のセキュリティ派生版は `network_security` を別途記録します。回執はパッチ、ビルダー、元の QtCore と実際の QtNetwork のダイジェスト、機能、実際の通信検証を結び付けます。ツールは回執を読む際に実際の SDK バイトを再確認します。この証拠は変更されたライブラリーにのみ適用され、上流 SPDX を派生版の証拠として書き換えず、他モジュールの未カバーの事実も変えません。入力と保守については [Qt パッチノート](../../../patches/qt/README.md)を参照してください。

モジュールのソース SPDX の有無、一致、欠落、未カバーの数、設定、パッチ状態は今回生成した JSON に従います。ある SDK の観察結果を他の SDK に一般化しないでください。ファイル単位の宣言がない場合、対応が通過したと推定せず、具体的な不足を保持します。

報告状態は常に evidence-collected-not-build-equivalence です。[実際の Qt ソースビルドと読み込みまたは置換](../../../developers/engineering/candidate-verification.md#qt-source-and-compatible-replacement)には別の手順がありますが、元の SDK のバイト単位の再現を証明するものではありません。完全なツールチェーンとシステム SDK の資料、未カバーまたは生成されたファイル、ランタイムの出所、ライセンス履行がすべて明確になるまで、対応ソースに関するすべての義務の履行やバイト単位で再現可能なビルドを主張してはいけません。既存の[ランタイム帰属](runtime-attribution.md)の未完了状態、ライセンス選択、プラットフォームごとのランタイムレビューは引き続き維持します。
