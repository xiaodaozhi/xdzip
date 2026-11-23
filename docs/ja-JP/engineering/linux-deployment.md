<a name="xdzip-linux-standalone-runtime-directory-and-verification"></a>

# Linux 単独実行ディレクトリと検証

[English](../../engineering/linux-deployment.md) | [简体中文](../../zh-CN/engineering/linux-deployment.md) | [繁體中文](../../zh-TW/engineering/linux-deployment.md) | 日本語 | [한국어](../../ko-KR/engineering/linux-deployment.md) | [Français](../../fr-FR/engineering/linux-deployment.md) | [Deutsch](../../de-DE/engineering/linux-deployment.md) | [Español](../../es-ES/engineering/linux-deployment.md) | [Português (Brasil)](../../pt-BR/engineering/linux-deployment.md) | [Русский](../../ru-RU/engineering/linux-deployment.md)

現在の開発用デプロイの基準は Ubuntu 24.04、x86_64、X11 です。目的は開発用 Qt SDK なしで現在の製品を実行することであり、他のディストリビューション、古い glibc、Wayland、完全なインストーラー対応を約束するものではありません。macOS CI はネイティブのビルド、テスト、起動検証を維持します。ローカル開発用 DMG は[Mac デプロイガイド](macos-deployment.md)に従い、このフローの影響を受けません。

<a name="xdzip-layout-and-generation"></a>

## 構成と生成

`scripts/deploy_linux.py` は信頼できるプロジェクト所有の ELF ファイルと明示指定された Qt 6.8.2 SDK のみを受け入れ、新しい build/ または dist/ ディレクトリを作成します。

- `xdzip` はランチャーで、開発用 SDK の検索パスを引き継がず、ローカルのライブラリー、プラグイン、QML を使用します。
- `bin/xdzip` は製品本体で、隣接する `qt.conf` は相対 Qt パスを使用します。
- `lib/` は実際にインポートされた Qt/SDK 共有ライブラリーを含み、`qml/` は Basic Controls スタイルと必要な実装サブモジュールを保持します。他の Controls スタイル、NativeStyle、Effects、スタイル固有のダイアログ変種、デザイナーファイル、QML 型メタデータはネイティブ依存関係の収集前に削除し、それらだけが必要とするライブラリーをデプロイしません。
- `plugins/` は xcb、オフスクリーン検証、画像、アイコン、プラットフォーム入力、xcb グラフィック統合、必須の `tls/libqopensslbackend.so` を含みます。OpenSSL バックエンドがなければデプロイは失敗します。EGLFS、フレームバッファー、Wayland のプラットフォームプラグインは検証済み配布として扱いません。
- `translations/` は Qt 標準コントロールの翻訳を保持し、`share/notices/` は現在の通知とソース依存関係の資料を提供します。
- `share/deployment.json` は QML、システム実行時依存関係、コピーした SDK ライブラリー、範囲を記録し、README.md に使用方法を記載します。

