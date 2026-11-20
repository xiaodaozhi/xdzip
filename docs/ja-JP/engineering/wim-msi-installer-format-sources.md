<a name="xdzip-wim-msi-and-installer-format-sources-and-licenses"></a>

# WIM、MSI、インストーラー形式の資料とライセンス

[English](../../engineering/wim-msi-installer-format-sources.md) | [简体中文](../../zh-CN/engineering/wim-msi-installer-format-sources.md) | [繁體中文（台灣）](../../zh-TW/engineering/wim-msi-installer-format-sources.md) | 日本語 | [한국어](../../ko-KR/engineering/wim-msi-installer-format-sources.md) | [Français](../../fr-FR/engineering/wim-msi-installer-format-sources.md) | [Deutsch](../../de-DE/engineering/wim-msi-installer-format-sources.md) | [Español](../../es-ES/engineering/wim-msi-installer-format-sources.md) | [Português (Brasil)](../../pt-BR/engineering/wim-msi-installer-format-sources.md) | [Русский](../../ru-RU/engineering/wim-msi-installer-format-sources.md)

このページでは、WIM、MSI、インストーラー形式の技術資料、ライセンス上の制限、および対応に必要な条件を説明します。参照資料は `build/toolchain/references` に置くか、公開仕様へのリンクとして扱います。いずれも製品の依存関係ではありません。

<a name="xdzip-wim"></a>

## WIM

- 参照実装：`ebiggers/wimlib`、コミット `cd5e231c348c255ae5088873b5a66ee0eb96fa07`。
- リポジトリはプロジェクトに GPLv3+ を提供し、ライブラリ部分には条件付きで LGPL-2.1+ も選択できます。LGPL を選ぶには、`libntfs-3g` などの第三者依存関係が GPL 条件を強制しないことが必要です。
- WIM は通常のアーカイブではなくイメージコンテナーです。制限付きリーダーは、リソーステーブル、メタデータストリーム、分割されたリソース、Xpress/LZX/LZMS 圧縮、整合性テーブル、ハードリンクのメタデータ、イメージの選択を処理しなければなりません。マウントやセキュリティ権限の復元は行いません。

参照実装は形式とテスト資料の調査に役立ちますが、現在の第三者ランタイムを導入しない方針の下では xdzip にリンクできません。WIM は**製品の判断により対象外**です。必要となるイメージ専用リーダー、展開増幅のモデル、整合性リソース、ハードリンクの意味、完全なテスト資料をプロジェクトでは引き受けません。この判断を明示的に見直さない限り、レジストリー、MIME、ランタイム、CLI の入口は追加できません。

<a name="xdzip-compound-file-and-msi"></a>

## Compound File と MSI

- Microsoft [MS-CFB] が主要な公開仕様です：https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/53989ce4-7b05-4f8d-829b-d08d6148375b 。このページは PDF/DOCX の改訂版と、記載技術の実装を認める知的財産権通知を公開しています。特許ライセンスには引き続き該当する Microsoft のプログラムが適用されます。
- CFB リーダーは MSI 固有の解釈より前に、セクターと FAT/DIFAT の割り当て、ディレクトリーの赤黒木の走査、ミニストリームの割り当て、ストリームサイズの上限、循環の検出を処理する必要があります。

独立した CFB メタデータリーダーは技術的に実現可能であり、現在は `xdzip-codec-cfb` の読み取り専用部分として実装されています。ストレージとストリームの行を公開する前に、512 バイトと 4096 バイトのセクター基準を含む CFB 3.0/4.0 ヘッダー、FAT/DIFAT チェーン、ディレクトリーツリー、ミニストリームの割り当て、ストリームサイズの上限、循環を検証します。生ストリームのコピーは、不変のメタデータスナップショットが一致した後にのみ可能です。MSI テーブル、埋め込み cabinet、カスタムアクション、変換は契約の対象外です。インストーラーの実行、カスタムアクションの解釈、パッケージのインストール、書き戻しはいずれも行わず、`.msi` 拡張子によって実行権限が付与されることもありません。テスト用データとランタイムの証拠はコーデックとランタイムのテストで自動化されています。

<a name="xdzip-nsis-and-other-sfx"></a>

## NSIS とその他の SFX

- NSIS 参照ソース：`kichik/nsis`、コミット `a96303eae3115221471bb127fa916a13d8bf9ecd`。
- `COPYRIGHT` には、大部分のソースと zlib コンプレッサーは zlib/libpng、bzip2 モジュールは bzip2、LZMA モジュールは Common Public License 1.0 と記載されています。
- NSIS ファイルはスクリプトと埋め込みデータを持つ実行可能なインストーラーです。パーサーは実行境界の外にとどまり、ヘッダーとデータの範囲を制限し、圧縮方式とインストーラーの動作を分離しなければなりません。
- 公開ソースは固定 `firstheader` レコードを定義しています。内容は `flags`、`0xDEADBEEF`、`NullsoftInstallSystem` の署名ワード、`length_of_header`、`length_of_all_following_data` です。静的リーダーは PE スタブを開かずにこのレコードを特定して検証できますが、圧縮ヘッダーブロックと完全な自前のインストーラーには別途独立したモデルが必要です。

NSIS は現在、**静的メタデータのみのプロファイルとして採用**されています。コーデックは制限された PE オーバーレイ内の 512 バイト境界にそろったレコードだけを走査し、記録済みの 2 種類の `firstheader` マジック値を受け入れ、フラグと長さを制限し、宣言された後続範囲が物理入力に収まることを要求します。その後、`NSIS/firstheader` メタデータを 1 行公開します。インストーラーヘッダーは展開せず、データのバイトも公開しません。`.exe` の登録は意図的にドラッグ＆ドロップの候補にもシェル関連付けにもせず、内容検出を優先します。パーサーは Qt とインストーラー実行 API から独立しています。PE スタブ、スクリプト、カスタムページ、プラグイン、抽出されたデータは決して実行しません。その他の SFX 形式は、内部アーカイブが内容検証に合格した場合にのみ別名として扱います。

<a name="xdzip-admission-gates"></a>

## 採用条件

1. 参照ツールを使用する前に、ソースのリビジョンとライセンス文のハッシュを固定します。
2. 採用するコンテナーと圧縮方式ごとに、再配布可能な完全なテストデータを作成します。ヘッダーだけのサンプルでは不十分です。
3. CFB/WIM/NSIS の解析は Qt およびインストール、マウント、レジストリー、スクリプト実行の API から独立させます。
4. 正確な物理 ID、重複項目、循環の上限、累積入力・処理・出力の制限、キャンセル、ソースの変更、最終的な失敗のテストに合格してから、ランタイム、CLI、デスクトップの入口を追加します。
5. GPL/LGPL/CPL の参照コンポーネントは、依存関係とライセンスについての別の決定が製品方針を明示的に変更しない限り、build/toolchain に残します。
