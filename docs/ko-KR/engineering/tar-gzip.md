<a name="xdzip-targzip-implementation-contract"></a>

# TAR/GZIP 구현 계약

[English](../../engineering/tar-gzip.md) | [简体中文](../../zh-CN/engineering/tar-gzip.md) | [繁體中文](../../zh-TW/engineering/tar-gzip.md) | [日本語](../../ja-JP/engineering/tar-gzip.md) | 한국어 | [Français](../../fr-FR/engineering/tar-gzip.md) | [Deutsch](../../de-DE/engineering/tar-gzip.md) | [Español](../../es-ES/engineering/tar-gzip.md) | [Português (Brasil)](../../pt-BR/engineering/tar-gzip.md) | [Русский](../../ru-RU/engineering/tar-gzip.md)

이 계약은 TAR/TAR.GZ 탐색, 생성과 안전한 압축 풀기의 경계를 정의합니다. 플랫폼 결과와 수동 수락은 이슈 또는 CI 산출물에 기록합니다. [안전한 압축 풀기](zip-safe-extraction.md)와 [ZIP 생성](zip-creation.md)의 입력 권한, 임시 준비, 충돌, 취소와 전체 아카이브 커밋을 유지합니다.

<a name="xdzip-scope-and-capabilities"></a>

## 범위와 기능

TAR과 TAR.GZ의 탐색, 생성과 안전한 압축 풀기를 지원합니다. TAR은 컨테이너이고 GZIP은 압축 필터입니다. 읽을 때 확장자로 백엔드를 선택하지 않고 내용을 감지합니다. ZIP 동작은 호환성을 유지합니다. 중첩 자동 확장, 암호화와 분할 TAR은 활성화하지 않습니다. 일반 단일 파일 GZIP은 별도의 [스트림 계약](single-file-streams.md)을 사용합니다.

TAR 헤더, 숫자, PAX, 스트리밍 인코딩과 다시 쓰기는 독립 Rust 레코드 계층을 사용합니다. GZIP 프레이밍, CRC-32와 Deflate LZ77/Huffman은 공통 `xdzip-compression`을 사용합니다. `tar`와 `flate2`는 개발 테스트 참고용일 뿐이며 이 crate들의 제품 의존성 집합에 들어가지 않습니다. 사양, 출처와 제한은 [자체 형식 계약](../../engineering/independent-rust-codecs.md)을 참고하세요.

TAR은 일반 파일과 디렉터리를 허용하고 PAX 경로와 GNU 긴 이름을 분석합니다. 심볼릭 링크, 하드 링크, 장치, FIFO, 스파스 파일과 알 수 없는 유형은 추출할 수 없습니다. 모든 출력 이름은 계속 PortablePath와 호스트 충돌 사전 검사를 통과해야 하며 형식 라이브러리는 파일 경로나 압축 해제 권한을 받지 않습니다. 디렉터리 권한, 소유자와 특수 모드는 복원하지 않습니다. 날짜는 표시용 메타데이터이며 복원을 보장하지 않습니다.

<a name="xdzip-bounded-streaming"></a>

## 제한된 스트리밍

기존 파일 이름 인코딩, GNU 긴 이름과 PAX UTF-8/BINARY 선언은 [인코딩 계약](filename-encoding.md)에 따라 통일해서 해석합니다. 전체 검색을 완료한 후에만 인덱스를 게시하며 검색 중에는 항목 수와 합계를 알 수 없습니다. 메타데이터에는 항목 100,000개, 이름당 4,096바이트와 누적 이름 예산 제한이 유지됩니다. 스트리밍 원본의 인코딩 입력과 디코딩 작업은 각각 최대 24 GiB이며 GNU/PAX 확장 헤더 하나는 최대 64 KiB, 항목당 확장 헤더는 최대 네 개입니다. 전역 PAX는 git archive 호환을 위해 파일 의미가 없는 주석 텍스트만 허용합니다. 선언된 레코드 길이를 완전히 검증한 뒤 버리며 항목에 상속하지 않습니다. 전역 path, size, mtime, linkpath, sparse와 알 수 없는 필드는 명시적으로 거부하며 GNU 긴 링크와 스파스 변형은 지원하지 않습니다. 전역 주석도 항목당 네 헤더와 스트림 바이트 예산에 포함되며 미완성 로컬 확장을 유효하게 만들지 않습니다. TAR 이름의 흔한 선행 `./`는 정규화하고 루트 표시는 출력 항목이 아닙니다. 순차 읽기와 건너뛰기에서도 취소와 바이트 예산을 확인하며 위치 이동으로 손상된 스트림 검사를 우회할 수 없습니다. GZIP은 EOF에 도달하여 CRC와 ISIZE를 검증해야 합니다. 여러 멤버는 하나의 연속된 TAR 바이트 스트림만 구성합니다. 끝의 0이 아닌 불필요한 데이터, 잘림과 손상은 실패합니다. 검증되지 않은 부분 인덱스는 거부합니다.

