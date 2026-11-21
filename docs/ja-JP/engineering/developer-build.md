# 開発ビルドとプラットフォーム検証

[English](../../engineering/developer-build.md) | [简体中文](../../zh-CN/engineering/developer-build.md) | [繁體中文](../../zh-TW/engineering/developer-build.md) | 日本語 | [Español](../../es-ES/engineering/developer-build.md) | [Português (Brasil)](../../pt-BR/engineering/developer-build.md)

このページは、3 つのプラットフォームで環境の準備、ビルド、テスト、パッケージ化、検証を行うための共通入口です。共通の処理は [scripts/desktop.py](../../../scripts/desktop.py) で実行します。各プラットフォームの文書は、通常のコマンドを別々に管理するのではなく、配置構成、ツールの動作、システム上の制約を補足します。

単独の CLI は Qt、X11、グラフィカルセッションを使わずに `python3 scripts/cli.py build`、`test`、`package` で扱います。Linux でのコンパイルには `libssl-dev pkg-config` も必要で、実行時にはシステムの OpenSSL と証明書を使用します。デスクトップビルドでは専用の CLI ディレクトリーも自動生成します。コマンド、インストール、信頼できる更新の境界は [CLI 契約](command-line.md)を参照してください。

開発ビルドは実験的なものであり、正式な Release インストーラーではありません。機能と制限はルートの [README](../../../README.ja-JP.md)、サポート範囲は[サポートポリシー](support-policy.md)を参照してください。過去の `v0.1.0` リリース審査は [#4](https://github.com/xiaodaozhi/xdzip/issues/4) と [#53](https://github.com/xiaodaozhi/xdzip/issues/53)に記録されており、今後のバージョンにはそれぞれの候補について新たな審査記録が必要です。

<a id="environment-and-entry-points"></a>

## 環境と入口

最初に Python 3.10 以降、Git、CMake 3.24 以降、Ninja、rustup とプラットフォームのコンパイラー／SDK をインストールします。Rust のバージョンはルートの rust-toolchain.toml で指定し、Qt は 6.8.2 に固定します。依存関係と更新の規則は[ツールチェーン方針](toolchain.md)を参照してください。コマンドはリポジトリーのルートで実行します。Windows では例の `python3` を利用可能な `python` に置き換え、x64 MSVC 開発環境を使用します。

## ローカルツールチェーンのディレクトリー

Visual Studio の PowerShell 環境を読み込むときは `USERPROFILE`、`LOCALAPPDATA`、`APPDATA` の Unicode 値を保持し、読み込み処理でエンコーディングが変わった場合は復元します。Qt は既定のキャッシュや設定パスにこれらの場所を使うため、値が壊れるとコンパイラー自体が動作してもファイルシステム操作が失敗することがあります。

`prepare` は既存 SDK 内に、CVE-2026-116337 と CVE-2026-105946 に対応するセキュリティ修正済み QtNetwork をビルドして検証します。再ビルドしたライブラリーで他の実行時モジュールを置き換えることはありません。Perl、make（Windows では nmake）、OpenSSL コマンドラインツールも必要です。Ubuntu には `zlib1g-dev libpcre2-dev libicu-dev libglib2.0-dev libzstd-dev libkrb5-dev` が必要です。ソース、生成宣言、ビルドキャッシュは `build/` 内に保持し、既定では `build/toolchain/qt-network-security/<platform>` を再利用します。入力と実際のライブラリーのダイジェストが一致しない場合、製品ビルドとパッケージ化は直ちに失敗します。`prepare` を再実行してください。来歴、ABI、TLS 機能、障害からの復旧は[パッチ保守ノート](../../../patches/qt/README.md)を参照してください。

共通入口のローカル依存関係には、Git が無視しコミットしない `build/toolchain/` のみを使います。

| 内容 | 固定ディレクトリー | 備考 |
| --- | --- | --- |
| Qt SDK 6.8.2 | `build/toolchain/Qt/6.8.2/<platform-subdirectory>` | `<platform-subdirectory>` は `macos`、`msvc2022_64`、`gcc_64` のいずれか。qmllint、qmake、CMake 設定、Qt ランタイムはここから取得 |
| Python ツール環境 | `build/toolchain/desktop-tools` | aqtinstall、パッケージ化、検証スクリプト用の固定仮想環境 |
| Qt ダウンロード一時領域 | `build/toolchain/qt-install-tool` | Qt インストーラーのダウンロードと一時入力のみ |

`scripts/desktop.py prepare --install-qt`、`doctor`、`build`、`test` は既定でこれらの場所を共有します。開発パイプラインごとに `.cache/Qt`、`.cache/desktop-tools` などの別コピーを作成しません。`--qt-prefix`、`--qt-root`、`--tools-dir` を明示するのは、異なる Qt バージョン、プラットフォーム、実験ビルドを分離する場合に限ります。CI では runner の一時ディレクトリーを明示できますが、CI のフォールバックパスはローカルの既定値を変更しません。CMake、Ninja、rustup、Rust、プラットフォームの C++ コンパイラーはホストツールチェーンに属し、環境の事前確認に従ってインストール・呼び出しを行います。Qt ツールディレクトリーではコピーしません。

開発サイクルが終わり、ブランチと worktree を整理した後は、そのサイクルで `build/` に作成した一時ディレクトリーとファイルを削除します。`build/toolchain/` など再利用可能な共有ツールチェーンは残し、サイクル固有のビルド、診断、ダウンロード一時領域を削除します。規則と手動での補完方法は[リポジトリー内容と履歴の管理](../../../developers/engineering/repository-hygiene.md#研发周期工件收尾)を参照してください。

| プラットフォーム | 最初に準備するシステム環境 | Qt SDK サブディレクトリー |
| --- | --- | --- |
| macOS | ネイティブ Apple Silicon ターミナル、Xcode Command Line Tools、有効な macOS SDK | macos |
| Windows | MSVC 2022 x64 C++ ツールと Windows SDK、x64 Developer PowerShell／コマンドライン | msvc2022_64 |
| Ubuntu 24.04 | x86_64 C++ コンパイラー、OpenGL／EGL／xkbcommon の開発パッケージ、X11 実行時ライブラリー | gcc_64 |

macOS では `xcode-select --install` でコマンドラインツールを準備します。Ubuntu の基本開発パッケージには `libgl1-mesa-dev libegl1-mesa-dev libxkbcommon-dev libx11-dev libxext-dev libnautilus-extension-dev pkg-config` が含まれます。パッケージ化には patchelf、binutils（readelf と strip）、Debian パッケージツールも必要で、分離した実行時検証には利用可能な Docker が必要です。Windows では Visual Studio をインストールしただけで開発環境に入らないと、cl、link、rc、dumpbin が使えない場合があります。事前確認で実際に不足するツールを特定します。共通入口は、システムコンパイラー、SDK、管理者権限が必要なシステムパッケージを自動インストールしません。

Windows 開発者は[Community 無料ライセンスの利用方法](windows-licensing.md)に従い、そのバージョンの条件を読んで同意したうえでツールを準備できます。Build Tools または VC_redist があるだけでは利用資格を証明できません。企業利用と GitHub 標準ホスト runner での関連する再頒布には別の公開された権限があります。メンテナーはホストビルドのために Enterprise を購入したり、ローカルに Community IDE をインストールしたりする必要はありませんが、その権限はセルフホストマシンには適用されません。ライセンス審査のために既存の 2019 ツールをアンインストールする必要はありません。ローカル代替手段と現在の 2022 リリース SDK の互換性は別途検証してください。

すべてのコマンドで次のオプションを共有します。

| オプション | 既定値と用途 |
| --- | --- |
| `--qt-prefix` | まず `XDZIP_QT_PREFIX`、なければ `build/toolchain/Qt/6.8.2/<platform-subdirectory>`。bin ではなく SDK プレフィックスを指す |
| `--build-dir` | `build/desktop`。CMake のビルドとテスト用ディレクトリー |
| `--output-dir` | `build/package`。成果物と検証出力用ディレクトリー |
| `--tools-dir` | `build/toolchain/desktop-tools`。分離した Python ツールと準備資料 |
| `--configuration` | `Release`。別の開発構成を明示的に選択可能 |

build-dir と output-dir は、基礎となるパッケージ化と検証の出力境界に合わせ、リポジトリーの build/ 内の独立したサブディレクトリーにする必要があります。同じ実行の各段階では同じオプションを使います。明示した `--qt-prefix` は環境変数より優先され、グローバルな CMAKE_PREFIX_PATH の変更は不要です。入口スクリプトは子プロセス向けに Qt とツールの検索環境を準備します。通常のデスクトップ起動にオフスクリーンテスト用変数を継承させないでください。生成物は無視対象の build/、target/、dist/ またはツールキャッシュにのみ置き、コミットしません。

## 段階別の実行

各段階は独立して実行でき、前段階の準備、ビルド、テストを暗黙に再実行しません。前提条件が不足するとエラーになるため、対応する段階を実行してから再試行してください。各段階は処理前に必要な環境を確認し、`doctor` はビルドやパッケージ化を行わずに問題を確認します。

保護された Windows ビルドは、既存のコンソール内のプロセスを調整します。異なるコンソールは並行実行できますが、同じ既存コンソールを共有するビルドは順次実行します。ビルド中に他のコードが保護対象プロセスをそのコンソールから切断したり再接続したりしてはいけません。コンソールを持たないプロセスが作成した一時コンソールはそのプロセスと子プロセス用であり、外部プロセスは接続してはいけません。古いスクリプトはウィンドウハンドルのないコンソールに共通のフォールバックロックを使い、新しいスクリプトと調整できません。古いビルドが完了するまで、同じウィンドウのないコンソールで新バージョンのビルドを始めないでください。プロセス一覧を判定できない場合はコードページを変更せずに停止します。

非対話ビルドでは、標準入力、標準出力、標準エラーをすべてリダイレクトした独立コンソールラッパーを明示的に利用できます。PowerShell の例：

```powershell
New-Item -ItemType Directory -Force build\isolated-logs | Out-Null
cmd /c "python scripts\windows_detached_console.py --timeout 7200 -- python scripts\desktop.py build <NUL >build\isolated-logs\desktop.log 2>&1"
$LASTEXITCODE
```

ラッパーはコマンドごとに独立した非表示コンソールを作成し、タイムアウト時または監督プロセスの終了時に子プロセスを終了します。完了シグナルがない場合は失敗します。並行ビルドには引き続き別々の `--build-dir` が必要です。ラッパーはビルドディレクトリーをロックしません。既存の desktop-build と `windows_ninja_safety.py` の依存関係チェックは実行されますが、他のコマンドをラップしてもそれらのチェックが自動追加されるわけではありません。この入口は非対話コマンド専用で、プロンプト、パスワード入力、疑似コンソールでの対話はサポートしません。

`prepare` は現在の製品用フォーマットソースの一覧を検証します。ZIP、TAR/GZIP、7z、RAR、CAB、光ディスクコンテナーは独立した Rust 実装を使い、製品向け公式ソース一覧は空で、公式 UnRAR／7-Zip エンジンをダウンロードまたはコンパイルしません。独立したテスト参照はそれぞれ固定されたソースから準備し、製品の実行時依存関係と分離します。限定的な RAR マッピングの元のソース通知はパッケージと対応ソース資料に残します。[ライセンス契約](licensing.md)を参照してください。`build` は準備済み入力のみを使い、固定されていないバージョンをダウンロードしません。Rust のみのテストでは、まず `xdzip-native-codec` をビルドして `XDZIP_NATIVE_CODEC` にそのワーカーを指定し、ワークスペーステストを実行します。ネイティブバックエンドのタスクフローでは、追加で `cargo test -p xdzip-runtime --locked native_backend_tests -- --ignored` を実行します。共通 `test` と Foundation CI の両方がこれを要求します。ignored はフォーマットチェックの合格を意味しません。ソース、ストリームの認可、ライセンスの境界は[ネイティブバックエンド契約](native-codecs.md)を参照してください。

| コマンド | 用途と前提条件 |
| --- | --- |
| `doctor --for-stage build` | PATH、固定 Rust、Qt、コンパイラー、SDK とバージョンを確認。既定でビルド環境を確認 |
| `prepare` | 共通ツールと依存関係を準備。Qt、Rust ツールチェーン、パッケージ資料のダウンロード・インストールには以下の明示スイッチが必要 |
| `build` | 実際の製品、テスト、診断を構成してビルド。環境準備が必要で、未コミットのソース変更は許可 |
| `test` | 既定で Rust ワークスペーステスト、qmllint、CTest を実行。同じ構成の完了済みビルドが必要で、未コミットの変更は許可 |
| `package` | ランタイムの配置、ライセンス・来歴資料の収集、配置監査、選択した成果物の生成を実行。クリーンなソース、同一コミットのビルド・テスト回執、パッケージツールが必要 |
| `verify` | 既存ランタイムまたはパッケージを再ビルド、製品インストール、既定の関連付け変更なしで検証。成果物とプラットフォーム検証環境が必要 |

事前確認では、必要なツールの不足、バージョン不一致、現在の段階に不要なツールを区別します。Docker または Inno Setup がなくても通常の開発ビルドは妨げられません。パッケージ化の事前確認では Rust／Cargo の呼び出しと固定依存関係のメタデータを実際に確認し、PATH や依存キャッシュの問題をライセンス収集時まで見逃さないようにします。失敗したコマンドと検証の制限はローカル／CI 出力に残し、部分的な完了を合格と報告しません。

まず固定された Qt と Rust のツールチェーンを準備します。

```sh
python3 scripts/desktop.py prepare --install-rust --install-qt --qt-root build/toolchain/Qt
python3 scripts/desktop.py doctor --for-stage build
python3 scripts/desktop.py build
```

`--install-rust` はインストール済み rustup を使ってリポジトリーで固定したツールチェーンを準備し、rustup 自体はインストールしません。`--install-qt` は既定で固定 Qt SDK を `build/toolchain/Qt` に明示的にダウンロードします。既存 SDK を再ダウンロードする必要はありません。各段階にそのパスを指定してください。

```sh
python3 scripts/desktop.py doctor --qt-prefix build/toolchain/Qt/6.8.2/gcc_64
python3 scripts/desktop.py build --qt-prefix build/toolchain/Qt/6.8.2/gcc_64
```

ビルド後、完全な `native_backend_tests` バッチに必要な公開テスト入力を明示的に準備します。CAB、ISO/UDF、基本 RAR のフィクスチャーは `tests/fixtures/cab-iso/` と `tests/fixtures/native-archives/` のプロジェクトソースに保持します。7z テストは新しくビルドした Rust ワーカーで独自のアーカイブを作成します。このランタイムバッチは公式 7-Zip／UnRAR ツールやマウント済みイメージを必要としません。他の Foundation 相互運用参照は[ネイティブバックエンド契約](native-codecs.md)に従います。

固定された公開 RAR PPM、旧パスワードなどの入力を初めて取得する場合は、明示的に次を実行します。

```sh
python3 scripts/prepare-rar-rust-fixtures.py --output build/native-test-inputs
```

サイズと SHA-256 で検証済みの固定キャッシュがある場合は、キャッシュパスを置き換え、マニフェストの `junrar/` サブディレクトリー構造を維持して、明示的にオフラインモードを使用します。

```sh
python3 scripts/prepare-rar-rust-fixtures.py --output build/native-test-inputs --archives /path/to/fixed-rar-cache --offline
```

固定ソースと通知は `scripts/rar-rust-fixtures.json` とその公開互換性一覧にあります。出力される `sources.json.environment` にはソースルートからの相対パスを使います。現在の対応ソース資料にはこれら 7 個の公開キャッシュファイルが含まれず、Cargo の `--offline` でも提供されません。準備スクリプトはシェル環境変数をエクスポートしません。完全な `test` の前に、以下の明示的な引き渡しで子プロセスを起動します。

```sh
python3 -c "
import json, os, pathlib, subprocess, sys
root = pathlib.Path.cwd()
worker = pathlib.Path(sys.argv[1]).resolve(strict=True)
prepared = json.loads((root/'build/native-test-inputs/sources.json').read_text(encoding='utf-8'))
environment = os.environ.copy()
environment.update({name: str(root/value) for name, value in prepared['environment'].items()})
environment.update(XDZIP_NATIVE_CODEC=str(worker), XDZIP_NATIVE_CODEC_WORKER=str(worker), XDZIP_CAB_BACKEND='rust')
environment['XDZIP_RAR_FIXTURE_PASSWORD'] = 'password'
subprocess.run([sys.executable, 'scripts/desktop.py', 'test', *sys.argv[2:]], env=environment, check=True)
" build/desktop/tools/native-codec/xdzip-native-codec
```

Windows ではワーカーパスに `.exe` を追加します。独自の SDK、build-dir、tools-dir を使う場合は、ワーカーパスの後に同じオプションを追加します。例えば `--qt-prefix /path/to/Qt/6.8.2/gcc_64` を指定し、ワーカーパスを実際のビルドディレクトリーに合わせます。`XDZIP_CAB_BACKEND=rust` は固定 Rust CAB チェックを選択します。`password` は既存の公開フィクスチャーのテスト値で、テスト子プロセスにのみ渡します。この引き渡しは既定のワークスペーステストと完全な ignored バッチを維持し、入力のダウンロードやワーカーの暗黙ビルドは行いません。

`test` は子プロセスでのみオフスクリーン／ソフトウェア描画を有効にし、後続のデスクトップコマンドを汚染しません。通常の変更は影響範囲に応じて再ビルド・検証します。Rust コアのみの検証なら Qt を使わず対象クレートで `cargo test --locked` を実行できます。`test --skip-rust` は qmllint と Qt CTest を実行しますが、Rust ワークスペースと完全なネイティブバックエンドテストを省略し、回執の Rust は `not-run` になります。Rust について結論が必要な場合は別途実行し、実際の結果を記録してください。

ブラウザーテストは既存のスイート入口を通じて直接またはグループ単位で実行できます。各スイートプロセスはプロジェクトの `build/test-tmp/` 内に一意で短い一時ルートを作成します。終了時にはテストオブジェクトの破棄後、Qt アプリケーションの破棄前に TEMP、TMP、TMPDIR を復元してクリーンアップします。既定の設定ファイルも分離します。Unix では一時ルートと既存 IPC 名のバイト長が実際の sun_path 容量に収まるか確認します。パスが長すぎる場合はフィクスチャーの前提条件が明示的に失敗します。プロジェクト外に移動したり製品の IPC を変更したりせず、チェックアウトパスを短くしてください。グループ内の補助プロセスは単一インスタンス IPC のディレクトリー識別を維持するため親の一時ルートを継承し、専用の事前確認プローブを使い、親のクリーンアップ前に終了します。グループ間でルートを共有しません。言語再起動とオープン方式の子プロセスに明示的に渡したテスト設定は、新しい空の設定を使わず共有を続けます。実行前には子ファイルの作成、ロック、一時配置、アトミックなコミット、設定の読み戻しを実際に検証します。失敗時は `Fixture precondition` と非ゼロ終了を報告します。テストを省略したりウィンドウ非表示のアサーションを弱めたりしないでください。ディレクトリーの存在や QTemporaryDir::isValid だけでは書き込み可能性を証明できず、フィクスチャーを通すためにシステム権限を変更しません。

テスト設定のパスには `tests/test_environment` によるスコープ付き保存・復元を使用します。Windows はシステム環境ブロックでワイド文字値を保持し、未設定と明示的な空の値を区別します。Unix は元の環境バイト列を保持します。MSVC CRT の設定関数は空値を削除するため、ネイティブ環境ブロックを別途復元します。空値の存在は Qt の CRT における空／null の区別を保証するのではなく、ヘルパーの processValue または QProcessEnvironment で確認します。このヘルパーは直列テストのメインスレッド専用です。関係するエンジンやワーカーを作成する前に設定し、破棄／join の後に復元します。製品コードや並行する環境変更には使用しません。ASCII、中文、ANSI で表せない文字について、元のパス上の実ファイル、言語復元の子プロセス、失敗時のウィンドウとドラフトの保持を確認します。自動化結果は実際の人による設定操作と分けて記録します。

実際の製品名はすべてのプラットフォームで `xdzip` です。既定のビルド入口は次のとおりです。

| プラットフォーム | 入口 |
| --- | --- |
| macOS | `build/desktop/apps/desktop/xdzip.app/Contents/MacOS/xdzip` |
| Windows | `build/desktop/apps/desktop/xdzip.exe` |
| Linux | `build/desktop/apps/desktop/xdzip` |

`xdzip-ui-preview` コンポーネント例は実ファイルの受け入れ検証には使いません。build-dir を変更した場合はパスを調整してください。実際のビルド識別にはバージョン、チャンネル、コミット、変更の有無を記録します。モジュールと低水準の CMake デバッグは [Qt 開発アプリケーション](../../../apps/desktop/README.md)を参照してください。

## UI フォント生成

`prepare` は共有ツール環境に固定された FontTools／Brotli をインストールし、ハッシュ検証済みの上流 UI フォントをキャッシュします。各デスクトップビルドは対応言語のカタログと固定 QML／C++ テキストを自動収集し、フォントサブセットを `<build-directory>/generated/fonts/` に生成します。文字の追加にフォントバイナリーのコミットは不要です。準備後のビルドはオフラインで動作し、ソースキャッシュが欠落または破損している場合は明示的に失敗します。直接 CMake を使うビルドでは `Python3_EXECUTABLE` でその環境を選択する必要があります。

HTML のみのプレビュー、オフライン出力、手動ソース取り込み、フォントチェックは[共有フォントガイド](../../../design/assets/fonts/README.en.md)を参照してください。パッケージ化は一致する製品ビルドのフォントマニフェストを再生成せずに検証します。固定 UI テキストとユーザーデータのシステムフォント選択は、それぞれ従来の役割を維持します。

## パッケージ化と検証

`test` と `package` はビルド回執、ソースのフィンガープリント、構成、SDK 識別と実際の製品ファイルを確認します。ソースまたは構成を変更した後は、まず再ビルドします。パッケージ化前に納品するソースをコミットし、そのクリーンなコミットからビルド・テストしてください。package は変更ありの状態を無視したり、成果物の識別を変更したり、古いレポートを再利用したりしてチェックを回避してはいけません。ライセンス資料、Rust メタデータ、実際の CLI バージョンはこのソースと一致する必要があります。プラットフォームツールは既存パッケージディレクトリーの使用・上書き制限を確認します。古いファイルが混ざるのを防ぐため、再ビルド候補には新しい output-dir を推奨します。

Windows／Linux ではパッケージツールを準備し、共通の各段階を実行します。

```sh
python3 scripts/desktop.py prepare --packaging
python3 scripts/desktop.py doctor --for-stage package
python3 scripts/desktop.py build
python3 scripts/desktop.py test
python3 scripts/desktop.py package --kind all
python3 scripts/desktop.py verify --kind all
```

`prepare --packaging` は分離された Python 検証依存関係、固定ライセンステキストとスキーマキャッシュ、プラットフォームのパッケージ資料を準備します。Windows では固定 CRT と Inno Setup を含みます。システムツールは開発者が先にインストールする必要があります。`package --kind runtime` は配置済みランタイムディレクトリーと監査資料のみを準備します。その後、同じ output-dir で `package --kind installer` を実行して検証済みランタイムからインストーラーを作成できます。Linux では実行権限を保持する tar.gz も作成します。既定の `--kind all` は一つのコマンドで両方を実行します。Windows のパッケージ化は Release または RelWithDebInfo を使い、Debug Qt ライブラリーは配置しません。Windows は配置済みディレクトリーとユーザー単位のインストーラー、Linux はランタイムディレクトリー、tar.gz、.deb を提供します。Windows の検証には同じ構成のブラウザーテスト実行ファイルと Qt Test／オフスクリーンテスト入力が必要です。Linux の検証にはビルドテストで生成した m2-flat.zip が必要です。テストファイルは製品ディレクトリーに入りません。verify の `--kind runtime` はランタイムを検証し、既定の `--kind all` は選択した成果物を検証します。通常の検証ではインストール、アップグレード／ロールバック、ユーザーのファイル関連付け変更は行いません。

macOS DMG には、通常の開発・テスト用コピーが「このアプリケーションで開く」に入らないよう、書類ハンドラー候補として登録できる、別途選択した主ビルドが必要です。

```sh
python3 scripts/desktop.py prepare --packaging
python3 scripts/desktop.py build --build-dir build/desktop-dmg --macos-handler-rank Alternate
python3 scripts/desktop.py test --build-dir build/desktop-dmg
python3 scripts/desktop.py doctor --for-stage package --build-dir build/desktop-dmg
python3 scripts/desktop.py package --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
python3 scripts/desktop.py verify --build-dir build/desktop-dmg --output-dir build/package-macos --kind all
```

通常の macOS ビルドのハンドラーランクは既定で `None` であり、ユーザーの既定値を変更しません。`Alternate` は長期間使う主アプリまたは DMG の入力にのみ選択してください。ローカル macOS パッケージは `all` のみをサポートし、必要な ad-hoc 完全性署名を付けた arm64 開発 DMG を生成します。verify は移動後の Cocoa／Metal と実際の解凍を確認します。通常 CI は Mac パッケージの実行やアプリパッケージの配置、署名、圧縮、アップロードを行ってはいけません。専用 Release の例外は[リリースプロトコル](release-protocol.md)を参照してください。制限の全体とコピー登録の保守は [Mac 配置ガイド](macos-deployment.md)を参照してください。

配置構成と基礎ツールは [Linux 配置](linux-deployment.md)、[Mac DMG](macos-deployment.md)、[インストールとファイル関連付け](installers-associations.md)を参照してください。Windows／Linux インストーラーには正式な署名がなく、macOS は Developer ID 署名と公証のいずれもありません。インストール・実行時の問題は実際の製品と該当プラットフォームの検証で診断します。SDK 下で動作することは単独パッケージの検証を意味しません。

## CI との関係

コミットまたはプッシュ前に、変更内容に適したローカルビルドまたは検証を実行します。文書変更は `python3 scripts/check-docs.py` を実行し、影響する生成出力を確認します。Rust の変更ではフォーマットを確認し、少なくとも影響するクレートをビルドまたはテストします。分離フォーマットバックエンドを含むテストでは、上記の実際のワーカーと必要なフィクスチャーを準備します。C++、ブリッジ、QML の変更では `python3 scripts/desktop.py build` で対象製品をビルドし、関連テストを実行します。すべての変更に全フォーマットの相互運用テストやブラウザースイート全体の 10 回反復が必要なわけではありません。

QML ウィンドウを追加・変更する場合は `python3 scripts/check-window-contracts.py --self-test`、`python3 scripts/check-operation-components.py`、影響する二言語・複数字号のレイアウト回帰を実行します。ビルド後の `python3 scripts/desktop.py test --skip-rust` は Qt コンポーネントと CTest の全チェックを実行します。代わりに Qt SDK パスとオフスクリーン環境を設定し、`ctest --test-dir build/desktop -R qt_task_components --output-on-failure` のような既存入口から関連ケースを実行することもできます。ラジオボタン／チェックボックスの変更では、`accessibleRadioActionsCommitSelection` や `accessibleCheckboxActionsCommitIntent` など、変更に適した QAccessibleActionInterface 回帰も実行します。

[通常 CI](../../../.github/workflows/docs.yml) は軽量な静的チェックのみを実行し、ローカルの製品ビルドを代替しません。[Foundation](../../../.github/workflows/foundation.yml) と [Qt ワークフロー](../../../.github/workflows/qt-components.yml) は必要に応じて手動実行します。Qt は同じ desktop.py を使ってビルド、Qt テスト、Windows／Linux のパッケージ化と実行時検証を行います。[ci-desktop.ps1](../../../scripts/ci-desktop.ps1) は必要時の反復、視覚、相互運用、インストール互換性チェックを保持し、通常のローカル検証には必須ではありません。

過去の製品全体、実際のインストールのアップグレード／ロールバック、関連付けのライフサイクルは、使い捨て Windows／Linux CI ホストまたは分離コンテナーでのみ実行します。共通入口はこれらを通常のローカル verify の既定動作にはしません。必要時の Qt ワークフローの macOS ジョブは SDK モード配置監査と Cocoa 起動／ファイルオープン検査を行い、ログと JSON のみを保持します。全チェックと証拠の境界は [CI 方針](../../../developers/engineering/ci-policy.md)を参照してください。

## ファイルとタスクの境界

開く操作は閲覧のみで、自動的に解凍しません。同名の通常ファイルには確認、スキップ、名前変更、明示的な置換を利用できます。ファイル全体は内容の検証と同期が終わってから公開します。コミット前のキャンセルまたは書き込み失敗では旧ファイルを保持し、すでにコミット済みの解凍項目は残ります。タスクウィンドウを閉じても詳細を隠すだけです。終了時はアクティブタスクの判断と非同期シャットダウンバリアを使用します。

ZIP は制限された POSIX 条件下で相対シンボリックリンクと実行ビットを復元できますが、特権ビット、拡張属性、元の更新日時は復元しません。これはアプリバンドルのインストールや署名検証を意味しません。ソースと出力先は[安全な解凍契約](zip-safe-extraction.md)を満たす必要があります。クラッシュ時の一時配置は報告して調査用に保持するだけで、未知の内容を自動復旧または削除しません。

## Microsoft Store MSIX

Store 提出は[メンテナー手順](../../../developers/market/windows/build.md)に従ってください。

<a id="windows-msix-实验构建"></a>

## 実験的な Windows MSIX ビルド

MSIX は同一コミットの Windows Release デスクトップビルド、テスト、監査済みランタイムディレクトリーを再利用します。上記の `prepare --packaging`、`build`、`test` を完了し、同じオプションで次を実行します。

```powershell
python scripts/desktop.py package --kind runtime
python scripts/desktop.py package --kind msix --msix-name xdzip.Development --msix-publisher 'CN=xdzip development' --msix-publisher-display-name xdzip
python scripts/desktop.py verify --kind msix
```

例の識別情報はローカル開発専用です。正式な Store 識別情報のパッケージ名、発行者、発行者表示名は Partner Center と一致する必要があります。出力 `build/package/xdzip-windows-x64.msix` は署名のない実験パッケージであり、そのまま配布可能なサイドロードパッケージや Store 認定済みとはなりません。通常のサイドロードには発行者と一致しデバイスが信頼する署名が必要です。Windows 11 の制限付き未署名開発モードではさらに指定された発行者 OID が必要で、実行可能コンテンツの場合は通常管理者権限も必要です。この例の識別情報はそのモードに対応しません。スクリプトは証明書の作成、署名、インストール、Store 提出を行いません。技術上の最低要件は Windows 10 22H2 x64 であり、サポート対象プラットフォームの約束を広げるものではありません。バージョン `1.0.<product build>.0` は表示バージョンとは別であり、ビルド番号を使い切る前に Store バージョン方針を見直す必要があります。

ビルドにはクリーンなソース、一致するビルド／テスト回執、完全に監査されたランタイムディレクトリー、Windows SDK の MakeAppx と MakePRI が必要です。スクリプトはランタイムとその更新、関連付け、メニューの機能マーカーを再確認し、別の一時領域でマニフェストとアイコンリソースを生成し、パッケージ内の各ファイルを比較してハッシュを記録します。既存のパッケージや一時領域は上書きしません。マニフェストはフォーマット登録表の既存拡張子、完全信頼デスクトップ入口、チャンネル別メニュークラスを宣言します。

アプリアイコンは倍率バリエーションと正確なターゲットサイズに加え、既定、暗い背景用の台座なし、明るい背景用の台座なしの形式を含みます。MakePRI はこれらを `resources.pri` に索引化するため、Windows はシステムの台座を加えず目的のサイズを選べます。ファイル関連付けは形式固有のロゴを宣言し、既存 Windows ICO の 16–256 ピクセル PNG フレームを、画像、可視サイズ、余白、位置を変えず再利用します。リソース方針は [msix-icons.json](../../../specs/visual/msix-icons.json)を参照してください。これらはインストールパッケージのリソースであり、Partner Center のストア掲載ロゴとは別です。`verify --kind msix` は寸法、すべてのバリエーション、正規の形式画像、リソース索引の存在を含め、新しいアイコン回執を実際のパッケージと照合します。変更は更新パッケージをビルドして配布した後にインストール済みユーザーへ届きます。

例では既定の `--msix-channel sideload` を使用し、Windows 11 のクラシックメニューのみを登録します。Store にアップロードする場合は、Partner Center の正確な識別情報を使い `package --kind msix --msix-channel store` を実行します。これによりモダンとクラシックの両メニューが宣言されますが、モダン項目はパッケージに実際の Store システム署名が付くまで非表示です。チャンネルマーカーはパッケージ時に追加され、監査済みランタイムの再ビルドは不要です。出力は別々のディレクトリーに保管します。[メニューチャンネル方針](shell-actions.md#windows-パッケージの境界)を参照してください。ライセンス、実際のインストール、関連付け、アンインストール、Store 署名、対象システムでの受け入れは、パッケージ化の成功とは別に検証します。

MSIX マニフェストは共有言語レジストリーの 10 言語を宣言し、内蔵カタログと対応します。回執には言語一覧を記録して検証します。この MakeAppx フローは Qt の翻訳リソースをパッケージ言語宣言へ自動変換しません。Partner Center の「パッケージでサポートされる言語」はアップロード済みパッケージを反映します。ソースを修正した場合は再ビルドしてドラフトのパッケージを差し替えて初めて更新されます。「追加の Store 掲載言語」は別に設定され、マニフェスト宣言の代わりにはなりません。[Microsoft のマニフェスト言語説明](https://learn.microsoft.com/windows/apps/design/globalizing/manage-language-and-region)を参照してください。

## 証拠と再検証

[共通検証](../../../developers/qa/README.md)と[必要時のデスクトップ検証](../../../developers/qa/private-test-manual.md)に従い影響するチェックを選択します。通常作業に手動受け入れフォームは必要ありません。ビルドとテストの回執は build-dir 内の `desktop-build.json` と `desktop-test.json` です。準備、パッケージ化、検証の回執は output-dir 内の `<stage>.json` で、基礎となる証拠にリンクします。自動化回執は実際の段階とソースを記録します。オフスクリーン／ソフトウェア描画、ネイティブ Cocoa チェック、実行した手動チェックは別々に記録し、互いの代わりにはなりません。既存の受け入れ結果は [#5](https://github.com/xiaodaozhi/xdzip/issues/5) にあり、候補一覧の再生成だけを理由に繰り返しません。

Windows／Linux の成果物には同一コミットのソースとライセンス通知が含まれ、Actions は成果物を 14 日間保持します。期限後は識別情報と固定入力を使って再ビルドします。deployment-audit.json とソース／ファイル単位の CycloneDX 一覧は来歴検証を支援しますが、不完全な場合があり、ライセンス承認を証明するものではありません。ソース再構築と Qt 置換については[ソース資料](source-delivery.md)と[自動検証契約](../../../developers/engineering/candidate-verification.md)を参照してください。

ビルド、スクリーンショット、測定値、レポート、パッケージは、無視対象のローカルまたは CI 成果物としてのみ保持します。生成された未完了フォーム、オフスクリーンチェック、部分的な SBOM を、クリーン環境と完全なリリースゲートの合格として説明してはいけません。

## macOS 更新フレームワーク

ネイティブビルドは `prepare-sparkle.py` で `build/toolchain/Sparkle/2.10.0-xdzip-proxy.2` を準備します。フレームワークのメインライブラリーは固定上流ソースとプロキシパッチからコンパイルし、インストールヘルパー、リソース、署名ツールは公式 SDK の来歴を維持します。[依存関係契約](../engineering/dependencies.md#macos-更新提供者)を参照してください。有効なビルド回執は再利用できますが、古い公式 SDK でプロキシインターフェースを持つ派生フレームワークを置き換えることはできません。CI は一時 SDK ディレクトリーを明示的に選択できます。

`python3 scripts/prepare-sparkle.py --output build/toolchain/Sparkle/2.10.0-xdzip-proxy.2` は既存の Xcode Command Line Tools または Xcode のネイティブ Clang を使用し、別の Qt SDK を必要としません。元の SDK とソースアーカイブは `build/source-cache/` にキャッシュし、オフライン動作では `--archive` と `--source-archive` を別々に指定できます。ビルドはダウンロードしたソースのスクリプトを実行せず、システムプロキシを変更せず、本番署名鍵にアクセスせず、プロジェクトのソース資料アーカイブを収集しません。

`check-macos-proxy.py` は既定でプロジェクト所有のループバックダウンローダーチェックのみを行います。`--system-conflicts` は専用ワークフローの使い捨てホスト Apple Silicon runner に限定されます。制限付きヘルパーがシステムプロキシと固定テストドメインの hosts 設定を一時配置、復元、検証します。このフラグをローカルで追加しないでください。
