<a name="xdzip-iso9660-and-joliet-creation"></a>
# ISO9660 및 Joliet 이미지 만들기

[English](../../engineering/iso-creation.md) | [简体中文](../../zh-CN/engineering/iso-creation.md) | [繁體中文（台灣）](../../zh-TW/engineering/iso-creation.md) | [日本語](../../ja-JP/engineering/iso-creation.md) | 한국어 | [Français](../../fr-FR/engineering/iso-creation.md) | [Deutsch](../../de-DE/engineering/iso-creation.md) | [Español](../../es-ES/engineering/iso-creation.md) | [Português (Brasil)](../../pt-BR/engineering/iso-creation.md) | [Русский](../../ru-RU/engineering/iso-creation.md)

첫 번째 쓰기 모듈은 2048바이트 섹터, ISO9660 기본 이름 공간과 Joliet 보조 이름 공간을 갖춘 부팅 불가 단일 데이터 이미지를 만듭니다. 부록 C에 실린 Joliet 사양을 비롯하여 [ECMA-119 제5판](https://ecma-international.org/wp-content/uploads/ECMA-119_5th_edition_december_2024.pdf)을 따릅니다. 외부 이미지 제작 엔진을 링크하거나 실행하지 않습니다.

일반 파일과 명시된 폴더만 허용합니다. Joliet 이름 공간은 이름을 빅 엔디언 UCS-2로 그대로 보존하며 구성 요소별 최대 64자까지 지원합니다. 파일 버전 바이트와 상위 폴더 개수를 포함한 전체 인코딩 Joliet 경로는 최대 240바이트입니다. 보충 평면 문자, 세미콜론, 안전하지 않은 이식 가능 경로, 중복 경로, 없는 상위 폴더, 파일과 폴더의 충돌은 쓰기 전에 거부합니다. 기본 식별자는 고유하고 결정적으로 생성한 ASCII 별칭입니다. Joliet 보기에서는 원래 이름을 유지합니다. 폴더 깊이는 루트를 포함하여 8단계, 폴더 개수는 루트를 포함하여 65535개, 파일 크기는 4 GiB 미만, 전체 이미지 크기는 20 GiB, 계획된 메타데이터는 64 MiB로 제한합니다. 시간 기록은 UTC를 사용하고 디스크에 표현할 수 있는 범위로 제한합니다.

파일 내용을 스트리밍하기 전에 두 폴더 트리와 리틀 엔디언 및 빅 엔디언 경로 테이블을 계획합니다. 폴더 레코드는 섹터 경계를 넘지 않습니다. 각 섹터는 0으로 채우며 두 수치 표현은 일치합니다. 고정된 입력 인덱스는 스트림에만 접근할 수 있게 하며 파일 시스템 경로에 대한 권한을 주지 않습니다. 취소, 입력 길이 부족 또는 증가, 출력 오류, 자원 한도 초과가 발생하면 인코딩을 중단합니다. 임시 출력 파일은 호스트만 최종 저장할 수 있습니다.

최종 저장 전에 완성된 이미지를 다시 열고, 열거 순서와 관계없이 디코딩한 Joliet 이름으로 레코드를 대응시키고, 디코딩된 파일 바이트를 고정된 원본과 비교합니다. 임의 ISO 이미지의 구조 검사는 이 비교와 별개입니다. ISO는 일반 파일의 체크섬을 제공하지 않습니다. 독립 읽기 모듈은 기본 별칭, 정확한 Joliet 이름 및 파일 바이트를 모두 확인해야 합니다.

UDF, Rock Ridge 쓰기, El Torito 및 부팅 이미지, 압축, 암호, 다중 볼륨 및 기존 이미지 수정은 지원하지 않습니다. CLI와 데스크톱은 동일한 생성 레지스트리와 작업 경로를 사용합니다.
