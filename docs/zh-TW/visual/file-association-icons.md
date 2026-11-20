<a name="xdzip-文件关联图标"></a>

# 檔案關聯圖示

[English](../../visual/file-association-icons.md) | [简体中文](../../zh-CN/visual/file-association-icons.md) | 繁體中文（台灣）

系統檔案圖示沿用 xdzip 的折角文件、連續曲率徽章、四條交錯帶與開放 U 形標記。設計入口為[高保真審閱頁](../../../design/hifi/sources/file-association-icons.html)，引數來源為 [file-association-icons.json](../../../specs/visual/file-association-icons.json)，文件圖示源和產品所需平臺容器統一位於 [file-associations](../../../design/assets/file-associations)。預設應用仍由使用者選擇。

<a name="xdzip-形状与识别"></a>

## 形狀與識別

紙張與品牌幾何複用 [brand_assets.py](../../../scripts/brand_assets.py)。格式之間保持相同的紙張、折角、徽章位置和光學中心，透過標籤識別格式，輔助色幫助掃視。TAR.GZ、TGZ、GZ 屬於同一視覺家族，使用青綠色。通用後備採用中性灰藍和文件橫線，不加格式標籤。

標籤嚴格沿用原 ZIP 的字形風格：高度 61、筆畫 14、方形端點、圓角連線，頂邊位於母版 Y=817，顏色為 #39465D。ZIP 直接複用原母版的完整文字路徑，其他字母在相同度量與筆畫規則下補齊；不使用 SVG text、系統字型、第三方圖示或嵌入點陣圖。最大寬度 504，保留完整的 TAR.GZ，不拉伸字形，只有超過寬度預算才等比縮小並垂直居中。顏色不承擔唯一識別職責。

母版為 1024 單位。通用 PNG／ICO 沿用既有 `32 32 960 960` 取景框，紙張主體高度約 90%；macOS PNG／ICNS 沿用既有 `-20 -20 1064 1064` 取景框，主體高度約 81.2%。這是專案現有光學校準的延續，不宣稱為系統強制標準。圖示使用透明畫布，不按桌面主題替換紙張，不帶桌面底色或額外圓角遮罩。

16、24 畫素採用放大的徽章和簡化紙張，32 畫素採用稍小的徽章；這三檔省去標籤與大圖陰影，保留品牌標記的原始寬高比。此時無法僅靠圖示可靠地區分全部格式，尤其是同色的 GZIP 家族，檔名／型別列承擔精確識別。48 畫素起使用帶標籤母版，長標籤在 64 畫素以上更易讀。各尺寸直接從向量渲染，不把 1024 PNG 連續縮小。

介面中的 24 單位圖示、主視窗和選單密度繼續遵循 [layout-scale.json](../../../specs/visual/layout-scale.json)。本素材不新增 QML 視窗，也不改變主視窗或選單的邏輯尺寸。平臺 PNG 尺寸是影象取樣尺寸，不能寫入 QML 元件邏輯幾何。

<a name="xdzip-格式与平台映射"></a>

## 格式與平臺對映

| 圖示標識 | 標籤 | 字尾／用途 | 輔助色 |
| --- | --- | --- | --- |
| `zip` | ZIP | `.zip` | 品牌藍 |
| `7z` | 7Z | `.7z` | 靛藍 |
| `rar` | RAR | `.rar` | 紫色 |
| `tar` | TAR | `.tar` | 琥珀色 |
| `tar-gz` | TAR.GZ | 已確認的 `.tar.gz` 複合字尾 | 青綠 |
| `tgz` | TGZ | `.tgz` | 青綠 |
| `gz` | GZ | `.gz` 系統候選 | 青綠 |
| `cab` | CAB | `.cab` | 鋼藍 |
| `iso` | ISO | `.iso` | 湖藍 |
| `udf` | UDF | `.udf` | 青藍 |
| `archive` | 無 | 已識別歸檔無專屬圖示時的後備 | 中性灰藍 |

此表是設計意圖，不是跨平臺統一登錄檔。檔名對映在可用時優先匹配完整 `.tar.gz`，再匹配 `.gz`；按最終副檔名識別的系統可能只能為兩者顯示 GZ 圖示。不要承諾所有平臺都能自動區別複合字尾。GZ 圖示只表示副檔名，普通 GZIP 與 TAR.GZ 外層共享該圖示，實際讀取能力仍由內容校驗和格式契約決定。

