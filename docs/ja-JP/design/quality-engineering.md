<a name="xdzip-coding-conventions-and-quality-engineering"></a>

# コーディング規約と品質管理

[English](../../design/quality-engineering.md) | [简体中文](../../zh-CN/design/quality-engineering.md) | [繁體中文（台灣）](../../zh-TW/design/quality-engineering.md) | 日本語 | [한국어](../../ko-KR/design/quality-engineering.md) | [Français](../../fr-FR/design/quality-engineering.md) | [Deutsch](../../de-DE/design/quality-engineering.md) | [Español](../../es-ES/design/quality-engineering.md) | [Português (Brasil)](../../pt-BR/design/quality-engineering.md) | [Русский](../../ru-RU/design/quality-engineering.md)

<a name="xdzip-code-and-ownership"></a>

## コードと所有権

- **Rust。** ワークスペースのツールチェーン、lint、ロックファイルに従います。外部入力の長さ、オフセット、整数変換を検証します。不明なサイズと型付きエラーを保持します。unsafe と FFI の前提、所有権、ライフタイム、失敗時の挙動を記載します。同期のクリティカルセクションは短く保ち、アーカイブ I/O はその外で実行します。
- **C++ と Qt。** C++17、RAII、明確な QObject の所有権を使います。モデルは GUI スレッドで変更し、スレッド間では所有権のある値を渡し、キューに入れた通知は有効な受信側に結び付けます。イベントの処理に GUI が必要な間は、終了を非同期にします。アーカイブとファイルのポリシーは Rust が担当します。[Qt/Rust ブリッジ](qt-rust-bridge.md)
- **QML。** 状態とユーザーの意図をコントローラーに伝えます。共有コントロールと生成された視覚・文言リソースを再利用します。ファイル I/O とアーカイブの解釈はデリゲートの外で行います。ウィンドウの変更は登録情報、ネイティブ動作、ビジュアル標準に従います。[Qt Quick インターフェース](qt-quick-ui.md)、[ビジュアル設計手順](../visual/design-workflow.md)

不変条件と失敗後に残る状態を記載します。ファイル操作では、確定許可が何を保護するか、どの識別情報を確認するか、失敗時にどの部分がすでに表示され得るかを説明します。[ファイルの確定](file-security.md)

<a name="xdzip-verification-layers"></a>

## 検証の層

| 変更 | 確認する証拠 |
| --- | --- |
| ドメイン、クエリー、サービス | 識別、上限、同名、順序、古い応答、上限付き転送、キャンセルの Rust テスト。 |
| 形式の操作 | 自作の試験データ、必要に応じて独立実装での往復、拒否、破損、非対応プロファイル、上限の試験。 |
| ネイティブファイルとタスク | 実際のファイル識別情報と書き込んだバイト、キャンセルと確定の競合、既存出力の保護、対象プラットフォームでの後処理。 |
| デスクトップ、Qt、アクセシビリティ | モデルとコントローラー、QML と共有コントロールの検証、必要なネイティブ操作、対象の表示または支援技術の実機証拠。 |
| 性能と配布 | コミットを特定した負荷、レンダラーとリソースの測定、またはチャンネル別のビルド識別、インストールと更新結果、依存の記録。 |

変更に必要な検証は[検証ガイド](../../../developers/qa/README.md)で決めます。プロトコル適合、テストデータやオフスクリーン、ネイティブ自動化、手動のデスクトップ観察では確認できる事項が異なります。既存の回帰試験と成果物の範囲は[開発ビルドガイド](../engineering/developer-build.md)と[要件追跡](requirements-traceability.md)から参照できます。

<a name="xdzip-existing-ci-and-additional-techniques"></a>

## CI と追加の手法

通常の [CI](https://github.com/xiaodaozhi/xdzip/blob/main/.github/workflows/docs.yml) は文書、生成素材、レイアウト仕様、書式、ワークフロー構造を検証します。より重い Rust、形式、Qt のネイティブ検証は Foundation と Qt のワークフローを別途起動して実行します。リリース成果物は専用ジョブが扱います。起動条件と通過条件は [CI 方針](../../../developers/engineering/ci-policy.md) を参照してください。必要なローカル検証とワークフローは、変更したコードとプラットフォームで決まります。

有効な範囲でファジング、サニタイザー、モデル検査、性能ベンチマークを使います。上限があり再配布可能なテストデータを選び、結果はこのガイドと別に保存します。機密のある報告は[セキュリティ方針](../../../SECURITY.md)に従います。予定の検証や試験例を実行済みまたは受け入れ済みとして扱いません。

<a name="xdzip-maintaining-the-design"></a>

## 設計の保守

実装が識別子、コマンド、機能、タスク、書き込み境界を変えるときは、設計、技術リファレンス、受け入れ試験へのリンクを更新します。すべての言語版をそろえます。[アーキテクチャーの決定](architecture-decisions.md)は理由、[性能設計](performance.md)は測定規則をまとめます。実際の結果と制限は変更ごとに記録します。
