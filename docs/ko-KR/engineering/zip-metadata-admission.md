<a name="xdzip-zip-metadata-dependency-admission"></a>

# ZIP 메타데이터 의존성 선택

[English](../../engineering/zip-metadata-admission.md) | [简体中文](../../zh-CN/engineering/zip-metadata-admission.md) | [繁體中文（台灣）](../../zh-TW/engineering/zip-metadata-admission.md) | [日本語](../../ja-JP/engineering/zip-metadata-admission.md) | 한국어 | [Français](../../fr-FR/engineering/zip-metadata-admission.md) | [Deutsch](../../de-DE/engineering/zip-metadata-admission.md) | [Español](../../es-ES/engineering/zip-metadata-admission.md) | [Português (Brasil)](../../pt-BR/engineering/zip-metadata-admission.md) | [Русский](../../ru-RU/engineering/zip-metadata-admission.md)

이 문서는 첫 읽기 전용 디렉터리 스캔의 2026-10-11 결정을 보존합니다. 압축 풀기, 생성 및 타사 플러그인 실행은 이 결정의 범위 밖입니다. 현재 ZIP 메타데이터는 [독립 Rust 레코드 계층](independent-rust-codecs.md)을 사용하며 rawzip은 개발 테스트의 참조용입니다.

당시에는 [rawzip 0.5.2](https://github.com/nickbabcock/rawzip)를 선택했습니다. 정확한 버전과 체크섬은 [고정 매니페스트](../../../specs/engineering/zip-metadata-dependencies.json)와 루트의 Cargo.lock에 있습니다. 기본 기능을 끄고 std를 명시적으로 켰습니다. 실행 의존성과 build.rs는 없었습니다. 전체 MIT 라이선스는 [third_party/rawzip/LICENSE](../../../third_party/rawzip/LICENSE)에 보관하며 이 의존성을 사용하는 릴리스와 함께 제공합니다.

당시 검토는 README, Cargo 구성 및 디렉터리 읽기 코드를 확인했습니다. 빌린 Read + Seek 입력에서 중앙 디렉터리 레코드를 한 번에 하나씩 읽으며 물리적 순서와 중복 이름을 보존했습니다. 호스트는 읽기 바이트와 이름 합계 크기를 제한했습니다. 읽기 전용 스캔을 위해 zip 9.0.1도 평가했지만 이름으로 인덱싱하므로 물리적으로 중복된 레코드를 보존하는 데 맞지 않았습니다. AES 쓰기에는 나중에 zip-rs 9.0.3을 별도로 도입했으며 읽기 전용 물리 인덱스는 교체하지 않았습니다. 현재 독립 구현에서도 이 동작을 유지합니다.

RustSec 스냅숏 `7eebec69c352c7191b1f13eb95dd510eeca5d1de`에서 rawzip 권고가 발견되지 않았습니다. RUSTSEC-2026-0202는 1.0.195 이전 cxx의 let_cxx_string!에 해당하며 고정된 1.0.205에는 수정이 포함됩니다. 이는 해당 권고를 대상으로 한 검사 기록입니다.

메타데이터 어댑터는 디코더를 활성화하지 않으며 내용, CRC, 로컬 헤더와 중앙 디렉터리의 일치 여부를 검사하지 않습니다. 알 수 없는 방식은 번호만 표시합니다. 목록에 있다고 추출할 수 있는 것은 아닙니다. 읽기 전용 M2 범위에는 단일 볼륨 ZIP64 메타데이터도 추가해 레코드 100,000개와 2^53보다 큰 선언 크기의 정확성을 검증했습니다. [다중 볼륨 호스트](archive-opening.md)는 지원하는 볼륨 집합을 논리적인 단일 볼륨으로 변환합니다. 변환하지 않은 다중 디스크 레코드는 계속 거부합니다. Unicode 이름 확장과 이전 인코딩은 [이름 인코딩 계약](filename-encoding.md)을 따릅니다. 실제 내용은 [압축 풀기 의존성](zip-extraction-admission.md)과 [안전한 압축 풀기](zip-safe-extraction.md)에서 별도로 다룹니다. 한도와 실패 동작은 [구현 계약](readonly-browser.md)을 참조하세요.

회귀 검사는 실제 ZIP64, 잘못된 로케이터 및 끝 레코드, ZIP64 확장의 시작 디스크, 이름과 표시 및 읽기 한도, 취소 및 소스 변경을 포함합니다. 독립 구현에서는 자동 압축 풀기 오프셋, 모호한 끝 레코드 및 고정된 시드를 사용한 제한된 변이도 검사합니다. 제품 의존성 검사는 형식 라이브러리가 제거되었는지 확인합니다. 잠금 파일에는 테스트 의존성이 남을 수 있습니다.

압축 풀기 단계 M3의 내용 디코딩과 호스트 출력은 [압축 풀기 의존성](zip-extraction-admission.md)에 있습니다. 이 문서는 M2의 읽기 전용 범위를 보존합니다.
