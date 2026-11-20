# 압축 파일 만들기 및 수정

[English](../../user-guide/create-and-modify.md) | [简体中文](../../zh-CN/user-guide/create-and-modify.md) | [繁體中文](../../zh-TW/user-guide/create-and-modify.md) | [日本語](../../ja-JP/user-guide/create-and-modify.md) | 한국어 | [Français](../../fr-FR/user-guide/create-and-modify.md) | [Deutsch](../../de-DE/user-guide/create-and-modify.md) | [Español](../../es-ES/user-guide/create-and-modify.md) | [Português (Brasil)](../../pt-BR/user-guide/create-and-modify.md) | [Русский](../../ru-RU/user-guide/create-and-modify.md)

<a name="xdzip-create-an-archive"></a>

## 압축 파일 만들기

1. 시작 화면이나 **만들기** 명령에서 만들기 창을 엽니다. 파일이나 폴더를 추가하고 입력 목록을 확인하세요.
2. 출력 이름과 위치를 지정합니다. 형식과 사용할 수 있는 압축 설정을 선택합니다. ZIP은 공유에 자주 쓰입니다. 만들기 옵션은 [형식 표](../../../README.ko-KR.md#xdzip-format-support)를 참조하세요. GZIP과 같은 단일 파일 스트림에는 일반 파일 하나를 넣을 수 있습니다.
3. ZIP과 7z는 선택한 옵션이 허용하면 암호화와 분할 볼륨을 지원합니다. 데이터 디스크 이미지에는 ISO를, 자동 압축 풀기 ZIP에는 Windows x64 SFX 프로필을 선택하세요. 작업을 시작하고 결과에서 실제로 저장된 파일을 확인하세요. [암호와 분할 볼륨](passwords-and-volumes.md)
4. 취소하거나 다시 시도해야 하면 결과를 먼저 확인하세요. 완성된 압축 파일이 확정되기 전에는 임시 위치에 기록됩니다.

<a name="xdzip-add-replace-or-move-files"></a>

## 파일 추가, 바꾸기 또는 이동

지원되는 ZIP/ZIPX, 7z, LHA/LZH, TAR 또는 외부 압축을 적용한 TAR 압축 파일을 열고 내부의 대상 폴더로 이동합니다. 로컬 파일이나 폴더를 목록 또는 격자에 끌어다 놓은 뒤 추가할 항목을 확인하고 승인합니다. 이름 충돌 시 바꿀 수 있는 일반 파일은 교체할 수 있습니다. 기존 항목을 옮기려면 같은 압축 파일의 다른 폴더로 끌어 놓고 확인하세요. 창 밖으로 끌면 복사본을 내보냅니다. [플랫폼별 끌어서 놓기](../engineering/archive-transfers.md)

압축 파일을 만들거나 내용을 추가할 때 `.DS_Store` 파일과 `__MACOSX`, `_MACOSX` 디렉터리는 제외합니다. 수정 가능 여부는 형식, 내용, 원본에 따라 달라집니다. 암호화되거나 분할되었거나 디스크 인덱스를 쓰는 압축 파일은 일반적인 방법으로 수정할 수 없습니다. 수정할 수 없으면 새 폴더에 압축을 풀고 새 압축 파일을 만드세요. [수정 및 끌어서 놓기 규칙](../engineering/archive-transfers.md)

<a name="xdzip-rename-or-delete-entries"></a>

## 항목 이름 바꾸기 및 삭제

편집할 수 있는 압축 파일의 일반 폴더에서 목록이나 그리드의 파일 또는 폴더를 마우스 오른쪽 버튼으로 클릭하세요. 선택된 항목을 클릭하면 기존 선택을 유지하고, 선택되지 않은 항목을 클릭하면 해당 항목만 선택합니다.

- **이름 바꾸기**는 항목 하나에 사용할 수 있습니다. 같은 폴더 안에서 새 이름을 입력하세요. 폴더 이름을 바꾸면 그 안의 모든 경로가 바뀝니다. 같은 이름이 이미 있으면 다른 이름을 선택하세요.
- **삭제**는 선택한 항목 하나 이상에 사용할 수 있습니다. 삭제를 확인하기 전에 항목 이름이나 선택한 항목 수를 확인하세요. 폴더를 삭제하면 그 안의 내용도 모두 삭제됩니다. 루트의 항목을 모두 삭제해도 빈 압축 파일은 남습니다. 빈 압축 파일이 독립 압축 스트림으로 열릴 수 있다면 `.tar.xz` 같은 복합 TAR 확장자를 사용하세요.

검색으로 찾은 항목을 편집하려면 해당 항목이 있는 폴더를 먼저 여세요. [압축 파일 수정 규칙](../engineering/archive-transfers.md)
