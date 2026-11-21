<a name="xdzip-repository-layout-and-dependencies"></a>

# 저장소 구조와 종속성

[English](../../engineering/repository-layout.md) | [简体中文](../../zh-CN/engineering/repository-layout.md) | [繁體中文](../../zh-TW/engineering/repository-layout.md) | [日本語](../../ja-JP/engineering/repository-layout.md) | 한국어 | [Français](../../fr-FR/engineering/repository-layout.md) | [Deutsch](../../de-DE/engineering/repository-layout.md) | [Español](../../es-ES/engineering/repository-layout.md) | [Português (Brasil)](../../pt-BR/engineering/repository-layout.md) | [Русский](../../ru-RU/engineering/repository-layout.md)

루트의 Cargo.toml과 CMakeLists.txt가 현재 제품 모듈을 정의합니다. 표는 실제 디렉터리를 설명합니다. 아키텍처에서 예정한 분리는 아직 독립 크레이트로 구현되지 않았습니다.

| 디렉터리 | 책임과 종속성 |
| --- | --- |
| apps/desktop | Qt/QML, 얇은 C++ 컨트롤러, 모델, 설정 및 시스템 연동. 브리지를 통해 코어에 액세스합니다. |
| crates/xdzip-cli | Qt 없이 인수, 터미널, JSON, 관리형 설치 및 업데이트 호스트를 처리합니다. 런타임, 안전한 I/O, 업데이트 코어에 직접 의존하고 형식 해석은 위임합니다. |
| crates/xdzip-domain | 순수 타입과 불변 조건. Qt나 구체적인 코덱에 의존할 수 없습니다. |
| crates/xdzip-codec-api | 기능, 빌린 입력, 인코딩 출력 및 취소 계약. 도메인에 의존합니다. |
| crates/xdzip-compression | Deflate/Deflate64/GZIP, LZMA/LZMA2, BZip2, PPMd7, MSZIP, LZX, Quantum의 공통 알고리즘과 래퍼. 운영용 타사 종속성이나 경로 및 출력 확정 권한이 없습니다. [계약](independent-rust-codecs.md)을 참고하세요. |
| crates/xdzip-crypto | 형식과 독립적인 고정 메모리 SHA/HMAC 및 BLAKE 해시. 민감한 상태를 0으로 지울 수 있으며 이미 채택한 원시 기능을 재사용합니다. 압축 파일 암호를 해석하거나 출력을 확정할지 결정하지 않습니다. |
| crates/xdzip-codec-zip, xdzip-codec-tar | ZIP 및 TAR/GZIP 구현. 임의의 출력 경로를 다루지 않습니다. |
| crates/xdzip-codec-lha, xdzip-codec-arj, xdzip-codec-arc, xdzip-codec-bh | [LHA](lha-containers.md)/[PMA](pma-streams.md), [ARJ](arj-streams.md), [ARC](arc-streams.md), [BH](bh-containers.md)의 독립 프레임 및 항목 프로필. 파일 시스템 권한이 없으며 제품 내 사용 가능 여부는 형식 레지스트리에서 결정합니다. |
| crates/xdzip-codec-7z | 자체 7z 컨테이너, 방식 그래프, 필터, AES 결합, 스트리밍 생성과 한도가 있는 다시 쓰기. 파일 경로에 대한 권한은 없습니다. |
| crates/xdzip-codec-cab | 호스트가 승인한 볼륨과 공통 알고리즘을 사용하는 자체 읽기 전용 CAB 컨테이너. 원본 이름, UTF-8 선언 및 일반적인 중복 이름 레코드를 보존합니다. 볼륨 검색과 출력 확정은 호스트의 책임입니다. [계약](independent-rust-codecs.md#cab只读规格) 및 [자료](cab-format-sources.md)를 참고하세요. |
| crates/xdzip-codec-optical | ISO9660/Joliet/Rock Ridge/UDF 읽기와 새 데이터 이미지를 위한 ISO9660/Joliet 쓰기. 경로 권한 없이 스트림을 사용합니다. |
| crates/xdzip-codec-bridge | 격리 프로세스와 Rust 7z, RAR, CAB 및 광학 매체 코덱을 잇는 얇은 CXX 경계. 승인된 스트림, 리소스 제한 및 취소를 다룹니다. |
| crates/xdzip-codec-native, tools/native-codec | Rust 7z/RAR/광학 매체/CAB 백엔드 격리 프로세스. 얇은 C++ 계층은 프로토콜과 시스템 달력 및 인코딩 서비스만 제공합니다. 네이티브 수신자는 미리 집계한 출력과 보조 작업량을 독립적으로 검사합니다. [I/O 계약](native-codecs.md)을 참고하세요. |
| crates/xdzip-extract-plan | 메타데이터 사전 검사와 한도가 있는 보고서. 대상 승인이나 쓰기 권한이 없습니다. |
| crates/xdzip-io | 입력 트리, 네이티브 승인 및 식별, 안전한 임시 준비와 파일 확정. |
| crates/xdzip-runtime | 세션, 한도가 있는 메모리 및 디스크 인덱스, 쿼리, 선택, 한도가 있는 미리 보기, 만들기 및 압축 풀기 작업과 예약. |
| crates/xdzip-qt-bridge | cxx 타입 브리지, 두 핸들과 수명 주기. 런타임에 의존합니다. |
| crates/xdzip-plugin-protocol | 표준 라이브러리만 사용하며 한도가 있는 CBOR 및 프레임 처리. 경로, 프로세스 또는 Qt에 대한 권한이 없습니다. |
| crates/xdzip-plugin-host, examples/sdk | 독립 읽기 전용 SDK 실험, 제어된 I/O, 네이티브 샌드박스 요건 및 독립 Python 통신 상대. 제품의 형식 검색과 실행 종속성에 포함되지 않습니다. |
| tools | 브리지, 형식, I/O, 프로세스 수를 위한 독립 검사 도구. GUI 제품에는 별도의 진입점이 있습니다. |
| scripts, .github | 생성, 검사, 배포 도구, CI 및 협업 템플릿. |
| release-notes | 검토할 수 있는 버전별 릴리스 노트와 버전 및 빌드 메타데이터. 실행 보고서와 생성한 패키지는 별도로 보관합니다. |
| design/hifi, design/assets | 고해상도 설계 이미지와 런타임 리소스 입력. |
| docs | 장기 관리하는 Markdown 설계, 기술 계약, 재사용할 수 있는 검증 방법 및 필요한 이미지. 실행 보고서는 별도로 보관합니다. |
| specs | 기계가 읽을 수 있는 시각 사양, 용어 및 종속성 목록. 제품 리소스, 생성기 및 검사에서 사용합니다. |
| third_party | 코드와 함께 보관하는 타사 소스 및 라이선스 자료. |

독립 xdzip-index, xdzip-platform 및 완전한 외부 형식 적합성 도구는 향후 아키텍처에 포함됩니다. 현재 인덱싱은 런타임에 있고, 플랫폼별 파일 작업은 io와 데스크톱 어댑터에 있습니다. 기존 plugin-host는 [읽기 전용 SDK 실험](sdk-trial.md)에 사용합니다. 테스트는 주로 Rust 모듈, apps/desktop/tests 및 스크립트 검사에서 관리합니다. 구현이 없는 빈 모듈은 만들지 않습니다.

제품의 CAB 경로는 Rust로 고정됩니다. 공식 Cab 처리기와 CAB 전용 LZX/Quantum은 컴파일에 포함되지 않습니다. 호출은 제품 실행 파일이나 테스트 스위치, 실패 시 대체 경로를 추가하지 않고 동일한 격리 프로세스를 사용합니다. codec-api는 완전한 프레임의 작업량을 미리 집계하는 방식을 정의합니다. 컨테이너는 프레임별 허용량을 보유하며 네이티브 수신자와 런타임은 각각의 허용량과 출력 제한을 독립적으로 확인합니다. [자체 코덱 계약](independent-rust-codecs.md#cab只读规格)을 참고하세요.

종속성 설계는 [시스템 아키텍처](../design/system-architecture.md)를 참고하세요. 도메인에서 런타임으로의 역방향 종속성, 코덱의 직접적인 QML 알림, 플랫폼 계층의 UI 문구 결정을 피하세요. 생성한 바인딩, 샘플, 보고서 및 빌드 결과는 소스 디렉터리 밖의 build/, target/, dist/에 둡니다. [설치 및 연결](installers-associations.md#兼容基线的长期输入)에는 이전 버전과의 호환성을 확인하기 위해 유지할 입력이 나와 있습니다.

`tools/image-preview/`는 정적 이미지용 내부 QtGui/QtSvg 프로세스입니다. 검증되고 한도가 적용된 입력을 받아 한도가 적용된 RGBA를 반환하며 압축 파일이나 대상의 경로는 보유하지 않습니다. 플랫폼 지원과 실패 시 동작은 [미리 보기 계약](content-preview.md)을 참고하세요.
