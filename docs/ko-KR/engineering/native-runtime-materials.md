<a name="xdzip-native-runtime-provenance-and-notices"></a>

# 네이티브 런타임 출처와 고지

[English](../../engineering/native-runtime-materials.md) | [简体中文](../../zh-CN/engineering/native-runtime-materials.md) | [繁體中文（台灣）](../../zh-TW/engineering/native-runtime-materials.md) | [日本語](../../ja-JP/engineering/native-runtime-materials.md) | 한국어 | [Français](../../fr-FR/engineering/native-runtime-materials.md) | [Deutsch](../../de-DE/engineering/native-runtime-materials.md) | [Español](../../es-ES/engineering/native-runtime-materials.md) | [Português (Brasil)](../../pt-BR/engineering/native-runtime-materials.md) | [Русский](../../ru-RU/engineering/native-runtime-materials.md)

이 계약은 실제 패키지에 포함된 Qt 이외의 네이티브 런타임을 다루며 [런타임 귀속](runtime-attribution.md)과 [대응 소스](source-delivery.md) 설명과 함께 사용합니다. 소스 버전, 파일 식별, 라이선스 조건과 실제 이행을 각각 확인합니다. 공급업체 파일 해시는 라이선스 승인이 아닙니다. `v0.1.0` 후보의 과거 결론과 보충 증거는 [#4](https://github.com/xiaodaozhi/xdzip/issues/4)와 [#53](https://github.com/xiaodaozhi/xdzip/issues/53)에 있습니다. 이후 후보에는 새로운 릴리스 이슈를 사용하며 증거는 build/ 또는 CI 산출물에만 생성합니다.

| 구성 요소 | 원래 근거와 전달 자료 | 검토 경계 |
| --- | --- | --- |
| Qt 6.8.2 | SDK SPDX, 원본 모듈 소스와 패치, Qt 고지, LGPLv3 및 GPLv3 전문, 앱 재빌드와 호환 교체 지침 | 런타임은 LGPL 선택지를 사용하며 도구와 타사 라이선스는 별도입니다. SDK 파일 대응 관계는 상위 빌드와의 완전한 동등성이 아닙니다 |
| Rust 런타임 코드 | Cargo.lock, 실제 링크 입력, 런타임 의존성 후보, 완전한 vendor 및 원본 고지, Rust 표준 라이브러리 소스 | 빌드 그래프의 고지를 보수적으로 전달하되 build/dev/proc-macro 도구를 링크된 런타임 코드로 잘못 보고하지 않습니다 |
| Noto Sans SC, TC, JP, KR 및 Noto Sans 파생 글꼴 | 글꼴 매니페스트, 원본 OFL 전문, 이름을 바꾼 글꼴, 생성 진입점 | 프로젝트 Apache-2.0은 글꼴에 적용되지 않으며 변경과 명명 설명을 보존합니다 |
| ICU 73.2 | 실제 라이브러리 버전, 타사 고지를 포함한 상위 ICU LICENSE 전문, 출처 기록 | 일반 MIT 텍스트로 ICU 자체 고지를 대체하지 않고 시스템 래퍼를 실제 패키지 파일과 혼동하지 않습니다 |
| Microsoft VC++ 런타임 | 고정된 공식 VC_redist 패키지와 DLL 해시, 원본 라이선스, 버전 및 배포 목록 | 파일은 수정하지 않고 유지 관리자의 자격과 재배포 조건은 별도로 검토합니다 |
| Windows SDK 그래픽 컴파일러 라이브러리 | 새 패키지에서 제외하며 과거 패키지에는 원본 SDK 라이선스, DLL별 해시와 출처를 보존합니다 | 과거 배포 조건은 별도로 검토하며 새 패키지에서 제거해도 바뀌지 않습니다 |
| Inno Setup 6.7.3 설치 엔진 | 고정된 컴파일러 패키지와 소스 커밋, 원본 Inno 및 RemObjects Pascal Script 라이선스, 이중 언어 Installer-CREDITS.txt | 기존 설치 엔진 저작권 UI를 보존합니다. Pascal Script는 정보 화면 또는 문서에 이름과 소스 URL을 별도로 요구하며 일반 Inno 라이선스만으로는 부족합니다 |

Microsoft의 [런타임 재배포 지침](https://learn.microsoft.com/en-us/cpp/windows/redistributing-visual-cpp-files)과 [VS 2022 재배포 목록](https://learn.microsoft.com/en-us/visualstudio/releases/2022/redistribution)은 재배포 권리를 적용되는 소프트웨어 라이선스에 연결합니다. 설치 프로그램의 라이선스 텍스트는 유지 관리자의 자격을 증명하지 않습니다. Qt의 [오픈 소스 의무](https://www.qt.io/development/open-source-lgpl-obligations)는 사용자가 대응 소스를 얻고 수정하고 다시 링크하며 수정된 라이브러리를 실행할 권리를 보존하도록 요구합니다. 동적 링크는 다른 의무를 대신하지 않습니다.

무료 Community 경로, 별도의 호스팅 Enterprise 권한과 재배포 조건은 [Windows 라이선스](windows-licensing.md)를 참고하세요. 배포 시 원본 CRT/SDK 라이선스, 출처와 이중 언어 Windows-runtime-NOTICE.txt를 런타임과 자료 양쪽에 포함합니다. 컴파일 전에 Setup은 원본 CRT 계약의 해시를 확인하고 라이선스 페이지를 표시합니다. Microsoft DLL은 Apache-2.0으로 재라이선스하지 않습니다.

<a name="xdzip-generation-and-verification"></a>

## 생성과 검증

`scripts/native-runtime-sources.json`은 ICU 라이선스 버전, 공식 URL과 SHA-256을 고정합니다. `collect-notices.py`는 Linux에서 ICU 라이선스 전문을 수집하고 캐시와 다운로드의 해시를 모두 다시 확인합니다. 현재 Windows 개발 패키지는 `opengl32sw.dll`을 제외하고 해당 파일의 Mesa/LLVM 고지도 더 이상 수집하지 않습니다. 고지 출력 디렉터리에 오래된 전용 자료가 있으면 수집은 재사용을 거부하여 새 후보에 잘못 추가되지 않도록 합니다.

`prepare-source-delivery.py`는 ICU 라이선스와 고정된 소스 목록을 포함합니다. 자료는 원래 바이트를 유지하고 전달 매니페스트는 크기와 해시를 다시 계산합니다. Mesa 소프트웨어 렌더링 라이브러리를 포함한 과거 패키지에는 대응하는 과거 소스, 고지와 의무가 여전히 필요합니다. 새 정책으로 과거 전달을 바꿀 수 없습니다.

플랫폼 자료 첨부에는 귀속 보고서의 `license_materials_base`가 참조하는 디렉터리가 포함되어야 합니다. 패키징 전에 각 원본 라이선스 텍스트의 해시를 다시 확인합니다. 런타임 디렉터리 다른 곳에 동일한 텍스트가 있어도 보고서가 없는 상대 디렉터리를 가리키는 것을 정당화하지 못합니다.

Windows 배포는 소프트웨어 OpenGL, D3D 컴파일러와 DXC에 대한 windeployqt 환경 검색을 비활성화하고 Windows SDK에서 세 그래픽 컴파일러 라이브러리를 더 이상 복사하지 않습니다. 이러한 이름이 들어 있는 준비 영역은 거부합니다. 관리되는 준비 영역의 루트에 남은 소프트웨어 OpenGL DLL은 제거하고 다른 위치의 같은 이름 잔여물은 거부합니다.

macOS 패키징은 최종 서명된 앱이 아직 존재할 때 `deployment-audit.json`, `macos-deployment.json`, `runtime-attribution.json`을 생성합니다. 파일 SBOM은 같은 감사 및 귀속 보고서를 사용합니다. Qt 네이티브 파일은 arm64 슬라이스, 모든 섹션 바이트, UUID와 가져오기 버전으로 비교하며 배포에 필요한 Qt 경로 재배치를 허용합니다. 서명, 로드 명령 배치와 LINKEDIT는 이 비교 결론에 포함되지 않습니다. 비네이티브 리소스는 정확한 바이트로 연결합니다. 연결되지 않은 파일과 원본 SPDX의 빈틈은 남으며 이 절차는 완전한 바이너리 동등성이 아닙니다.

게시 후 누락된 고지를 발견한 경우 보충 자료가 원래 패키지에 이미 포함되었다고 설명할 수 없습니다. 원래 버전 자산과 체크섬은 변경하지 않으며 수정 패키지는 새 버전과 재검증이 필요합니다. 보충 자료와 그 연결 대상을 명시적으로 기록합니다.
