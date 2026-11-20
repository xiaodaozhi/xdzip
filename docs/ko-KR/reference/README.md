<a name="xdzip-product-design-and-technical-reference"></a>

# 제품 설계 및 기술 참조

[English](../../reference/README.md) | [简体中文](../../zh-CN/reference/README.md) | [繁體中文](../../zh-TW/reference/README.md) | [日本語](../../ja-JP/reference/README.md) | 한국어 | [Français](../../fr-FR/reference/README.md) | [Deutsch](../../de-DE/reference/README.md) | [Español](../../es-ES/reference/README.md) | [Português (Brasil)](../../pt-BR/reference/README.md) | [Русский](../../ru-RU/reference/README.md)

데스크톱과 CLI 실행 경로는 [시스템 아키텍처](../design/system-architecture.md)를 참조하세요. 아래 절에서 각 동작을 정의하는 계약을 찾을 수 있습니다. [지원 형식 표](../../../README.ko-KR.md#xdzip-format-support)는 지원 작업을 요약합니다. 사용 절차나 개발 환경 준비는 [사용자 설명서](../user-guide/README.md), [CLI 예제](../integration/README.md), [기여 안내](../contributing/README.md)를 참조하세요.

<a name="xdzip-architecture-and-interfaces"></a>

## 아키텍처와 인터페이스

- [설계 개요](../design/index.md)
- [제품 요구 사항 및 작업 범위](../design/product-requirements.md)
- [도메인 모델, 항목 식별자 및 소스 버전](../design/domain-data.md)
- [Qt/Rust 명령 및 이벤트 브리지](../design/qt-rust-bridge.md)
- [작업 스케줄링, 진행, 취소 및 종료](../design/task-scheduling.md)
- [성능 한도 및 검증](../design/performance.md)
- [용어 및 구현 제한](../design/glossary.md)
- [요구 사항 및 검증 추적](../design/requirements-traceability.md)
- [아키텍처 결정](../design/architecture-decisions.md)
- [프로토타입에서 데스크톱과 Rust 실행 환경까지](../design/architecture-evolution.md)
- [저장소 구조와 의존성](../engineering/repository-layout.md)
- [인터페이스 계약과 검사](../engineering/contracts.md)
- [제품 실행 환경과 Qt 수명 주기](../engineering/product-runtime.md)
- [앱 세션과 종료](../engineering/application-sessions.md)
- [cxx 브리지 선택 이유](../engineering/cxx-admission.md)
- [형식 작업 프로세스용 호스트 달력 및 인코딩 서비스](../engineering/host-platform-services.md)

<a name="xdzip-reading-and-writing-archives"></a>

## 압축 파일 읽기와 쓰기

- [파일 권한, 임시 출력 및 안전한 최종 저장](../design/file-security.md)
- [읽기 전용 탐색 구현](../engineering/readonly-browser.md)
- [읽기 전용 ZIP 탐색](../engineering/zip-readonly-browsing.md)
- [대용량 압축 파일의 인덱싱과 필요 시 탐색](../engineering/large-index.md)
- [분할 볼륨 읽기 및 임시 열기](../engineering/archive-opening.md)
- [파일 이름 인코딩 자동 판별](../engineering/filename-encoding.md)
- [내용 미리보기](../engineering/content-preview.md)
- [압축 풀기 전 메타데이터 검사](../engineering/extraction-preflight.md)
- [안전한 ZIP 압축 풀기 구현](../engineering/zip-safe-extraction.md)
- [압축 파일 무결성 검사](../engineering/archive-checking.md)
- [항목 열기, 끌어서 놓기 및 압축 파일 수정](../engineering/archive-transfers.md)
- [압축 파일 암호와 암호화된 디렉터리 열기](../engineering/password-extraction.md)
- [ZIP 생성 계약](../engineering/zip-creation.md)
- [ISO9660 및 Joliet 생성](../engineering/iso-creation.md)
- [Windows용 내장 ZIP 자동 압축 풀기 파일](../engineering/sfx-creation.md)

<a name="xdzip-format-support-and-implementation"></a>

## 형식 지원 및 구현

- [내장 기능, 생성 프로필 및 검사 범위](../engineering/format-registry.md)
- [형식 어댑터 및 외부 SDK 시험 설계](../design/codec-extension-sdk.md)
- [읽기 전용 외부 SDK 시험 및 실행 범위](../engineering/sdk-trial.md)
- [격리된 형식 작업 프로세스 계약](../engineering/native-codecs.md)
- [자체 Rust 형식 및 알고리즘 계약](../engineering/independent-rust-codecs.md)
- [보조 디코딩 작업량 계산](../engineering/auxiliary-decoding-work.md)
- [ZIP 메타데이터 의존성 허용](../engineering/zip-metadata-admission.md)
- [ZIP 압축 풀기 및 출력 의존성 허용](../engineering/zip-extraction-admission.md)
- [7z 형식 출처 및 독립 구현](../engineering/sevenzip-format-sources.md)
- [RAR 독립 구현의 근거 및 호환성 경계](../engineering/rar-format-sources.md)
- [CAB 형식 출처 및 구현 근거](../engineering/cab-format-sources.md)
- [TAR/GZIP 구현 계약](../engineering/tar-gzip.md)
- [단일 파일 스트림과 ZIP 파생 패키지](../engineering/single-file-streams.md)
- [LZ4 스트림 계약](../engineering/lz4-streams.md)
- [Zstandard 스트림 계약](../engineering/zstandard-streams.md)
- [Unix compress Z 계약](../engineering/compress-streams.md)
- [일반 Brotli 스트림 계약](../engineering/brotli-streams.md)
- [LHA 데이터 구조 및 알고리즘](../engineering/lha-containers.md)
- [PMA 컨테이너와 PMarc 방식](../engineering/pma-streams.md)
- [ARJ 압축 스트림과 컨테이너 허용 범위](../engineering/arj-streams.md)
- [ARC 컨테이너 및 압축 스트림 프로필](../engineering/arc-streams.md)
- [BH 컨테이너와 허용된 BlakHole 프로필](../engineering/bh-containers.md)
- [ACE, ALZ 및 EGG의 출처와 라이선스 조건](../engineering/ace-alz-egg-format-sources.md)
- [읽기 전용 AES Crypt 엔벌로프](../engineering/aescrypt-envelopes.md)
- [DEB 컨테이너 계약](../engineering/deb-containers.md)
- [ASAR 컨테이너 계약](../engineering/asar-containers.md)
- [UU, UUE 및 XXE 텍스트 엔벌로프](../engineering/text-envelopes.md)
- [WIM, MSI 및 설치 프로그램 형식의 출처와 라이선스](../engineering/wim-msi-installer-format-sources.md)
- [디스크 이미지와 기타 압축 형식의 출처 및 라이선스](../engineering/disk-image-archive-format-sources.md)
- [Bandizip 형식 목록과 xdzip 적용 가능성](../engineering/bandizip-format-assessment.md)

<a name="xdzip-desktop-interaction-and-visual-design"></a>

## 데스크톱 상호 작용 및 시각 디자인

- [Qt Quick 화면과 컨트롤러 책임](../design/qt-quick-ui.md)
- [데스크톱 시각 사양](../visual/README.md)
- [UI 디자인 표준](../visual/design-standard.md)
- [UI 디자인 및 수정 절차](../visual/design-workflow.md)
- [파일 연결 아이콘](../visual/file-association-icons.md)
- [HTML 시각 일치 검토](../visual/html-fidelity.md)
- [추가 상호 작용 및 문구 사양](../ux/interaction-spec.md)
- [기본 창의 기능, 구성 요소 및 상호 작용](../ux/main-window-components.md)
- [디렉터리 트리 디자인 및 통합](../ux/directory-tree.md)
- [기본 창의 시작 페이지](../ux/welcome.md)
- [시작 페이지의 로딩 오버레이와 생성 완료](../ux/welcome-opening.md)
- [압축 풀기 암호 입력](../ux/password-extraction.md)
- [작업 창과 무결성 검사](../ux/task-windows.md)
- [설정 창과 인터페이스 언어](../ux/settings-localization.md)
- [중국어 간체 UI 용어](../ux/chinese-terminology.md)
- [시스템 메뉴와 알림 영역](../ux/desktop-integration.md)
- [플랫폼 창 및 제목 표시줄](../ux/platform-windows.md)
- [고해상도 디자인 상태 및 구현 계약](../ux/hifi-state-spec.md)
- [v0.1 상호 작용 프로토타입과 기능 매핑](../ux/prototype-spec.md)

<a name="xdzip-platform-integration-releases-and-trust"></a>

## 플랫폼 통합, 릴리스 및 신뢰

- [설치 프로그램과 파일 연결](../engineering/installers-associations.md)
- [시스템 지원 및 유지보수 정책](../engineering/support-policy.md)
- [파일 관리자 동작과 독립 작업 호스트](../engineering/shell-actions.md)
- [사용자가 시작하는 업데이트 계약](../engineering/software-update.md)
- [데이터 수명 주기 및 진단](../engineering/privacy.md)
- [제품 버전 및 빌드 식별자](../engineering/versioning.md)
- [GitHub Release 프로토콜 v1](../engineering/release-protocol.md)
- [macOS 로컬 개발용 DMG](../engineering/macos-deployment.md)
- [Linux 독립 실행 디렉터리 및 검증](../engineering/linux-deployment.md)
- [배포 의존성 검증 및 SBOM](../engineering/deployment-audit.md)
- [실제 구성 요소의 저작권 표시와 라이선스 자료](../engineering/runtime-attribution.md)
- [네이티브 실행 구성 요소의 출처 및 고지](../engineering/native-runtime-materials.md)
- [Qt SDK와 소스의 일치 검증](../engineering/qt-sdk-correspondence.md)
- [Windows 개발 도구 및 실행 구성 요소 라이선스](../engineering/windows-licensing.md)
- [라이선스 결정과 타사 자료](../engineering/licensing.md)
- [의존성 및 형식 백엔드 허용](../engineering/dependencies.md)
- [대응 소스 및 재빌드 자료](../engineering/source-delivery.md)
- [CLDR에서 ICU 73.2 데이터 다시 빌드](../engineering/icu-cldr-rebuild.md)

<a name="xdzip-build-and-cli-reference"></a>

## 빌드 및 CLI 참조

- [빌드, 배포 및 릴리스 설계](../design/build-release.md)
- [코딩 규칙 및 검증 계층](../design/quality-engineering.md)
- [개발 환경 준비](../engineering/development.md)
- [개발 빌드 및 플랫폼 검증](../engineering/developer-build.md)
- [도구 체인 및 플랫폼 버전 고정](../engineering/toolchain.md)
- [CLI 명령, JSON, 설치 및 업데이트 영수증](../engineering/command-line.md)
