# CAB 形式の資料と実装根拠

[English](../../engineering/cab-format-sources.md) | [简体中文](../../zh-CN/engineering/cab-format-sources.md) | [繁體中文](../../zh-TW/engineering/cab-format-sources.md) | 日本語 | [Français](../../fr-FR/engineering/cab-format-sources.md) | [Deutsch](../../de-DE/engineering/cab-format-sources.md) | [Español](../../es-ES/engineering/cab-format-sources.md) | [Português (Brasil)](../../pt-BR/engineering/cab-format-sources.md)

この文書は CAB コンテナ、MSZIP、LZX、Quantum の資料、範囲、独立実装の制約を記録します。製品機能、リソース管理、形式別の置換規則はそれぞれ[分離形式ワーカー契約](native-codecs.md)、[自社 Rust 契約](../../engineering/independent-rust-codecs.md)、[分割ボリューム契約](archive-opening.md)に定義します。資料の記録は相互運用性の受け入れやリリース時のライセンス審査の代わりにはなりません。

## Microsoft 技術文書

Microsoft の形式文書を優先します。表には実際に読んだコピーを固定します。日付は文書の版を示し、URL が常に同一バイトを返すことを意味しません。ダウンロードしたコピーは SHA-256 で検証してください。タイトルの一致だけでは同じ版と確認できません。

