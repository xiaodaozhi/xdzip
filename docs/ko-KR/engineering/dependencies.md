# 의존성과 형식 백엔드 채택

[English](../../engineering/dependencies.md) | [简体中文](../../zh-CN/engineering/dependencies.md) | [繁體中文](../../zh-TW/engineering/dependencies.md) | [日本語](../../ja-JP/engineering/dependencies.md) | 한국어 | [Français](../../fr-FR/engineering/dependencies.md) | [Deutsch](../../de-DE/engineering/dependencies.md) | [Español](../../es-ES/engineering/dependencies.md) | [Português (Brasil)](../../pt-BR/engineering/dependencies.md) | [Русский](../../ru-RU/engineering/dependencies.md)

## 외부 SDK 시험

독립 시험 호스트는 고정된 serde 1.0.229, serde_json 1.0.154, sha2 0.10.9, getrandom 0.4.2, libc 0.2.190, tempfile 3.27.0을 재사용하며 타사 버전이나 형식/CBOR 라이브러리를 추가하지 않습니다. 프로토콜 크레이트는 std만 사용하고 프로젝트 소유의 제한된 CBOR은 RFC 8949에 따라 독립적인 Python 표준 라이브러리 상대와 상호 운용됩니다. 호스트는 Qt를 링크하거나 데스크톱 또는 CLI 실행 의존 집합에 들어가지 않으며 공식 릴리스 자료를 변경하지 않습니다.

