<a name="xdzip-adaptive-filename-encoding-contract"></a>

# 적응형 파일 이름 인코딩 계약

[English](../../engineering/filename-encoding.md) | [简体中文](../../zh-CN/engineering/filename-encoding.md) | [繁體中文](../../zh-TW/engineering/filename-encoding.md) | [日本語](../../ja-JP/engineering/filename-encoding.md) | 한국어 | [Français](../../fr-FR/engineering/filename-encoding.md) | [Deutsch](../../de-DE/engineering/filename-encoding.md) | [Español](../../es-ES/engineering/filename-encoding.md) | [Português (Brasil)](../../pt-BR/engineering/filename-encoding.md) | [Русский](../../ru-RU/engineering/filename-encoding.md)

<a name="xdzip-resolution-and-identity"></a>

## 해석과 신원

디코딩은 Qt와 독립적인 Rust 형식 계층에서 수행합니다. 호스트 시스템 코드 페이지, UI 언어, 소스 시스템 추측, 파일 내용이나 네트워크 서비스를 사용하지 않습니다. ZIP의 raw_name은 이름 필드의 원래 바이트를 보존합니다. TAR은 PAX/GNU가 선택한 원시 이름에서 기존 규칙에 따라 앞의 `./`를 제거해 보존합니다. 물리 EntryId는 이름과 무관합니다. decoded_name은 트랜스코딩이나 추가 인코딩 선언이 필요한 UTF-8 표기를 저장하고 name_encoding은 해석 출처를 보존하며 name_bytes는 고정된 유효 이름을 제공합니다. display_name은 이스케이프된 표시 전용이며 출력 경로가 될 수 없습니다.

ZIP은 다음 순서로 이름을 해석합니다.

1. UTF-8 플래그가 설정되면 원래 필드에 유효한 UTF-8을 요구하며 손상된 선언을 이전 인코딩으로 복구하지 않습니다.
2. 그렇지 않으면 Info-ZIP Unicode Path 0x7075를 읽고 원래 이름 CRC32가 일치하며 완전하고 유효한 UTF-8인 버전 1을 허용합니다. 알 수 없는 버전과 CRC 불일치는 사양에 따라 무시합니다. 일치하지만 유효하지 않은 UTF-8, 중복된 유효 필드 또는 손상된 필드 구조는 검토 대상으로 표시합니다.
3. 사용할 수 있는 확장이 없고 선언되지 않은 그룹의 모든 이름이 유효한 UTF-8이면 표시되지 않은 UTF-8을 허용합니다.
4. 그렇지 않으면 선언되지 않은 비 ASCII 이름에 압축 파일 전체의 통계 탐지를 적용합니다. GBK/GB18030, Big5, Shift-JIS, EUC-JP, 한국어와 탐지기가 지원하는 다른 문자 집합이 포함됩니다. ZIP의 서구권 기본 추정은 과거 기본값 CP437로 대체합니다. GBK가 감지되면 호환되는 엄격한 GB18030 디코더를 사용합니다.

일반 TAR 헤더, GNU 긴 이름과 명시적으로 hdrcharset=BINARY로 표시된 PAX 경로는 같은 미선언 이름 전략을 사용합니다. 일반 PAX 경로에는 유효한 UTF-8이 필요하며 탐지로 손상된 선언을 숨겨서는 안 됩니다. UTF-8과 BINARY 이외의 명시적 hdrcharset은 지원하지 않습니다. TAR은 ZIP의 CP437 기본값을 상속하지 않습니다. TAR.GZ는 TAR과 동일한 이름 해석을 사용합니다.

