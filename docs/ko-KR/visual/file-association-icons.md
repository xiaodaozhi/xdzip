<a name="xdzip-file-association-icons"></a>

# 파일 연결 아이콘

[English](../../visual/file-association-icons.md) | [简体中文](../../zh-CN/visual/file-association-icons.md) | [繁體中文（台灣）](../../zh-TW/visual/file-association-icons.md) | [日本語](../../ja-JP/visual/file-association-icons.md) | 한국어 | [Français](../../fr-FR/visual/file-association-icons.md) | [Deutsch](../../de-DE/visual/file-association-icons.md) | [Español](../../es-ES/visual/file-association-icons.md) | [Português (Brasil)](../../pt-BR/visual/file-association-icons.md) | [Русский](../../ru-RU/visual/file-association-icons.md)

시스템 파일 아이콘은 xdzip의 접힌 모서리 문서, 연속 곡률 배지, 엮인 네 띠와 열린 U자 마크를 재사용합니다. 디자인 진입점은 [고충실도 검토 페이지](../../../design/hifi/sources/file-association-icons.html), 매개변수는 [file-association-icons.json](../../../specs/visual/file-association-icons.json), 문서 아이콘 소스와 제품 플랫폼 컨테이너는 [file-associations](../../../design/assets/file-associations)에 있습니다. 기본 애플리케이션은 계속 사용자가 선택합니다.

<a name="xdzip-shape-and-identification"></a>

## 모양과 식별

종이와 브랜드 기하 형태는 [brand_assets.py](../../../scripts/brand_assets.py)를 재사용합니다. 형식들은 같은 종이, 접힌 부분, 배지 위치와 시각적 중심을 공유하며 라벨로 형식을 식별하고 강조색으로 구분을 돕습니다. TAR.GZ, TGZ, GZ는 청록색 시각 계열입니다. 일반 대체 아이콘은 중립적인 회청색과 문서 선을 사용하며 형식 라벨은 붙이지 않습니다.

라벨은 원래 ZIP 글리프 스타일을 엄격히 따릅니다. 높이 61, 획 14, 사각 끝, 둥근 접합, 마스터 Y=817의 상단과 색상 #39465D를 사용합니다. ZIP은 원래 마스터의 전체 텍스트 경로를 직접 재사용하고 다른 글자는 같은 치수와 획 규칙으로 완성합니다. SVG 텍스트, 시스템 글꼴, 제3자 아이콘이나 내장 비트맵을 사용하지 않습니다. 최대 너비는 504입니다. TAR.GZ 라벨 전체를 보존하고 글리프를 늘리지 않으며 너비 예산을 초과할 때만 균등 축소하여 세로 중앙에 둡니다. 색상만으로 식별하지 않습니다.

마스터는 1024 단위입니다. 일반 PNG/ICO는 기존 viewBox `32 32 960 960`을 유지하고 종이 본체 높이는 약 90%입니다. macOS PNG/ICNS는 `-20 -20 1064 1064`를 유지하고 본체 높이는 약 81.2%입니다. 이는 프로젝트의 기존 시각 보정을 이어가는 것이며 시스템 요구 사항이라고 주장하지 않습니다. 아이콘은 테마별 종이, 데스크톱 배경이나 추가 둥근 마스크 없이 투명 캔버스를 사용합니다.

16과 24픽셀에서는 배지를 확대하고 종이를 단순화하며 32픽셀에서는 배지가 조금 더 작습니다. 이 세 크기는 라벨과 큰 이미지 그림자를 생략하면서 브랜드 마크의 원래 종횡비를 유지합니다. 특히 같은 색의 GZIP 계열을 포함해 이 크기에서는 아이콘만으로 모든 형식을 확실히 구분할 수 없습니다. 파일 이름과 유형 열로 정확히 식별합니다. 48픽셀부터 라벨이 있는 마스터를 사용하며 긴 라벨은 64픽셀 이상에서 더 잘 읽힙니다. 1024 PNG를 반복 축소하지 않고 각 크기를 벡터에서 직접 렌더링합니다.

