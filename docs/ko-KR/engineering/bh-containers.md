<a name="xdzip-bh-containers-and-the-admitted-blakhole-profile"></a>

# BH 컨테이너와 허용된 BlakHole 프로필

[English](../../engineering/bh-containers.md) | [简体中文](../../zh-CN/engineering/bh-containers.md) | [繁體中文（台灣）](../../zh-TW/engineering/bh-containers.md) | [日本語](../../ja-JP/engineering/bh-containers.md) | 한국어 | [Français](../../fr-FR/engineering/bh-containers.md) | [Deutsch](../../de-DE/engineering/bh-containers.md) | [Español](../../es-ES/engineering/bh-containers.md) | [Português (Brasil)](../../pt-BR/engineering/bh-containers.md) | [Русский](../../ru-RU/engineering/bh-containers.md)

`xdzip-codec-bh`는 호스트에서 관리하는 소스에서 BlakHole(`.bh`) 항목을 순차적으로 읽습니다. 프로필은 `BH 05 07` 접두부, 관찰된 고정 헤더 상수, 일반 파일 메타데이터, 정확한 압축 및 디코딩 크기, 각 항목에서 선언한 IEEE CRC-32를 요구합니다. 물리적 순서를 보존하므로 같은 이름의 항목도 별개로 유지됩니다. 고정된 행 식별자는 표시 이름 디코딩에 의존하지 않습니다.

항목 내용에 허용된 방식은 RFC 1951의 원시 Deflate인 방식 8뿐입니다. 한도가 있는 공용 Deflate 읽기 구현은 완전한 스트림과 정확한 압축 범위 소비를 요구합니다. 뒤에 남은 바이트, 잘린 스트림, 디코딩 크기 불일치 또는 CRC 불일치가 있으면 작업이 실패합니다. 알 수 없는 방식은 경고와 함께 표시됩니다. 해당 항목은 추출하거나 검증된 내용에 포함할 수 없습니다.

허용된 헤더는 고정된 5바이트 접두부, 리틀 엔디언 16비트 헤더 길이, 고정 바이트 `25 d8 0c 00`, 방식, DOS 형태의 4바이트 시간 기록, 압축 크기, 디코딩 크기, CRC-32, 속성 `0x20`, 0으로 채운 예약 5바이트, 4바이트 파일 이름 길이, 파일 이름 바이트로 구성됩니다. 헤더 길이는 `37 + filename_length`와 같아야 합니다. 항목은 연속되고 마지막 내용은 실제 파일 끝에서 정확히 종료됩니다. 압축 파일 트레일러, 디렉터리 레코드, SFX 접두부, 재동기화, 패딩 및 인접 파일 검색은 허용하지 않습니다.

BH에는 현재 관리되는 공개 사양이 없습니다. 형식 관찰은 `ashang/libxad`의 `support/formats/BLAKHOLE.TXT`(리비전 `dbe67a865207e49cd917ad9170a11b9eaa83998a`, SHA-256 `a77d570bba1ceafc443fa2404dfaccc8d8872a6a`)와 Hawkynt/CompressionWorkbench의 `docs/BLAKHOLE-ON-DISK.md`(커밋 `ab61c59f789d82c628d3c5b4bac264afbdc0c7a7`, SHA-256 `2b7572396683e96f8b5a47ff0b3f0dde2de4b839`)의 고정 버전을 따릅니다. 후자는 출처가 확인되는 항목 다섯 개짜리 샘플 하나입니다. Store, 암호화, 디렉터리, 솔리드 블록, 볼륨 동작을 확인하기에는 부족합니다.

IZArc Command Line Add-On v1.1은 빌드 참조용으로만 보관합니다. 공식 다운로드의 MD5는 `9002c7634768e5336244f9accb2223b0`이며, 로컬 `IZArcCL.exe`의 SHA-256은 `959050AFC513C64FD61F9EE15837054D0FAEE5EFB98C9107B4DC5D9A528A62D4`입니다. 이 환경에서 추출한 `IZARCC.exe`와 `IZARCE.exe`는 모두 출력 전에 Windows 코드 `0xC0000005`로 종료됩니다. 이 도구로 Store, 압축 수준, 암호 또는 압축 풀기 동작을 입증할 수 없습니다. 참조 자료는 `build/toolchain`에 있으며 제품은 이에 의존하지 않습니다.

BH는 읽기 전용입니다. 생성, 수정, 암호화, 다중 볼륨 압축 파일과 Linux 전용 MIME 등록은 지원 범위에서 제외됩니다. 형식 레지스트리와 [압축 파일 검사 계약](archive-checking.md)이 이 경계를 설명합니다. `.bh` 확장자로 탐지를 시작할 수 있습니다. 실제 결과는 전체 콘텐츠 검증에 따릅니다.