tempfile은 범용 임시 디렉터리 구현을 유지하는 대신 검증된 비공개 실행 파일 사본을 보존하기 위해 기존 잠금 파일에서 시험의 직접 의존성으로 승격합니다. 안전한 출력에는 계속 xdzip-io를 사용하며 tempfile은 커밋을 인가하지 않습니다. 소스는 [Stebalien/tempfile](https://github.com/Stebalien/tempfile), 라이선스는 MIT 또는 Apache-2.0입니다. 기본 기능과 fastrand, getrandom, once_cell, rustix/windows-sys를 포함한 실제 대상 의존 집합은 Cargo.lock과 메타데이터에 따라 고정 버전을 재사용하며 빌드 스크립트를 다운로드하거나 C 라이브러리를 추가하지 않습니다. 기존 기반 라이브러리의 사용, 소스와 라이선스도 계속 적용되며 과거 취약점 공지 조회를 새로운 보안 결론으로 주장하지 않습니다. 테스트와 조건은 [SDK 시험](sdk-trial.md)을 참조하세요. Windows에서는 AppContainer 구현이 없으면 실행을 명시적으로 거부합니다.

패키지 구성원 접근은 채택된 cap-std/cap-fs-ext 4.0.3의 정확한 버전과 기존 기능을 재사용하고, 보유한 디렉터리를 기준으로 각 구성 요소를 링크를 따르지 않고 엽니다. 문자열 검사와 환경 경로를 통한 재열기 사이의 경합을 방지하기 위함입니다. 두 라이브러리는 기존 보안 I/O 출처, MIT 또는 Apache-2.0 라이선스, 전이 의존성 버전과 네이티브 동작을 유지하며 타사 버전을 추가하지 않습니다.

## CLI 프런트엔드와 HTTPS

CLI는 잠금된 clap 4.6.7을 실제 제품 프런트엔드 의존성으로 승격하고 std, help, usage, error-context, suggestions, string만 활성화하며 derive와 터미널 색상은 사용하지 않습니다. ureq 3.5.1은 기본 기능을 끄고 추가해 native-tls-no-default와 socks-proxy만 활성화하고 NativeTls와 시스템 신뢰 루트를 명시적으로 선택합니다. 기본 GZIP 디코딩, 쿠키, JSON 도우미, 외부 curl 또는 Qt 네트워크 실행 의존성을 도입하지 않습니다. serde, serde_json, zeroize, sha2, getrandom과 플랫폼 바인딩은 고정 버전을 재사용합니다.

ureq와 ureq-proto는 [algesten/ureq](https://github.com/algesten/ureq)에서 가져오며 MIT 라이선스를 선택합니다. 주요 전이 입력은 native-tls 0.2.18, ureq-proto 0.7.1, http 1.5.0, httparse 1.10.1입니다. 전체 버전, 기능과 대상 의존 집합은 Cargo.lock과 호스트로 필터링한 Cargo 메타데이터를 따릅니다. Windows는 schannel 0.1.30, macOS는 security-framework 3.7.0과 시스템 프레임워크, Linux는 openssl 0.10.82/openssl-sys 0.9.118과 시스템 OpenSSL 3을 사용합니다. RustCrypto der/PKI 유형은 일반 TLS 래핑만 지원하며 압축 파일 형식을 해석하지 않습니다.

Linux Native TLS 빌드는 pkg-config로 시스템 OpenSSL을 찾으며 전용 OpenSSL을 포함, 다운로드 또는 컴파일하지 않습니다. Mac과 Windows는 시스템 TLS를 사용합니다. 필요한 Cargo 빌드 스크립트, 네이티브 링크, 대상 조건과 알려진 공지를 실제 고정 버전에 대해 확인합니다. 조회 기록은 로컬/CI 아티팩트에 두고 요약은 CLI issue에 기록합니다. 일회성 무결과 조회는 영구적인 안전 보장이 아닙니다. 프로젝트 소유 CLI의 터미널, 신호와 OS 조회 FFI는 플랫폼 어댑터에 한정하며 실패하면 작업을 거부하고 암호 컨테이너의 영점화 요구 사항을 유지합니다.

패키지 공지는 CLI와 작업 프로세스의 일반 의존 집합에서 각각 수집하며 빌드/개발/proc-macro 의존성은 실행 시 출처 증거에서 제외합니다. 데스크톱 실행 시 출처에는 CLI 루트를 추가합니다. 공개 Windows 실행기만 컴파일러 CRT를 정적 링크해 버전이 있는 앱 로컬 DLL을 로드하기 전에 진입점이 시작되도록 합니다. 실제 페이로드와 작업 프로세스에는 고정된 VC++ 런타임과 원본 라이선스를 계속 포함하며 SDK 그래픽 라이브러리는 포함하지 않습니다. 실제 릴리스 자료에서는 의존성 출처, 보존되는 정적/동적 기계 코드와 라이선스 의무를 계속 검토합니다.

의존성을 처음 추가할 때는 목적, 대안, 버전, 소스, 라이선스, 전이 의존성, 기능, 네이티브 빌드 동작, 알려진 취약점과 유지 관리를 기록합니다. 크레이트 이름만으로 채택이 완료되지 않으며 가상의 감사 버전을 나열하지 않습니다.

| 후보 방향 | 필요한 검증 |
| --- | --- |
| Rust ZIP 구현 | Store/Deflate, ZIP64, CRC, 중복, 원시 이름 바이트, 취소, 호스트 출력 대상 |
| Rust TAR + GZIP 구현 | 스트리밍 인덱스, PAX/GNU 메타데이터, 링크/장치 거부, 알 수 없는 크기, 손상된 스트림 |
| cxx | 소유권, 생성기 일관성, 플랫폼 간 정적 코어 링크, 경계를 넘지 않는 예외 |
| Qt 모듈 | 실제 실행 및 전이 의존성, 교체 가능한 배포, 라이선스, 보안 업데이트 채널 |
| 아이콘 세트 | 명시적인 재배포 라이선스, 일관된 선 두께, 접근성, 에셋 목록 |

검증 순서는 최소 상호 운용 → 악성/잘린 입력 → 취소/리소스 제한 → 라이선스/SBOM → 깨끗한 빌드 → 결정 기록입니다. 라이브러리의 범용 압축 해제로 호스트 경로 보호를 우회하지 않습니다.

애플리케이션 Rust 워크스페이스는 Cargo.lock을 커밋합니다. 새 네이티브 라이브러리와 네트워크에서 다운로드하는 빌드 스크립트는 별도 검토가 필요하며 사용하지 않는 기능은 비활성화합니다. 의존성이 방치되거나 취약점 수정이 없으면 실제 도달 가능성에 따라 업그레이드, 교체 또는 기능 비활성화를 결정하고 이유를 기록합니다.

## 업데이트 메타데이터와 게시자 서명 검증

`xdzip-update`는 매니페스트, 채널과 서명 검증을 위한 Qt 독립 코어이며 제품은 별도 cxx 경계로 호출합니다. 릴리스 도구와 설치 도우미도 같은 규칙을 사용합니다. `ed25519-dalek = 2.2.0`의 안전한 `VerifyingKey::verify_strict`를 사용해 약한 공개 키를 거부하고 정확한 원시 바이트를 검증합니다. 서명 연산을 직접 구현하거나 TLS 또는 SHA-256을 게시자 서명으로 취급하지 않습니다. 업스트림 fast, std, zeroize는 기본 활성화되며 batch, hazmat, legacy_compatibility, rand_core, PKCS#8은 비활성화됩니다. 릴리스 도구는 소유자가 설정한 32바이트 시드만 가져오며 프로덕션 개인 키를 생성하지 않습니다.

| 직접 또는 주요 전이 의존성 | 정확한 버전 | 소스, 라이선스와 목적 |
| --- | --- | --- |
| ed25519-dalek, curve25519-dalek | 2.2.0, 4.1.3 | [dalek-cryptography](https://github.com/dalek-cryptography/curve25519-dalek), BSD-3-Clause, Ed25519와 곡선 연산 |
| base64ct | 1.8.3, alloc | [RustCrypto/formats](https://github.com/RustCrypto/formats), Apache-2.0 또는 MIT, Sparkle 호환 서명 인코딩 |
| sha2 | 0.10.9 | [RustCrypto/hashes](https://github.com/RustCrypto/hashes), MIT 또는 Apache-2.0, 패키지 다이제스트 |
| semver | 1.0.28 | [dtolnay/semver](https://github.com/dtolnay/semver), MIT 또는 Apache-2.0, 제한된 제품 버전 분석 |
| subtle | 2.6.1 | [dalek-cryptography/subtle](https://github.com/dalek-cryptography/subtle), BSD-3-Clause, 업스트림 상수 시간 연산 지원 |
| zeroize | 1.9.1 | [RustCrypto/utils](https://github.com/RustCrypto/utils), Apache-2.0 또는 MIT, 릴리스 과정의 임시 개인 키 컨테이너 영점화 |

serde 1.0.229와 serde_json 1.0.154는 잠금 버전을 재사용합니다. digest, block-buffer, crypto-common, generic-array, typenum, cpufeatures, derive 매크로를 포함한 다른 전이 의존성은 Cargo.lock과 실제 기능 트리를 따릅니다. curve25519-dalek 빌드 스크립트는 로컬 컴파일러 능력만 감지하며 실행 시 업스트림 CPU별 구현을 선택할 수 있습니다. 외부 소스를 다운로드하거나 C/C++ 암호화 라이브러리 또는 OpenSSL 실행 의존성을 추가하지 않습니다. 기존 공지와 벤더 수집은 Cargo 메타데이터로 이 패키지를 포함하며 실제 패키지 출처와 라이선스는 릴리스 자료 검토가 필요합니다.

시스템 암호화 API를 사용하면 세 플랫폼의 검증 규칙이 갈라지고 외부 OpenSSL CLI는 실행 의존성과 인수 경계를 넓히므로 하나의 Rust 코어를 사용합니다. 과거 [RUSTSEC-2022-0093](https://rustsec.org/advisories/RUSTSEC-2022-0093.html)의 구 Keypair API는 사용하지 않습니다. [RUSTSEC-2024-0344](https://rustsec.org/advisories/RUSTSEC-2024-0344.html)가 요구하는 수정된 curve25519-dalek 경계는 4.1.3이며 고정 버전은 이를 충족합니다. 이 검사로 모든 의존성에 취약점이 없음을 증명하지는 않으며 업그레이드 시 유지 관리와 공지를 계속 검토합니다.

회귀 검사는 정확한 바이트 변경, 잘못된 서명, 알 수 없거나 중복되거나 약한 공개 키, 빈 프로덕션 신뢰 집합, 채널 폴백, 버전/빌드 결합, 여섯 패키지의 완전성과 서명된 피드를 검증합니다. 임시 테스트 키는 테스트 코드와 격리된 아티팩트에만 존재하며 제품 신뢰 리소스에는 넣지 않습니다. 네이티브 설치와 플랫폼 간 결과는 별도로 기록하며 한 플랫폼의 코어 테스트는 설치 승인을 대체하지 않습니다.

## Windows 소스 신원

M2는 Windows 런타임에서 정확한 `winapi-util = 0.1.11`의 안전한 `file::information(&File)` 래퍼를 직접 사용해 볼륨 일련번호와 파일 인덱스를 비교합니다. 길이와 수정/생성 시간만으로는 같은 이름의 NTFS 교체를 안정적으로 감지할 수 없습니다. 표준 라이브러리의 해당 파일 인덱스 API는 아직 안정적이지 않고 직접 작성한 Win32 unsafe는 검토 범위를 넓힙니다. 이미 잠금된 라이브러리를 재사용하며 패키지 버전을 추가하지 않습니다.

소스는 crates.io/[BurntSushi/winapi-util](https://github.com/BurntSushi/winapi-util)이며 업스트림의 MIT 또는 Unlicense 이중 라이선스에서 MIT를 선택합니다. Windows 전이 의존성은 고정된 windows-sys 0.61.2와 windows-link 0.2.1로 둘 다 MIT 또는 Apache-2.0입니다. 업스트림이 선언한 Win32 모듈만 활성화하고 기본 기능을 끄며 빌드 스크립트, 외부 네이티브 빌드 또는 네트워크 다운로드는 없습니다. Windows 이외의 대상은 이 실행 의존성을 컴파일하지 않습니다. 업스트림 안전 래퍼는 유효한 File 핸들을 빌리며 시스템 호출이 실패하면 이전 소스가 무효가 됩니다. 핸들은 FFI를 넘지 않으며 신원 검사는 안전한 쓰기 인가가 아닙니다.

버전 출처, 기능과 네이티브 컴파일 동작을 확인했고 Windows GNU 대상 형식 검사가 통과했습니다. 네이티브 Windows 회귀는 수정 시간을 유지한 같은 이름/같은 크기의 교체를 다뤄야 합니다. 업스트림은 작은 안전 Win32 래퍼이며 업그레이드는 잠금 파일과 네이티브 회귀에 의해 제한됩니다. 2026-10-13의 세 패키지 정확한 버전에 대한 OSV 조회에서는 알려진 공지가 없었습니다. 과거 응답은 원래 실행 아티팩트에 두고 새 조회는 [검증](../../../developers/qa/README.md)에 따라 실제 응답을 기록합니다. 이는 취약점 부재의 증거가 아닙니다. 개발 패키지는 계속 모든 업스트림 라이선스를 수집합니다.

## Rust 형식 백엔드와 공통 작업 프로세스 브리지

`xdzip-codec-7z`는 프로젝트 API/도메인, 타사 제품 의존성이 없는 공통 압축 알고리즘과 범용 암호화 기본 요소를 사용합니다. AES 0.9.3은 zeroize를 활성화하며 getrandom 0.4.2와 zeroize 1.9.1은 ZIP 버전을 재사용합니다. 영점화 가능한 SHA-256은 형식에 독립적인 `xdzip-crypto`로 분리해 형식마다 래퍼를 중복하지 않고 채택된 sha2 0.10.9 compress 기본 요소 위에서 프로젝트 소유의 증분 상태를 관리합니다. 공통 크레이트는 공개 수학 명세에 따라 HMAC-SHA256과 BLAKE2s/BLAKE2sp도 구현합니다. 소스와 경계는 해당 [README](../../../crates/xdzip-crypto/README.md)를 참조하세요. 압축 파일이나 공개 권한을 부여하지 않으며 타사 암호화 버전을 업그레이드하지 않습니다. 형식 속성, CBC 구성, KDF, 생성과 다시 쓰기는 독립 구현이며 기반 라이브러리가 제공하는 압축 파일 처리가 아닙니다. 소스, 분석 역할과 라이선스 차이는 [7z 근거](sevenzip-format-sources.md)를 참조하세요.

`xdzip-codec-optical` 형식 로직은 표준 라이브러리와 프로젝트 codec API/도메인만 사용하며 ISO/UDF 또는 범용 압축 파일 라이브러리를 사용하지 않습니다. 명세, 범위와 오류는 [독립 구현 계약](independent-rust-codecs.md#iso与udf只读规格)을 참조하세요. `xdzip-codec-bridge`는 잠금된 cxx/cxx-build 버전과 생성을 재사용하며 새 외부 버전, 다운로드하는 빌드 스크립트 또는 형식 알고리즘 라이브러리를 추가하지 않습니다. 프로세스 내부 라우팅으로 교체하면 기존 광학 작업 프로세스 관리가 사라지므로 작업 프로세스와 제어된 스트림 프로토콜을 유지합니다.

`xdzip-codec-cab`은 프로젝트 API, 도메인과 타사 제품 의존성이 없는 `xdzip-compression`에만 의존합니다. MSZIP은 ZIP/GZIP Deflate를 재사용하며 LZX와 Quantum은 추가 Deflate나 범용 압축 파일 라이브러리가 없는 독립 알고리즘 모듈입니다. 7z, CAB와 광학 백엔드는 `xdzip-codec-bridge`와 호스트가 인가한 스트림을 공유합니다. 생성은 범용 경계를 중복하지 않고 제어 입력과 무작위 임시 배치 콜백만 추가합니다. 제품은 Rust를 사용하며 공식 Cab 및 CAB 전용 LZX/Quantum은 빌드에 들어가지 않습니다. 리소스와 형식은 [자체 구현 계약](independent-rust-codecs.md#cab只读规格), 참조 분석 범위와 라이선스는 [CAB 소스](cab-format-sources.md)를 참조하세요.

`xdzip-codec-rar`는 RAR이나 다른 압축 파일 라이브러리를 도입하지 않고 같은 codec API, 공통 압축 알고리즘과 암호화 기본 요소를 재사용합니다. PPMd H 통계 모델은 7z와 RAR이 공유하며 각 형식이 자체 범위, 재설정과 종료를 정의합니다. 특수한 기존 암호 호환 조합은 RAR 모듈에 유지합니다. 시스템 달력과 OEM/ANSI/로컬 인코딩은 얇은 작업 프로세스 콜백을 사용하며 Rust가 형식 필드, 암호 절단과 검증을 처리합니다. Qt는 형식 로직에 들어가지 않습니다.

제품 의존성 정책은 형식과 브리지 크레이트의 일반/빌드 의존 집합을 검사합니다. 공식 7-Zip과 UnRAR 컨테이너, 알고리즘 및 범용 유틸리티는 모두 제품 CMake 매니페스트에서 제거되었으며 제품 소스 준비 목록은 비어 있고 실패 폴백도 없습니다. 과거 제공물의 라이선스 의무는 유지됩니다. 독립 pycdlib, Microsoft IMAPI, mkudffs/udfclient 픽스처, 공식 참조 작업 프로세스와 명세 벡터는 테스트 전용입니다. 소스와 라이선스를 별도로 관리하며 애플리케이션 실행 의존성에 들어가지 않습니다. 작고 고정된 프로젝트 소유 입력과 생성기는 테스트 디렉터리에서 버전 관리하고 확장된 이미지와 실행 증거는 build/에 둡니다.

## 인터페이스 글꼴과 도구

검토 장치의 시스템 폴백 글꼴 대신 공식 Noto Sans SC 2.004-H2의 오프라인 가변 부분 집합을 사용합니다. 고정 Google Fonts 커밋, 원본 소스 해시와 생성 방법, 저작권과 OFL-1.1 라이선스는 [자료 목록](../../../specs/dependencies/ui-font.json)을 참조하세요. 빌드는 굵기 400–750 WOFF2와 같은 소스의 네이티브 TTF를 빌드 디렉터리에 생성합니다. 소스 저장소에는 생성된 글꼴 바이너리를 보관하지 않으며 외부 서비스, 전이 실행 의존성 또는 네이티브 컴파일도 필요하지 않습니다. 시스템 글꼴은 플랫폼별 텍스트 측정값이 달라지므로 검토 디자인에 글꼴을 포함합니다. 각 빌드는 현재 텍스트의 문자 범위를 검사하지만 알 수 없는 생성 디자인 글꼴이 일치한다고 가정할 수 없습니다. 업스트림은 유지 관리 중이며 유지 상태가 바뀌면 고정 버전을 재평가합니다.

데스크톱 리소스 정적 검사는 Pillow(HPND), FontTools(MIT), Brotli(MIT), 워크플로 검사에 PyYAML(MIT)을 사용합니다. 버전은 [데스크톱 에셋 도구 요구 사항](../../../scripts/desktop-asset-requirements.txt)에 고정하고 별도 가상 환경에 설치합니다. Playwright(Apache-2.0)와 Chromium은 필요할 때 로컬 리소스 렌더링에 사용하며 제품 런타임 코드에 들어가지 않습니다.

## 자체 ZIP과 범용 암호화 기본 요소

ZIP 컨테이너, 스트리밍 생성, 일반 다시 쓰기와 AES 래핑은 [독립 Rust 구현](independent-rust-codecs.md)을 사용하며 Deflate/CRC-32는 `xdzip-compression`을 재사용합니다. `rawzip = 0.5.2`, `zip = 9.0.5`, `flate2 = 1.1.10`은 개발 테스트 참조 전용으로 ZIP 제품 또는 빌드 의존 집합에 들어가지 않습니다. 범용 압축 해제를 호출하거나 임의 경로를 인가하지 않습니다. 이를 사용한 과거 제공물의 라이선스 의무는 유지됩니다.

암호화 기본 요소는 기존 전이 의존성에서 직접 의존성으로 승격한 정확한 버전을 재사용합니다. RustCrypto `aes = 0.9.3`, `hmac = 0.13.0`, `sha1 = 0.11.0`, `pbkdf2 = 0.13.0`과 `getrandom = 0.4.2`, `constant_time_eq = 0.6.1`입니다. PBKDF2는 hmac만 활성화합니다. SHA1은 기본 alloc/oid를 비활성화하고 zeroize만 활성화하며 AES/HMAC도 zeroize를 활성화합니다. 형식 라이브러리, 네이티브 압축 라이브러리 또는 실행 시 다운로드를 추가하지 않습니다. CTR, 무작위 솔트 래핑, 파생 결과 분할, 암호 검증 값과 인증된 EOF는 프로젝트가 구현합니다. 범용 기본 요소가 ZIP 파서를 다시 도입해서는 안 됩니다.

이 패키지와 전이 cipher/digest 의존성은 Cargo.lock에 잠그며 crates.io와 각 업스트림에서 가져옵니다. AES/HMAC/SHA1/PBKDF2/zeroize/getrandom은 MIT 또는 Apache-2.0, constant_time_eq는 CC0-1.0 또는 MIT-0 또는 Apache-2.0이며 실제 고정 패키지의 텍스트를 따릅니다. SHA-1/PBKDF2 매개변수는 WinZip AES에서 가져왔으며 별도로 선택한 사용자 암호 저장 방식에는 사용하지 않습니다. 빌드 스크립트는 플랫폼 설정만 수행하며 네트워크 다운로드나 C 디코더 컴파일은 없습니다.

`zeroize`는 AES 0.9.3 키 정리 기능을 충족하도록 1.9.1로 통일해 고정합니다. 호스트, 네이티브 어댑터, 업데이트와 프로브의 암호 컨테이너는 같은 메이저 버전 두 개 대신 한 버전을 유지합니다. 비밀 Debug는 계속 가려지며 파생 버퍼와 확장 키는 라이브러리가 지원하는 영점화 수명을 사용하되 모든 과거 메모리 사본을 지울 수 있다고 주장하지 않습니다. 채택에는 잠금 파일, 라이선스, 실제 기능 트리, 실패 테스트와 독립 7-Zip 상호 운용이 포함됩니다. 원본 취약점 조회 응답은 빌드 아티팩트에 두며 공지 없는 성공 조회는 취약점 부재를 보장하지 않습니다.

## WP-11 TAR/GZIP

[tar 0.4.46](https://docs.rs/tar/0.4.46/tar/)을 정확히 고정하고(crates.io, [업스트림 소스](https://github.com/alexcrichton/tar-rs), MIT 또는 Apache-2.0) 기본 xattr 기능을 비활성화합니다. 독립적인 개발 테스트 읽기/쓰기 참조 전용입니다. 제품 TAR 헤더, PAX, 생성과 다시 쓰기는 타사 TAR 파서나 Builder, 범용 압축 해제 또는 소유자/권한 복원이 아닌 [독립 Rust 레코드 계층](independent-rust-codecs.md)을 사용합니다. 제품 의존 집합에 tar와 filetime이 돌아오는 것을 금지하며 테스트 및 과거 제공물 라이선스는 유지합니다. 인코딩 시 플랫폼 간 이름 상호 운용을 위해 PAX UTF-8 경로와 범위를 벗어난 숫자를 명시적으로 기록합니다.

GZIP은 타사 제품 의존성이 없는 독립 `xdzip-compression` 구현을 사용하며 RFC 1951 Deflate, RFC 1952 프레이밍과 CRC-32를 포함합니다. 압축 입력과 디코딩 출력에는 호스트 취소와 바이트 예산이 적용됩니다. 완전한 디코딩은 각 구성원의 CRC와 ISIZE를 검증합니다. `flate2` 1.1.10은 개발 테스트 참조입니다. ZIP과 GZIP은 같은 자체 Deflate 알고리즘을 사용합니다. 실행 라우팅은 논리 파일 하나를 제공하는 [독립 GZIP](single-file-streams.md)과 TAR 디렉터리 트리를 제공하는 [TAR.GZ](tar-gzip.md)를 구분합니다.

tar 테스트 전이 의존성에는 filetime 0.2.29(MIT 또는 Apache-2.0)가 포함되며 cfg-if와 libc를 재사용합니다. filetime은 codec 제품 의존 집합에 들어가지 않습니다. 새로운 다운로드 빌드 스크립트, 실행 시 네트워크 요청 또는 외부 프로세스 형식 백엔드는 없습니다. 기존 collect-notices.py는 잠금 파일에서 라이선스를 수집합니다. 잠금 파일에 테스트 패키지가 있다고 제품이 링크하는 것은 아닙니다. 시스템 tar와 Python tarfile은 테스트 전용이며 실행 의존성에 들어가지 않습니다.

2026-10-15에 tar 0.4.46, filetime 0.2.29, flate2 1.1.10의 정확한 버전을 OSV에서 조회한 결과 공지가 없었습니다. 원본 응답은 build/review/wp11에만 보관합니다. 이는 취약점 부재를 보장하지 않습니다. 양방향 상호 운용, 잘못된 입력, 할당량, 취소와 세 플랫폼 검증은 실제로 실행해야 하며 증거는 [WP-11 계약](tar-gzip.md)에서 연결합니다.

## 보안 패치된 QtNetwork 파생판

데스크톱 Qt 6.8.2 Network 동적 라이브러리는 고정 QtBase 아카이브에 CVE-2026-116337과 CVE-2026-105946에 대한 수정되지 않은 공식 패치 다섯 개를 적용한 `6.8.2-xdzip-network.1`을 사용합니다. 범위는 HTTP/CONNECT 상태 줄과 헤더 수신 제한 및 인증 분석 경계이며 QtCore, 다른 모듈과 TLS 플러그인은 교체하지 않습니다. 기존 기능 스위치, 내보낸 심볼과 원본 SDK QtCore의 가져오기 호환성을 유지합니다. 소스, 순서, 해시, 라이선스, 교체와 실제 입력 검증은 [유지 관리 노트](../../../patches/qt/README.md)를 참조하세요. 제품은 LGPL-3.0 동적 링크를 유지합니다. 업스트림 GPL 전용 테스트는 배포하지 않으며 OpenSSL 3.5.9는 컴파일 선언 생성에만 사용하고 새 암호화 라이브러리는 빌드하거나 배포하지 않습니다. 실제 패키지 출처, 공지와 재링크 조건은 별도 검토가 필요합니다.

Linux에는 기존 SDK 내장 유형 상수를 명시적으로 인스턴스화하는 다이제스트 고정 자체 ABI 보존 단위도 추가해 네트워크 동작을 늘리지 않고 원본 내보내기를 유지합니다. 소스와 Apache-2.0 공지가 자료에 포함됩니다. 다섯 공식 패치는 수정하지 않았으며 다른 플랫폼의 제품 라이브러리에는 이 단위를 적용하지 않습니다.

<a id="macos-更新提供者"></a>

## macOS 업데이트 제공자

Mac 업데이트 통합은 Sparkle 2.10.0과 `xdzip-proxy.2` 프록시 확장을 고정합니다. SDK, 소스 아카이브, 패치와 전체 라이선스 텍스트 해시는 [의존성 설정](../../../scripts/sparkle-dependency.json)에 있습니다. 애플리케이션 업데이트만 처리하며 압축 파일 디코딩은 하지 않습니다. 공식 SDK 원본 라이선스 텍스트에는 MIT, BSD-2-Clause, Zlib가 포함되며 Mac 공지에 유지합니다. 로컬 대응 소스 수집기는 오프라인 준비와 동적 프레임워크 재링크를 위해 고정 소스와 원본 SDK 아카이브를 유지합니다. 릴리스 프로토콜에 따라 이 아카이브는 Release 또는 Actions 다운로드 자산으로 제공하지 않습니다.

주 프레임워크의 업스트림 Objective-C 단위 67개와 Ed25519 C 단위 10개는 고정 소스와 [제한된 패치](../../../patches/sparkle/2.10.0-download-proxy.patch)에서 네이티브 Apple Silicon Clang으로 컴파일하며 업스트림 빌드 기능과 설치 IPC를 유지합니다. 리소스, 설치 도우미와 서명 도구는 공식 SDK 출처를 유지하며 소스에서 다시 빌드했다고 주장하지 않습니다. 파생 프레임워크는 별도 변형, 소스/패치/빌더 해시와 파일별 빌드 회신을 기록합니다. 배포 출처와 SBOM은 이 출처를 공식 SDK 바이트와 구분합니다. [패치 유지 관리 노트](../../../patches/sparkle/README.md)를 참조하세요.

확장은 사용자 지정 프록시에 시스템 Network.framework를 사용하되 직접 연결로 폴백하지 않으며 명시적인 직접 연결에는 CFNetwork 세션 프록시 스위치를 사용합니다. 재배포 가능한 실행 라이브러리를 추가하거나 시스템 프록시 설정을 변경하지 않습니다.

필요할 때 실행하는 Qt 워크플로의 Mac 작업은 SDK를 외부 링크하며 프레임워크를 배포하지 않습니다. 전용 패키징은 임시 사본에 프레임워크를 포함하고 샌드박스가 아닌 애플리케이션에서 활성화하지 않는 XPC 서비스를 제거하며 필요한 설치 도우미를 유지합니다. 축소와 재배치 후 도우미, 프레임워크, 애플리케이션 순서로 ad-hoc 무결성 서명을 수행합니다. 도우미는 필요한 Hardened Runtime과 특정 entitlement를 유지하며 범용 네이티브 라이브러리 재서명 명령을 적용하지 않습니다. 배포 출처, SBOM, 라이선스와 소스 자료는 버전과 함께 기록합니다.

## 실험적 설치 도구

Windows 빌드는 프로젝트 런타임 디렉터리의 사용자별 설치 프로그램을 컴파일하기 위해서만 Inno Setup 6.7.3을 도입합니다. 소스는 [jrsoftware/issrc 공식 릴리스](https://github.com/jrsoftware/issrc/releases/tag/is-6_7_3)이며 불변 소스 커밋과 공식 바이너리, 중국어 리소스, 라이선스 텍스트의 SHA-256은 [installer-toolchain.json](../../../scripts/installer-toolchain.json)에 고정합니다. 사용자별 모드와 누적 제거 로그를 사용합니다. NSIS와 WiX/MSI는 대안이며 이 작업은 자체 정리 데이터베이스나 기업 배포 규칙을 유지하지 않습니다.

고정 소스의 Inno Setup License를 사용하고 컴파일러를 수정하지 않은 채 저작권, 원본 출처와 엔진 UI를 유지합니다. 텍스트는 Windows 공지에 포함합니다. 전이 코드는 공식 컴파일러/설치 스텁의 Pascal Script, 압축과 플랫폼 구성 요소를 사용하며 여기서 엔진을 다시 빌드하지 않습니다. 포함 구성 요소의 소스와 라이선스를 실제 제공물에 대해 확인합니다. 컴파일러는 CI의 build/에 설치하며 Qt 런타임 디렉터리에 들어가지 않습니다. 설치 스텁은 최종 설치 프로그램에 들어가며 회신은 그 버전과 설치 프로그램 해시를 기록합니다. 런타임 디렉터리 SBOM을 설치 스텁 전체 SBOM으로 표현하지 않습니다.

표준 파일 복사, 사용자별 레지스트리, 프로젝트 바로 가기와 뮤텍스 보호만 활성화합니다. 애플리케이션 자동 종료/재시작은 비활성화합니다. 설치 중 네트워크 다운로드, 사용자 지정 네이티브 플러그인 또는 새 Rust 빌드 스크립트는 없습니다. Linux .deb는 Ubuntu dpkg-deb와 시스템 유지 관리 도구를 사용하며 애플리케이션 codec을 추가하지 않습니다. Qt 다운로드 도구 작업 디렉터리는 build/ 아래로 고정해 aqtinstall.log가 소스 신원을 오염하지 않게 합니다.

공식 저장소의 공개 보안 공지 API는 2026-10-16에 빈 목록을 반환했으며 원본 응답은 build/review/installer-tool-advisories.json에 남아 있습니다. 이는 취약점 부재의 증거도 완전한 NVD 검색도 아닙니다. [공식 버전 기록](https://jrsoftware.org/files/is6-whatsnew.htm)은 이후 보안 유지 관리 검토에 활용합니다. 6.7.3 고정은 최신 또는 장기 지원 버전이라는 주장이 아닙니다. 해시 검증은 서명 신원이나 엔진 전체의 라이선스/보안 채택 검토를 대체하지 않습니다. 실제 컴파일, 설치, 시스템 열기, 업데이트/롤백/제거와 실패는 [설치 계약](installers-associations.md)과 실제 CI에 기록합니다.

## 기존 파일 이름 인코딩 감지와 디코딩

[chardetng 1.0.0](https://docs.rs/chardetng/1.0.0/chardetng/)과 [encoding_rs 0.8.42](https://docs.rs/encoding_rs/0.8.42/encoding_rs/)를 정확히 고정하며 둘 다 crates.io와 hsivonen 업스트림에서 가져옵니다. 전자는 MIT 또는 Apache-2.0을 사용하고 후자의 코드는 MIT 또는 Apache-2.0, WHATWG 매핑 데이터에는 추가로 BSD-3-Clause가 적용됩니다. LICENSE-WHATWG와 COPYRIGHT를 유지합니다. 공지 수집기는 전체 라이선스 텍스트뿐 아니라 COPYRIGHT도 포함합니다.

둘 다 기본 기능을 비활성화합니다. encoding_rs는 직접 alloc만 활성화하고 감지기 멀티스레딩/rayon은 비활성화합니다. 새로 고정한 전이 패키지는 core_detect 1.0.0, multiversion_no_op 1.0.0, scopeguard 1.2.0, simdutf8 0.1.5로 모두 MIT 또는 Apache-2.0입니다. cfg-if 1.0.5, memchr 2.8.3과 빌드 매크로 rustversion 1.0.23은 고정 버전을 재사용합니다. core_detect는 로컬 CPU 능력을 감지하고 simdutf8은 업스트림 SIMD UTF-8 검증, multiversion_no_op는 컴파일 시 동작이 없는 매크로입니다. 새 C/C++ 네이티브 라이브러리, 네트워크 빌드 스크립트 또는 실행 시 다운로드를 추가하지 않습니다. encoding_rs 빌드 스크립트는 컴파일러 버전에 따라 로컬 설정만 선택합니다. CPU별 unsafe는 유지 관리되는 업스트림 코드에 남으며 호스트는 unsafe/FFI를 추가하지 않습니다.

하나의 Rust 감지기는 Qt, 시스템 iconv와 Windows 코드 페이지 사이의 규칙 차이를 방지합니다. CJK 통계 모델을 유지하거나 외부 압축 해제 프로그램을 호출하면 구현과 배포 경계가 넓어집니다. chardetng 업스트림은 수동적 유지 관리로 표시되어 있으며 버전/모델 변경 시 독립 파일 이름 샘플을 다시 실행해야 합니다. ZIP CP437은 플랫폼 API 없이 [Unicode CP437 표](https://www.unicode.org/Public/MAPPINGS/VENDORS/MICSFT/PC/CP437.TXT)에 기반한 고정 단일 바이트 매핑입니다. Python 표준 라이브러리가 테스트 원시 바이트를 별도로 생성합니다. 감지는 휴리스틱이며 경로 인가나 내용 검증의 증거가 아닙니다.

2026-10-24에 새로 고정한 여섯 패키지의 정확한 버전을 OSV에서 조회한 결과 공지가 없었습니다. 원시 응답은 로컬 아티팩트에만 남습니다. 이는 대상이 한정된 검사이지 취약점 부재를 보장하지 않습니다. 채택 시 실제 Cargo 기능, 소스와 전체 라이선스 텍스트도 확인합니다. 회귀 범위, 리소스와 실패 규칙은 [파일 이름 인코딩 계약](filename-encoding.md)을 참조하세요. 세 플랫폼 빌드, 내용 상호 운용과 실제 결과는 관련 issue/CI에서 추적합니다.

## Qt 이미지 미리 보기 모듈

고정 Qt 6.8.2를 유지하고 공식 qtimageformats를 추가합니다. 소스 커밋과 아카이브 다이제스트는 [구성 요소 설정](../../../scripts/corresponding-sources.json)에 있습니다. qwebp, qtiff, qtga, qicns를 사용합니다. macOS는 시스템 ImageIO를 사용하는 qmacheif도 사용합니다. libheif/libavif나 추가 이미지 백엔드를 도입하지 않습니다. GIF, BMP, ICO/CUR와 JPEG는 QtGui를 재사용하고 SVG는 제한 없는 리소스 로드 없이 기존 QtSvg를 재사용합니다.

QtImageFormats에는 libwebp 1.5.0이 포함됩니다. XDZip은 libpng 1.6.59로 QtGui를, libtiff 4.7.2로 TIFF 플러그인을 다시 빌드합니다. [고정 소스](../../../scripts/qt-image-security.json)와 SDK 출처 기록으로 제공되는 구성 요소를 식별합니다. 원본 COPYING/COPYRIGHT 텍스트는 고정 커밋과 SHA-256으로 수집합니다. 일반적인 NOASSERTION 표식은 그 자체로 확인된 라이선스 결함이 아닙니다. 동적 Qt 모듈은 기존 라이선스 경로를 유지하며 실제 제공된 타사 구성 요소의 공지를 포함합니다. 정확한 버전 의존성 조회와 세 플랫폼의 디코딩, 잘못된 입력, 배포 결과는 issue와 아티팩트에 남습니다. 공지가 없는 한 번의 조회는 취약점 부재 주장이 아닙니다.

이미지 런타임은 고정된 도우미를 사용합니다. 원본 이미지, 픽셀, 작업 프로세스, 입력/출력과 실패 제한은 [미리 보기 계약](content-preview.md)을 참조하세요. 기본 준비는 기존 도구체인을 보완하고 빌드 전에 플러그인을 검사하며 배포 시 실제 도우미를 검증합니다. 두 번째 Qt SDK는 추가하지 않습니다.

## 파일 관리자 메뉴 어댑터

Windows 확장은 Windows SDK 시스템 API와 정적 MSVC 런타임에만 의존합니다. macOS 서비스 도우미는 시스템 AppKit과 C++ 표준 라이브러리에만 의존합니다. 어느 쪽도 파일 관리자 호스트에 Qt나 형식 라이브러리를 로드하지 않습니다. Ubuntu 24.04는 배포판의 `libnautilus-extension4`와 GLib/GObject/GIO를 사용하고 빌드 의존성으로 `libnautilus-extension-dev`를 사용합니다. 파일 애플리케이션은 제품 구성 요소가 아닙니다.

[Nautilus 46.0 MenuProvider 헤더](https://github.com/GNOME/nautilus/blob/46.0/libnautilus-extension/nautilus-menu-provider.h)는 GNU Library General Public License 2 이상을 선언합니다. 새 확장은 이 공개 인터페이스만 사용하고 시스템 라이브러리를 동적 링크하며 이후 LGPL-3.0 조건을 선택합니다. Linux 배포 감사는 실제 시스템 패키지 버전과 소스를 기록하며 별도 Nautilus/GLib 사본은 포함하지 않습니다. 프로젝트 어댑터 코드, 인터페이스 사용과 전체 Nautilus 애플리케이션의 라이선스는 별개입니다. 공식 후보는 실제 실행 자료에 대한 검토가 계속 필요하며 인터페이스 컴파일 성공은 릴리스 라이선스 승인을 의미하지 않습니다.
