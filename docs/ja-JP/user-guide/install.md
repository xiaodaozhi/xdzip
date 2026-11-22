# XDZip をインストールしてアーカイブを開く

[English](../../user-guide/install.md) | [简体中文](../../zh-CN/user-guide/install.md) | [繁體中文](../../zh-TW/user-guide/install.md) | 日本語 | [한국어](../../ko-KR/user-guide/install.md) | [Français](../../fr-FR/user-guide/install.md) | [Deutsch](../../de-DE/user-guide/install.md) | [Español](../../es-ES/user-guide/install.md) | [Português (Brasil)](../../pt-BR/user-guide/install.md) | [Русский](../../ru-RU/user-guide/install.md)

[GitHub の最新安定版](https://github.com/xiaodaozhi/xdzip/releases/latest)からシステムに合ったパッケージを入手してください。Windows では Microsoft Store も利用できます。GitHub のページにはファイル名、バージョン、直接ダウンロードしたファイルを検証するための `SHA256SUMS` があります。動作要件は[サポートポリシー](../engineering/support-policy.md)を参照してください。

| デスクトップ環境 | インストールまたは起動 |
| --- | --- |
| Windows 11 x64 | Microsoft Store で **XDZip** を検索するか、GitHub の `-setup.exe` を実行して現在のユーザーにインストールします。GitHub のインストーラーは署名されていません。ポータブル ZIP を使う場合はパッケージ全体を展開し、展開先から `xdzip.exe` を起動します。 |
| macOS 14 以降、Apple Silicon | DMG を開き、`xdzip.app` を **アプリケーション** にドラッグします。`.app.zip` の場合はまず展開し、アプリをアプリケーションフォルダーに移動します。アプリケーションフォルダーのコピーを起動してください。GitHub のビルドは整合性を保つためのアドホック署名付きで、公証されていません。macOS が警告を表示したら、配布元とチェックサムを確認してください。 |
| Ubuntu 24.04 LTS x64、X11、glibc 2.39 以降 | パッケージマネージャーで `.deb` をインストールします。ポータブル版の `.tar.gz` はすべて展開し、展開した実行環境のディレクトリから `./xdzip` を実行します。ランチャーは実行環境のファイルと同じ場所に置いてください。 |

デスクトップ版は Windows ARM64、Intel Mac、Linux ARM64 および Wayland に対応していません。Linux の独立した CLI は X11 なしで動作し、システムの OpenSSL 3 と証明書が必要です。`xdzip-cli-` から始まるファイルを選び、[CLI クイックスタート](../integration/README.md)に従ってください。

DEB パッケージでインストールした XDZip は、Ubuntu の App Center のインストール済み一覧からアンインストールできます。

<a name="xdzip-open-an-archive"></a>

## アーカイブを開く

XDZip の **アーカイブを開く** を選ぶか、初期画面にアーカイブをドロップします。インストール済みのパッケージが登録されていれば、ファイルマネージャーの **プログラムから開く** も使えます。特定のファイルの種類で XDZip を使うには、システム設定で既定のアプリに指定します。Microsoft Store 版では、ファイルの関連付けページから **Windows の既定のアプリを開く** を開けます。既定のアプリと、コンテキストメニューの圧縮操作は別々に設定します。[ファイルマネージャーからの展開](extract.md#xdzip-from-the-file-manager)と[インストールとファイルの関連付け](../engineering/installers-associations.md)を参照してください。

macOS であるファイルの種類を開けるアプリが XDZip だけの場合、関連付けをオフにしようとするとメッセージが表示され、既定のアプリは XDZip のままになります。

次に、[ファイルを閲覧してプレビューする](browse.md)方法を確認します。
