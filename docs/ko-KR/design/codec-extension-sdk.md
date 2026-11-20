<a name="xdzip-format-adapters-and-the-extension-trial"></a>

# 형식 어댑터와 확장 시험

[English](../../design/codec-extension-sdk.md) | [简体中文](../../zh-CN/design/codec-extension-sdk.md) | [繁體中文](../../zh-TW/design/codec-extension-sdk.md) | [日本語](../../ja-JP/design/codec-extension-sdk.md) | 한국어 | [Français](../../fr-FR/design/codec-extension-sdk.md) | [Deutsch](../../de-DE/design/codec-extension-sdk.md) | [Español](../../es-ES/design/codec-extension-sdk.md) | [Português (Brasil)](../../pt-BR/design/codec-extension-sdk.md) | [Русский](../../ru-RU/design/codec-extension-sdk.md)

<a name="xdzip-three-execution-paths"></a>

## 세 가지 실행 경로

| 경로 | 구현과 사용 가능성 |
| --- | --- |
| 호스트의 내장 어댑터 | 데스크톱과 CLI는 같은 고정 형식 레지스트리와 런타임 디스패치를 사용합니다. ZIP, TAR, 스트림 필터와 DEB, ASAR 같은 지원 컨테이너의 Rust 구현을 호스트 프로세스에서 호출합니다. |
| 함께 제공되는 아카이브 작업자 | 호스트의 `xdzip-codec-native`가 `xdzip-native-codec`를 시작합니다. C++ 프로토콜 계층은 `xdzip-codec-bridge`를 통해 Rust 7z, RAR, CAB와 광학 미디어 백엔드를 호출합니다. 적용 가능한 7z 생성과 다시 쓰기도 같은 경로를 사용합니다. ISO 생성은 프로세스 내부의 광학 기록기를 사용합니다. |
| 외부 SDK 시험 | `xdzip-plugin-host`와 `xdzip-plugin-protocol`은 독립적이고 명시적인 XDP1 실험을 구성합니다. 데스크톱과 CLI 제품 빌드는 타사 확장을 찾거나 설치하거나 실행하지 않습니다. |

처음 두 경로는 제품 내장 구성 요소입니다. 제공되는 작업자는 전용 프로토콜과 리소스 감독을 가진 별도 프로세스입니다. 외부 시험에는 자체 패키지 검증, 버전 협상과 플랫폼 권한 조건이 있습니다. [시스템 아키텍처](../../design/system-architecture.md), [아카이브 작업자 계약](../engineering/native-codecs.md), [SDK 시험 계약](../engineering/sdk-trial.md)

<a name="xdzip-product-routing-and-operation-capabilities"></a>

## 제품 라우팅과 작업 능력

