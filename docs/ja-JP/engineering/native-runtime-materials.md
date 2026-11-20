<a name="xdzip-native-runtime-provenance-and-notices"></a>

# ネイティブランタイムの出所と通知

[English](../../engineering/native-runtime-materials.md) | [简体中文](../../zh-CN/engineering/native-runtime-materials.md) | [繁體中文（台灣）](../../zh-TW/engineering/native-runtime-materials.md) | 日本語 | [한국어](../../ko-KR/engineering/native-runtime-materials.md) | [Français](../../fr-FR/engineering/native-runtime-materials.md) | [Deutsch](../../de-DE/engineering/native-runtime-materials.md) | [Español](../../es-ES/engineering/native-runtime-materials.md) | [Português (Brasil)](../../pt-BR/engineering/native-runtime-materials.md) | [Русский](../../ru-RU/engineering/native-runtime-materials.md)

この契約は実際のパッケージ内の Qt 以外のネイティブランタイムを対象とし、[ランタイムの帰属](runtime-attribution.md)と[対応するソース](source-delivery.md)と併せて使用します。ソースのバージョン、ファイルの同一性、ライセンス条件、実際の履行は別々に確認します。供給元ファイルのハッシュはライセンス承認ではありません。`v0.1.0` の過去の候補に関する結論と補足証拠は [#4](https://github.com/xiaodaozhi/xdzip/issues/4) と [#53](https://github.com/xiaodaozhi/xdzip/issues/53) にあります。今後の候補には新しいリリース issue を使用し、証拠は build/ または CI 成果物にのみ生成します。

| コンポーネント | 元の根拠と配布資料 | レビューの境界 |
| --- | --- | --- |
| Qt 6.8.2 | SDK SPDX、元のモジュールソースとパッチ、Qt 通知、LGPLv3 と GPLv3 の全文、アプリの再ビルドと互換置換の手順 | ランタイムは LGPL 分岐を使用し、ツールと第三者のライセンスは別に扱います。SDK ファイルの対応関係は上流ビルドとの完全な等価性ではありません |
| Rust ランタイムコード | Cargo.lock、実際のリンク入力、ランタイム依存候補、完全な vendor 通知と元の通知、Rust 標準ライブラリーのソース | ビルドグラフの通知を保守的に配布し、build/dev/proc-macro ツールをリンク済みランタイムコードと誤報しません |
| Noto Sans SC、TC、JP、KR と Noto Sans の派生フォント | フォントマニフェスト、元の OFL 文書、改名したフォント、生成入口 | プロジェクトの Apache-2.0 はフォントに適用されません。変更と命名の説明を保持します |
| ICU 73.2 | 実際のライブラリーバージョン、第三者通知を含む上流 ICU LICENSE 全文、出所記録 | 一般的な MIT 文書で ICU 自体の通知を代用せず、システムラッパーを実際にパッケージ化されたファイルと混同しません |
| Microsoft VC++ ランタイム | 固定された公式 VC_redist パッケージと DLL ハッシュ、元のライセンス、バージョンと配布の一覧 | ファイルは変更しません。保守担当者の資格と再配布条件は別途レビューが必要です |
| Windows SDK のグラフィックスコンパイラーライブラリー | 新しいパッケージから除外。過去のパッケージでは元の SDK ライセンス、DLL ごとのハッシュ、出所を保持 | 過去の配布条件は別途レビューし、新パッケージからの削除によって変更されません |
| Inno Setup 6.7.3 インストーラーエンジン | 固定されたコンパイラーパッケージとソースコミット、元の Inno と RemObjects Pascal Script のライセンス、二言語の Installer-CREDITS.txt | インストーラーエンジンの既存の著作権 UI を保持します。Pascal Script は別途、名前とソース URL を About または文書に記載する必要があり、一般の Inno ライセンスだけでは不十分です |

Microsoft の[ランタイム再配布ガイダンス](https://learn.microsoft.com/en-us/cpp/windows/redistributing-visual-cpp-files)と [VS 2022 再配布可能ファイル一覧](https://learn.microsoft.com/en-us/visualstudio/releases/2022/redistribution)は、再配布権を適用されるソフトウェアライセンスに結び付けています。インストーラーのライセンス文は保守担当者が資格を満たす証明ではありません。Qt の[オープンソース義務](https://www.qt.io/development/open-source-lgpl-obligations)は、利用者が対応ソースを取得、変更、再リンクし、変更したライブラリーを実行する権利の保持を求めます。動的リンクは他の義務の代わりにはなりません。

無料の Community 経路、ホスト型 Enterprise の別個の権限、再配布条件については [Windows ライセンス](windows-licensing.md)を参照してください。デプロイでは元の CRT/SDK ライセンス、出所、二言語の Windows-runtime-NOTICE.txt をランタイムと資料の両方に含めます。コンパイル前に Setup は元の CRT 契約のハッシュを確認し、ライセンスページを表示します。Microsoft DLL を Apache-2.0 に再ライセンスすることはありません。

<a name="xdzip-generation-and-verification"></a>

## 生成と検証

`scripts/native-runtime-sources.json` は ICU ライセンスのバージョン、公式 URL、SHA-256 を固定します。`collect-notices.py` は Linux で ICU ライセンス全文を収集し、キャッシュとダウンロードの両方のハッシュを再確認します。現在の Windows 開発パッケージには `opengl32sw.dll` が含まれず、そのための Mesa/LLVM 通知も収集しません。通知出力ディレクトリーに古い専用資料が残る場合、収集は再利用を拒否し、新しい候補に誤って追加することを防ぎます。

`prepare-source-delivery.py` は ICU ライセンスと固定されたソース一覧を含めます。資料は元のバイト列を保持し、配布マニフェストはサイズとハッシュを再計算します。Mesa ソフトウェアレンダリングライブラリーを含む過去のパッケージには、対応する過去のソース、通知、義務が引き続き必要です。新しい方針で過去の配布を変更することはできません。

プラットフォーム資料の添付には、帰属レポートの `license_materials_base` が参照するディレクトリーを含める必要があります。パッケージ化前に元の各ライセンス文書のハッシュを再確認します。ランタイムディレクトリーの別の場所に同じ文書があっても、レポートが存在しない相対ディレクトリーを指すことは正当化されません。

Windows デプロイはソフトウェア OpenGL、D3D コンパイラー、DXC について windeployqt の環境検索を無効にし、Windows SDK から 3 つのグラフィックスコンパイラーライブラリーをコピーしません。これらの名前を含むステージングは拒否します。管理されたステージングのルートに残ったソフトウェア OpenGL DLL は削除し、他の場所に残った同名ファイルは拒否します。

macOS パッケージ化は最終署名済みアプリがまだ存在する間に `deployment-audit.json`、`macos-deployment.json`、`runtime-attribution.json` を出力します。ファイル SBOM は同じ監査と帰属レポートを使用します。Qt のネイティブファイルは arm64 スライス、すべてのセクションバイト、UUID、インポートバージョンで比較し、デプロイに必要な Qt パスの再配置を許可します。署名、ロードコマンドの配置、LINKEDIT はこの比較の結論に含まれません。非ネイティブリソースは完全に一致するバイト列で関連付けます。関連付けられないファイルと元の SPDX の不足は残ります。この処理は完全なバイナリー等価性ではありません。

公開後に通知の欠落が見つかった場合、補足資料が元のパッケージに含まれていたと説明することはできません。元バージョンの成果物とチェックサムは不変です。修正パッケージには新しいバージョンと再検証が必要です。補足資料とその結び付き先を明示的に記録します。