UI의 24단위 아이콘과 기본 창 및 메뉴 밀도는 계속 [layout-scale.json](../../../specs/visual/layout-scale.json)을 따릅니다. 이 자산은 QML 창을 추가하거나 기본 창과 메뉴의 논리 크기를 바꾸지 않습니다. 플랫폼 PNG 크기는 이미지 샘플링 크기이며 QML 구성 요소의 논리 기하에 기록해서는 안 됩니다.

<a name="xdzip-format-and-platform-mapping"></a>

## 형식과 플랫폼 매핑

| 아이콘 ID | 라벨 | 확장자 / 용도 | 강조색 |
| --- | --- | --- | --- |
| `zip` | ZIP | `.zip` | 브랜드 파랑 |
| `7z` | 7Z | `.7z` | 남색 |
| `rar` | RAR | `.rar` | 보라 |
| `tar` | TAR | `.tar` | 호박색 |
| `tar-gz` | TAR.GZ | 확인된 복합 접미사 `.tar.gz` | 청록 |
| `tgz` | TGZ | `.tgz` | 청록 |
| `gz` | GZ | 시스템 후보 `.gz` | 청록 |
| `cab` | CAB | `.cab` | 강철 파랑 |
| `iso` | ISO | `.iso` | 호수 파랑 |
| `udf` | UDF | `.udf` | 청색 |
| `archive` | 없음 | 전용 아이콘이 없는 인식된 압축 파일의 대체 | 중립 회청색 |

이 표는 디자인 의도이며 플랫폼 간 동일한 등록을 의미하지 않습니다. 가능하면 전체 `.tar.gz`를 `.gz`보다 먼저 매핑합니다. 마지막 확장자만 인식하는 시스템은 둘 다 GZ로 표시할 수 있습니다. 모든 플랫폼이 복합 접미사를 자동으로 구분한다고 약속하지 않습니다. GZ 아이콘은 독립 GZIP과 TAR.GZ 외부 스트림이 공유하는 확장자를 나타내며 실제 읽기는 내용 검증과 형식 계약을 따릅니다.

암호화와 분할은 압축 파일 상태이며 확장자 아이콘에서 추론하지 않습니다. 자물쇠나 성공 표시를 그리지 않습니다. 인식된 ZIP, 7z, RAR 볼륨은 일반적인 `.001`, `.rNN`, `.zNN`, `.img`를 등록하지 않고 앱에서 해당 계열을 재사용할 수 있습니다. ISO/UDF 아이콘은 마운트나 실행을 의미하지 않습니다. 확장자 일치만으로 압축 파일 내용의 유효성을 입증할 수 없습니다.

일반 Windows 설치 프로그램은 확장자별 ProgID와 ICO를 사용합니다. MSIX는 해당 ICO 프레임에서 만든 PNG로 연결 프로그램 후보를 선언합니다. 둘 다 생성된 연결 매핑을 따르며 접미사 간 이미지를 재사용할 수 있습니다. macOS는 압축 TAR과 독립 GZIP을 포함해 문서 유형별 ICNS를 사용합니다. Linux는 시스템 MIME 아이콘을 사용합니다. 선택한 기본 앱과 시스템 아이콘 캐시가 실제 표시에 영향을 줍니다. [설치와 연결](../engineering/installers-associations.md)을 참조하세요.

MSIX는 각 형식 연결에 별도 `uap:Logo`를 선언합니다. 대상 크기 PNG는 Windows 정규 ICO 프레임을 바이트 단위로 재사용하여 작은 크기의 단순화, 큰 크기의 형식 라벨, 보이는 경계와 중앙 정렬을 유지합니다. 앱 로고는 기본형과 두 테마의 플레이트 없는 형태를 가진 별도 리소스입니다. [msix-icons.json](../../../specs/visual/msix-icons.json)이 패키지 크기와 변형을 정의하고 MakeAppx 패키징 전에 MakePRI가 인덱싱합니다. 시스템 배경판이나 앱 로고 대체는 의도된 문서 이미지에 포함되지 않습니다.

