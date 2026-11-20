<a name="xdzip-requirements-and-verification-traceability"></a>

# 要件と検証のトレーサビリティ

[English](../../design/requirements-traceability.md) | [简体中文](../../zh-CN/design/requirements-traceability.md) | [繁體中文](../../zh-TW/design/requirements-traceability.md) | 日本語 | [한국어](../../ko-KR/design/requirements-traceability.md) | [Français](../../fr-FR/design/requirements-traceability.md) | [Deutsch](../../de-DE/design/requirements-traceability.md) | [Español](../../es-ES/design/requirements-traceability.md) | [Português (Brasil)](../../pt-BR/design/requirements-traceability.md) | [Русский](../../ru-RU/design/requirements-traceability.md)

[製品要件](product-requirements.md)は要件 ID と現在の範囲を定義します。各行は動作を変更した際の検証内容を示します。実行結果、対象プラットフォーム、制限は該当する変更記録に残します。[受け入れケース](../../../developers/qa/acceptance.md)は検証の入口を示します。

<a name="xdzip-requirement-matrix"></a>

## 要件マトリクス

| 要件 | 設計資料 | 検証の重点 |
| --- | --- | --- |
| P01 展開せずに閲覧 | [ドメイン](domain-data.md), [デスクトップ](qt-quick-ui.md) | T01 実際のアーカイブを開き、閲覧によってファイルが展開されないことを確認します。 |
| P02 閲覧、検索、選択 | [ドメイン](domain-data.md), [性能](performance.md) | T02 同名項目の識別、安定した検索、古いページ、大規模索引の保存、ページをまたぐ選択を検証します。 |
| P03 指定範囲を展開 | [ファイルの権限](file-security.md), [タスク](task-scheduling.md) | T03 ディレクトリー展開、重複、名前変更、スキップ、上書き、ディスク上の正確な結果を確認します。 |
| P04 アーカイブを作成 | [形式](codec-extension-sdk.md), [ファイルの権限](file-security.md) | T04 変更したプロファイルの独立した作成・読み取り、準備した出力の検証、既存ファイルの保護を確認します。 |
| P05 形式ごとの機能を尊重 | [ブリッジ](qt-rust-bridge.md), [形式](codec-extension-sdk.md) | T05 レジストリ、セッション固有の操作、非対応プロファイルの実行時拒否を確認します。 |
| P06 タスクを制限しキャンセルを可能にする | [ブリッジ](qt-rust-bridge.md), [タスク](task-scheduling.md) | T06 待ち行列の満杯、問い合わせの振り分け、キャンセルと確定の競合、非同期終了を確認します。 |
| P07 ファイルとリソースの境界を保護 | [ファイルの権限](file-security.md), [形式](codec-extension-sdk.md) | T07 対象となるネイティブのリンク・識別検査、パス検証、上限、保護対象を確認します。 |
| P08 アクセス可能なデスクトップを提供 | [デスクトップ](qt-quick-ui.md) | T08 モデルとコントロール、ネイティブのキーボード、IME、アクセシビリティ、拡大率とフォーカスを対象プラットフォームで確認します。 |
| P09 リソースを制限し計測 | [性能](performance.md) | T09 比較可能なビルド、プロセスツリーの指標、起動、並べ替え、出力速度、パッケージサイズを必要に応じて確認します。 |
| P10 形式ロジックから Qt を分離 | [アーキテクチャ](../../design/system-architecture.md), [形式](codec-extension-sdk.md) | T10 Qt を含まないコア、明示的なアダプター選択、形式固有の回帰を確認します。 |
| P11 秘密情報とデータを保護 | [ファイルの権限](file-security.md), [プライバシー](../engineering/privacy.md) | T11 用途を限定したパスワード、機密を除いた診断、意図されたネットワークアクセスを確認します。 |
| P12 配布物を監査 | [品質](quality-engineering.md), [リリース](build-release.md) | T12 固定入力と成果物の識別、構成一覧、ライセンス資料、パッケージの信頼性、必要なインストール検査を確認します。 |
| P13 外部拡張を制御 | [形式](codec-extension-sdk.md), [SDK 試験](../engineering/sdk-trial.md) | T13 試験に限った相互運用性とネイティブ権限を検証します。製品での拡張インストールは設計案です。 |
| P14 プレビューと開く操作を制限 | [デスクトップ](qt-quick-ui.md), [ファイルの権限](file-security.md) | T14 自動プレビュー、対応内容、処理プロセスの上限、非公開の一時出力を確認します。 |
| P15 実際の結果を報告 | [ブリッジ](qt-rust-bridge.md), [ファイルの権限](file-security.md), [タスク](task-scheduling.md) | T15 入力元や出力先の変更、確定後の失敗とキャンセル、後処理、不確実な結果を確認します。 |

<a name="xdzip-cross-cutting-verification"></a>

## 横断的な検証

| 検証群 | 対象となる変更 | 検証の重点 |
| --- | --- | --- |
| T16 ブリッジの所有権とライフサイクル | ブリッジ、接続、コントローラーの寿命に関する変更 | 現在の要求 ID、受信側の起床、保持されたタスク、安全な破棄を確認します。 |
| T17 形式の相互運用性と異常系 | 形式またはコーデックの変更 | 必要な独立した正常サンプル、切り詰めや非対応プロファイル、出力の上限を確認します。 |
| T18 インストール、更新、削除 | プラットフォームまたは配布経路の変更 | 実パッケージ、クリーン環境の実行時依存、インストール ID、更新の引き継ぎ、対応する削除や復旧を確認します。 |

<a name="xdzip-representative-workflows"></a>

## 代表的な操作

1. ZIP を開いて検索し、選択結果を固定してから展開し、出力バイトを比較します。「すべて展開」はアーカイブ全体を対象とします。
2. 同名または衝突する項目を定めた競合方針で展開し、結果のファイル名と状態を確認します。
3. 確定前と確定中にキャンセルし、完了済みファイルが報告されること、作成が確定前に失敗した場合に検証済みの既存アーカイブが保護されることを確認します。
4. 管理下のテスト中に入力元または出力先を変更して実際の結果を確認します。異常終了後は自動再実行を前提にせず残存物を調べます。
5. 同梱の形式処理プロセスまたは独立 SDK 試験を契約に従って失敗させ、未完了のデータが成功結果として公開されないことを確認します。

文章、形式、リンクの静的検査で文書を検証します。フィクスチャー、オフスクリーン、ネイティブ自動、手動の受け入れ記録はそれぞれの範囲に適用します。[品質管理](quality-engineering.md)、[検証ガイド](../../../developers/qa/README.md)