スキャナーは固定された Qt の [qmlimportscanner](https://doc.qt.io/qt-6.8/qtqml-tooling-qmlimportscanner.html) を使用します。Xdzip.Settings と Xdzip.Native は組み込み登録モジュールで、開発用静的ライブラリーはコピーしません。通常の依存関係の欠落、SDK 外に出る QML パス、同名ライブラリーの衝突、不明な外部ライブラリーは失敗します。ネイティブインポートの再帰収集は SDK ライブラリーだけをコピーし、ホストの glibc やグラフィックドライバーはコピーしません。システム依存関係は実際の Debian パッケージの所有関係に従って列挙します。

削除前にコピーされた QML ツリー、必要な Basic プラグイン、保持する強いインポート、既知のスタイル構成を検証します。予期しないリンク、不明なスタイルやダイアログセレクター、省略されたモジュールへの強い依存関係は失敗します。既定のダイアログ、入力方式、アクセシビリティ、画像プラグイン、TLS、システム翻訳は既存の規則を維持します。`share/deployment.json` は削除された QML ファイルとバイト数を記録します。非公開の `cli/` は固定されたコンテキストパスを通じて `bin/xdzip-native-codec` を共有し、独立した CLI パッケージは自己完結したままです。

HTTPS には Ubuntu の `libssl3t64` と `ca-certificates` も必要です。`libssl.so.3` と `libcrypto.so.3` がシステムパッケージに属すること、およびシステム CA 信頼ストアが空でないことを確認します。Qt OpenSSL バックエンドはこれらを実行時に読み込むことがあるため、ldd だけでは依存関係の完全性を証明できません。デプロイマニフェストはインストール済みバイナリパッケージのバージョン、ソースパッケージとそのバージョンを記録します。インストーラーは両依存関係を宣言し、実ファイル SBOM はシステムライブラリーをアプリに同梱せず、必要な OS パッケージとして記録します。

patchelf は各 ELF に相対 `$ORIGIN` RPATH を設定し、各プラグインから lib/ までの相対距離を計算します。Qt のパスリダイレクトは[公式 qt.conf 規則](https://doc.qt.io/qt-6.8/qt-conf.html)に従い、ディレクトリ全体を移動できます。ライブラリーは動的リンクされた交換可能な状態を保ち、互換 Qt の置換を制限しません。一般的な範囲は [Qt の Linux デプロイ](https://doc.qt.io/qt-6.8/linux-deployment.html)を参照してください。

相対 RPATH の書き換え後、デプロイは不要なネイティブシンボルテーブルを削除し、すべての DT_NEEDED と RPATH/RUNPATH エントリーが変わらないことを検証します。各 ELF のツールバージョン、前後のハッシュとバイト数を `symbol_stripping` に記録し、SDK 入力と元のビルドバイナリは変更しません。診断用コピーには下位のデプロイスクリプトで `--keep-symbols` を指定できます。最終的な SDK 帰属とアプリのハッシュは変換後のファイルを表します。これはパッケージサイズの削減であり、メモリーや処理速度の改善を測定したという意味ではありません。

通常の生成と検証には[統一ビルド入口](../engineering/developer-build.md)を段階的に使用します。先に同じ構成でビルド、テスト、パッケージ準備を完了してから実行します。

```sh
python3 scripts/desktop.py doctor --for-stage package
python3 scripts/desktop.py package --kind all
python3 scripts/desktop.py verify --kind all
```

既定のビルドディレクトリは build/desktop、配布出力は build/package、Qt SDK は `build/toolchain/Qt/6.8.2/gcc_64` です。他の場所を使う場合は各段階に共通オプションを渡します。`package --kind runtime` は実行ディレクトリと監査資料を生成し、続いて同じ output-dir で `package --kind installer` を実行して tar.gz と .deb を生成します。既定の `all` は両方を順に実行します。下位の [deploy_linux.py](../../../scripts/deploy_linux.py)、[package_linux.py](../../../scripts/package_linux.py)、[verify_linux_runtime.py](../../../scripts/verify_linux_runtime.py) がプラットフォームロジックを保持します。問題調査には各 `--help` を参照してください。

デプロイはネイティブ Linux x86_64 上でのみ実行し、既存ディレクトリを上書きしません。パッケージングの事前検査は qmlimportscanner、ldd、patchelf、dpkg-query、インストーラーに必要な dpkg-deb を確認します。これらのビルドツールは製品に含まれません。コンテナーの実行時検証には Docker とアクセス可能なデーモンも必要で、ビルドは固定された Ubuntu イメージと記録された OS パッケージに限定します。Docker がなくても通常のビルド・テストは妨げませんが、verify は明示的に失敗し、未実行の分離検査を合格として記録しません。通常の verify はホストに .deb をインストールせず、ファイル関連付けも変更しません。インストールのライフサイクルは使い捨て CI で検査します。

<a name="xdzip-experimental-strict-snap"></a>

## 実験的な strict Snap

strict confinement の `core24` Snap は別個の実験的な配布です。制限されたアプリはホストの XKB 設定を `$SNAP/usr/share/X11/xkb` に引き継ぎません。ランチャーが `XKB_CONFIG_ROOT` をこのパスに設定する場合、Snap は固定された Ubuntu ビルド入力から XKB ツリー全体を含める必要があります。これがないと、Qt Wayland プラグインは読み込まれてコンポジターに接続できても、キーボードコンテキストの作成時に `xkbcommon` のインクルードパスエラーで失敗します。`xkb-data` の著作権通知を同梱し、パッケージバージョン、決定的なディレクトリハッシュ、ファイル数、通知のハッシュを候補の記録に残します。完成した Snap を展開して検証し、オフスクリーンと Wayland のスモークテストを別々に実行します。これらは実際の strict Snap デスクトップセッションでのインストールと操作の検証に代わるものではありません。

<a name="xdzip-automated-evidence-without-the-sdk"></a>

## SDK なしの自動証拠

コンテナーはダイジェストで固定した Ubuntu 24.04 を使用し、実際の OS 実行時依存関係と Xvfb、フォント、ロケールのテストサポートをインストールしますが、Qt、Rust、コンパイラーはインストールせず、その不在を再確認します。開発用 SDK と完全なソースツリーはマウントせず、読み取り専用の製品ディレクトリ、別のプロジェクト所有の 100,000 件 ZIP、検証スクリプトだけをマウントします。アーカイブの実行時検証はネットワークと権限昇格を無効にし、一時設定ディレクトリのみを書き込み可能にします。

空白を含む新しいパスにアプリをマウントし、開発用検索変数を消去してからランチャーで確認します。

1. 実際の `--version` が動作すること。
2. X11 下で実際の QML ウィンドウが起動し、安全に閉じること。
3. ネイティブ製品プローブが初回フレーム、100,000 件の読み取り、アクティブ要求のキャンセル、終了を完了し、件数とキャンセル結果が契約を満たすこと。
4. ホストの開発用変数を消去後、SDK へのフォールバックなしでパッケージ内の全 ELF をデプロイ済みモードで検証すること。実ファイル SBOM は同じバージョンとコミットの識別情報を保持し、公式スキーマを通過すること。
5. 同じ SDK なしイメージの新しいプロセスでネットワークを有効にし、分離された `XDZIP_SETTINGS_PATH` で製品の `--update-probe` を実行して GitHub HTTPS に実際に要求すること。利用可能な OpenSSL バックエンドを報告して更新検査を完了し、`.https.json` と `.https.log` を別々に保持する必要があります。ネットワーク、証明書、TLS の失敗は検証を阻止します。

HTTPS プローブは OpenSSL バックエンド、システム CA ファイル、実際の GitHub 要求結果を別々に記録します。実要求の成功だけでは署名検証やネイティブ更新のインストールを証明しません。署名解析と SDK なしの更新ワークフローは[別の更新回帰ワークフロー](../../../.github/workflows/update-regression.yml)で検証し、これらの証拠は互いに代用できません。

プローブの各試行の生ログと構造化レポートを `.https.attempt-N.log` と `.https.attempt-N.json` として保持し、最終試行の `.https.log` も保持します。レポートの欠落や無効、タイムアウト、プロセス終了は失敗状態のままです。製品コントローラーが匿名 GitHub リリース一覧のレート制限を確認し、状態、要求段階、応答ヘッダー、診断情報がすべて揃った場合にのみ再試行します。サーバー期限まで待つか制限付きバックオフを使用し、macOS の受け入れプローブと同じく最大 3 回、合計待機 3,700 秒までとします。期限が残り予算を超えた場合、待機が中断された場合、TLS、ネットワーク、その他のエラーは直ちに失敗します。合格には実際の更新要求を完了する必要があります。各待機判断と実際の時間を `.https-waits.json` に記録します。

出力パスを再利用する前に、以前のレポートとログを同じ親ディレクトリ内のレポート名に対応する `.history` ディレクトリに移し、実行ごとに別サブディレクトリを設けます。新しい実行が途中で失敗しても古い成功レポートが現在の出力パスに残らず、混同を防ぎます。過去の成果物は個別に確認できます。

linux-runtime.json は実際のイベントと実行ファイルの SHA-256 を記録します。Xvfb/Qt Quick のソフトウェアレンダリングが証明するのは実行フローのみであり、物理 GPU、中国語候補ウィンドウ、実際のスクリーンリーダー、完全なデスクトップインストールは証明しません。

<a name="xdzip-download-delivery-and-remaining-gates"></a>

## ダウンロード配布と残りの条件

CI は xdzip-linux-x86_64.tar.gz を生成し、SHA-256 と実行ファイルのハッシュを記録します。tar.gz は実行権限を保持し、Actions の ZIP 展開でランチャーの実行権限が失われることを防ぎます。アーカイブのパス、UID/GID、タイムスタンプは個人ディレクトリを含めず正規化します。ダウンロード後は展開してディレクトリ内で `./xdzip` を実行し、bin/xdzip だけを移動しないでください。生の linux/ ディレクトリは再アップロードしません。

ここで生成する `build/package` 実行ディレクトリ、tar.gz、.deb は実験的な開発用配布物であり、自動インストールや既定の関連付けの変更は行いません。公式 Release の Ubuntu tar.gz と .deb は専用 Release ワークフローが生成し、[リリースプロトコル](release-protocol.md)に従って提供します。プラットフォーム範囲は[サポートポリシー](support-policy.md)に従い、他のディストリビューションやデスクトップ、Wayland などの拡張には別の issue が必要です。既存のデスクトップ受け入れは [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5) にあります。後続の変更では影響を受ける実際の IME・スクリーンリーダーとクリーンなインストール・アップグレード・アンインストールを再検証します。ディレクトリの再ビルドだけで既存の受け入れを繰り返す必要はありません。過去の `v0.1.0` のコンポーネントレビューは [#4](https://github.com/xiaodaozhi/xdzip/issues/4) と [#53](https://github.com/xiaodaozhi/xdzip/issues/53) にあり、ディレクトリの実行成功だけでは将来のリリース条件は自動的に完了しません。

Ubuntu .deb のインストール、アップグレード、ロールバック、アンインストール、実行時ロック、デスクトップ MIME の自動化は[インストールと関連付け](installers-associations.md)を参照してください。実行時コンテナーの合格は実際のデスクトップ検査やリリースライセンスレビューの代わりにはなりません。
