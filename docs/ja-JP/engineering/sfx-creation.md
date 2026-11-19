# xdzip で作成する Windows 用 ZIP 自己展開アーカイブ

[English](../../engineering/sfx-creation.md) | [简体中文](../../zh-CN/engineering/sfx-creation.md) | [繁體中文（台灣）](../../zh-TW/engineering/sfx-creation.md) | 日本語 | [한국어](../../ko-KR/engineering/sfx-creation.md) | [Français](../../fr-FR/engineering/sfx-creation.md) | [Deutsch](../../de-DE/engineering/sfx-creation.md) | [Español](../../es-ES/engineering/sfx-creation.md) | [Português (Brasil)](../../pt-BR/engineering/sfx-creation.md) | [Русский](../../ru-RU/engineering/sfx-creation.md)

最初の SFX プロファイルは Windows x64 専用で、作成時に `.sfx.exe` を指定します。プロジェクトがビルドしたスタブ `xdzip-sfx.exe` に、暗号化されていない単一ボリュームの Store または Deflate ZIP を格納します。作成時にスタブを実行することはありません。一般の `.exe` ファイルは作成、ドラッグ＆ドロップおよびシステムの関連付けの対象外です。

アプリはスタブをアプリのディレクトリまたは専用の `cli` ディレクトリの親からのみ読み込みます。リンクをたどらず、サイズに上限のある通常ファイルを開き、PE32+、x64、セクション範囲、プロジェクトのバージョン識別子を検証します。ZIP 終端レコードのコメントには、スタブと ZIP の長さ、およびそれ以前のバイト列の SHA-256 を含む 64 バイトのバージョン付きエンベロープがあります。SFX は実行時にこれを検証してから、共通の Rust の ZIP と展開の処理を使います。破損や不正なエンベロープを検出できます。発行元の身元や任意の実行可能コードの真正性を保証するものではありません。

スタブは Qt や xdzip をインストールしなくても動作します。Windows のダイアログで既存の展開先ディレクトリと競合時の処理を選択し、進行状況を確認してキャンセルできます。アーカイブ処理は UI スレッド以外で Rust が行います。既存の権限、固定した識別情報、リソース上限、ファイル単位の検証、一時書き込み、競合処理と後片付けを再利用します。キャンセルすると保存が確定したファイルを保持し、未完了の一時ファイルを削除します。実行結果にかかわらず、展開したファイルを開いたり実行したり、スクリプトを起動したり、ソフトをインストールしたり、権限を昇格したり、関連付けを変更したりすることはありません。

自動化では、書き込まずに検証する `--test` と、展開する `--extract-to <existing-directory> --conflict skip|rename|overwrite` を使用できます。エンベロープが不正な場合や ZIP プロファイルが未対応または暗号化されている場合は、出力前に拒否します。ペイロードのパスワード、分割ボリューム、任意の起動コマンド、サイレントインストール、展開後の自動実行には対応していません。生成されたファイルは署名されません。署名と発行元の信頼性は、公開時に個別に扱います。[PE と Authenticode の規則](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format)

デスクトップアプリは Qt Quick のままです。スタンドアロン SFX のネイティブダイアログは [ADR-013](../design/architecture-decisions.md#xdzip-adr-013-standalone-windows-sfx-host) に定められた限定的な例外です。
