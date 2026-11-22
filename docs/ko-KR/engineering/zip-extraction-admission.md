<a name="xdzip-zip-extraction-and-output-dependency-admission"></a>

# ZIP 압축 풀기와 출력 의존성의 도입 기준

[English](../../engineering/zip-extraction-admission.md) | [简体中文](../../zh-CN/engineering/zip-extraction-admission.md) | [繁體中文](../../zh-TW/engineering/zip-extraction-admission.md) | [日本語](../../ja-JP/engineering/zip-extraction-admission.md) | 한국어 | [Français](../../fr-FR/engineering/zip-extraction-admission.md) | [Deutsch](../../de-DE/engineering/zip-extraction-admission.md) | [Español](../../es-ES/engineering/zip-extraction-admission.md) | [Português (Brasil)](../../pt-BR/engineering/zip-extraction-admission.md) | [Русский](../../ru-RU/engineering/zip-extraction-admission.md)

이 문서는 안전한 압축 풀기와 암호 압축 풀기에 공통으로 적용되는 의존성 경계를 정의합니다. 최초 도입 당시의 의존성 출처, 라이선스와 기능 발췌는 [고정된 기준 자료](../../../specs/engineering/zip-extraction-dependencies.json)에 있습니다. AES를 포함한 현재 정확한 버전과 crates.io 체크섬은 루트 Cargo.lock을 따릅니다. 전체 활성 의존성 그래프는 `cargo tree --locked -e features`로 확인하세요. 기준 발췌를 현재의 전체 의존성 목록으로 취급하지 마세요.

| 의존성 | 용도와 결정 |
| --- | --- |
| xdzip-compression | 프로젝트 자체의 공통 Deflate와 CRC-32 구현으로 고정 버퍼를 사용하고 마지막 블록 및 압축 바이트의 정확한 소비를 엄격히 검사합니다. 독립 형식 계층은 ZIP 컨테이너, 디스크립터와 AES 래핑을 처리하며 호스트의 입력과 출력 대상을 빌려 쓰되 출력 권한과 완전한 검증의 경계를 유지합니다. flate2는 개발 테스트 참고용일 뿐입니다. |
| cap-std / cap-fs-ext 4.0.3 | 유지된 디렉터리 권한, nofollow 하위 디렉터리 열기, 배타적 임시 준비와 상대 경로 게시를 제공합니다. 문자열을 결합하고 canonicalize 후 생성하는 방식과 달리 호스트가 각 구성 요소의 권한을 확인하고 식별자를 다시 검사하며 일반 unpack을 호출하지 않습니다. cap-primitives 같은 전이 패키지도 고정합니다. |
| getrandom 0.4.2 | 비공개 작업 영역의 nonce에 운영체제 난수를 사용하며 타임스탬프, 의사 난수 값 또는 압축 파일 이름은 사용하지 않습니다. 기본 기능은 비활성화합니다. |
| rustix 1.1.6 | 고정된 버전을 재사용하여 POSIX 유효 사용자 ID와 파일 시스템 기본 연산을 제공합니다. 기본 기능을 비활성화하고 std/process와 전이적으로 필요한 fs만 활성화합니다. |
| windows-sys 0.61.2 | Windows 객체 소유자와 DACL을 조회하고 SID를 검사합니다. 직접 작성한 unsafe 코드를 격리하고 유효한 핸들, DWORD 정렬 버퍼와 RAII LocalFree를 빌려 사용합니다. 읽기 전용 속성만 검사하는 것과 달리 DACL 검사는 비공개 출력의 전제 조건을 확인합니다. 새 FFI는 핸들, 버퍼, 해제의 전제 조건과 조회 실패 시 동작을 검증해야 합니다. |
| unicode-normalization 0.1.25 | 작업 이름 별칭의 도입 판정 키를 NFD와 소문자로 생성하고 원래 출력 표기는 유지합니다. 이 키를 파일 시스템의 완전한 대소문자 접기로 취급하지 않습니다. ASCII 소문자를 넘어 macOS의 일반적인 Unicode 동등 이름을 다룹니다. |
| AES, HMAC, SHA-1, PBKDF2, constant_time_eq | 도입된 범용 암호 기본 요소입니다. 정확한 버전과 기능은 [의존성 계약](../engineering/dependencies.md)에 있습니다. ZIP CTR 계수, 솔트와 검증 값의 배치, 암호문 인증과 오류 분류는 프로젝트가 독립적으로 구현합니다. zip 9.0.5은 개발 테스트 참고용일 뿐입니다. |
| zeroize 1.9.1 | 암호, 파생 키와 테스트 프로브의 제한된 stdin 버퍼를 0으로 지우는 생명주기를 제공합니다. AES/HMAC/SHA-1도 자체 정리 기능을 활성화합니다. Qt, IME 또는 과거의 모든 메모리 복사본이 지워진다고 보장하지 않습니다. |

