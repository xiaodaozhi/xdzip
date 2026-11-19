# アーカイブを検証してタスクを管理する

[English](../../user-guide/checks-and-tasks.md) | [简体中文](../../zh-CN/user-guide/checks-and-tasks.md) | [繁體中文（台灣）](../../zh-TW/user-guide/checks-and-tasks.md) | 日本語 | [한국어](../../ko-KR/user-guide/checks-and-tasks.md) | [Français](../../fr-FR/user-guide/checks-and-tasks.md) | [Deutsch](../../de-DE/user-guide/checks-and-tasks.md) | [Español](../../es-ES/user-guide/checks-and-tasks.md) | [Português (Brasil)](../../pt-BR/user-guide/checks-and-tasks.md) | [Русский](../../ru-RU/user-guide/checks-and-tasks.md)

<a name="xdzip-test-an-archive"></a>

## アーカイブを検証する

1. アーカイブを開き、ツールバーの **検証** を選びます。
2. パスワードを求められたら入力し、結果を待ちます。対応する内容を検証し、展開したファイルは書き込みません。
3. 失敗した場合は原因を確認します。対象の項目が表示されている場合は、それも確認してください。チェックサムや認証に対応した形式もあれば、構造と読み取り可否を調べる形式もあります。検証が成功しても、確認できるのは形式が対応する範囲です。[整合性検証の動作](../engineering/archive-checking.md)

検証できないアーカイブもあります。たとえば、ディスクにインデックスを置く ZIP はタスクの上限内で選択ファイルを展開できます。NSIS で参照できるのはインストーラーのメタデータのみです。[大きなアーカイブの上限](../engineering/large-index.md)と[対応形式](../../../README.ja-JP.md#xdzip-format-support)を参照してください。

<a name="xdzip-progress-cancellation-and-results"></a>

## 進行状況、キャンセル、結果

タスクメニューを開くと、進行中の作業に戻ったり結果を確認したりできます。アーカイブのタブを閉じてもバックグラウンドのタスクは続きます。パスワードやファイル名の重複について応答するまで処理が待機することがあります。

タスクのウィンドウで **キャンセル** を選ぶと停止を要求します。最終状態が表示されるまで待ってください。保存が確定したファイルは残ることがあります。結果の詳細を開き、完了、スキップ、未完了を区別します。表示される件数と理由から再試行する項目を判断してください。完了したタスクの履歴を消去するとタスクの記録が消え、ディスク上のファイルは残ります。[タスクウィンドウの動作](../ux/task-windows.md)
