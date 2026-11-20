<a name="xdzip-file-association-icons"></a>

# ファイル関連付けアイコン

[English](../../visual/file-association-icons.md) | [简体中文](../../zh-CN/visual/file-association-icons.md) | [繁體中文（台灣）](../../zh-TW/visual/file-association-icons.md) | 日本語 | [한국어](../../ko-KR/visual/file-association-icons.md) | [Français](../../fr-FR/visual/file-association-icons.md) | [Deutsch](../../de-DE/visual/file-association-icons.md) | [Español](../../es-ES/visual/file-association-icons.md) | [Português (Brasil)](../../pt-BR/visual/file-association-icons.md) | [Русский](../../ru-RU/visual/file-association-icons.md)

システムのファイルアイコンは xdzip の折り返した角の文書、連続曲率のバッジ、交差する四本の帯、開いた U 字のマークを再利用します。設計入口は[高忠実度レビュー画面](../../../design/hifi/sources/file-association-icons.html)、パラメーターは [file-association-icons.json](../../../specs/visual/file-association-icons.json)、文書アイコンのソースと製品のプラットフォーム用コンテナーは [file-associations](../../../design/assets/file-associations) にあります。既定のアプリは引き続きユーザーが選択します。

<a name="xdzip-shape-and-identification"></a>

## 形状と識別

紙とブランドの幾何形状は [brand_assets.py](../../../scripts/brand_assets.py) を再利用します。各形式は同じ紙、折り目、バッジ位置、視覚上の中心を共有し、ラベルで形式を識別しアクセント色で見分けやすくします。TAR.GZ、TGZ、GZ は青緑の視覚的な系列です。汎用の代替は中間的な青灰色と文書の罫線を使い、形式ラベルは付けません。

ラベルは元の ZIP グリフのスタイルに厳密に従います。高さ 61、線幅 14、四角い端、丸い接合、上端はマスターの Y=817、色は #39465D です。ZIP は元のマスターの完全なテキストパスを直接再利用し、他の文字は同じ寸法と線の規則で補います。SVG テキスト、システムフォント、第三者アイコン、埋め込みビットマップは使いません。最大幅は 504 です。TAR.GZ ラベル全体を保持してグリフを引き伸ばさず、幅の予算を超える場合だけ均等に縮小して垂直方向の中央に配置します。色だけに識別を頼りません。

マスターは 1024 単位です。汎用 PNG/ICO は既存の viewBox `32 32 960 960` を維持し、紙本体の高さは約 90% です。macOS PNG/ICNS は `-20 -20 1064 1064` を維持し、本体の高さは約 81.2% です。これはプロジェクトの視覚調整の継続であり、システム要件を主張するものではありません。アイコンは透明なキャンバスを使い、テーマ別の紙、デスクトップ背景、追加の角丸マスクは使いません。

16 と 24 ピクセルではバッジを拡大して紙を簡略化し、32 ピクセルでは少し小さいバッジを使います。この三つのサイズはラベルと大きな画像の影を省き、ブランドマークの元の縦横比を保持します。これらのサイズでは、特に同色の GZIP 系列など、アイコンだけですべての形式を確実に区別できません。正確な識別にはファイル名や種類の列を使います。48 ピクセルからラベル付きマスターを使い、長いラベルは 64 ピクセル以上で読みやすくなります。1024 PNG を繰り返し縮小せず、各サイズをベクターから直接描画します。

UI の 24 単位アイコンとメインウィンドウ、メニューの密度は引き続き [layout-scale.json](../../../specs/visual/layout-scale.json) に従います。これらの素材は QML ウィンドウを追加せず、メインウィンドウやメニューの論理寸法も変えません。プラットフォームの PNG 寸法は画像のサンプリング寸法であり、QML コンポーネントの論理ジオメトリーに書き込んではいけません。

<a name="xdzip-format-and-platform-mapping"></a>

## 形式とプラットフォームの対応

| アイコン ID | ラベル | 拡張子 / 用途 | アクセント |
| --- | --- | --- | --- |
| `zip` | ZIP | `.zip` | ブランドブルー |
| `7z` | 7Z | `.7z` | インディゴ |
| `rar` | RAR | `.rar` | 紫 |
| `tar` | TAR | `.tar` | 琥珀色 |
| `tar-gz` | TAR.GZ | 確認済みの複合拡張子 `.tar.gz` | 青緑 |
| `tgz` | TGZ | `.tgz` | 青緑 |
| `gz` | GZ | システム候補 `.gz` | 青緑 |
| `cab` | CAB | `.cab` | スチールブルー |
| `iso` | ISO | `.iso` | レイクブルー |
| `udf` | UDF | `.udf` | シアンブルー |
| `archive` | なし | 専用アイコンがない認識済みアーカイブの代替 | 中間的な青灰色 |

この表は設計意図であり、プラットフォーム共通の登録を示すものではありません。可能な場合は完全な `.tar.gz` を `.gz` より優先して名前を対応付けます。最後の拡張子だけを認識するシステムでは両方に GZ が表示される場合があります。すべてのプラットフォームが複合拡張子を自動で区別できると約束してはいけません。GZ アイコンは独立した GZIP と TAR.GZ の外側ストリームが共有する拡張子を示し、実際の読み取りは内容の検証と形式契約に従います。

