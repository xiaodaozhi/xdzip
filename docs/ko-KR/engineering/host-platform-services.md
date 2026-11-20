<a name="xdzip-host-calendar-and-encoding-services-for-format-workers"></a>
# 형식 워커를 위한 호스트 달력 및 인코딩 서비스

[English](../../engineering/host-platform-services.md) | [简体中文](../../zh-CN/engineering/host-platform-services.md) | [繁體中文](../../zh-TW/engineering/host-platform-services.md) | [日本語](../../ja-JP/engineering/host-platform-services.md) | 한국어 | [Français](../../fr-FR/engineering/host-platform-services.md) | [Deutsch](../../de-DE/engineering/host-platform-services.md) | [Español](../../es-ES/engineering/host-platform-services.md) | [Português (Brasil)](../../pt-BR/engineering/host-platform-services.md) | [Русский](../../ru-RU/engineering/host-platform-services.md)

Rust는 형식 필드와 암호 호환성 규칙을 처리하고 데이터를 검증합니다. 운영체제의 과거 시간대 규칙, OEM 및 ANSI 코드 페이지, 로컬 C 인코딩은 호스트 환경에 속하며 선택적 `ReadSource` 서비스로 제공됩니다. 형식 계층은 파일 이름, HostOS 또는 현재 UTC 오프셋으로 이를 추측해서는 안 됩니다. 이 서비스는 Qt에 의존하지 않으며 경로 접근이나 파일 확정 권한을 주지 않습니다.

<a name="xdzip-calendar-projection"></a>

## 날짜와 시간 변환

`Timestamp`는 1601년부터 100 ns 단위로 센 FILETIME 값과 1970년부터 부호 없는 정수 초로 센 Unix 값을 구별합니다. `ReadSource::local_calendar`는 선택적 `Calendar`를 반환합니다. 일반 파일과 메모리 소스는 기본으로 항상 같은 UTC 변환을 사용합니다. 제품 워커는 동기 CXX 콜백으로 운영체제의 로컬 시간대 규칙을 사용합니다. Windows는 시스템 시간대 API를, Unix는 `localtime_r`를 사용하며 모두 변환 결과를 확인합니다. 변환할 수 없으면 달력을 반환하지 않으며 초기화되지 않은 필드를 읽어서는 안 됩니다.

Rust는 DOS 표시 값을 만들기 전에 연월일과 시분초를 검증합니다. 7z는 기존 UTC와 1980..2107 표시 범위를 유지합니다. RAR은 내부 모듈러 연산, 시간값 재정의, 플랫폼 차이에 대해 [고정된 호환성 규칙](rar-format-sources.md)을 적용한 뒤 호스트 달력과 기존 연도 순환 처리를 사용합니다. 원래의 정밀도, 미설정 표식, 완전한 물리 헤더는 비공개 형식 인덱스에 남으며 표시 정밀도가 이를 덮어쓰지 않습니다. 변환할 수 없는 시간에는 RAR의 안전 규칙에 따라 0을 표시합니다. 참조 구현이 운영체제 실패를 확인하지 않으면 정의되지 않은 데이터를 복사할 수 있습니다.

<a name="xdzip-controlled-encoding-mappings"></a>

## 제한된 인코딩 변환

| 서비스 | 입력과 출력 | 호스트 범위 |
| --- | --- | --- |
| `oem_to_ansi_wide` | NUL을 포함한 UTF-16 512단위에서 UTF-16 512단위 | Windows OEM과 ANSI를 거치는 전체 변환, 실제 매핑과 치환 동작 유지 |
| `wide_to_ansi` | NUL을 포함한 UTF-16 최대 512단위에서 512바이트 | 현재 Windows ANSI 코드 페이지. 공통 치환 문자를 가정하지 않음 |
| `wide_to_local_prefix` | NUL을 포함한 와이드 스칼라 최대 128개에서 512바이트 | 현재 Unix C 인코딩. 실패 시 고정 호환성 규칙에 따라 지우고 다시 시도하며 이미 변환한 접두부는 보존 |

반환하는 길이에 NUL은 포함되지 않습니다. 호스트 서비스가 없는 기본 구현은 `None`을 반환합니다. 형식 계층 호출자는 정의된 이식 가능한 대체 동작을 사용하거나 명시적으로 거부할 수 있습니다. 코드 페이지를 추측해서는 안 됩니다. 제품 콜백은 크기, NUL과 반환 범위를 검증합니다. 사용 불가능한 매핑에는 일관되게 Unsupported를 반환합니다.

Windows의 [CharToOemBuffW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-chartooembuffw) 길이 매개변수는 와이드 입력 문자 개수이며 NUL 뒤 패딩도 계속 변환합니다. 바이트 단위의 대상 용량을 나타내지 않습니다. 호스트는 먼저 `GetCPInfo(CP_OEMCP)`로 단위당 최대 4바이트인 코드 페이지를 허용한 뒤 임시 2,048바이트 버퍼로 512단위 전부를 변환합니다. 그 후에만 511번 바이트에 NUL을 쓰고 ANSI로 변환합니다. 필요한 용량을 확인할 수 없으면 변환을 거부합니다. 와이드 문자 512개를 512바이트 대상에 받거나 용량 요구를 피하려고 입력을 먼저 잘라서는 안 됩니다. 실제 운영체제 검증에는 DBCS 511단위 경계 전체와 WinAPI 직접 호출의 독립 비교가 포함됩니다.

512단위 허용, 첫 NUL, OEM 보정 조건, 127단위 및 바이트 단위 절단, 이전 표식 프로토콜, macOS의 명시적 UTF8은 Rust 형식 호환성 규칙입니다. 원래 운영체제 변환이 먼저라면 그 전에 암호를 잘라서는 안 됩니다. DBCS 시퀀스 내부에서의 절단을 임의로 문자 경계 절단으로 바꾸어서도 안 됩니다.

<a name="xdzip-ownership-and-failure"></a>

## 소유권과 실패

슬라이스는 동기 호출 중에만 빌립니다. C++는 포인터를 보관하지 않습니다. 비밀 입력 및 출력의 삭제 시점은 Rust 호출자가 관리합니다. 크기가 제한된 C++ 와이드·내로 임시 버퍼는 모든 반환 경로에서 지웁니다. 운영체제 내부나 이전 메모리 복사본 모두의 삭제를 보장하지는 않습니다. 기본 연산은 암호를 기록하거나 프로세스 인수 또는 환경 변수에 넣지 않습니다.

읽기 작업은 서비스를 부르기 전에 취소 여부를 확인합니다. 서비스 결과는 압축 풀기 출력으로 계산하지 않으며 읽거나 디코딩하거나 기록한 바이트 수를 바꾸지 않습니다. 실패는 기존 소스, 취소, 할당량 분류를 그대로 유지합니다. 암호 오류로 잘못 분류하면 안 됩니다. 실제 검증값 상태에 따라 암호 오류와 무결성 실패를 구분할 수 없는지는 형식 계층만 판정합니다.