加密與分卷是歸檔狀態，不由副檔名圖示猜測，不繪製鎖或成功標記。已識別 ZIP、7z、RAR 分卷可在應用內複用對應家族，不註冊通用 `.001`、`.rNN`、`.zNN` 或 `.img`。ISO／UDF 圖示不表示掛載或執行能力。不能僅因字尾匹配就宣告歸檔內容有效。

普通 Windows 安裝版使用逐副檔名 ProgID 和 ICO。MSIX 為“開啟方式”候選使用從對應 ICO 幀提取的 PNG。兩者使用生成的關聯對映，多個字尾可以共用圖示。macOS 按文件型別使用 ICNS，包括壓縮 TAR 和獨立 GZIP。Linux 使用系統 MIME 圖示。實際顯示受預設應用和系統圖示快取影響，見[安裝與關聯](../engineering/installers-associations.md)。

MSIX 為每個格式關聯單獨宣告 `uap:Logo`，各目標尺寸 PNG 逐位元組複用 Windows 標準 ICO 幀，保留小尺寸簡化、大尺寸格式標籤、可見邊界及居中位置。應用標誌使用獨立資源，提供預設和兩種主題的無底板變體。[msix-icons.json](../../../specs/visual/msix-icons.json) 定義包內尺寸與變體，由 MakePRI 建立索引後再交給 MakeAppx 打包。文件圖示的預期設計不包含系統底板，也不以應用標誌作為後備圖案。

Windows 預設應用與“開啟方式”裡的應用入口使用 [品牌應用 ICO](../../../design/assets/brand/xdzip.ico)，主程式和關聯啟動器均嵌入此資源；不使用逐格式文件圖示代替應用身份。系統應只展示一個名稱為 xdzip、帶品牌圖示的可選入口，舊共享命令僅用於相容既有選擇。該調整複用已有素材，不改變主視窗、選單或 QML 邏輯幾何。原生應用圖示、文件圖示及候選數量分別驗收，不能將作業系統的通用應用方框判作格式文件圖示問題。

<a name="xdzip-生成与交付"></a>

## 生成與交付

在倉庫根目錄執行：

```sh
python scripts/build-association-icons.py
python scripts/build-design-assets.py --metadata-only
python scripts/build-association-icons.py --check --check-runtime
# 匯出至本機工件，使用現有的 Pillow、Playwright 環境與 Chromium。
python scripts/build-association-icons.py --export build/file-association-icons
python scripts/build-association-icons.py --check --export build/file-association-icons
# 將需要的 ICO/ICNS 匯出結果複製到正式素材目錄後，驗證並更新清單。
python scripts/build-design-assets.py --metadata-only --refresh-association-icons
```

`XDZIP_CHROMIUM` 可指定已有 Chromium／Edge 路徑。SVG 母版、小尺寸源、規範和高保真審閱頁納入版本管理；素材源在主 [manifest.json](../../../design/assets/manifest.json) 中登記。生成入口是 [build-association-icons.py](../../../scripts/build-association-icons.py)，組合定義見 [association_icon_assets.py](../../../scripts/association_icon_assets.py)。

匯出工件每格式包含 16 至 1024 畫素的通用與 macOS PNG，以及 ICO 和 ICNS。產品 ICO、ICNS 由格式能力表登記，與向量源同放 `file-associations/`；ZIP 容器保留遷移前原位元組，其他容器來自相應匯出。普通構建直接讀取這些容器，不依賴瀏覽器。臨時 PNG、完整匯出、聯絡表和回執保留在 `build/` 或 `dist/`。

<a name="xdzip-验证与验收"></a>

## 驗證與驗收

生成檢查驗證 SVG 與規格一致、無字型／外部影象依賴。平臺檢查包括畫布與 RGBA、透明角、可見主體高度和居中、向量摘要以及每種 ICO／ICNS 表示解碼後的畫素一致性。主體邊界按 alpha ≥ 128 測量，容差沿用品牌的 1.25 物理畫素。

視覺審閱同時覆蓋淺色、深色與選中背景，16 至 256 畫素，以及普通和高 DPR 的實際渲染。審閱頁按 CSS 畫素呈現，不將瀏覽器截圖示記為 Explorer、Finder 或 Linux 原生驗收。後續接入須檢查真實檔名、複合字尾、圖示快取、預設選擇前後和安裝／解除安裝後的顯示。階段狀態和實際執行範圍在 GitHub issue 中記錄，不在本規範累積測試報告。