暗号化と分割はアーカイブの状態であり、拡張子のアイコンから推測しません。鍵や成功のマークを描いてはいけません。認識された ZIP、7z、RAR の分割ファイルは汎用の `.001`、`.rNN`、`.zNN`、`.img` を登録せずにアプリ内で対応系列を再利用できます。ISO/UDF アイコンはマウントや実行を意味しません。拡張子の一致だけでアーカイブの内容が有効とは判断できません。

通常の Windows インストーラーは拡張子ごとの ProgID と ICO を使います。MSIX は対応する ICO フレームから作った PNG を使い、「プログラムから開く」の候補を宣言します。両方とも生成された関連付けに従い、拡張子間で画像を再利用できます。macOS は圧縮 TAR と独立した GZIP を含め、文書タイプごとに ICNS を使います。Linux はシステムの MIME アイコンを使います。選択された既定アプリとシステムのアイコンキャッシュが実際の表示に影響します。[インストールと関連付け](../engineering/installers-associations.md)を参照してください。

MSIX は形式の関連付けごとに別々の `uap:Logo` を宣言します。ターゲットサイズの PNG は Windows の正規 ICO フレームをバイト単位で再利用し、小さいサイズの簡略化、大きいサイズの形式ラベル、見える範囲、中央配置を保持します。アプリのロゴは別リソースで、既定と両テーマのプレートなし形式を持ちます。[msix-icons.json](../../../specs/visual/msix-icons.json) がパッケージのサイズと種類を定義し、MakeAppx でのパッケージ化前に MakePRI が索引を作ります。システムの背景プレートやアプリロゴへの代替は意図した文書画像に含みません。

Windows の既定アプリと「プログラムから開く」のアプリ項目には[ブランドのアプリ ICO](../../../design/assets/brand/xdzip.ico) を使い、メイン実行ファイルと関連付けランチャーの両方に埋め込みます。形式別の文書アイコンはアプリの識別を置き換えません。システムにはブランドアイコン付きの選択可能な xdzip 項目を一つ表示し、古い共有コマンドは既存の選択との互換性に使います。既存素材を再利用し、メインウィンドウ、メニュー、QML の論理ジオメトリーは変えません。ネイティブアプリアイコン、文書アイコン、候補数は別々に受け入れ、OS の汎用アプリ枠を形式別文書アイコンの問題と分類してはいけません。

<a name="xdzip-generation-and-delivery"></a>

## 生成と提供

リポジトリーのルートで実行します。

```sh
python scripts/build-association-icons.py
python scripts/build-design-assets.py --metadata-only
python scripts/build-association-icons.py --check --check-runtime
# 既存の Pillow、Playwright、Chromium でローカル成果物を出力する。
python scripts/build-association-icons.py --export build/file-association-icons
python scripts/build-association-icons.py --check --export build/file-association-icons
# 必要な ICO/ICNS 出力を正規素材フォルダーへコピーした後に実行する。
python scripts/build-design-assets.py --metadata-only --refresh-association-icons
```

`XDZIP_CHROMIUM` には既存の Chromium/Edge のパスを指定できます。SVG マスター、小サイズのソース、仕様、レビュー画面はバージョン管理し、素材ソースはメインの [manifest.json](../../../design/assets/manifest.json) に登録します。生成入口は [build-association-icons.py](../../../scripts/build-association-icons.py)、構成定義は [association_icon_assets.py](../../../scripts/association_icon_assets.py) にあります。

出力成果物は形式ごとに 16〜1024 ピクセルの汎用と macOS 用 PNG、および ICO と ICNS を含みます。形式レジストリーで管理する製品 ICO と ICNS は `file-associations/` のベクターソースの隣に置きます。ZIP コンテナーは移行前の元バイトを保持し、他は対応する出力から生成します。通常のビルドはこれらを直接読み取り、ブラウザーを必要としません。一時 PNG、完全な出力、コンタクトシート、記録は `build/` または `dist/` に保持します。

<a name="xdzip-verification-and-acceptance"></a>

## 検証と受け入れ

生成検査は SVG が仕様に一致し、フォントや外部画像に依存しないことを確認します。プラットフォーム検査はキャンバスと RGBA、透明な角、本体の見える高さと中央配置、ベクターのダイジェスト、各 ICO/ICNS 表現のデコード後のピクセル一致を対象とします。本体の境界は alpha ≥ 128 で測定し、ブランドの許容差 1.25 物理ピクセルを使います。

視覚レビューは明色、暗色、選択状態の背景、16〜256 ピクセル、通常と高 DPR での実際の描画を対象とします。レビュー画面は CSS ピクセルを使い、ブラウザーのスクリーンショットを Explorer、Finder、Linux のネイティブ受け入れとして扱ってはいけません。後の統合では実際のファイル名、複合拡張子、アイコンキャッシュ、既定選択の前後、インストールやアンインストール後の表示を確認します。段階の状態と実際の実行範囲は GitHub issue に記録し、この文書にテスト報告を蓄積しません。
