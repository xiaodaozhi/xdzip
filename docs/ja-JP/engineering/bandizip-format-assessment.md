<a name="xdzip-bandizip-format-inventory-and-xdzip-feasibility"></a>

# Bandizip の形式一覧と xdzip の実現可能性

[English](../../engineering/bandizip-format-assessment.md) | [简体中文](../../zh-CN/engineering/bandizip-format-assessment.md) | [繁體中文](../../zh-TW/engineering/bandizip-format-assessment.md) | 日本語 | [한국어](../../ko-KR/engineering/bandizip-format-assessment.md) | [Français](../../fr-FR/engineering/bandizip-format-assessment.md) | [Deutsch](../../de-DE/engineering/bandizip-format-assessment.md) | [Español](../../es-ES/engineering/bandizip-format-assessment.md) | [Português (Brasil)](../../pt-BR/engineering/bandizip-format-assessment.md) | [Русский](../../ru-RU/engineering/bandizip-format-assessment.md)

この評価は Bandizip が列挙する形式と xdzip の実装範囲を比較し、ソース、テストデータ、自動検証が必要な機能を示します。一覧は Bandizip の製品ページと形式ヘルプに基づき、2026-11-15 に確認しました。

- [Bandizip 製品ページ](https://www.bandisoft.com/bandizip/)には `7Z`、`ACE`、`AES`、`ALZ`、`ARJ`、`BH`、`BIN`、`BR`、`BZ`、`BZ2`、`CAB`、`Compound(MSI)`、`DAA(1.0)`、`DEB`、`EGG`、`GZ`、`IMG`、`ISO`、`ISZ`、`LHA`、`LZ`、`LZH`、`LZMA`、`PMA`、`RAR`、`RAR5`、`SFX(EXE)`、`TAR`、`TBZ/TBZ2`、`TGZ`、`TLZ`、`TXZ`、`UDF`、`WIM`、`XPI`、`XZ`、`Z`、`ZIP`、`ZIPX`、`ZPAQ`、`PEA`、`UU`、`UUE`、`XXE`、`ASAR`、`ZSTD`、`NSIS`、`LZ4` が記載されています。
- 同ページは `ZIP`、`7Z`、`ZIPX`、`EXE (SFX)`、`TAR`、`TGZ`、`LZH`、`ISO`、`GZ`、`XZ` を作成可能としています。長い一覧は主に展開対象です。
- [Bandizip の ALZ と EGG のヘルプ](https://www.bandisoft.com/bandizip/help/how-to-open-alz-and-egg-files/)は、ALZ の Deflate、難読化 Deflate、BZip2、ZipCrypto、および EGG の Deflate、BZip2、LZMA、AZO、AES/LEA 暗号化、ソリッドブロック、分割アーカイブを説明しています。
- [Bandizip の ZPAQ ヘルプ](https://www.bandisoft.com/bandizip/help/what-is-a-zpaq/)は ZPAQ を独自のジャーナル型アーカイブ形式と説明し、Bandizip が扱える圧縮方式を列挙しています。
- [ディスクイメージとその他のアーカイブ形式のソースガイド](disk-image-archive-format-sources.md)は DAA、ISZ、ZPAQ、PEA、BOND、J2J の公開ソース、ライセンス、コーパス、クリーンルームの確認を記録します。BOND と J2J は現在の公式 Bandizip アーカイブ項目として確認されておらず、過去の調査手がかりにとどまります。

<a name="xdzip-current-xdzip-disposition"></a>

## xdzip の現在の扱い

表の意味は次のとおりです。**受理済み**は現在のレジストリ、runtime、CLI、デスクトップ経路に記載プロファイルの自動証拠があることを示します。**別名/プロファイル**は内容検証後に既存コンテナーを通じて拡張子を受理することを示します。**候補**は将来の入口として有用ですが、デコーダーは主張しません。**保留**は公開され独立にテスト可能な形式仕様と互換実装または自前アルゴリズムが利用可能になるまで、責任を持って受理できないことを示します。**対象外**はプロジェクトが非対応を明示的に決定した形式であり、その製品判断を見直さない限りレジストリ項目や検出器を追加しません。

| Bandizip 名 | xdzip の状態 | 現在説明可能な範囲または理由 |
| --- | --- | --- |
| 7Z | 受理済み | 独立した Rust 7z バックエンド。宣言されたパスワード、ソリッド、分割プロファイルを含む。 |
| ACE | 対象外 | 製品判断で ACE 対応を永久に断念。デコーダー、検出器、拡張子、MIME、関連付けは追加しない。 |
| AES | 受理済み | AES Crypt v0–v3 のパスワードと HMAC プロファイル。これは `.aes` エンベロープであり、任意の暗号化アーカイブではない。 |
| ALZ | 受理済み | 通常の単一ボリューム Store と raw Deflate を読み取り専用で扱い、正確なサイズと CRC-32 を検証。改変 BZip2、ZipCrypto、分割は対象外。 |
| ARJ | 受理済み | Store と方式 1–4 を読み取り専用で扱い、完全なヘッダー、サイズ、メンバー CRC-32 を検証。 |
| BH | 受理済み | 通常の `BH 05 07` メンバー、方式 8 raw Deflate、正確なサイズと CRC-32 を読み取り専用で扱う。Store、暗号化、分割、他方式は対象外。 |
| BIN | 候補 | ディスク、実行ファイル、データで曖昧な拡張子。検証済みの内部形式のみを検出し、汎用 BIN をアーカイブデコーダーとして登録しない。 |
| BR | 受理済み | 制限付き Brotli ストリームと TAR.BR プロファイル。 |
| BZ / BZ2 | 受理済み | BZip2 単一ファイルストリームと宣言済み TAR.BZ2 別名。 |
| CAB | 受理済み | Store、MSZip、Quantum、LZX、許可された cabinet セットを扱う独立 Rust CAB バックエンド。 |
| Compound(MSI) | 受理済み | 内容検証済み `.msi` について読み取り専用 CFB 構造、ストレージ/ストリーム名、制限付き raw ストリームコピー。MSI テーブル、埋め込み CAB の解釈、変換、カスタムアクション、インストールは対象外。 |
| DAA(1.0) | 対象外 | 製品判断により永久に非対応。検出器、codec、レジストリ、リーダー、ライターは受理しない。 |
| DEB | 受理済み | 一般的な `ar` エンベロープ、control/data ビュー、受理済み圧縮メンバー。 |
| EGG | 受理済み | 非暗号化、非ソリッドの通常単一ボリューム Store と raw Deflate を正確なブロックサイズと CRC-32 で読み取り専用に扱う。AZO、BZip2、LZMA、暗号化、分割、他のプロファイルは対象外。 |
| GZ | 受理済み | 連結ストリームと TAR.GZ を含む GZIP メンバー。 |
| IMG | 候補 | 曖昧なディスクイメージ拡張子。内容による ISO/UDF の受理は汎用 IMG 対応を意味しない。 |
| ISO | 受理済み | ISO9660/Joliet/Rock Ridge の読み取りと通常の ISO9660 + Joliet データイメージ作成。 |
| ISZ | 対象外 | 製品判断で ISZ を断念。検出器、レジストリ、鍵処理は追加しない。 |
| LHA / LZH | 受理済み | LHA レベル 0–3、Store、LH1–LH7、LZS/LZ5 と LH5 作成。 |
| LZ | 受理済み | Lzip v1 と TAR.LZ。 |
| LZMA | 受理済み | LZMA Alone と TAR.LZMA。 |
| PMA | 受理済み | CP/M フレーミングと CRC 検査を伴う PM0/PM1/PM2 読み取り専用プロファイル。 |
| RAR / RAR5 | 受理済み | 宣言範囲内で旧形式、RAR5、RAR7 拡張、パスワード、ソリッド、分割を含む独立 Rust RAR バックエンド。 |
| SFX(EXE) | 別名/プロファイル | ZIP SFX プレフィックスを ZIP リーダーで検証。汎用的な実行ファイルの抽出や実行は一切許可しない。独自の Windows x64 Store/Deflate ZIP 作成は明示的な `.sfx.exe` プロファイルで提供し、自動実行しない。 |
| TAR | 受理済み | レジストリに記載された TAR とフィルター付き TAR.GZ/TAR.BZ2/TAR.LZMA/TAR.LZ/TAR.XZ/TAR.Z/TAR.BR/TAR.LZ4/TAR.ZST。 |
| TBZ / TBZ2 | 別名/プロファイル | BZip2 フィルター付き TAR 別名。 |
| TGZ | 別名/プロファイル | GZIP フィルター付き TAR 別名。 |
| TLZ | 別名/プロファイル | LZMA フィルター付き TAR 別名。 |
| TXZ | 別名/プロファイル | XZ フィルター付き TAR 別名。 |
| UDF | 受理済み | ハイブリッドイメージの名前空間規則を含む読み取り専用 UDF イメージ。 |
| WIM | 対象外 | 製品判断により、WIM に必要なイメージ専用リーダー、リソースモデル、コーパスを追求しない。 |
| XPI | 別名/プロファイル | XPI 拡張子の ZIP 内容。内容検証が基準。 |
| XZ | 受理済み | LZMA2、宣言された検査、受理済みフィルターを伴う XZ と TAR.XZ。 |
| Z | 受理済み | Unix compress LZW と TAR.Z/TAZ 別名。 |
| ZIP | 受理済み | ZIP/ZIP64、AES-256、対応する分割、方式、書き換えプロファイル。 |
| ZIPX | 別名/プロファイル | XZ などの検証済み ZIP 方式に対して受理し、独立したコンテナー実装ではない。ZIPX 作成は現在提供していない。 |
| ZPAQ | 対象外 | このソリッドなジャーナル型形式に必要な ZPAQL 仮想マシンを実装しないという製品判断。 |
| PEA | 受理済み | PEA 1.x 単一ボリューム、UTF-8、非暗号化 PCOMPRESS0 格納ファイル/ディレクトリの読み取り専用プロファイル。検査、暗号化、圧縮、分割、作成、変更は対象外。 |
| UU / UUE | 受理済み | 物理メンバーと行を検証する制限付き UU/UUE エンベロープ。 |
| XXE | 受理済み | 制限付き XXE エンベローププロファイル。 |
| ASAR | 受理済み | 順序付きメタデータ、格納範囲、宣言された SHA-256 検査。 |
| ZSTD | 受理済み | 標準 Zstandard フレームと TAR.ZST/TZST プロファイル。 |
| NSIS | 受理済み | 静的 `firstheader` メタデータのみ。内容検出で制限付き PE overlay を検証し、`.exe` はドラッグ/ドロップやシェル関連付けの入口ではなく、PE、スクリプト、プラグイン、ペイロードの実行は許可しない。 |
| LZ4 | 受理済み | 標準 LZ4 フレームと TAR.LZ4。 |

したがって一覧は `BIN`、`IMG`、`SFX(EXE)` の汎用ハンドラー追加を正当化しません。これらの名前は実行ファイル、ディスク、インストーラーのデータを表すことがあり、拡張子だけではアーカイブ機能を確定できません。NSIS は内容を検証する、より限定的な静的メタデータリーダーを持ち、実行やペイロードへの権限を与えず、`.exe` 入口はドラッグ/ドロップとシェル関連付けから隠されます。Compound(MSI) は内容検証済み CFB リーダーを持ちますが、`.msi` 拡張子も実行権限を与えません。

DAA、ISZ、ZPAQ、PEA、BOND、J2J は[ディスクイメージとアーカイブ形式のソース](disk-image-archive-format-sources.md)を参照してください。PEA は文書化された保守的な読み取り専用プロファイルで利用できます。ACE、DAA、ISZ、ZPAQ は製品判断で対象外です。BOND と J2J は Bandizip のアーカイブ形式であることが確認されていないため対象外です。

<a name="xdzip-delivery-plan"></a>

## 配布計画

1. **一覧とレジストリの条件。** 受理済みプロファイルごとに `specs/engineering/archive-formats.json` に機械可読の項目を一つ維持します。Rust、C++、JavaScript、関連付けの利用側を生成し、両方のレジストリ検査を実行します。新形式ごとに形式契約と対応する受け入れ項目を追加します。
2. **コードより先に形式契約。** 公開ソースのリビジョンと SHA-256 を固定し、ヘッダーフィールド、名前のエンコーディング、物理的な終端、方式、チェックサム、重複の識別、リソース上限を文書化します。参照ツールは独自の完全なサンプルを生成した場合のみ証拠となり、クラッシュや単一の Store サンプルは制限として記録します。
3. **raw アルゴリズムのテスト。** Qt と独立した制限付きリーダーを実装します。空と境界のペイロード、受理済みの全方式、入力/出力/処理/メモリーの正確な予算、キャンセル、型付き I/O、ソース変更、切り詰め、末尾バイト、CRC、エラー後の終端状態を対象とします。不明な方式はペイロード権限なしで表示を維持します。
4. **コンテナーと製品の受理。** codec crate、runtime の弱い/強い検出、CLI list/test/extract、デスクトップのラベル、関連付けのテストデータを追加します。固定されたメタデータと物理的な重複の選択を検証し、関連する自動 Qt とレイアウトの検査を実行します。この調査・形式部分には手動の視覚受け入れは必要ありません。
5. **優先順位。** ALZ の保守的な Store/raw Deflate、EGG の非暗号化・非ソリッド Store/raw Deflate、CFB/Compound(MSI) メタデータ、PEA の保守的な格納プロファイル、NSIS の静的 `firstheader` メタデータは受理済みです。ACE、DAA、WIM、ISZ、ZPAQ、BOND、J2J は製品判断で永久に対象外です。ZIPX 作成は現在のライターの対象外です。[自前の SFX ライター](sfx-creation.md)は別途実装されており、読み取り別名の対応から自動的に得られるものではありません。
6. **リリース境界。** codec、runtime、CLI、デスクトップの証拠が揃うまで候補形式を公開機能表に入れません。未対応方式と参照ツールの制限は issue に記録し、黙って Store や別のコンテナーにフォールバックしません。

現在の BH 部分はこの計画に従います。読み取り専用で、パスワードと分割の経路はなく、独立した制限付き Deflate リーダー、必須 CRC-32、正確な物理終端を使用します。Compound(MSI) も同じ範囲に従い、CFB FAT/DIFAT、ディレクトリツリー、mini-stream、循環の検証は Qt と独立し、不変のメタデータスナップショットが一致した後にのみ raw ストリームをコピーします。NSIS は固定された firstheader メタデータレコードまでで、ペイロードへのアクセスは未対応を返します。MSI や NSIS のインストーラー動作は実行しません。