| 文書と入口 | 固定した版 | 範囲と制限 |
| --- | --- | --- |
| [MS-CAB 公式アーカイブ入口](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxprotlp/5641c1b2-35ed-4542-9058-a07ccab5b0a1)、[PDF](https://officeprotocoldocs-f5hpbjgea6b8gneq.b02.azurefd.net/files/Archive_Exchange/%5BMS-CAB%5D.pdf) | フッター `v20110304`、2011-03-04 公開 | §2 CFHEADER、CFFOLDER、CFFILE、CFDATA、§3.1 チェックサム方式。Microsoft はアーカイブ済みと分類し、アーカイブ資料に依存しないよう明示的に警告しています。現在保守されている仕様と呼ばないでください。圧縮アルゴリズム内部のビットストリームは定義しません。 |
| [MS-MCI](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-mci/27f0a9bf-9567-4e40-ad66-6ae9ab9d2786)、[PDF](https://officeprotocoldocs-f5hpbjgea6b8gneq.b02.azurefd.net/files/MS-MCI/%5bMS-MCI%5d.pdf) | 2016-10-17、改訂 7.3 | §2 MSZIP 構造、CK マーカー、完全な Deflate ストリーム、ブロックサイズ。Deflate ビットストリームは [RFC 1951](https://www.rfc-editor.org/rfc/rfc1951.html) にも従います。 |
| [MS-PATCH](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-patch/cc78752a-b4af-4eee-88cb-01f4d8a4c2bf)、[PDF](https://officeprotocoldocs-f5hpbjgea6b8gneq.b02.azurefd.net/files/MS-PATCH/%5bMS-PATCH%5d.pdf) | 2025-08-19、改訂 14.1 | §2.1 ビットストリーム、ウィンドウと距離、§2.2 ヘッダーと E8、§2.3 ブロック、§2.4 Huffman 木。共通機構と差異の比較にのみ使用します。LZX Delta の説明であり、CAB LZX の代替仕様ではありません。 |

| 固定 PDF | SHA-256 |
| --- | --- |
| MS-CAB | `09f796a493547697ed7d4bc36222ac132bf3ca1bd6b1da76fbdc3b38b5307709` |
| MS-MCI | `cb097c434e1cfccb368f0d7e9084ab4d47a65809a2eea94bbf41f9722c1a0343` |
| MS-PATCH | `ea17466b97c9a9d4e2c815538387bf442fb2393ed29632c6f58b2c47ef1f29fe` |

Microsoft Open Specifications 文書には、実装に必要な文書ライセンスや特許通知を含む独自の著作権・知的財産条件があります。それによってこのプロジェクトの Apache-2.0 資料になるわけではなく、ダウンロードできることはすべての特許・再配布上の問題が解決したことを示しません。

## 1997 年の CAB SDK 文書

元のタイトルは Microsoft Cabinet File Format、Microsoft MSZIP Data Compression Format、Microsoft LZX Data Compression Format です。読んだコピーには Microsoft の 1997 年の著作権表示があり、第三者がホストしています。

| ミラーされた原本 | 読んだ範囲 | SHA-256 |
| --- | --- | --- |
| [CABFMT.DOC](https://am.net/lib/tools/Microsoft/cab-sdk/DOCS/CABFMT.DOC) | §2 コンテナ配置とボリューム間セマンティクス、§3 固定コンテナ例、§4.1 チェックサム定義 | `5a220658b76f8a66e9013e342b4eb236f057b597a8e6f3939c86f05efc51e83a` |
| [MSZIPFMT.DOC](https://am.net/lib/tools/Microsoft/cab-sdk/DOCS/MSZIPFMT.DOC) | 導入と実装詳細、特に独立した Deflate ブロックと保持される履歴 | `046d283fa3b697e46c35b6a329bb90c86cbc9f4dedbed24c3ae43bd9b8ccf239` |
| [LZXFMT.DOC](https://am.net/lib/tools/Microsoft/cab-sdk/DOCS/LZXFMT.DOC) | 概念、LZX 圧縮データ形式、木の符号化、距離・長さ、E8 前処理 | `68ba4df86e08bece3259574133f1c61606d143d85f749b388f5f1e8ea46f8bef` |

これらの `.DOC` ファイルは実際には RTF です。タイトル、著作権表示、CAB の位置付けから Microsoft 作成文書のコピーとして使用できますが、ホストは Microsoft ではなく、Microsoft が公開した CAB SDK アーカイブとバイト単位で比較していません。[Microsoft KB 310618（アーカイブ）](https://mskb.pkisolutions.com/kb/310618)は元の SDK の内容を列挙し、古いダウンロードが利用できないと記録しています。このアーカイブ入口はミラーファイルを認証しません。読んだ範囲は形式文書であり、SDK の FCI/FDI 実装やサンプルコードではありません。

公開 TCOMP パラメーターは Windows SDK 10.0.19041.0 の `um/fdi_fci_types.h` にある定数宣言と [TCOMPfromLZXWindow 文書](https://learn.microsoft.com/en-us/windows/win32/api/fdi_fci_types/nf-fdi_fci_types-tcompfromlzxwindow)でも確認しました。ヘッダーの SHA-256 は `27a1f6f7728f5db64d11e434d101819d646ca49270930606bb76c7989ebe0254` です。公開パラメーター範囲とビット定義のみを使用し、この根拠に基づいて FCI/FDI アルゴリズムを呼び出したり分析したりしていません。

LZXFMT の概要、表、本文の一部には矛盾があります。MS-PATCH は共通機構の特定に役立ちますが、その参照データ、拡張された長い一致、128 KiB～32 MiB のウィンドウ、追加ブロックヘッダーは CAB に直接適用できません。CAB LZX のウィンドウは 15～21 ビットであり、原文、参照動作、ケースを区別するベクトルに基づく独立した検証が必要です。

## Quantum 資料の範囲

取得した元のアルゴリズム提供元資料は [Cinematronics QUANTUM ユーザーマニュアルのミラー](https://carrero.es/wp-content/uploads/2025/07/QUANTUM.pdf)で、copyright Cinematronics 1993、1994、1995、all rights reserved です。PDF の SHA-256 は `a2d6ed51e47db407ea5849b42004750f2f36c82a380f263b983c5282838a8212` です。第 6 節は独立した `.Q` アーカイブコンテナを説明しますが、CAB Quantum に必要な算術ビットストリーム、確率モデル更新、フレーム終端は定義しません。`.Q` のストリーム全体の状態やファイル検査を CAB の規則に転用しないでください。

[Matthew Russotto の Quantum 研究ページ](http://www.russotto.net/quantumcomp.html)と過去のアーカイブからは検証可能な原文を得られませんでした。そのため内容を読んだ証拠として扱っていません。原文がアルゴリズムを完全に定義するかどうかの判断や、ライセンス条件の推定はできません。

追加で読んだのは [David Carrero の UnQuantum README](https://github.com/dcarrero/unquantum/blob/main/README.md) だけで、コピーの SHA-256 は `187af81f8231cfe4cc31aec0c76476b1e33179fe517a365c820fc49c1b8106e0` です。そのプロジェクトのデコーダーソースは読んでいません。これは `.Q` プロジェクトの非公式概要で、Russotto、libmspack、バイナリ研究を参照すると述べています。Russotto の原文ではなく、CAB ビットストリーム仕様を完全に代替できません。プロジェクトの MIT 表示は、すべての参照資料とアルゴリズムソースが MIT ライセンスであることを示しません。

これらの不足は固定した公式参照実装の動作分析で補います。非公式資料を一律に排除するものではなく、参照実装を Microsoft の形式仕様とも呼びません。

## 固定した参照ソースと分析範囲

参照は[公式 7-Zip 26.05](https://github.com/ip7z/7zip/releases/tag/26.05)、具体的には [7z2605-src.tar.xz](https://github.com/ip7z/7zip/releases/download/26.05/7z2605-src.tar.xz) で、アーカイブ SHA-256 は `0b6080b8c3e7a579161d2d495224c0e51c467576cb4c7048b0ab485ef44cc22e` です。ビルドソースの一覧は [native-codec-sources.json](../../../scripts/native-codec-sources.json) を参照してください。その一覧は公式ソース全体を説明し、以下の記録は CAB 参照分析に実際に使ったファイルだけを示します。

アーカイブの `DOC/License.txt` によれば、以下のデコーダーと共通ファイルには既定の LGPL-2.1-or-later が適用され、これらのファイルに別のライセンス通知は見つかっていません。アーカイブには他のファイル用の BSD 条件もあります。アーカイブ全体を例外のない単一ライセンスに単純化したり、これらのデコーダーファイルを誤って BSD と表記したりしないでください。

| アーカイブ内ファイル | 実際の分析範囲 | SHA-256 |
| --- | --- | --- |
| `CPP/7zip/Compress/QuantumDecoder.cpp` | 算術ビットストリーム、モデル、トークン、フレーム終端、辞書状態 | `1a22429d792493573f542782a45a7f702b6e636e5aee1d47bdb3d1cf75ef11e2` |
| `CPP/7zip/Compress/QuantumDecoder.h` | モデルのアルファベットとセレクター定数 | `7571ee0360a7abe8f297f4cd4e34df1863003f252c6fab66c61f8d1bd25a21b4` |
| `CPP/7zip/Compress/Lzx.h` | LZX ビットストリーム定数 | `779b32ecaecfc5b8c135e0051394001786d680fca4cfda57a480884342500e99` |
| `CPP/7zip/Compress/LzxDecoder.h` | フレーム間で保持する状態の境界 | `18141528bc8587ade4f7b1e330ee35dc8e31d6c6ab34d86a2dff59d362d0d6c9` |
| `CPP/7zip/Compress/LzxDecoder.cpp` | CAB モードの木、非圧縮ブロック、パディング、繰り返し距離、E8、履歴、終端。最適化コードやメモリ配置を実装設計にはしません | `e78bf50f27f21e04966785fafe7f94fddc533a9e6103f79b81c4e7d7613d1aab` |
| `CPP/7zip/Compress/HuffmanDecoder.h` | 完全な木・空の木の受け入れのみ。固定コピーのおよそ 109～164 行と 430～497 行 | `22f93e29b2408ce7e1c1bca5f5f807c1383a8d1634fe3ae6cfcab15f084109c0` |
| `CPP/7zip/Archive/Cab/CabHandler.cpp` | 圧縮方法、CFDATA フレーム、履歴継続のみ。固定コピーのおよそ 1020～1252 行 | `77b82387eea40eda02929b90d3006877c08765089d6b96c5c12b9cfd3f56b8f7` |
| `C/7zVersion.h` | 固定版の識別 | `f7dc58b3a3c01569e061eda9cfe633ad8983667eb0fa10ab94a6c364ddd7e699` |
| `DOC/License.txt` | 既定のファイルライセンス規則と例外 | `9ac2b4a97ab5d523965534d8b2d5868e511b39096d51fff458ab72c38b80fccc` |
| `DOC/copying.txt` | LGPL 文書の識別ダイジェスト。法的助言として解釈しません | `1e7e6bae5a5bde32f1ae5a7c37a082d1ab03cf89354f7f936ac40be9e39a6531` |

行番号はその固定コピーの読書範囲を限定するだけで、ファイルの識別はダイジェストで確立します。参照版を更新する際は差分を再記録してください。同じ行番号を使うだけでは動作が変わっていないことを示せません。

## 分析と独立実装の分離

初期準備では形式文書とプロジェクト所有インターフェースだけを読みました。その後、指定された参照分析者が上記のソースファイルを明示的に読み、欠落または矛盾するビットストリーム、数学的更新、リセット、終端、エラー条件をコード表現のない動作仕様に変換し、それらのソースを読んでいない LZX と Quantum の実装者に渡しました。

各アルゴリズムの Rust 実装は、参照ソースを読まずに実装者が状態、型、制御フロー、テストを含め独立して構成しました。分析者はどちらのアルゴリズムも Rust で実装せず、CAB コンテナを別に実装しました。これは CAB 作業の関係者全員が参照ソースを見ていないという意味ではなく、厳密なクリーンルームとは呼べません。動作仕様に参照コード、行ごとの書き換え、移植した構造、翻訳したコメントを含めてはいけません。仕様名、定数、数学的関係は出典を保持します。

参照の読解、独立したコーディング、ブラックボックス相互運用は異なる問いに答えます。自社エンコーダーとデコーダーが相互に成功しても独立した互換性は確立しません。参照ツールでの成功は、許容的な動作がすべて本プロジェクトの完全性契約に適合することを証明しません。参照ソース、文書、ツール、サンプルはそれぞれのライセンスを維持します。Rust に書き直しても法的リスクがなくなったことにはなりません。ライセンスとリリースの要件は[ライセンス契約](licensing.md)と[対応ソース契約](../../engineering/source-delivery.md)を参照してください。

## 保持すべき技術上の相違

以下の動作境界は形式文書と限定した参照分析の両方から制約されます。Rust の内部構造や最適化方法を規定するものではありません。

- CAB フォルダーは圧縮履歴の単位で、ファイルはその中の出力範囲です。ファイル境界、CFDATA 境界、ボリューム間の継続は自動的に履歴をリセットせず、新しい論理フォルダーだけがリセットします。`cbUncomp=0` のボリューム間圧縮断片は 1 つの論理フレームに連結し、各物理 CFDATA チェックサムは個別に検証します。このチェックサムはバイト・ワードの XOR で予約データを含み、CRC や連結後の単一チェックサムで代替できません。メタデータ、長さ、ボリューム関係、ペイロード境界は別々に検査します。
- 各 MSZIP フレームには CK と完全な生 Deflate ストリームが含まれます。木とビットストリームはフレームごとにリセットし、フォルダーごとに最大 32 KiB の履歴を維持します。フレームの上限は圧縮 32,780 バイト、出力 32,768 バイトです。プロジェクト共通の Deflate プリセット履歴インターフェースを再利用し、2 つ目の Deflate 実装を作らないでください。切り詰められたストリームのゼロ埋めや強制パディングは厳格な終端検査を代替できません。
- CAB LZX はリトルエンディアンの 16 ビットワードを使用し、各ワード内では最上位ビットを先に読みます。15～21 ビットウィンドウの位置スロット数はそれぞれ 30、32、34、36、38、42、50 です。古い概要の大きなウィンドウの数は完全な距離表に照らして修正する必要があります。木の長さデルタは modulo 17 で減算し、整列木は主木・長さ木の前に置きます。主木、前木、整列木は完全でなければなりません。長さ木は完全に空でもよいですが、その空の木を要求した場合は失敗する必要があります。
- LZX の非圧縮ブロックにも共通のブロック長があります。生データ前に 1～16 個のゼロビットを飛ばし、既にワード境界にある場合はワード全体 1 個を飛ばします。奇数長の生ブロックのゼロパディングバイトは次フレームの先頭で消費される場合があります。省略は次のブロックがないフォルダーの最終末尾でのみ許容します。通常の圧縮フレームに許されるのは 0～15 個のゼロパディングビットだけで、追加の完全ワードや任意の末尾バイトは許しません。LZX ブロックは複数の CFDATA フレームにまたがる場合があります。最後の `finish` は宣言された出力が未完了のブロックを拒否する必要があります。旧 CAB 統合の緩い終端検査欠落を残さないでください。
- LZX の LZ 履歴には E8 逆変換前のバイトを格納します。新しいフォルダーには参照可能な有効なゼロ履歴はありません。E8 は各フレームの最後の 10 バイトを除いて走査します。変換サイズは符号なし 32 ビットの全値として解釈し、位置をフォルダー全体で累積します。最後の 6 バイトという古い説明や正の符号付きサイズでのみ有効な比較は直接適用できません。
- Quantum はフレームごとに 16 ビット閉区間の算術符号状態をリセットします。9 個の確率モデルと実際の辞書履歴はフォルダー全体で保持します。セレクター値が 7 個だからといってモデルも 7 個とは限りません。シンボル頻度は毎回 8 増加し、総頻度が厳密に 3800 を超えた場合のみ次の使用前に再スケーリングします。モデルごとの累積頻度の半減、4 回目とその後 50 回ごとの並べ替え、同頻度の順序はビットストリームの動作であり、言語の既定ソートで任意に置き換えられません。辞書ウィンドウは 10～21 ビット、レベルは 1～7 であり、レベルはデコードの意味を変えません。
- Quantum には独立した終了トークンがなく、コンテナが正確な出力長を与え、同じ圧縮バイトが複数の適法な宣言長を満たすことがあります。期待する出力に達した後、少なくとも 2 個のゼロビットとバイト境界までに必要な最小数を消費し、合計 2～9 ビットとした後、圧縮入力を正確に使い切ります。追加のゼロバイト、非ゼロ終端ビット、切り詰め、フレーム間の一致は成功してはいけません。新しいフォルダーはまだ生成されていないウィンドウ内容を参照できません。

ボリューム集合とソース識別情報の固定、名前解析、エントリ識別、パス事前検査、タスク予算、キャンセル、準備、確定は引き続きホストが担当します。形式層はホストが許可したストリームだけを読み、CAB 内部名で独自にファイルを開きません。元の名前と UTF-8 宣言を別々に保持します。通常の重複名レコードは統合せず、仕様が明示的に定義した継続ファイルだけを結合します。アルゴリズムエラーがホストのキャンセル、クォータ、I/O エラーを隠してはいけません。共有契約がインターフェースとリソース上限を定めます。参照のバッファサイズや HRESULT 分類から新たな製品権限を導かないでください。