TAR.GZ는 항목별 압축 크기가 없습니다. 높은 압축률의 허용 여부는 선택 항목의 선언된 합계와 고정된 전체 원본 크기를 비교합니다. 1000:1과 1 MiB를 넘으면 명시적 허가가 여전히 필요하며 이 허가로 검색 작업이나 출력 할당량을 우회할 수 없습니다. 메타데이터 사전 검사는 항목별 압축률을 만들어 내지 않습니다.

압축 풀기는 선택 항목을 물리적 순서로 처리합니다. 선택하지 않은 내용을 건너뛰는 작업도 계량되며 취소할 수 있습니다. 임의 읽기는 보장하지 않습니다. PayloadReader는 원본과 호스트 출력 대상만 받습니다. 호스트는 검색, 디코딩과 커밋 경계에서 원본 버전을 계속 검증합니다. 항목별 커밋과 전체 생성 실패 의미는 기존 안전 I/O 계약을 따릅니다.

<a name="xdzip-creation-and-interface"></a>

## 생성과 인터페이스

생성 창은 밀도를 바꾸지 않고 기존 TAR과 TAR.GZ 세그먼트를 활성화합니다. 형식을 바꾸면 확장자가 갱신됩니다. TAR은 압축 수준을 비활성화하고 두 TAR 형식 모두 암호와 분할 볼륨 값을 비활성화하고 지웁니다. TAR.GZ는 GZIP 수준 0/1/6/9를 사용합니다. 항목별 압축 크기는 알 수 없으므로 0바이트가 아니라 대시로 표시합니다. Rust는 원본 이름에서 아카이브 이름의 대상 폴더를 일관되게 도출하고 제한된 브리지를 통해 미리 보기에 제공합니다. TAR.GZ는 대소문자를 구분하지 않고 전체 복합 확장자를 제거하며 기본 이름이 비면 archive를 사용합니다. UI는 다른 대상을 도출하지 않으며 PortablePath와 승인된 루트가 다시 검증합니다. 기본 창의 형식 태그는 실제 형식과 텍스트 너비에 맞춰집니다. TAR 날짜는 UTC로 표시하며 기존 표시 범위 밖에서는 알 수 없는 상태로 둡니다. 생성 시 Unicode 또는 긴 이름에는 PAX UTF-8 경로를, 큰 파일에는 PAX 크기를 명시적으로 써서 참조 판독기의 로컬 인코딩 추측을 피합니다. 입력 트리를 고정하고 출력을 스트리밍하며 TAR/GZIP을 종료한 뒤 다시 검색하고 각 항목을 디코딩하여 검증합니다. 모두 성공한 후 전체를 게시합니다. 충돌로 이름을 바꿀 때 전체 복합 확장자를 유지합니다.

<a name="xdzip-verification-and-limits"></a>

## 검증과 제한

시스템 tar/Python tarfile과의 양방향 상호 운용, 긴 경로, Unicode, 빈 디렉터리, 중복, PAX/GNU, 특수 항목, 경로 이탈, 잘림, CRC, 리소스 제한, 취소와 기존 파일 보호를 검증합니다. 실제 Qt 생성, 다시 열기와 압축 풀기는 기존 작업 및 충돌 창을 재사용합니다. 실제 증거는 build/ 또는 CI 산출물에만 생성합니다.

영향받은 데스크톱 및 접근성 재검증은 [일반 검증](../../../developers/qa/README.md)에 따라 선택합니다. 네이티브 자동화와 오프스크린 검사는 수동 검사를 대신하지 않습니다. 실제 결과와 한계는 해당 이슈에 기록합니다. 기존 데스크톱 수락은 [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5)에 있으며 일회성 권한 또는 환경 차단은 이 계약에 유지하지 않습니다. `v0.1.0`의 과거 릴리스 라이선스 검토는 [#4](https://github.com/xiaodaozhi/xdzip/issues/4)와 [#53](https://github.com/xiaodaozhi/xdzip/issues/53)에 있으며 이후 후보에는 새 릴리스 이슈가 필요합니다.

CLI 생성 진입점은 .tgz를 .tar.gz와 같은 형식의 별칭으로 허용하며 동일한 GZIP 인코딩, 검증과 커밋을 사용합니다. 일반 비 TAR GZIP은 별도의 단일 파일 코덱을 사용합니다.