[형식 레지스트리 소스](https://github.com/xiaodaozhi/xdzip/blob/main/specs/engineering/archive-formats.json)는 확장자, 컨테이너 및 필터 유형, 지원 작업, 방법과 생성 옵션을 선언합니다. 생성기는 Rust 레지스트리, 데스크톱 목록과 디자인 산출물을 동기화합니다. 읽을 때 런타임은 원본 바이트를 확인하고 형식을 검증한 후 백엔드를 선택합니다. 생성할 때 요청한 출력 이름이 생성 프로필을 선택하고 런타임이 옵션을 검사합니다. 확장자 목록은 검색과 UI 안내를 돕습니다. 감지와 검색이 성공하면 열린 형식이 확정됩니다.

읽기, 미리 보기, 압축 풀기, 전체 아카이브 검사, 생성과 다시 쓰기에는 별도 제약이 있습니다. 목록의 항목이 보이더라도 지원되는 데이터 방식이 없을 수 있습니다. 암호, 볼륨과 특수 레코드는 허용에 영향을 주며 런타임은 ISO/UDF 구조 검사와 미지원 다시 쓰기 프로필 등 형식별 차이도 처리합니다. [런타임 라우팅](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/formats.rs), [형식 레지스트리 계약](../engineering/format-registry.md), [아카이브 검사](../engineering/archive-checking.md)

<a name="xdzip-internal-rust-adapter-interfaces"></a>

## 내부 Rust 어댑터 인터페이스

| 인터페이스 | 책임 |
| --- | --- |
| `ReadSource`와 `MetadataCodec` | 해당 볼륨 메타데이터를 포함해 호스트가 승인한 원본을 읽거나 위치를 이동하고 제한된 형식 레코드를 생성합니다. |
| `PayloadCodec`, `PayloadReader`, `DecodedSink` | 내용을 해석하고 적격 물리 레코드를 호스트 출력 대상으로 디코딩합니다. 백엔드가 보고하는 경우 일반 바이트, 건너뛴 디코딩 작업과 보조 작업을 구분합니다. |
| `CreationCodec`, `EncodeInput`, `EncodeOutput` | 고정된 입력 메타데이터와 인덱싱된 스트림에서 인코딩합니다. 호스트 소유 출력 대상에 기록하며 기록기가 필요로 하면 제한된 임의 쓰기를 수행합니다. |
| `RewriteCodec` | 보존된 원래 항목, 변경 사항과 승인된 추가 항목으로 지원 아카이브를 새 임시 아카이브에 다시 씁니다. |

경로, 원본 재검증, 취소, 암호, 할당량과 모든 최종 게시는 호스트가 소유합니다. 코덱은 작업에 필요한 인터페이스를 통해 권한을 받습니다. 아카이브 작업자는 허용된 원본 구간을 요청하고 제한된 레코드 또는 출력 블록을 전달합니다. 호스트는 동작 전에 프로토콜 상태와 바이트 할당량을 검증합니다. 작업자 오류나 잘못된 응답은 게시를 승인할 수 없습니다. 작업자 경계는 장애와 리소스를 격리합니다. 임의의 외부 코드에 대한 커널 권한 제한은 시험의 별도 요구 사항입니다. [메타데이터 계약](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/lib.rs), [데이터 계약](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/payload.rs), [인코딩과 다시 쓰기 계약](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/encode.rs), [파일 권한](file-security.md)

이 Rust trait는 내부 소스 인터페이스입니다. 제품 어댑터는 작업 공간과 명시적 런타임 라우팅에 통합됩니다. 허용된 각 작업은 유효한 파일, 미지원 기능, 리소스 제한, 손상과 출력 동작에 대한 별도 테스트가 필요합니다. [독립 Rust 코덱 계약](../../engineering/independent-rust-codecs.md)과 [개발 가이드](../contributing/README.md)를 사용하세요.

<a name="xdzip-implemented-external-sdk-trial"></a>

## 구현된 외부 SDK 시험

시험은 명시적으로 선택한 네이티브 패키지 매니페스트와 호스트가 선택한 원본을 받습니다. 프로젝트 소유 XDP1 시험 자료를 검색하고 일반 항목 하나를 읽을 수 있습니다. 패키지 허용 시 버전 1 매니페스트, 실행 대상과 해시, 패키지 내부 경로, 고정 권한 프로필과 제한을 검증합니다. 호스트는 검증된 비공개 사본을 실행합니다. 선언된 해시는 패키지 일관성을 입증하며 게시자 인증에는 별도 신뢰 정책이 필요합니다.

버전 1 시험 프로토콜은 길이 접두사가 있는 제한된 CBOR을 사용하고 작업 식별자와 순서를 검사합니다. 호스트는 불투명 토큰으로 원본 읽기와 출력 대상 하나를 허용합니다. 상대는 바이트 구간과 한 블록의 출력 크레딧을 받습니다. 호스트는 내용을 계량하고 검증하며 프로토콜 완료와 원본 상태를 확인하고 상대가 정상 종료한 후에만 덮어쓰지 않고 게시합니다. 샘플은 아카이브 암호, 분할 볼륨, 생성, 업데이트, 링크 또는 호스트 경로를 상대에게 노출하지 않습니다.

시험에는 macOS와 Linux의 명시적인 네이티브 권한 조건이 있으며 플랫폼 요구 사항과 실패 시 거부 동작은 시험 계약에 정의되어 있습니다. Windows는 실행을 거부합니다. 네이티브 상호 운용 및 거부 테스트는 데스크톱과 독립적으로 이 경계를 검증합니다. 실제 스키마, 예산과 테스트 진입점은 [시험 실행 및 통신 계약](../engineering/sdk-trial.md)을 참고하세요. [시험 호스트](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-plugin-host/src/lib.rs), [프레이밍 구현](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-plugin-protocol/src/lib.rs)

<a name="xdzip-design-options-outside-the-current-implementation"></a>

## 현재 구현 범위 밖의 설계 선택지

제품용 확장 관리자, 게시자 신뢰와 업그레이드 정책, 시험 프로토콜을 넘는 호환성, 외부 쓰기 또는 암호 기능은 아직 설계 작업입니다. 제품에 도입하려면 지원하는 각 플랫폼과 기능에 대해 권한 제한, 상호 운용성과 장애 동작을 검증해야 합니다. 제공되는 작업자와 XDP1 실험은 이 기능을 활성화하지 않습니다. [아키텍처 결정](architecture-decisions.md)
