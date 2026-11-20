<a name="xdzip-html-视觉一致性复核"></a>

# HTML 視覺一致性複核

[English](../../visual/html-fidelity.md) | [简体中文](../../zh-CN/visual/html-fidelity.md) | 繁體中文 | [日本語](../../ja-JP/visual/html-fidelity.md) | [한국어](../../ko-KR/visual/html-fidelity.md) | [Français](../../fr-FR/visual/html-fidelity.md) | [Deutsch](../../de-DE/visual/html-fidelity.md) | [Español](../../es-ES/visual/html-fidelity.md) | [Português (Brasil)](../../pt-BR/visual/html-fidelity.md) | [Русский](../../ru-RU/visual/html-fidelity.md)

高保真圖片是視覺依據，HTML 是可編輯實現。頂部與工具欄下方內容均從 [凍結的初版主視窗](../../../design/hifi/screens/main-light.png) 獨立量取尺寸，不從目前 CSS 反推“設計應當如此”。目前展示使用[膠囊標籤淺色稿](../../../design/hifi/screens/main-capsule-light.png)與[深色稿](../../../design/hifi/screens/main-capsule-dark.png)，它們由[可編輯源](../../../design/hifi/sources/main.html)匯出，不替代獨立量測。標題欄按 layout-scale.json 的顯式修訂驗收；原稿存在柔化邊緣和生成式字形，不能憑測試數量宣告整頁逐畫素一致。

<a name="xdzip-固定可重复条件"></a>

## 固定可重複條件

- 源畫布與 PNG 一致，為 1586×992；視窗 1548×940、原點 `(20,30)`。原稿錨點檢查仍使用 1588×1000 瀏覽器視口、裝置畫素比 1，頂部同坐標裁切 `(20,30)-(1568,167)`，內容區域裁切 `(20,167)-(1568,970)`，不拉伸原稿。
- 原稿 SHA-256 固定在 [header-reference.json](../../../design/html/header-reference.json) 和 [body-reference.json](../../../design/html/body-reference.json)。原稿變化後必須重新測量，不能默默替換。
- 字型為離線隨包的 [Xdzip Design Sans SC](../../../design/assets/fonts/README.md)，測量等待 `document.fonts.ready`。固定真實 400–750 字重，減少平臺回退字型改變寬度的問題。
- 參考字型的原始家族無法從生成圖片證明。字形輪廓與系統抗鋸齒仍可能不同；字形差異單獨記錄，不能偽稱已經還原原始字型。

<a name="xdzip-展示尺寸与源像素"></a>

## 展示尺寸與源畫素

正常設計展示沿用 [1024 邏輯畫素視窗寬度](README.md)，將源座標整體等比對映為 `1024 / 1548`。主視窗正常展示為 1024 × 約 621.81 CSS 畫素；原稿也使用同一倍率。這個比例來自顯式設計約定，不是根據 PNG 推斷 Retina 或 DPI。

`display-spec.js` 是所有圖板尺寸和展示比例的唯一來源。桌面預設正常設計尺寸，手機預設適應螢幕；適應倍率上限為正常設計尺寸，避免寬屏自動放大。原稿畫素模式則使用 `scale=1`，在 HTML 和圖片兩側同時保留原始畫素尺寸和滾動範圍。裝置畫素比只影響柵格化密度，不改變 CSS 邏輯寬度。

原稿和 HTML 禁止分別適應容器。頂部、正文錨點仍在源畫素座標中測量，展示縮放不修改已經校準的間距或字號。尺寸一致性另外覆蓋實際螢幕邊界、原稿自然尺寸、手機滾動、獨立預覽及離線單檔案，證據按[驗證方法](../../../developers/qa/README.md)儲存。

<a name="xdzip-画布与内容的滚动职责"></a>

## 畫布與內容的滾動職責

嵌入審閱工具的 HTML 是固定尺寸圖板，根文件使用 `overflow:hidden`，避免經典捲軸擠佔畫布寬高並互相觸發。審閱頁面自然上下滾動；外層畫布只在寬度不足時橫向平移。頁面的 `scrollbar-gutter:stable` 保持經典條預留寬度，避免臨界高度反覆改變適應倍率；獨立預覽使用 `auto`。檔案列表、長屬性及操作視窗保留內容容器的滾動，直接開啟非嵌入的源畫布也保留頁面滾動。該調整不改變源畫布、視窗或元件尺寸。

Playwright 預設攜帶 `--hide-scrollbars`，此前的截圖與尺寸矩陣沒有覆蓋實際經典捲軸。尺寸與預覽專項檢查現已明確移除該引數：同時檢查根捲軸佔位、視窗四邊可見、外層橫向平移和長屬性末尾按鈕可達。畫布少 1 px 的用例和移除嵌入規則的負向用例用於復現並捕獲捲軸相互擠佔。復現及修復截圖按[驗證方法](../../../developers/qa/README.md)保留在執行工件；這不等於驗證了所有瀏覽器和系統的捲軸外觀。

<a name="xdzip-本次顶部修正"></a>

