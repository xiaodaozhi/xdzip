<a name="xdzip-built-in-format-declarations"></a>

# 組み込み形式の宣言

[English](../../engineering/format-registry.md) | [简体中文](../../zh-CN/engineering/format-registry.md) | [繁體中文](../../zh-TW/engineering/format-registry.md) | 日本語 | [한국어](../../ko-KR/engineering/format-registry.md) | [Français](../../fr-FR/engineering/format-registry.md) | [Deutsch](../../de-DE/engineering/format-registry.md) | [Español](../../es-ES/engineering/format-registry.md) | [Português (Brasil)](../../pt-BR/engineering/format-registry.md) | [Русский](../../ru-RU/engineering/format-registry.md)

組み込み形式の宣言元は [archive-formats.json](../../../specs/engineering/archive-formats.json) です。形式の安定 ID、表示名、コンテナーの種類、外側の圧縮方式、既存の専用ワーカー ID、対応する方式の系統、作成用の拡張子、検証範囲、利用できる入口の候補を定義します。方式の系統は個別の形式仕様を要約したものです。利用できるパラメーター、旧来の派生形式、組み合わせは、実際のレコード、ホストの方針、プラットフォームの機能によって操作前に絞り込まれます。

検証範囲は[整合性検証](archive-checking.md)に成功したときに何を確認できたかを示します。`supported_content` は対応する項目の内容を検証します。`structure_and_readability` は構造と読み取り可否を確認しますが、内容のチェックサムは保証しません。`format_declared_checks` はストリーム内で宣言された検証を実行します。検証値を持たないプロファイルもこの分類に含まれます。検証操作の可否は検証範囲とは別です。たとえば NSIS プロファイルは、上限内でインストーラーのメタデータを認識できますが、検証操作には対応していません。作成方式 ID は、形式ごとに許可された書き込み方式の範囲で解決します。

`creation_methods` はホスト内の転送用 ID と方式名を対応付け、`creation_levels` は形式ごとに使用できるレベルを定めます。生成される GUI の選択肢と Rust の記述子は両方を保持します。CLI、GUI、ホストはいずれも同じ宣言を使い、Deflate と LZMA2 以外の方式にも対応します。Store と Copy は ID 0、Deflate は 8、LZMA2 は 32 を引き続き使います。Zstandard のホスト ID は 93 です。LZ4 の ID 256 は対象コンテナー内に限られ、ZIP 方式とは見なしません。作成に対応しない形式には作成レベルがありません。書き換えへの対応は作成とは別に決まります。ジェネレーターは形式内の ID の欠落や重複、無効なレベルを拒否します。

| 操作 | 組み込みの対応範囲 |
| --- | --- |
| 新しいアーカイブを作成 | ZIP、7z、TAR、TAR.GZ（`.tar.gz`、`.tgz`）、単独の GZIP、XZ/TAR.XZ、LZ4/TAR.LZ4、Zstandard/TAR.ZST、LHA（`.lha`、`.lzh`）、ISO9660/Joliet、Windows x64 用 ZIP SFX ラッパー。単独ストリームは通常ファイルを一つ受け取ります。[作成仕様](zip-creation.md)と[単一ファイルストリーム](single-file-streams.md)を参照してください。 |
| 既存のアーカイブを書き換え | 条件を満たす ZIP/ZIPX、7z、レベル 2 の LHA/LZH、TAR、TAR.GZ、TAR.XZ、TAR.ZST、TAR.LZ4、TAR.BZ2、TAR.LZMA、TAR.LZ、TAR.Z、TAR.BR が対象です。暗号化、分割ボリューム、非対応のレコードや保存方式は対象外です。[変更仕様](../../engineering/archive-transfers.md)を参照してください。 |
| 閲覧、展開、項目を開く、書き出し、検証 | 操作ごとに機能とレコードの条件を確認します。たとえば、一部のインストーラーで確認できるのはメタデータだけです。[読み取り専用の閲覧](readonly-browser.md)と[検証](archive-checking.md)を参照してください。 |

仕様を変更したら `python3 scripts/generate-format-registry.py` を実行します。Rust のコーデック記述子、ドラッグ＆ドロップ用とシェル用の C++ 形式候補、JavaScript UI の選択肢、インストーラー用の Python アイコン対応表が生成されます。`--check` はファイルを変更せず、生成物の欠落や古い内容を検出します。`python3 scripts/test-format-registry.py` は、矛盾する機能、ID や拡張子の重複、出力言語で無効な値、更新されていない参照先を検査します。通常の CI で両方の検査を実行します。

開くときは先に[アーカイブを開くときの仕様](archive-opening.md)で入口とソースの規則を適用し、その後、許可されたバイトを検出して検証します。拡張子や MIME タイプは検出の入口を提供します。ホストはソースの識別情報、操作パラメーター、出力の許可、公開も確認します。`worker_id` は組み込みの読み取り経路を選びます。ISO イメージはワーカーで読み取り、新規イメージは Rust ホストで作成します。レジストリーでは作成と既存アーカイブの変更を別々に扱います。

入口は必要に応じてドラッグ＆ドロップとシェルメニューを区別します。通常の `.gz` は両方の入口と単一ファイルからの作成に対応します。`.tar.gz` と `.tgz` は TAR コンテナーの意味を保ちます。XPI/APK/IPA/CBZ は ZIP の検証と許可されたプロファイルを使います。[単一ファイルストリーム](single-file-streams.md)を参照してください。分割ボリュームの命名とソースの許可には[アーカイブを開くときの仕様](archive-opening.md)が適用されます。レジストリーで番号付きファイルを別の形式に解釈することはありません。Windows のアイコンは最後の拡張子に従うため、`.tar.gz` は `gz` の関連付けを使います。

実装済みで自動検証がある宣言だけを追加します。ソースを変更する場合は、関係する[独自形式の仕様](../../engineering/independent-rust-codecs.md)、[形式ワーカーの仕様](native-codecs.md)、機能説明、[受け入れ項目](../../../developers/qa/acceptance.md)を更新します。計画や進捗、実行結果は Issue に、生成した検証結果は `build/` または CI の成果物に保存します。

`creation_wrappers` は Windows x64 の ZIP SFX ラッパーと、その専用拡張子 `.sfx.exe` を宣言します。ZIP の方式 ID を再利用し、暗号化と分割ボリュームを無効にします。一般の EXE ファイルは対象外です。ISO9660 の作成にはホスト方式 ID 260 と固定レベル 0 を使用します。[ISO 作成](iso-creation.md)と[組み込みの SFX 作成](sfx-creation.md)を参照してください。
