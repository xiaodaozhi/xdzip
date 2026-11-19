<a name="xdzip-read-only-zip-browsing-contract"></a>

# 읽기 전용 ZIP 탐색 계약

[English](../../engineering/zip-readonly-browsing.md) | [简体中文](../../zh-CN/engineering/zip-readonly-browsing.md) | [繁體中文（台灣）](../../zh-TW/engineering/zip-readonly-browsing.md) | [日本語](../../ja-JP/engineering/zip-readonly-browsing.md) | 한국어 | [Français](../../fr-FR/engineering/zip-readonly-browsing.md) | [Deutsch](../../de-DE/engineering/zip-readonly-browsing.md) | [Español](../../es-ES/engineering/zip-readonly-browsing.md) | [Português (Brasil)](../../pt-BR/engineering/zip-readonly-browsing.md) | [Русский](../../ru-RU/engineering/zip-readonly-browsing.md)

이 계약은 루트 바로 아래의 항목, 하위 폴더 검색, 안정적인 물리적 식별자, 폴더 우선 정렬을 정의합니다. ZIP 탐색은 메타데이터를 읽으며 항목의 내용을 디코딩하거나 CRC를 검사하지 않습니다. 파일도 추출하지 않습니다. 큰 일반 ZIP 압축 파일은 비공개 [디스크 메타데이터 인덱스](large-index.md)를 사용할 수 있습니다.

대용량 일반 ZIP의 현재 인덱스와 필요할 때 데이터를 읽는 Qt 모델은 [대용량 압축 파일 인덱싱](large-index.md)을 참조하세요. 이 문서는 최초 기준인 100,000개 항목과 쿼리 식별자 요구 사항을 유지합니다. 용량을 늘린 뒤에도 오류, 취소, 식별자 회귀를 확인해야 합니다.

<a name="xdzip-browsing-scope-and-limits"></a>

## 탐색 범위 및 제한

- ZIP/ZIP64 메타데이터는 실제 레코드 100,000개까지 지원합니다. 분할 볼륨은 [분할 볼륨 호스트](archive-opening.md)가 변환한 뒤 읽습니다. 코덱은 변환하지 않은 다중 디스크 메타데이터를 계속 거부합니다.
- 이름당 4 KiB, 원시 이름 합계 16 MiB, 디코딩과 이스케이프를 거친 표시용 이름 합계 16 MiB, 전체 읽기 64 MiB, 스캔 버퍼 256 KiB입니다. 선언된 64비트 크기는 정확한 정수로 유지합니다.
- 디렉터리 인덱스는 노드 32,768개, 깊이 64단계, 이름과 접두사 합계 4 MiB까지 허용합니다. 한도를 넘으면 탐색 인덱스 전체를 폐기하고 제한된 동작을 안내합니다. 불완전한 트리는 표시하지 않습니다.
- 디렉터리 바로 아래 항목의 인덱스와 쿼리 결과를 재사용합니다. 같은 쿼리의 전송 배치는 필터링이나 정렬을 다시 하지 않습니다. 각 배치의 이름 합계는 별도로 64 KiB로 제한합니다. UI는 페이지 이동 버튼 없이 현재 디렉터리 전체를 표시합니다.
- 페이지마다 세션, 쿼리 ID, GUI 세대, 스냅숏 리비전 및 오프셋을 검증합니다. 쿼리가 변경되면 이전 작업을 취소하고 최신 요청만 처리합니다. 오래된 응답은 새 결과를 덮어쓸 수 없습니다.
- 스캔 전후, 쿼리와 사전 검사 시점에 소스 길이, 수정 시각, Unix의 장치와 inode, Windows의 볼륨 번호와 파일 인덱스를 확인합니다. `source_changed`가 발생하면 Qt 쿼리와 사전 검사가 무효화된 목록, 선택, 디렉터리 트리 및 세부 정보를 지웁니다. 새 세션을 만들려면 압축 파일을 다시 여세요. 이 검사는 가능한 범위에서 변경을 감지하며 파일 시스템의 원자적 스냅숏이나 안전한 쓰기 권한을 제공하지 않습니다.

<a name="xdzip-acceptance-exit-criteria"></a>

## 인수 기준

실제로 생성한 100,000개 항목의 ZIP64에서 스캔, 전체 결과 정렬, 검색 결과 가져오기, 디렉터리 이동, Qt의 전체 목록 표시와 선택을 완료해야 합니다. 모든 배치는 같은 쿼리 식별자를 사용합니다. 손상된 ZIP64, 분할 볼륨, 이름·표시·읽기 한도, 취소, 오래된 응답, 소스 수정 및 교체를 검증합니다. 실제 시간, 자원 사용량, 이벤트 큐 및 델리게이트 수를 기록합니다. 필요에 따라 3개 플랫폼의 빌드 또는 Apple Silicon 개발 패키지를 검증할 때는 실제 결과를 별도로 기록합니다.

CI에서는 저장소 워크플로 실행 목록을 head_sha와 실제 이벤트로 필터링합니다. 일상 검사는 푸시나 PR에서 실행합니다. 전체 Foundation 및 Qt 검사는 필요에 따라 수동으로 실행합니다. 실제 결과와 미실행 범위는 관련 이슈에 기록합니다. 다른 이벤트에 결과가 없더라도 확인하려는 검사의 상태를 알 수는 없습니다.