범용 의존성의 Cargo 설정, 라이선스와 공개 API를 검토하세요. 독립 형식 구현의 출처 규칙은 [독립 구현 계약](independent-rust-codecs.md)에 있습니다. cap 계열은 Bytecode Alliance가 유지 관리하고 getrandom과 rustix는 시스템 인터페이스를 제공하며 Unicode normalization은 범용 Unicode 라이브러리입니다. 업그레이드에는 여전히 버전 고정과 네이티브 경계 회귀 테스트가 필요합니다. 유지 관리자는 회귀 테스트나 보안 공지 확인을 대신하지 않습니다. 새 패키지는 실행 중 네트워크 요청을 하지 않습니다. 빌드 스크립트는 컴파일러 또는 대상 설정을 확인하거나 Rust 시스템 호출 백엔드를 선택할 뿐이며 외부 코드를 다운로드하거나 C 디코더를 컴파일하지 않습니다. 라이선스 고지는 원본 전문을 수집하고 실제 라이선스 표현은 고정된 자료를 따릅니다.

[검증 방법](../../../developers/qa/README.md)에 따라 정확한 버전으로 조회한 OSV의 실제 응답을 보관하세요. 조회 실패를 공지가 없다는 뜻으로 해석해서는 안 됩니다. 이는 전체 cargo-audit이 아닙니다. 기존 cxx 공지와 버전 선택은 [M2 의존성 기록](../engineering/dependencies.md)에 유지됩니다.

메타데이터 도입이 디코딩을 자동 승인하지는 않습니다. 독립 어댑터는 로컬 헤더, 스트림 종료, CRC와 32/64비트 데이터 디스크립터를 검증하고 중앙 및 로컬 ZIP64 센티널에 따라 너비를 선택합니다. 독립적인 Python force_zip64 테스트 자료는 유효한 혼합 너비와 변조된 크기를 다룹니다. rawzip, zip, flate2는 ZIP과 호스트의 일반 및 빌드 의존성 집합에 없으며 과거의 고정 발췌는 당시 출처만 보존합니다.

고정 자료에는 대상별 Cargo 해결 의존성이 모두 나열됩니다. getrandom의 WASI/WIT/WebAssembly 도구 체인은 지원하지 않는 대상에만 적용되며 macOS, Windows 또는 Linux 데스크톱 컴파일과 실행에 포함되지 않습니다. 모든 대상에 대한 Cargo 목록을 데스크톱 런타임 범위로 해석해서는 안 됩니다.

개발 패키지 고지는 `cargo metadata --filter-platform <rustc host>`의 실제 해결 노드를 사용하여 현재 플랫폼의 작업 공간 및 빌드 도구 라이선스를 수집하고 target.json을 기록합니다. 컴파일되지 않은 WASI/UEFI 패키지는 데스크톱 배포 의존성으로 취급하지 않습니다. 라이선스 전문이 없으면 패키징은 여전히 실패하며 Cargo 라이선스 이름으로 전문을 대체할 수 없습니다. r-efi 6.0.0의 완전한 MIT 허가와 저작권은 AUTHORS에 있으며 수집기는 해당 버전과 고정된 SHA-256에 대해서만 이 텍스트 예외를 허용합니다.
