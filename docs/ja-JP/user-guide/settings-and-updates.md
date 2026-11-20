# 設定とアップデート

[English](../../user-guide/settings-and-updates.md) | [简体中文](../../zh-CN/user-guide/settings-and-updates.md) | [繁體中文](../../zh-TW/user-guide/settings-and-updates.md) | 日本語 | [한국어](../../ko-KR/user-guide/settings-and-updates.md) | [Français](../../fr-FR/user-guide/settings-and-updates.md) | [Deutsch](../../de-DE/user-guide/settings-and-updates.md) | [Español](../../es-ES/user-guide/settings-and-updates.md) | [Português (Brasil)](../../pt-BR/user-guide/settings-and-updates.md) | [Русский](../../ru-RU/user-guide/settings-and-updates.md)

<a name="xdzip-set-up-your-workspace"></a>

## 作業環境を設定する

アプリのメニューまたはツールバーから **設定** を開きます。表示言語、テーマ、文字サイズ、表示間隔をそれぞれの項目で設定します。自動検出ではシステムの言語が対応していればその言語を使います。アーカイブをタブとウィンドウのどちらで開くかを選びます。利用できる環境では、最近のファイルや通知領域のアイコンも設定できます。変更はすぐに適用されます。保存できなかった場合は、そのページのエラーを確認してください。[設定の動作](../ux/settings-localization.md)

ファイルの関連付けは OS の設定から変更します。コンテキストメニューの操作は別に設定します。macOS では Finder 拡張機能の有効化をシステムが管理します。登録済みであれば Finder サービスも使えます。[ファイルマネージャーの操作](../../engineering/shell-actions.md)

<a name="xdzip-update-the-app"></a>

## アプリをアップデートする

- **Microsoft Store 版。** Microsoft Store でアップデートします。アプリが MSIX パッケージとして動作している場合、情報ページに GitHub のアップデート操作は表示されません。
- **GitHub からインストールした場合。** **xdzip について** を開き、アップデートを確認します。アプリ内でインストールするには、対応する Windows のユーザー単位のインストール、macOS のアプリケーションフォルダーにある対応するコピー、または Ubuntu の DEB インストールが必要です。検証済みの配布パッケージも必要です。Windows はインストーラーの確認を求めることがあります。Ubuntu はシステムの認証を使用します。実行中のタスクによりインストールが待機中の場合は、タスクの完了を待つか、アップデートの待機をキャンセルしてください。
- **ポータブル版または開発ビルド。** 対応するパッケージをダウンロードします。Windows と Linux では新しい ZIP または TAR.GZ パッケージ全体を別のディレクトリに展開します。macOS では古いアプリを終了してからアプリケーションフォルダーで置き換えます。[インストール手順](install.md)を参照してください。

GitHub のアップデート確認は利用者が開始し、対応するプロキシ設定を使います。手動ダウンロードのみが表示された場合は、リリースへのリンクから取得してパッケージを検証します。ダウンロードが完了したらインストールが必要です。アップデート後に **xdzip について** を開き直し、実行中のバージョンを確認してください。Microsoft Store と GitHub は別々に公開されます。[アップデートの動作と復旧](../../engineering/software-update.md)