규칙은 [ZIP APPNOTE 6.3.10](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT), [POSIX pax](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/pax.html), [WHATWG Encoding Standard](https://encoding.spec.whatwg.org/)를 따릅니다. 로컬 헤더에는 Unicode Path 필드가 없을 수 있지만 제공된 유효한 값은 고정된 유효 이름과 일치해야 합니다. 데이터 검사는 원시 이름, 로컬 및 중앙 헤더, CRC, 크기, 암호 인증과 스트림 종료를 계속 확인합니다.

CAB는 원래 이름 바이트와 UTF-8 속성을 보존합니다. 명시적 UTF-8은 엄격히 검증하고 다른 이름에는 시스템 코드 페이지 없이 전체 압축 파일의 제한된 탐지를 재사용합니다. CAB 백슬래시는 형식상 디렉터리 구분자이지만 전체 디코딩 후에만 슬래시로 정규화하여 Shift-JIS 멀티바이트의 후속 바이트 0x5C가 분리되지 않게 합니다. 원시 바이트는 바뀌지 않으며 유효 경로에도 같은 안전 및 충돌 검사를 적용합니다. Joliet이 아닌 ISO 이름은 원시 바이트를 보존하고 같은 제한된 해석을 사용하여 Rock Ridge 이름의 호스트 OEM 변환을 피합니다. Joliet/UDF는 프로세서가 제공한 Unicode 이름을 사용하고 엄격히 검증하며 손실성 대체 문자를 원래 이름으로 취급하지 않습니다.

<a name="xdzip-detection-boundaries-and-resources"></a>

## 탐지 경계와 리소스

탐지기는 선언되지 않은 완전한 비 UTF-8 이름을 최대 1 MiB 읽고 레코드를 줄 바꿈으로 구분하여 잘린 멀티바이트 시퀀스가 이름 사이에서 결합되지 않게 합니다. 명시적 Unicode 선언과 유효하지 않은 확장은 표본이 아닙니다. 표본에 비 UTF-8 이름이 있으면 선언되지 않은 이름에 전체 압축 파일의 결과를 일관되게 적용하여 GBK의 우연히 유효한 UTF-8 조각 때문에 한 디렉터리가 두 이름으로 나뉘지 않게 합니다. 개별 이름의 엄격한 디코딩이 실패한 경우에만 유효한 UTF-8이나 그 이름의 로컬 결과를 시도할 수 있습니다. ISO-2022-JP 추측은 비활성화하며 이름의 이스케이프 상태를 해석하지 않습니다. 엄격한 디코딩은 대체 문자를 삽입하거나 BOM을 제거하지 않고 슬래시 변환, Unicode 정규화 또는 경로 복구를 수행하지 않습니다.

원시 이름의 합계는 최대 16 MiB입니다. 원시 또는 디코딩된 이름 하나는 최대 4,096바이트이고 디코딩 이름과 이스케이프 표시의 합계는 최대 16 MiB입니다. ZIP Unicode 확장의 항목별 및 누적 할당은 압축 파일 끝이 아니라 검사 중에 확인합니다. 해석, 탐지와 항목별 변환은 취소를 확인합니다. ZIP 데이터는 유효 이름 검증 표를 위해 추가로 최대 16 MiB를 유지하며 나중 디코딩은 고정된 이름을 바꿀 수 없습니다.

통계 탐지는 임의의 짧은 이름이나 혼합된 이전 인코딩을 유일하게 복구할 수 없고 다른 소프트웨어가 이미 유효한 Unicode로 저장한 깨진 문자도 식별할 수 없습니다. 명시적으로 선언된 Unicode와 이전 인코딩은 같은 압축 파일에 공존할 수 있지만 선언되지 않은 여러 인코딩은 가능한 범위에서 탐지합니다. 현재 수동 인코딩 선택기는 없습니다. 엄격히 디코딩할 수 없거나 선언이 손상되었거나 경로가 비정상인 이름은 이스케이프하여 표시하고 검토 대상으로 표시하며 추출, 임시 열기, 내보내기 또는 수정 권한을 주지 않습니다. 모든 과거 압축 파일의 정확한 식별은 보장하지 않습니다. 해결되지 않은 이름이 있으면 실제 디렉터리 소속을 모를 수 있으므로 성공 결과에서 항목을 조용히 누락하지 않도록 디렉터리 확장 추출을 거부합니다. 명시적으로 선택한 해결된 일반 파일은 계속 개별 추출할 수 있습니다.

<a name="xdzip-end-to-end-use-and-saving"></a>

## 전체 사용과 저장

디렉터리 트리, 직접 자식 인덱스, 재귀 검색, 디렉터리 선택 확장, 사전 검사, 추출, 임시 열기, 외부 내보내기와 압축 파일 수정은 같은 유효 이름을 사용합니다. 디코딩 후에도 PortablePath, Unicode NFD 및 대소문자 별칭, 중복 이름과 출력 권한을 검사합니다. Shift-JIS 멀티바이트의 후속 바이트 0x5C는 디렉터리 구분자가 아닙니다. CAB와 LHA의 형식 구분자를 제외하고 실제 디코딩된 백슬래시는 계속 거부합니다.

계획 후 일반 ZIP, TAR, TAR.GZ의 추가와 이동은 이전 인코딩, Unicode 확장, 표시되지 않은 UTF-8 이름을 UTF-8으로 다시 선언하면서 논리 이동 계획의 원래 위치와 건너뛰기 의미를 유지합니다. ZIP은 압축 데이터를 보존하고 TAR은 명시적 UTF-8 PAX 경로를 사용합니다. 쓰기 후 다시 검사해 각 이름과 내용을 확인한 다음 기존 동일 볼륨 교체를 수행합니다. TAR 다시 쓰기에서 PAX 경로의 BINARY 선언을 제거하면 다른 비 UTF-8 PAX 텍스트의 해석이 바뀌는 경우 명시적으로 실패하고 원본 압축 파일을 보존합니다. 암호화 및 분할 압축 파일도 읽기와 추출에 같은 이름을 사용하며 추가와 이동 제한은 바뀌지 않습니다.

<a name="xdzip-reproducible-acceptance"></a>

## 재현 가능한 승인

프로젝트 소유 원시 바이트 테스트 데이터와 Python 표준 라이브러리로 독립적으로 인코딩한 압축 파일을 사용하여 표시되지 않은 UTF-8, GBK, 4바이트 GB18030, Big5, 0x5C가 포함된 Shift-JIS, 한국어, 러시아어, ZIP CP437과 Unicode Path를 다룹니다. 전체 이름, 원래 바이트, 디렉터리, 검색, 선택, 사전 검사, 실제 출력 바이트, 임시 열기, 내보내기, 일반 압축 파일 추가와 이동, AES 및 분할 볼륨을 검증합니다. 손상된 선언, 확장 우선순위, 경로 거부, Unicode 및 대소문자 충돌, 취소와 예산도 검사합니다.

scripts/check-filename-encoding.py는 실제 서비스 프로브로 Python 생성 ZIP/TAR/TAR.GZ를 추출합니다. Python도 Rust가 다시 쓴 압축 파일을 독립적으로 읽어 UTF-8 플래그/PAX, 이름과 내용을 확인합니다. 테스트 데이터는 소스 상수에서만 얻으며 사용자 압축 파일을 사용하지 않습니다. 실제 결과와 남은 수동 승인은 issue와 build/CI 산출물에 기록하며 이 계약에 통과 상태로 기록하지 않습니다.

LHA는 원래 dirname/filename 필드 경계와 dirname의 원시 `0xff` 구분자를 보존합니다. 공통 제한 탐지는 각 완전한 필드를 따로 표본 추출합니다. 디코딩된 백슬래시를 해석하고 `/`를 조합하기 전에 각 구성 요소를 엄격히 디코딩합니다. 원래 바이트 경계는 필드 간 멀티바이트 차용을 방지합니다. 디렉터리 확장과 출력은 같은 유효 이름을 사용합니다. [LHA 계약](../../engineering/lha-containers.md)을 참조하세요.
