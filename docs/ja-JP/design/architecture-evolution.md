<a name="xdzip-architecture-evolution-and-current-boundaries"></a>

# アーキテクチャーの変遷と現在の範囲

[English](../../design/architecture-evolution.md) | [简体中文](../../zh-CN/design/architecture-evolution.md) | [繁體中文（台灣）](../../zh-TW/design/architecture-evolution.md) | 日本語 | [한국어](../../ko-KR/design/architecture-evolution.md) | [Français](../../fr-FR/design/architecture-evolution.md) | [Deutsch](../../de-DE/design/architecture-evolution.md) | [Español](../../es-ES/design/architecture-evolution.md) | [Português (Brasil)](../../pt-BR/design/architecture-evolution.md) | [Русский](../../ru-RU/design/architecture-evolution.md)

<a name="xdzip-from-framework-experiments-to-the-product"></a>

## フレームワークの試作から製品へ

初期のフレームワーク試作は、架空のアーカイブ一覧と模擬タスクで検証しました。製品は Qt Quick と型付き CXX ブリッジを採用し、その後 Rust に実際のアーカイブ、タスク、ファイル操作を実装しました。試作の JSON、C ABI、処理時間は当時の実験に対応します。選定理由と再検討の条件は[アーキテクチャーの決定記録](architecture-decisions.md)にあります。

製品は行モデルの上限を設け、クエリーの識別子を検証して古い結果が新しい表示を置き換えないようにします。物理項目の識別、形式ごとの機能、ファイルへの許可、一時出力と検証後の確定を実装しています。独立した CLI は Qt を介さず同じコアを呼び出します。[システム構成](system-architecture.md)と[ドメインモデル](domain-data.md)も参照してください。

<a name="xdzip-current-experimental-and-proposed-work"></a>

## 現行機能、実験、提案

| 領域 | 現在の状態 |
| --- | --- |
| ランタイムと索引 | ホストに実装済みです。通常の単一ファイル ZIP/ZIP64 では非公開のディスク索引に切り替えられます。タスクとメタデータには別々の上限があります。[性能設計](performance.md) |
| 形式ごとの隔離 | 組み込みルーティングは、プロセス内の Rust 実装か付属の形式ワーカーを選びます。登録内容、プロファイル、ランタイムの受理条件で対応操作が決まります。[形式アダプター](codec-extension-sdk.md) |
| 外部拡張 | 独立した読み取り専用の XDP1 試験は、上限付きプロトコルとプラットフォーム別の条件を備え、別のホストで実行します。製品による拡張の検出、インストール、有効化は提案段階です。[SDK 試験](../engineering/sdk-trial.md) |
| 永続状態と並行処理 | 現在のスケジューラーは負荷の高いデスクトップ操作を一度に一つ受け付けます。起動をまたぐ索引の保存、タスクの復旧、汎用の共有スケジューリングは設計案です。[タスクスケジューリング](task-scheduling.md)、[ファイルの確定](file-security.md) |

現行ソースの機能は[対応形式の表](../../../README.ja-JP.md#xdzip-format-support)にあります。公開済みのビルドは該当するリリースノートと成果物を参照してください。ネイティブ実装、実験用ツール、測定結果にはそれぞれ対象範囲があります。
