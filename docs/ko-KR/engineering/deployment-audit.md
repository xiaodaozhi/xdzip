<a name="xdzip-deployment-dependency-verification-and-sbom"></a>

# 배포 의존성 검증과 SBOM

[English](../../engineering/deployment-audit.md) | [简体中文](../../zh-CN/engineering/deployment-audit.md) | [繁體中文（台灣）](../../zh-TW/engineering/deployment-audit.md) | [日本語](../../ja-JP/engineering/deployment-audit.md) | 한국어 | [Français](../../fr-FR/engineering/deployment-audit.md) | [Deutsch](../../de-DE/engineering/deployment-audit.md) | [Español](../../es-ES/engineering/deployment-audit.md) | [Português (Brasil)](../../pt-BR/engineering/deployment-audit.md) | [Русский](../../ru-RU/engineering/deployment-audit.md)

배포 검증은 기본적으로 `incomplete`로 명시된 개발 검토 자료를 생성하며 릴리스 준비가 완료되었음을 자동으로 입증하지 않습니다. 공식 Release는 동일 후보의 자료를 별도로 수집하고 릴리스 절차 검토 후 패키지와 함께 제공합니다. 형식은 [공식 CycloneDX 1.6 스키마](https://github.com/CycloneDX/specification/blob/55343ba19dee1785acf1ce9191540d5fd7b590db/schema/bom-1.6.schema.json)를 사용합니다. 상위 커밋과 세 스키마 파일의 SHA-256은 [설정](../../../scripts/sbom-schemas.json)에 고정되어 있습니다. 검증은 이 리소스만 다운로드하여 분석하고 build/에 캐시하며 SBOM의 임의 URL에서 스키마를 다운로드하지 않습니다.

| 산출물 | 데이터 출처와 목적 | 입증할 수 없는 사항 |
| --- | --- | --- |
| source-build.cdx.json | 개발 및 빌드 도구를 포함한 Cargo의 호스트 기준 작업 공간 의존성 그래프, 선언된 Qt 모듈, 글꼴과 브랜드 소스 해시 | Rust 패키지가 반드시 제품에 정적으로 링크되었는지, 모든 Qt 플러그인과 내장 타사가 식별되었는지 |
| deployment-audit.json | 지정한 실제 산출물 루트의 파일 SHA-256, 네이티브 바이너리 가져오기, SDK 및 시스템 및 미해결 의존성, 실제 CLI 빌드 식별 | 이름으로 동적 로드되는 모든 QML이나 플러그인, 깨끗한 설치, 시스템 ABI, 서명 또는 라이선스 준수 |
| artifact-files.cdx.json | 실제 산출물과 관찰된 외부 SDK 파일, 네이티브 가져오기의 알려진 관계, 소스 목록과 실제 실행 파일 버전 및 커밋 및 dirty 상태에 연결 | 배포된 구성 요소의 완전한 식별, 정적 링크 귀속, Qt 내장 라이브러리 라이선스 또는 완성된 공식 제품 SBOM |
| 선택적 데스크톱 기록 | 변경과 사용 가능한 환경에 따라 실행한 데스크톱 검사와 제한 | 실행하지 않은 동작, 다른 플랫폼 또는 릴리스 승인 |

소스 기록은 Cargo의 경로 기반 불투명 ID를 포함하지 않으며 작업 공간 패키지에 가짜 crates.io purl을 만들지 않습니다. 산출물 경로는 지정 루트 또는 Qt SDK 기준 상대 경로입니다. 다른 외부 의존성은 비공개 SDK 절대 경로 없이 콘텐츠 해시와 파일 이름으로 식별합니다. 글꼴은 OFL-1.1을 사용하고 원본 브랜드 소스는 자산 라이브러리의 Apache-2.0을 유지합니다. Qt 모듈 선언만으로 실제 배포된 모든 구성 요소의 라이선스를 추론하지 않습니다.

<a name="xdzip-automated-verification"></a>

## 자동화 검증

`deployment_audit.py`는 명시적으로 지정된 신뢰할 수 있는 프로젝트 빌드만 검사하며 사용자 아카이브는 검사하지 않습니다. macOS는 가져오기와 RPATH에 otool을, Windows는 일반 및 지연 DLL 가져오기에 MSVC dumpbin을, Linux는 ldd를 사용합니다. ldd는 출처를 알 수 없는 실행 파일에 사용해서는 안 됩니다. 지정된 산출물 루트를 벗어나는 파일 심볼릭 링크는 검증을 실패시킵니다. 보고서가 자기 출력을 목록에 포함하지 않도록 루트 밖에 기록해야 합니다.

- `--mode sdk`는 외부 Qt SDK로의 해석을 허용하고 빌드가 독립적이지 않음을 명시합니다. 미해결 가져오기는 여전히 실패합니다. 재귀는 산출물과 선언된 Qt SDK를 대상으로 하며 다른 외부 라이브러리는 해시와 탐색하지 않은 수만 기록하고 의존성 폐쇄가 검증된 것으로 보지 않습니다.
- `--mode deployed`는 시스템에 속하지 않는 외부 의존성과 미해결 가져오기를 허용하지 않습니다. Windows의 MSVCP와 VCRUNTIME 같은 VC++ 가져오기는 패키지에서 제공되어야 하며 개발 컴퓨터의 System32로 누락을 감출 수 없습니다. 시스템 UCRT와 API 계약은 계속 시스템에서 제공합니다. Qt 개발 검색 경로를 제거한 상태로 네이티브 시작 스모크 테스트도 수행합니다.
- 미해결 가져오기, 소스와 실제 바이너리의 버전 및 커밋 및 dirty 상태 불일치, 공식 스키마 해시나 형식 불일치, 중복 또는 대상 없는 구성 요소 참조는 해당 검사를 차단합니다.
- 네이티브 가져오기 폐쇄는 이름으로 로드되는 플러그인 범위나 깨끗한 컴퓨터 검사를 대신하지 않습니다. 배포된 Windows 디렉터리의 가져오기가 통과해도 설치 프로그램 수락은 아닙니다.

필요할 때 실행하는 세 플랫폼 Qt 워크플로는 루트의 cdx.json 두 개와 deployment-audit.json을 저장하며 빈 수동 양식을 자동 생성하지 않습니다. macOS는 로컬에서 빌드한 .app과 SDK만 읽습니다. 일반 검사는 macdeployqt를 실행하거나 앱 패키지에 서명, 압축, 업로드하지 않습니다. Linux는 [독립 런타임 디렉터리](linux-deployment.md)를 통해 Qt/QML/플러그인을 제공하고 deployed 검증과 SDK 없는 컨테이너를 사용합니다. 다운로드한 tar.gz는 실행 비트를 유지합니다. 현재 Ubuntu 24.04, x86_64, X11 기준만 선언하며 범용 Linux 설치 프로그램이라고 하지 않습니다.

신뢰할 수 있는 프로젝트 소유 Windows 산출물은 설치 프로그램을 별도로 추출하고 `python scripts/check-windows-package-payload.py --installer-app <extracted-app-directory> --portable <portable-package.zip> --output build/review/payload-comparison.json`으로 그 앱 디렉터리와 같은 후보의 포터블 ZIP을 비교합니다. 파일별 해시, 설치 매니페스트와 포함해서는 안 되는 SDK 그래픽 DLL을 비교합니다. 출력 경로는 새로워야 합니다. 이 정적 비교는 설치 프로그램이나 업데이트를 실행하지 않으며 실제 설치 검증을 대신하지 않습니다.

일관된 소스에서 다시 빌드한 후 로컬 macOS 예제:

```sh
python3 scripts/collect-notices.py build/review/wp12/notices
python3 scripts/deployment_audit.py \
  --root build/ui/apps/desktop/xdzip.app \
  --app build/ui/apps/desktop/xdzip.app/Contents/MacOS/xdzip \
  --qt-prefix build/toolchain/Qt/6.8.2/macos --mode sdk \
  --output build/review/wp12/deployment-audit.json
python3 scripts/sbom.py \
  --audit build/review/wp12/deployment-audit.json \
  --source build/review/wp12/notices/source-build.cdx.json \
  --output build/review/wp12/artifact-files.cdx.json
```

격리된 검증 환경에 jsonschema==4.23.0을 설치한 후 그 환경의 Python으로 위 두 cdx.json 파일을 지정하여 `scripts/validate-sbom.py`를 실행합니다. 레지스트리 사용법은 [jsonschema 4.23 문서](https://python-jsonschema.readthedocs.io/en/v4.23.0/referencing/)를 참고하세요. 검증 도구는 제품에 배포하지 않습니다.

<a name="xdzip-attribution-results-and-verification-limits"></a>

## 귀속 결과와 검증 한계

[실제 구성 요소 귀속](runtime-attribution.md)이 통합되어 있습니다. SDK SPDX는 실제 파일, Qt 모듈과 내장 타사를 연결하며 고정 해시의 공식 라이선스 텍스트를 전달하고 SDK와 상위 소스의 해시 차이 및 확인되지 않은 귀속을 기록합니다. 일반 Rust 런타임 후보는 빌드, 개발, proc-macro 도구와 분리합니다. 새 구성 요소 관계는 실제 감사에 연결되어 산출물 SBOM에 병합되지만 SBOM은 명시적으로 미완성 상태입니다.

설치된 트리에서 제공된 각 DLL, 프레임워크와 플러그인을 상위 구성 요소, 버전, 라이선스와 대응 소스에 연결합니다. Qt 내장 구성 요소, 정적 Rust 링크 귀속과 도구 체인 출처는 별도로 검토합니다. 후보 결정에는 [릴리스 검토 가이드](../../../developers/qa/release-review.md)를 사용하세요.
