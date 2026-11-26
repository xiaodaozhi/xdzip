# スクリプトで XDZip を使う

[English](../../integration/README.md) | [简体中文](../../zh-CN/integration/README.md) | [繁體中文](../../zh-TW/integration/README.md) | 日本語 | [한국어](../../ko-KR/integration/README.md) | [Français](../../fr-FR/integration/README.md) | [Deutsch](../../de-DE/integration/README.md) | [Español](../../es-ES/integration/README.md) | [Português (Brasil)](../../pt-BR/integration/README.md) | [Русский](../../ru-RU/integration/README.md)

スタンドアロン CLI は Qt やデスクトップセッションなしで動作します。[GitHub のリリース](https://github.com/xiaodaozhi/xdzip/releases)から、お使いのシステム向けの `xdzip-cli-` パッケージを選択してください。パッケージ全体を展開し、同梱のワーカーを実行ファイルの隣に置きます。展開したディレクトリから、macOS または Linux では `./bin/xdzip --help`、PowerShell では `.\bin\xdzip.exe --help` を実行します。

例では PATH にある `xdzip` を使います。展開した CLI の `setup` でユーザー用のランチャーを登録し、必要に応じてコマンドのディレクトリを PATH に追加します。`xdzip --version-json` を実行し、見つかったプログラムが `"component":"cli"` を返すことを確認します。デスクトップランチャーと CLI のコマンド名が同じ場合があります。競合したら、展開した CLI を明示的なパスで指定してください。[コマンドのインストールと削除](../engineering/command-line.md#xdzip-build-and-command-installation)

<a name="xdzip-create-and-check"></a>

## 作成と検証

新しく用意した書き込み可能で非公開の作業ディレクトリに、`hello.txt` を含む `docs` フォルダーを作ります。別のファイル `notes.txt` をそのフォルダーの隣に置きます。次の手順で ZIP を作成し、2 番目のファイルを既存のアーカイブ内ディレクトリ `docs/` に追加して、結果を検証します。

```sh
xdzip create sample.zip docs --level normal --json
xdzip add sample.zip notes.txt --into docs --json
xdzip test sample.zip --json
```

作成時の拡張子とオプションは[対応形式の表](../../../README.ja-JP.md#xdzip-format-support)から選択してください。出力は入力側のディレクトリツリーの外に置き、既存の親ディレクトリを指定します。`add` には条件を満たす既存のアーカイブが必要です。`--into` はアーカイブ内の既存ディレクトリを指定します。省略するとルートに追加します。`test` は展開ファイルを書き出しません。成功した場合の検証範囲は `data.check_kind` に示されます。[形式の一覧](../engineering/format-registry.md)と[整合性の検証](../engineering/archive-checking.md)も参照してください。

<a name="xdzip-list-select-and-extract"></a>

## 一覧表示、選択、展開

同じ作業ディレクトリで先ほどのアーカイブを使用します。空の展開先を使うか、既存ファイルの扱いを明示的に選んでください。

```sh
xdzip list sample.zip --limit 2 --json
xdzip extract sample.zip docs/hello.txt -C output --conflict skip --json
```

メンバーのパスは正確に指定し、大文字と小文字を区別し、区切りにはスラッシュを使います。同じパスのレコードが複数ある場合は、`list` が返した `id` で選択します。ID は 1 から始まります。例えば一覧で対象ファイルの ID が 2 なら、2 番目のコマンドの `docs/hello.txt` を `--entry-id 2` に置き換えます。複数のメンバーには `--entry-id` を繰り返し指定します。照合には `path`、表示には `display_name` を使います。アーカイブを変更した後は一覧を取り直してください。

最初のページのオフセットは 0 です。同じ `--limit` を使い、`data.next_offset` を `--offset` として渡すと次のページを読めます。`next_offset` が null になったら終了です。オフセットは 0 から始まる行位置で、ID は物理レコードを識別します。各コマンドはアーカイブを開き直すため、ページ取得の途中でソースを変更しないでください。[項目の選択と一覧表示の上限](../engineering/command-line.md#xdzip-archive-behavior)

<a name="xdzip-run-without-prompts"></a>

## 確認を求めずに実行

`--json` は対話形式の確認を無効にします。無人で処理を終えるには競合時の方針を明示してください。パスワードに対応する `extract`、`test`、暗号化付きの `create` などには `--password-stdin` を指定し、専用の標準入力パイプで UTF-8 のパスワードを 1 行送ります。暗号化して作成する場合は `--encrypt` も必要です。必要な判断やパスワードを受け取れない場合、CLI は `interaction_required` を返します。[パスワードと形式のオプション](../engineering/command-line.md#xdzip-archive-behavior)

<a name="xdzip-interpret-results"></a>

## 結果を解釈

`--json` でアーカイブを操作した場合は、stdout の UTF-8 オブジェクトを解析します。`schema_version`、`status`、`error_code`、`data`、`summary`、`warnings` を参照します。診断に stderr を利用できるようにしておいてください。`--version-json` はビルド情報を持つ別のオブジェクトを返します。

| 終了コード | 取り扱い |
| --- | --- |
| 0 | 結果とスキップ件数を確認します。デスクトップへの受け渡しに対応するコマンドでは `handed_off` が受け渡しの完了を示します。 |
| 1 | エラー、警告、概要で失敗、部分的な完了、後片付けの警告を確認します。 |
| 2 | コマンドまたは引数を修正します。 |
| 3 | 対話操作、認証、プラットフォーム、インストールについて報告された要件を確認します。 |
| 130 | キャンセルが確認されました。保存が確定したファイルがあるか確認します。 |

不明な値は null の場合があります。失敗やキャンセルの前に保存が確定したファイルは残ることがあります。再試行する前に、`summary.committed_files` があれば確認してください。デスクトップへの受け渡し、更新結果の記録、出力の全仕様は [CLI リファレンス](../engineering/command-line.md) を参照してください。
