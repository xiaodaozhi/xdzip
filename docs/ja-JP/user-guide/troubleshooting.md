# XDZip のトラブルシューティング

[English](../../user-guide/troubleshooting.md) | [简体中文](../../zh-CN/user-guide/troubleshooting.md) | [繁體中文](../../zh-TW/user-guide/troubleshooting.md) | 日本語 | [한국어](../../ko-KR/user-guide/troubleshooting.md) | [Français](../../fr-FR/user-guide/troubleshooting.md) | [Deutsch](../../de-DE/user-guide/troubleshooting.md) | [Español](../../es-ES/user-guide/troubleshooting.md) | [Português (Brasil)](../../pt-BR/user-guide/troubleshooting.md) | [Русский](../../ru-RU/user-guide/troubleshooting.md)

| 現象 | 確認すること |
| --- | --- |
| アーカイブを開けない | [対応形式](../../../README.ja-JP.md#xdzip-format-support)、圧縮方式、暗号化方式、分割ボリュームの不足がないかを確認します。元の場所にアクセスできない場合は、アーカイブとすべてのパートを名前を変えずにアクセス可能なローカルフォルダーにコピーし、開き直します。 |
| ファイル名が正しく表示されない、または書き込めない | UTF-8 の名前でアーカイブを作り直せるか確認します。XDZip は一部の古い文字コードを検出しますが、短い名前や複数の文字コードが混在する場合に制限があります。安全に出力先を決められる項目だけを展開します。[ファイル名の扱い](../engineering/filename-encoding.md) |
| プロパティは表示されるがプレビューできない | 形式、サイズ、プラットフォーム、文字コードを確認します。プレビューの制限は展開とは別です。[プレビューの制限](../engineering/content-preview.md) |
| 保存先が拒否された、または出力が不完全 | 書き込み可能なプライベートフォルダーを選びます。名前の重複や未対応の項目がないか確認し、再試行の前に結果を見ます。[ファイルの展開](extract.md) |
| XDZip の処理中に元のファイルが置き換わった | 新しいファイルを開き、項目を選び直します。XDZip は書き込み前に元のファイルを再確認します。 |
| Explorer、Finder、またはファイルに操作が表示されない | インストール元と拡張機能の有効化を確認します。Windows 11 のクラシックメニューは **その他のオプションを確認** にあります。macOS はシステム設定で Finder 拡張機能を有効にするか、登録済みの Finder サービスを試します。XDZip でアーカイブを直接開くこともできます。[ファイルマネージャーからの展開](extract.md#xdzip-from-the-file-manager) |
| 大きな ZIP の検証または変更を使えない | ディスクにインデックスを置く場合は、必要な通常ファイルだけを選び、タスクの上限内で展開します。[大きなアーカイブの上限](../engineering/large-index.md) |
| アップデートまたはインストールに失敗する | バージョンと配布元を確認します。手動ダウンロードはチェックサムを確認し、対応するインストール手順に従います。[アップデート時の復旧](settings-and-updates.md) |

問題を報告するには、**xdzip について** のバージョンとビルド番号、OS とアーキテクチャ、形式、操作、再現手順、期待した結果と実際の結果を記載してください。可能なら自分で作った小さなサンプルを使い、共有前にパスワードと非公開のパスを削除してください。[問題を報告する](https://github.com/xiaodaozhi/xdzip/issues/new/choose)