## 本次頂部修正

| 部位 | 修正前 | 按原稿修正 |
| --- | --- | --- |
| 活動 Tab | 起點 x299、寬244 | x293、寬246 |
| 非活動 Tab | x557、寬244 | x542、寬262 |
| Tab 間距 | 14 px | 3 px；按柔化邊緣允許約1 px不確定性 |
| 加號與關閉按鈕 | 分屬兩套垂直佈局，中線差5 px | 使用同一標籤行，按原稿的約1 px光學偏移定位 |
| 關閉控制元件 | 一個按鈕、一個裸 span | 同一圖示／名稱／關閉控制元件結構與明確槽位 |
| 設定圖示 | 可見圖形小、偏左偏上 | 中心 `(1532,61)`；檢查實際圖形而非僅檢查SVG容器 |
| 活動 Tab 下邊框 | 與標題欄橫線疊加 | 與工具欄白色表面接通，左肩過渡獨立定義 |
| 搜尋與解壓 | 左右、垂直邊界有偏移 | 分別對照 `(906,101,449,51)`、`(1368,101,183,51)` |
| 工具欄左側圖示 | 通用按鈕尺寸和間距決定佈局 | 按原稿的圖形錨點定位與檢查 |

頂部元件的槽位和排版集中在 `hifi.css` 的標題欄、標籤和工具欄段落，不依賴通用 `.icon-button` 的尺寸或繼承行高。淺深主題共用幾何；Windows/Linux 的標題裝飾仍是瀏覽器演示，正式產品原生視窗單獨驗收。

<a name="xdzip-工具栏下方内容区域"></a>

## 工具欄下方內容區域

正文範圍為歸檔卡片、目錄樹、路徑欄、列表表頭與行、選擇狀態、檔案預覽和底部狀態列。記錄列邊界、行高、內邊距、圖示中心及可見墨跡，按 [獨立基準](../../../design/html/body-reference.json) 檢查。嚴格幾何與隔離圖示墨跡容差均為 2 px，文字輪廓只記錄供人工複核。資料夾的光學大小是明確的設計修訂：新輪廓在共享素材內等比放大約25.7%；五處舊原稿目錄墨跡保留作對照，按獨立記錄的新設計邊界檢查，其他原稿錨點不變。

正文複核使用同尺寸內容裁切，並為側欄、列表、預覽及狀態列保留區域對照。修復前後截圖、提交和測量只記錄在對應 issue／Actions 工件，不在規範中維護某次執行結果。手機按區域檢視，不能改變桌面排版隱藏差異。

原稿選中一項但全選框為空、標註節省26%但比例條填充約56%，屬於靜態狀態矛盾。HTML 分別保留 `indeterminate` 半選與實際26%比例，作為明確記錄的語義例外。正常預覽的溢位和長內容滾動由預覽專項檢查負責；選擇、導航、排序、搜尋和示例操作由現有互動檢查負責，不以幾何檢查替代功能驗證。

按[驗證方法](../../../developers/qa/README.md)分別儲存正文複核、同尺寸區域對照和量測證據。原稿的未知字型、生成軟邊、光影和抗鋸齒差異必須保留說明；本輪 HTML 證據不能用於宣稱 Qt 產品已完成對應校準。

<a name="xdzip-检查与交付"></a>

## 檢查與交付

執行：

```sh
python scripts/build-html-design.py
python scripts/check-html-header.py
python scripts/check-html-body.py
python scripts/check-html-preview.py
python scripts/check-html-references.py
python scripts/check-html-scale.py
python scripts/check-html-design.py
python scripts/build-html-header-review.py
python scripts/build-html-body-review.py
```

頂部檢查使用原稿獨立錨點：DOM 邊界／中心最大偏差不超過 1 px，隔離區域中的實際圖示墨跡邊界偏差不超過 2 px。下拉箭頭白色墨跡邊界容差 1 px；Tab 右肩按原稿的白色表面和弧線取樣，單通道顏色容差10/255，以容納原稿柔化。字形輪廓只記錄實測值。疊加圖和差異圖包含抗鋸齒、光影和字形差異，不能將總畫素差轉換成虛構的相似度。

頂部離線檔案是 `dist/xdzip-header-review.html`；正文離線檔案是 `dist/xdzip-body-review.html`。兩者均可切換原稿、修復前、修復後、透明疊加和畫素差異；正文增加手機區域選擇。介面單檔案是 `dist/xdzip-hifi-design.html`。

上述 HTML 檢查與離線複核按需在本地執行，不屬於桌面產品預設 CI。錨點失敗時應檢查差異；幾何透過後仍要看疊加證據。HTML 主視窗檢查不替代其他頁面校準，也不替代 Qt 產品與原生桌面驗收。

主視窗與選單的正常密度按[介面規範](design-standard.md)執行。必須同時核驗 Qt 選單／列表字號比例；在本地複核 HTML 時按[驗證方法](../../../developers/qa/README.md)檢查實際頁面縮放，舊源圖座標一致不能替代這一項。
