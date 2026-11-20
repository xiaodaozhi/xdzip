<a name="xdzip-wim-msi-and-installer-format-sources-and-licenses"></a>

# WIM, MSI 및 설치 프로그램 형식의 자료와 라이선스

[English](../../engineering/wim-msi-installer-format-sources.md) | [简体中文](../../zh-CN/engineering/wim-msi-installer-format-sources.md) | [繁體中文（台灣）](../../zh-TW/engineering/wim-msi-installer-format-sources.md) | [日本語](../../ja-JP/engineering/wim-msi-installer-format-sources.md) | 한국어 | [Français](../../fr-FR/engineering/wim-msi-installer-format-sources.md) | [Deutsch](../../de-DE/engineering/wim-msi-installer-format-sources.md) | [Español](../../es-ES/engineering/wim-msi-installer-format-sources.md) | [Português (Brasil)](../../pt-BR/engineering/wim-msi-installer-format-sources.md) | [Русский](../../ru-RU/engineering/wim-msi-installer-format-sources.md)

이 문서는 WIM, MSI 및 설치 프로그램 형식을 지원하는 데 필요한 기술 자료, 라이선스 제한과 요구 사항을 설명합니다. 참조 자료는 `build/toolchain/references`에 보관하거나 공개 사양으로 연결하며, 어느 것도 제품 의존성이 아닙니다.

<a name="xdzip-wim"></a>

## WIM

- 참조 구현: `ebiggers/wimlib`, 커밋 `cd5e231c348c255ae5088873b5a66ee0eb96fa07`.
- 저장소는 프로젝트에 GPLv3+를 제공하며 라이브러리 부분에는 조건부 LGPL-2.1+ 선택지도 제공합니다. LGPL 선택은 `libntfs-3g` 같은 타사 의존성이 GPL 조건을 강제하지 않는다는 전제에 달려 있습니다.
- WIM은 일반 압축 파일이 아니라 이미지 컨테이너입니다. 제한이 있는 판독기는 리소스 테이블, 메타데이터 스트림, 청크 단위 리소스, Xpress/LZX/LZMS 압축, 무결성 테이블, 하드 링크 메타데이터와 이미지 선택을 처리해야 하며 마운트나 보안 권한 복원은 하지 않아야 합니다.

참조 구현은 형식과 테스트 자료 연구에 유용하지만 현재 타사 런타임을 도입하지 않는 정책에 따라 xdzip에 연결할 수 없습니다. WIM은 **제품 결정으로 지원 대상에서 제외**됩니다. 프로젝트는 지원에 필요한 이미지 전용 판독기, 압축 해제 증폭 모델, 무결성 리소스, 하드 링크 의미와 완전한 테스트 자료를 수용하지 않습니다. 이 결정을 명시적으로 재검토하지 않는 한 레지스트리, MIME, 런타임 또는 CLI 항목을 추가할 수 없습니다.

<a name="xdzip-compound-file-and-msi"></a>

## Compound File과 MSI

- Microsoft [MS-CFB]가 주요 공개 사양입니다: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/53989ce4-7b05-4f8d-829b-d08d6148375b . 이 페이지는 PDF/DOCX 개정판과 설명된 기술의 구현을 허용하는 지식 재산권 고지를 제공합니다. 특허 라이선스는 해당 Microsoft 프로그램의 적용을 받습니다.
- CFB 판독기는 MSI 고유 내용을 해석하기 전에 섹터와 FAT/DIFAT 할당, 디렉터리 적흑 트리 순회, 미니 스트림 할당, 스트림 크기 제한과 순환 감지를 처리해야 합니다.

