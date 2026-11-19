# XDZip 문제 해결

[English](../../user-guide/troubleshooting.md) | [简体中文](../../zh-CN/user-guide/troubleshooting.md) | [繁體中文（台灣）](../../zh-TW/user-guide/troubleshooting.md) | [日本語](../../ja-JP/user-guide/troubleshooting.md) | 한국어 | [Français](../../fr-FR/user-guide/troubleshooting.md) | [Deutsch](../../de-DE/user-guide/troubleshooting.md) | [Español](../../es-ES/user-guide/troubleshooting.md) | [Português (Brasil)](../../pt-BR/user-guide/troubleshooting.md) | [Русский](../../ru-RU/user-guide/troubleshooting.md)

| 문제 | 확인할 사항 |
| --- | --- |
| 압축 파일이 열리지 않음 | [지원 형식](../../../README.ko-KR.md#xdzip-format-support), 압축 또는 암호화 방식, 볼륨 세트가 완전한지 확인하세요. 원본에 접근할 수 없으면 압축 파일과 모든 볼륨을 이름 변경 없이 접근 가능한 로컬 폴더로 복사한 뒤 다시 여세요. |
| 파일 이름이 잘못 보이거나 쓸 수 없음 | UTF-8 이름으로 압축 파일을 다시 만들 수 있는지 확인하세요. XDZip은 일부 이전 인코딩을 감지하지만 이름이 짧거나 인코딩이 섞이면 제한됩니다. 안전한 경로를 정할 수 있는 항목만 푸세요. [파일 이름 처리](../engineering/filename-encoding.md) |
| 속성은 표시되지만 미리 보기가 없음 | 형식, 크기, 플랫폼, 인코딩을 확인하세요. 미리 보기에는 별도 제한이 있으며 압축 풀기와는 독립적입니다. [미리 보기 제한](../engineering/content-preview.md) |
| 대상 위치가 거부되거나 결과가 불완전함 | 쓸 수 있는 개인 폴더를 선택하세요. 파일명 충돌과 지원되지 않는 항목을 확인하고 다시 시도하기 전에 작업 결과를 보세요. [파일의 압축 풀기](extract.md) |
| XDZip으로 작업 중 원본 파일이 교체됨 | 새 복사본을 다시 열고 항목을 새로 선택하세요. XDZip은 쓰기 전에 원본을 다시 확인합니다. |
| 파일 탐색기, Finder 또는 파일 앱에 동작이 없음 | 설치 채널과 확장 사용 여부를 확인하세요. Windows 11의 클래식 메뉴는 **더 많은 옵션 표시**에 있습니다. macOS에서는 시스템 설정에서 Finder 확장을 켜거나 등록된 서비스를 시도하세요. XDZip에서 압축 파일을 먼저 열 수도 있습니다. [파일 관리자에서 압축 풀기](extract.md#xdzip-from-the-file-manager) |
| 큰 ZIP을 검사하거나 수정할 수 없음 | 디스크 인덱스를 사용한다면 필요한 일반 파일만 선택하고 작업 제한 안에서 압축을 푸세요. [큰 압축 파일 제한](../engineering/large-index.md) |
| 업데이트 또는 설치 실패 | 버전과 채널을 확인하세요. 수동 다운로드의 체크섬을 확인하고 해당 설치 절차를 따르세요. [업데이트 복구](settings-and-updates.md) |

문제를 신고할 때는 **xdzip 정보**의 버전과 빌드, 운영체제와 아키텍처, 형식과 작업, 재현 단계 및 예상 결과와 실제 결과를 포함하세요. 가능하면 직접 만든 작은 예제를 사용하세요. 공유하기 전에 암호와 개인 경로를 제거하세요. [문제 신고](https://github.com/xiaodaozhi/xdzip/issues/new/choose)
