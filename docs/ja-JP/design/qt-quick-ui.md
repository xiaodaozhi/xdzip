<a name="xdzip-qt-quick-desktop-interface"></a>

# Qt Quick デスクトップインターフェース

[English](../../design/qt-quick-ui.md) | [简体中文](../../zh-CN/design/qt-quick-ui.md) | [繁體中文](../../zh-TW/design/qt-quick-ui.md) | 日本語 | [한국어](../../ko-KR/design/qt-quick-ui.md) | [Français](../../fr-FR/design/qt-quick-ui.md) | [Deutsch](../../de-DE/design/qt-quick-ui.md) | [Español](../../es-ES/design/qt-quick-ui.md) | [Português (Brasil)](../../pt-BR/design/qt-quick-ui.md) | [Русский](../../ru-RU/design/qt-quick-ui.md)

デスクトップは表示と操作に Qt Quick を使用します。C++ コントローラーはユーザーの操作を要求に変換し、ビューの状態を維持します。Rust ランタイムはアーカイブの解釈、タスクの方針、ファイルの操作権限を担当します。以下の呼び出しは[システムアーキテクチャ](system-architecture.md)と[ブリッジの契約](qt-rust-bridge.md)に記載しています。

<a name="xdzip-windows-sessions-and-models"></a>

## ウィンドウ、セッション、モデル

| コンポーネント | 現在の責務 |
| --- | --- |
| `Browser.qml` と `SessionPage.qml` | ホーム画面、アーカイブのタブ、パスと検索、ディレクトリーの移動、一覧またはグリッド、プロパティ、操作の入口。ページはコントローラーの現在の状態を表示してコマンドを呼び出します。 |
| `ApplicationSessions` | 開く要求を独立した閲覧セッションに振り分け、タブやウィンドウ、共通のタスク入口、アプリの終了を調整します。開き方の設定は以後の要求に適用します。ページを閉じると閲覧表示は解放されますが、タスクのコントローラーは動作を続けられます。 |
| `BrowserController` と `ArchiveTableModel` | 選択、一覧の状態、結果の識別情報を維持します。小さな結果は完全なモデルとして反映します。大きな結果は論理上の行数を示し、常駐する行は最大 1,024 行で、不足分をページ単位で取得します。未取得の行から項目の操作を要求することはできません。 |
| `TaskListModel` と操作ウィンドウ | 上限付きのタスクスナップショット、進行状況、結果と処理待ちの判断を表示します。タスクコマンドは複合のデスクトップ識別情報を使って担当コントローラーへ送ります。 |
| 設定とプラットフォームアダプター | 外観、言語、文字サイズ、アプリの動作を管理します。通知領域、アクティブ化、ファイルダイアログ、関連付け、転送の引き渡しは OS に応じてネイティブアダプターが扱います。 |

[デスクトップの入口](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/browser_main.cpp)、[セッションページ](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/ui/SessionPage.qml)、[コントローラーとモデル](https://github.com/xiaodaozhi/xdzip/blob/main/apps/desktop/BrowserController.h)、[アプリのセッション](../../engineering/application-sessions.md)、[設定と言語](../ux/settings-localization.md)

<a name="xdzip-browsing-and-operation-flow"></a>

## 閲覧と操作の流れ

開く処理は完成したメタデータのスナップショットを公開します。検索、並べ替え、移動の新しい要求にはそれぞれ結果の識別情報があります。古いページで現在のモデルを置き換えることはできません。新しい要求が保留中も、コントローラーが有効と判断する間は、ビューに確定済みの結果を表示します。コアからの転送は一回に最大 128 行と 64 KiB の名前データです。[ドメインデータ](domain-data.md)、[大きなアーカイブの索引](../engineering/large-index.md)

ディレクトリーを開くとその中へ移動します。対象となる通常ファイルをダブルクリックするか、対応する開く操作を選ぶと、一時的に開くタスクを開始します。プロパティには独立したプレビュー操作があります。競合やパスワードのダイアログは現在の操作の識別情報を保持します。閲覧ページを閉じたあとも、タスクの入口から進行状況と結果を確認できます。[閲覧ガイド](../user-guide/browse.md)、[一時的に開く](../engineering/archive-opening.md)、[内容のプレビュー](../engineering/content-preview.md)、[タスクウィンドウ](../ux/task-windows.md)

ドラッグしてドロップすると、場所に応じてアーカイブを開くか、作成用の入力を追加します。対応するアーカイブ内の移動と外部へのドラッグは、タスクの準備とプラットフォームごとの引き渡しを使います。非同期コンテンツと出力先への書き込みを含め、転送機能はプラットフォームで決まります。[アーカイブの転送](../../engineering/archive-transfers.md)

<a name="xdzip-shared-controls-and-native-boundaries"></a>

## 共通コントロールとネイティブの範囲

操作画面は `OperationButton`、`OperationFooterButton`、`OperationFrame`、`OperationSelect`、`OperationSegmentedGroup` を再利用します。寸法と操作は共通コンポーネントで定めます。選択用の共通コントロールはマウス、キーボード、アクセシビリティの操作を同じ状態モデルに送ります。UI 文言とファイルデータは[デザイン標準](../../visual/design-standard.md)のフォントと言語の規則に従います。アーカイブ内の名前はデータとして安全に表示します。

独立した操作ウィンドウはウィンドウレジストリと各ウィンドウのモダリティおよび閉じる動作に従います。macOS ではネイティブのタイトルバーコントロールを使い、Windows と Linux では各 OS のアダプターを使います。[プラットフォームのウィンドウ契約](../ux/platform-windows.md)に実ウィンドウの受け入れ基準があり、[デスクトップ統合の契約](../ux/desktop-integration.md)に通知領域とアクティブ化の動作があります。

<a name="xdzip-design-and-verification-scope"></a>

## 設計と検証の範囲

ソースコードとテストが製品の動作を記述します。高忠実度の HTML はレイアウトと模擬状態をレビューするためのものです。ネイティブのフォーカス、メニュー、IME、アクセシビリティの操作、画面のスケーリング、システムのファイルマネージャーに影響する変更は、対象プラットフォームでの検証記録が必要です。デスクトップの対応範囲や操作ウィンドウを増やす場合は、対応する実装と受け入れ検証を行います。[ビジュアル作業手順](../visual/design-workflow.md)、[品質管理](quality-engineering.md)