독립적인 CFB 메타데이터 판독기는 기술적으로 가능하며 현재 `xdzip-codec-cfb`의 읽기 전용 부분으로 구현되어 있습니다. 저장소와 스트림 행을 공개하기 전에 512바이트와 4096바이트 섹터 기준을 포함한 CFB 3.0/4.0 헤더, FAT/DIFAT 체인, 디렉터리 트리, 미니 스트림 할당, 스트림 크기 제한과 순환을 검증합니다. 원시 스트림 복사는 불변 메타데이터 스냅숏이 일치한 후에만 가능합니다. MSI 테이블, 내장 cabinet, 사용자 지정 동작과 변환은 계약 범위 밖입니다. 설치 프로그램을 실행하거나 사용자 지정 동작을 해석하거나 패키지를 설치하거나 내용을 다시 쓰지 않으며, `.msi` 확장자는 실행 권한을 부여하지 않습니다. 테스트 자료와 런타임 증거는 코덱 및 런타임 테스트에서 자동화되어 있습니다.

<a name="xdzip-nsis-and-other-sfx"></a>

## NSIS와 기타 SFX

- NSIS 참조 소스: `kichik/nsis`, 커밋 `a96303eae3115221471bb127fa916a13d8bf9ecd`.
- `COPYRIGHT`는 대부분의 소스와 zlib 압축기에 zlib/libpng, bzip2 모듈에 bzip2, LZMA 모듈에 Common Public License 1.0을 명시합니다.
- NSIS 파일은 스크립트와 내장 데이터를 포함한 실행 가능 설치 프로그램입니다. 파서는 실행 경계 밖에 머물고 헤더와 데이터 구간을 제한하며 압축 방법과 설치 프로그램 동작을 분리해야 합니다.
- 공개 소스는 고정된 `firstheader` 레코드를 정의합니다. 여기에는 `flags`, `0xDEADBEEF`, `NullsoftInstallSystem` 서명 워드, `length_of_header`, `length_of_all_following_data`가 포함됩니다. 정적 판독기는 PE 스텁을 열지 않고 이 레코드를 찾아 검증할 수 있지만 압축된 헤더 블록과 완전한 자체 설치 프로그램을 위한 독립 모델은 여전히 필요합니다.

NSIS는 현재 **정적 메타데이터 전용 프로필로 허용**됩니다. 코덱은 제한된 PE 오버레이에서 512바이트 경계에 맞춘 레코드만 검사하고 기록된 두 `firstheader` 매직 값을 받아들이며 플래그와 길이를 제한하고 선언된 후속 구간이 실제 입력 안에 들어가는지 확인합니다. 그다음 `NSIS/firstheader` 메타데이터 행 하나를 공개합니다. 설치 프로그램 헤더를 압축 해제하거나 내장 데이터 바이트를 노출하지 않습니다. `.exe` 레지스트리 항목은 의도적으로 끌어 놓기 후보나 셸 연결에 포함하지 않으며 콘텐츠 감지 결과를 따릅니다. 파서는 Qt와 설치 프로그램 실행 API로부터 독립적입니다. PE 스텁, 스크립트, 사용자 지정 페이지, 플러그인 또는 추출된 데이터는 절대 실행하지 않습니다. 다른 SFX 형식은 내부 압축 파일이 콘텐츠 검증을 통과할 때만 별칭으로 유지됩니다.

<a name="xdzip-admission-gates"></a>

## 허용 조건

1. 참조 도구를 사용하기 전에 소스 리비전과 라이선스 문서 해시를 고정합니다.
2. 허용되는 각 컨테이너와 압축 방법에 대해 완전하고 재배포 가능한 테스트 자료를 만듭니다. 헤더만 있는 샘플은 충분하지 않습니다.
3. CFB/WIM/NSIS 파싱은 Qt와 설치, 마운트, 레지스트리 또는 스크립트 실행 API로부터 독립시킵니다.
4. 정확한 물리 ID, 중복 항목, 순환 제한, 누적 입력, 작업, 출력 제한, 취소, 원본 변경과 최종 실패 테스트에 통과한 후에만 런타임, CLI와 데스크톱 항목을 추가합니다.
5. 별도의 의존성 및 라이선스 결정으로 제품 정책을 명시적으로 변경하지 않는 한 GPL/LGPL/CPL 참조 구성 요소는 build/toolchain에 보관합니다.
