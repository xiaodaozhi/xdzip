# コマンドラインのランタイム契約

[English](../../engineering/command-line.md) | [简体中文](../../zh-CN/engineering/command-line.md) | [繁體中文](../../zh-TW/engineering/command-line.md) | 日本語 | [한국어](../../ko-KR/engineering/command-line.md) | [Français](../../fr-FR/engineering/command-line.md) | [Deutsch](../../de-DE/engineering/command-line.md) | [Español](../../es-ES/engineering/command-line.md) | [Português (Brasil)](../../pt-BR/engineering/command-line.md) | [Русский](../../ru-RU/engineering/command-line.md)

CLI は独立した Rust ホストで、同じ `xdzip-runtime` を通して形式と安全な I/O のサービスを呼び出します。Qt、QML、画面セッションには依存しません。7z、RAR、CAB、ISO、UDF は引き続き同梱の隔離ワーカーを使い、別の展開ツールを PATH から探しません。プラットフォームとアーキテクチャーは[サポート方針](support-policy.md)に従います。独立した Linux CLI に X11 は不要です。実行にはシステムの OpenSSL 3 と証明書、ビルドにはさらに libssl-dev と pkg-config が必要です。

`v0.2.0` 以降の公式リリースは三つのプラットフォーム向けの独立 CLI パッケージを提供し、デスクトップパッケージには非公開 CLI を含めます。以前のリリースファイルは遡って変更されません。ダウンロードは[リリース](https://github.com/xiaodaozhi/xdzip/releases)を参照してください。ソースからのビルド、インストール、更新の境界は以下の節と[リリースプロトコル](release-protocol.md)に記載します。実装と受け入れの進捗は [#254](https://github.com/xiaodaozhi/xdzip/issues/254) で追跡します。

## 使用方法と出力

```sh
xdzip --help
xdzip version
xdzip list archive.zip --json
xdzip extract archive.zip docs/ -C output
xdzip extract archive.zip --entry-id 3 -C output
xdzip create backup.7z docs/ --level normal
xdzip create data.iso docs/
xdzip create portable.sfx.exe docs/ --level normal
xdzip create private.zip docs/ --encrypt --password-stdin
xdzip add backup.zip notes.txt --into docs
xdzip test backup.zip
xdzip update --check
xdzip update --download --output downloads
xdzip update --yes
xdzip update --status
xdzip gui archive.zip
```

引数なしまたは `--help` ではヘルプを表示します。`version`、`--version`、`--version-json` は `VERSION.json` とビルド時の Git 状態に基づく識別情報を報告します。`--version-json` は product、version、build、revision、dirty、component、channel、configuration、distribution、platform、architecture を含む独立したオブジェクトを出力します。ヘルプとメッセージの言語は `--language zh-CN|en-US` で選べます。既定では `LC_ALL`、`LC_MESSAGES`、`LANG` を順に調べ、選択値が `zh` で始まる場合は中国語を使います。公開識別子とエラーコードは英語です。`l`、`x`、`c`、`a`、`t` はそれぞれ `list`、`extract`、`create`、`add`、`test` の別名です。ファイル引数をそのまま渡すには `--` を使います。

すべての操作コマンドは `--json` を受け付けます。stdout は固定フィールド schema_version（1）、command、status、error_code、data、summary、warnings を持つ JSON オブジェクトを必ず一つ出力します。不明なサイズは null です。共通の圧縮 TAR ストリームを共有する子も含まれます。DEB の生 ar メンバーは正確な圧縮後サイズを保持します。リストの data は形式とエントリーを含み、各エントリーには id、path、display_name、size、packed_size、method、directory、link、payload_supported、encrypted、warning が含まれます。path は固定された有効な名前で、不正または解決できない名前は null です。display_name はエスケープ表示専用です。タスクの summary は実際に読んだ、デコードした、検証した、確定した、スキップした件数と、ディレクトリー・分割数を含み、クリーンアップ警告や不確かな結果も保持します。

進捗、エラー、対話的な問い合わせは stderr に出力します。JSON、非端末、または `--non-interactive` の呼び出しでは問い合わせません。パスワードや競合の判断が不足すると、待機中のタスクをキャンセルしてクリーンアップを待ち、interaction_required を返します。終了コードは 0 が成功または明示的な引き渡し、1 が失敗・部分完了・クリーンアップ警告、2 が不正な引数、3 が必要な対話・権限・インストール条件の不足、130 が確認済みキャンセルです。明示的なスキップは実行された方針で、スキップ数は別に報告します。キャンセルが受け入れられず確定が完了している場合は、実際の完了結果を保持します。

<a name="xdzip-archive-behavior"></a>

## アーカイブの動作

`list` は物理レコードを返し、重複を保持します。`extract` はスラッシュ区切りで、大文字小文字を区別した完全かつ正確な有効メンバーパスを受け付けます。ディレクトリーは再帰的に展開し、ルート以下の階層を保持します。重複パスには `ambiguous_member` を返します。`list` が返す 1 から始まる数値 ID を `--entry-id` で指定し、複数 ID にはフラグを繰り返します。明示されたパスや ID を解決できなければ、出力先ディレクトリーの作成前に失敗します。ワイルドカード、stdin からのアーカイブ読み取り、展開内容の stdout 出力は対応していません。

`--limit` または `--offset` を指定するとページ分割します。オフセットは 0 から始まり、最大 1,000,000 です。`--limit` のみならオフセットは 0、`--offset` のみならリミットは 128 です。リミットの有効範囲は 1～4,096 です。JSON の `data` は `offset`、`total`、null になり得る `next_offset` を含みます。`summary.entries` は実際に返した行数です。通常の単一ファイル ZIP/ZIP64 は 100 万件をインデックスできますが、ページ分割しない一覧は合計 100,000 件を超えると拒否します。どちらもシリアライズした JSON 行と表示テキストを合わせて 16 MiB の予算を使います。超過時は部分行を返さず `resource_limit` を返します。ページ境界によって物理 ID は変わりません。[大規模アーカイブのインデックス](large-index.md)を参照してください。

extract の既定の出力先は現在のディレクトリーです。`-C` が存在しないディレクトリーを指定した場合、無条件に再帰作成せず、最も近い既存の安全な親からコンポーネントごとに権限を取得します。出力先、ソースの識別情報、リンク、Unicode・大文字小文字の衝突、ステージング、検証、確定は引き続き[安全な展開](zip-safe-extraction.md)と[ファイル名のエンコーディング](filename-encoding.md)に従います。名前の競合は既定で Ask とし、`--conflict ask|skip|rename|overwrite` で明示的に方針を選びます。`--allow-high-ratio` は高い圧縮率だけを確認し、厳格なリソース制限を解除しません。

`create` は出力の[拡張子](format-registry.md)からプロファイルを選び、出力先の親ディレクトリーが存在する必要があります。`--level store|fast|normal|best` は選択プロファイルに対して検証します。通常の TAR と ISO はレベルを省略するか `store` を使います。LHA（`.lzh` または `.lha`）は `fast`、`normal`、`best` に LH5 のレベル 1、6、9 を使います。ZIP と 7z は `--encrypt` と `--volume-size` に対応し、7z はプロファイルに応じて `--encrypt-names` と `--solid` にも対応します。分割サイズには正の整数のバイト数または `KiB`、`MiB`、`GiB` の接尾辞を指定できます。選択プロファイルに未対応の引数は書き込み前に失敗します。

パスワードは端末の非表示入力または `--password-stdin` からの UTF-8 一行だけで渡します。改行は除きますがパスワード自体の前後は削りません。対話式の暗号化作成では二度の確認が必要です。平文パスワードの引数は拒否し、環境変数からパスワードを読まず、設定やログに書きません。長さとゼロ化は[パスワード契約](password-extraction.md)と[作成契約](zip-creation.md)に従います。

`add` はホストに更新を提出し、既定ではアーカイブのルートを使います。`--into` はアーカイブ相対名で既存の安全なディレクトリーを選びます。受け入れ可能な入力はエントリーを追加または置換できます。内部移動はデスクトップ操作です。CLI は対応する形式・レコード、入力保護、一時的な再構築、最終置換について[アーカイブ変更](archive-transfers.md)の検証を共有します。`test` はメンバーファイルを生成せずアーカイブ全体を検証します。JSON の `data.check_kind` は[アーカイブ検証](archive-checking.md)で定義された完了検査の強さを示します。

各 CLI 操作は独立したランタイムを持ち、デスクトップのブラウズセッションには参加しません。Ctrl+C や終了シグナルは協調的なキャンセルを要求し、終端状態、クリーンアップ、ランタイムの終了を待ちます。既に確定したファイルは残ります。共有サービスの `submit_wait` は短い状態ロックだけを順に取得し、キューの空きは待たず GUI の非ブロッキング受け入れを変更しません。カタログページは物理エントリー最大 128 件です。SelectMembers の件数と名前の既存予算を維持し、すべてのリクエストは不変のセッション識別情報に結び付きます。

<a name="xdzip-build-and-command-installation"></a>

## ビルドとコマンドのインストール

```sh
python3 scripts/cli.py build
python3 scripts/cli.py test
python3 scripts/cli.py package --output build/package/cli
build/cli/bin/xdzip --help
```

この入口はリポジトリーで固定した Rust、CMake、プラットフォームの C++ ツールチェーン、既存の隔離ワーカーを使い、Qt を必要としません。Windows のパッケージ作成には、固定されたアプリローカル VC++ ランタイムと元のライセンスを用意する既存の `prepare-windows-runtime.py` も必要です。Qt の統一ビルド入口も、GUI 実行ファイル、アプリアイコン、関連付け入口の名前を変えずに非公開 cli ディレクトリーに CLI をビルドします。

macOS 同梱 CLI の実行ファイルは `Contents/Helpers/cli/` にあります。通常のコンテキスト JSON は `Contents/Resources/xdzip-cli-context.json` にあり、アプリのリソース署名で保護されます。CLI は固定の Contents/Helpers/cli 構造でのみこのリソースを発見し、スキーマ、デスクトップ種別、固定された GUI 相対パスを検証します。マーカーがなければポータブルの識別情報とし、不正なマーカーは拒否します。繰り返しビルドでは Helpers 内の古い生成マーカーを削除します。構造エラーの修正のために署名検証を緩めてはいけません。

新しいデスクトップバンドルはコンテキストに `worker_relative` も宣言し、デスクトップの形式ワーカーを共有します。受け入れるのはプラットフォーム固定の相対パスだけです。Windows では隣接するデスクトップワーカー、macOS では `Contents/MacOS/xdzip-native-codec`、Linux ではビルドに隣接するか配布された `bin/` のワーカーです。ワーカーがない、リンクである、または想定外の場合、PATH や別のコピーを探さずコンテキストを拒否します。このフィールドのない旧コンテキストと独立・管理対象 CLI パッケージは隣接ワーカーの検索を維持します。開発者の明示的なワーカー上書きは引き続き優先します。配布時の検索は `scripts/check-cli.py --default-worker` で検証します。通常のテスト上書きだけでは証明できません。

独立パッケージを展開したら bin/xdzip を実行し、Windows では .exe を付けます。`bin/xdzip setup` はユーザー単位の安定した入口とバージョン付きランタイムディレクトリーを明示的にインストールします。Unix の既定入口は ~/.local/bin、Windows はユーザー LocalAppData の xdzip/cli/bin です。管理ディレクトリーはプラットフォームのユーザーデータ位置を使います。`--bin-dir` と `--install-dir` で位置を明示的に変更できます。ユーザーが自分でディレクトリーを PATH に追加し、ツールはシェルの起動ファイルを変更しません。既存の入口や登録ファイルは上書きしません。

管理対象の識別情報には有効な installation.json が必要です。登録ファイルがなく versions という名前だけのディレクトリーはポータブルのままです。

実行中の Windows ランチャーを削除する場合、所有するコピーのハッシュが一致していれば同じコマンドディレクトリーの非公開 .xdzip-retired サブディレクトリーに移します。登録は直ちに削除し、結果に retained_launcher を記録します。後の明示的な setup は、上限付きの受領記録に含まれ、ハッシュが一致し、プロセスが終了したコピーだけを回収します。変更されたファイルは残し、上書きしません。バージョン付きディレクトリーに残った CLI で setup を再実行すれば登録を復元できます。

`distribution` フィールドはソースとリリースの識別情報を区別します。通常のソースビルドは、変更がなく Release 構成であっても確認とダウンロードのみ対応します。自動置換には、専用 Release パイプラインが付けた distribution 識別情報、有効な管理対象インストール、認証済みの対象署名がすべて必要です。

Windows デスクトップインストーラーの PATH オプションは既定でオフで、他の xdzip コマンドとの競合を拒否します。Mac/Linux ではデスクトップパッケージの非公開 CLI から setup を実行できます。デスクトップインストール内の CLI だけを登録し、個別に更新できるコアを外へコピーしません。`setup --remove` は、このツールが所有しハッシュを検証した入口と登録ファイルだけを削除します。バージョン付きディレクトリーと復旧資料は残し、ユーザー入力や使用中の可能性があるランタイムディレクトリーは削除しません。

安定ランチャーはシェルのエスケープを使いません。Unix では実際の CLI を exec し、Windows ではコンソール子プロセスを待ち、キャンセルと終了コードを転送します。アトミックな current.json ポインターが現在のランタイムディレクトリーを選びます。初回インストール、新バージョンの自己検証、ファイルの確定に失敗した場合、インストール成功の結果を返してはいけません。

## 更新、デスクトップへの引き渡し、復旧

update は現在のインストールを対象にします。独立 CLI は CLI だけを更新し、デスクトップパッケージ内の CLI はデスクトップ版全体を更新します。`--check` は照会、`--download` はダウンロードと検証、既定操作は確認後のインストール、`--status` は受領記録と実際の識別情報の確認を行います。`--yes` はこの更新だけを確認し、システム権限の代わりになりません。ポータブル、変更あり、ソース、識別不明のビルドは確認とダウンロードのみ可能です。独立 CLI の自動置換には管理対象で変更のない Release インストールが必要です。

CLI は固定の公開 GitHub Releases、チャネル、単調増加するビルド番号の規則を使います。三つのプラットフォームの CLI ファイルには独立した cli-release.json と cli-release.json.sig があり、埋め込み公開鍵で生バイトを検証してからサイズと SHA-256 を確認します。署名がないか鍵が信頼されない場合は手動取得の情報だけを提供し、不正な署名は明示的に失敗します。デスクトップ六パッケージ用 release.json は CLI エントリーを含まず、旧クライアントは元のプロトコルを使い続けます。

ネットワークは既定でシステム証明書と標準のプロキシ環境変数を使います。`--proxy` は資格情報のない HTTP/SOCKS5 プロキシ、`--no-proxy` は直接接続、`--mirror` は資格情報・クエリー・フラグメントのない HTTPS プレフィックスを選びます。元の GitHub URL、リダイレクト先ホスト、応答サイズ、タイムアウト、ダウンロード予算は引き続き制限されます。ネットワーク、TLS、オフライン、レート制限、互換性のある新バージョンがない結果を区別して報告します。自動ポーリングや再試行は行いません。

独立更新はユーザー所有の非公開ディレクトリーにステージングし、同じアーカイブランタイムで安全に展開します。固定された完全なファイル一覧と各ファイルのダイジェスト、製品、コンポーネント、ローカルのプラットフォームとアーキテクチャー、署名済みマニフェストのチャネルを検証し、ポインター切り替え前に新バージョンの `--version-json` 自己検証を実行します。トランザクションは相互排他的で、受領記録は対象の識別情報に結び付きます。失敗時は旧ポインターを保持するか明示的に復旧に入ります。自動ロールバックを装ってはいけません。

デスクトップの確認とダウンロードは、メインプログラムの GUI を使わないインストール識別入口だけを照会します。QApplication を作成せず、インストール受領記録も消費しません。インストール要求は同じユーザーの既存の単一インスタンスチャネルを通り、制限されたリクエストディレクトリーと候補バージョンだけを受け付けます。任意のコマンド、パッケージ、信頼ルートは受け付けません。リクエストと結果のファイルには Rust のネイティブなディレクトリー認可と安全な確定を再利用します。デスクトップコントローラーは候補を独立して再検証・照合し、「バージョン情報」、インストーラー、Sparkle、全タスクの終了バリアを再利用します。デスクトップ CLI タスクもインストールの使用中保護を保持し、インストールがタスクを破壊することはできません。

引き渡しが受け入れられた端末は handed_off だけを報告します。実際のインストールと復旧はデスクトップの受領記録と実行中の実際の識別情報に依存します。ダウンロードや引き渡しはインストール完了を意味しません。非対話モードは GUI やシステム権限が必要なデスクトップインストールを拒否します。インストールの安全性とネイティブ受け入れの境界は[更新契約](../../engineering/software-update.md)を参照してください。自動、ネイティブ、手動の受け入れは区別します。

`.iso` プロファイルはレベル 0 の非圧縮 ISO9660/Joliet データ画像を作ります。`--level` を省略するか `--level store` を使います。Windows x64 の `.sfx.exe` プロファイルは同梱された隣接 `xdzip-sfx.exe` を使って Store/Deflate ZIP を包みます。暗号化、ソリッドモード、分割を拒否します。どちらも既存出力の編集には対応しません。[ISO 作成](iso-creation.md)と[自前 SFX 作成](sfx-creation.md)を参照してください。