Windows 기본 앱과 연결 프로그램의 앱 항목은 [브랜드 앱 ICO](../../../design/assets/brand/xdzip.ico)를 사용하며 기본 실행 파일과 연결 실행기에 모두 내장합니다. 형식별 문서 아이콘은 앱 정체성을 대체하지 않습니다. 시스템에는 브랜드 아이콘이 있는 선택 가능한 xdzip 항목 하나가 표시되어야 하며 이전 공통 명령은 기존 선택과의 호환성에 사용합니다. 기존 자산을 재사용하며 기본 창, 메뉴 또는 QML 논리 기하를 바꾸지 않습니다. 네이티브 앱 아이콘, 문서 아이콘과 후보 수는 별도로 승인하며 운영체제의 일반 앱 상자를 형식 문서 아이콘 문제로 분류해서는 안 됩니다.

<a name="xdzip-generation-and-delivery"></a>

## 생성과 전달

저장소 루트에서 실행합니다.

```sh
python scripts/build-association-icons.py
python scripts/build-design-assets.py --metadata-only
python scripts/build-association-icons.py --check --check-runtime
# 기존 Pillow, Playwright, Chromium으로 로컬 산출물을 내보낸다.
python scripts/build-association-icons.py --export build/file-association-icons
python scripts/build-association-icons.py --check --export build/file-association-icons
# 필요한 ICO/ICNS 내보내기를 정규 자산 폴더에 복사한 뒤 실행한다.
python scripts/build-design-assets.py --metadata-only --refresh-association-icons
```

`XDZIP_CHROMIUM`으로 기존 Chromium/Edge 경로를 지정할 수 있습니다. SVG 마스터, 작은 크기 소스, 사양과 검토 페이지는 버전 관리하며 자산 소스는 기본 [manifest.json](../../../design/assets/manifest.json)에 등록합니다. 생성 진입점은 [build-association-icons.py](../../../scripts/build-association-icons.py), 구성 정의는 [association_icon_assets.py](../../../scripts/association_icon_assets.py)에 있습니다.

내보낸 산출물에는 형식마다 16~1024픽셀의 일반 및 macOS PNG와 ICO, ICNS가 있습니다. 형식 레지스트리에서 관리하는 제품 ICO와 ICNS는 `file-associations/`의 벡터 소스 옆에 있습니다. ZIP 컨테이너는 마이그레이션 전 원래 바이트를 유지하고 나머지는 해당 내보내기에서 생성됩니다. 일반 빌드는 브라우저 없이 이 컨테이너를 직접 읽습니다. 임시 PNG, 전체 내보내기, 연락 시트와 확인 기록은 `build/` 또는 `dist/`에 남깁니다.

<a name="xdzip-verification-and-acceptance"></a>

## 검증과 승인

생성 검사는 SVG가 사양과 일치하고 글꼴이나 외부 이미지 의존성이 없는지 확인합니다. 플랫폼 검사는 캔버스와 RGBA, 투명한 모서리, 보이는 본체 높이와 중앙 정렬, 벡터 다이제스트와 각 ICO/ICNS 표현의 디코딩된 픽셀 일치를 다룹니다. 본체 경계는 alpha ≥ 128에서 측정하며 브랜드 허용 오차인 1.25 물리 픽셀을 사용합니다.

시각 검토는 밝은 배경, 어두운 배경, 선택된 배경, 16~256픽셀 크기와 일반 및 높은 DPR의 실제 렌더링을 다룹니다. 검토 페이지는 CSS 픽셀을 사용하며 브라우저 스크린샷을 Explorer, Finder 또는 Linux 네이티브 승인으로 표시해서는 안 됩니다. 이후 통합에서는 실제 파일 이름, 복합 접미사, 아이콘 캐시, 기본 선택 전후 및 설치와 제거 후 표시를 확인해야 합니다. 단계 상태와 실제 실행 범위는 GitHub issue에 기록하고 여기에는 테스트 보고서를 누적하지 않습니다.
