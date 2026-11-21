<a name="xdzip-actual-component-attribution-and-license-materials"></a>

# 실제 구성 요소 귀속과 라이선스 자료

[English](../../engineering/runtime-attribution.md) | [简体中文](../../zh-CN/engineering/runtime-attribution.md) | [繁體中文](../../zh-TW/engineering/runtime-attribution.md) | [日本語](../../ja-JP/engineering/runtime-attribution.md) | 한국어 | [Français](../../fr-FR/engineering/runtime-attribution.md) | [Deutsch](../../de-DE/engineering/runtime-attribution.md) | [Español](../../es-ES/engineering/runtime-attribution.md) | [Português (Brasil)](../../pt-BR/engineering/runtime-attribution.md) | [Русский](../../ru-RU/engineering/runtime-attribution.md)

이 절차는 실제 Qt 라이브러리와 플러그인을 SDK의 SPDX 선언에 연결하여 파일 단위 SBOM을 보완합니다. 출력은 검토 가능한 증거이며 라이선스 자동 선택이나 준수 승인과 다릅니다. 정식 릴리스에 필요한 완전성은 아직 미완료입니다.

<a name="xdzip-inputs-and-sources"></a>

## 입력과 출처

Qt 6.8부터 빌드에 대응하는 [SPDX 목록](https://doc.qt.io/qt-6.8/licenses-used-in-qt.html)을 제공합니다. 도구는 고정된 Qt 6.8.2 SDK의 sbom/에서 qtbase, qtdeclarative, qtsvg, qttranslations 등의 런타임 선언만 읽고 qttools 빌드 및 테스트 도구는 제외합니다. SPDX 버전, 모듈 이름, Qt 버전을 검증하고 원본 문서의 SHA-256, 네임스페이스, 안정적인 패키지 참조를 보존합니다. 출력에는 필요한 필드만 남기며 개발자 컴퓨터 경로가 포함된 업스트림 빌드 주석은 복사하지 않습니다.

`scripts/qt-license-sources.json`은 공식 Qt 소스와 SPDX license-list-data 커밋, 라이선스 텍스트 URL 및 SHA-256을 고정합니다. 다운로드는 build/ 아래에 캐시하며 캐시나 다운로드 해시가 일치하지 않으면 실패합니다. 자료 다운로드에 임의의 SPDX URL을 따라가지 않습니다. LICENSES/의 표준 텍스트 외에도 Qt GUI의 부드러운 크기 조정 알고리즘과 SVG의 원본 고지, 시스템 의존성 선언에서 참조하는 LGPL-2.1 텍스트를 포함합니다. 업스트림 LicenseRef 텍스트와 저작권은 선언 JSON에 유지합니다.

Qt 런타임 고지에는 Qt Network가 사용하는 고정된 편집 가능한 Public Suffix List 소스, MPL-2.0 텍스트, 취득 방법도 포함됩니다. 수집기는 업스트림 blob, SHA-256, 라이선스 바이트를 검증합니다. 소스 제공은 이 입력이 Qt 아카이브의 생성된 데이터 두 형태를 재현하는지 별도로 검증합니다. [해당 소스 자료](source-delivery.md)를 참조하세요. 이는 다른 Qt 모듈의 소스 및 대체 검사를 대신하지 않습니다.

Qt Core의 Tika MIME 고지는 Apache Software Foundation과 영국 국립기록원의 PRONOM에 대한 관련 귀속, Apache-2.0 및 OGL-UK-3.0 라이선스 텍스트, 고정된 전체 업스트림 NOTICE를 보존합니다. 전체 NOTICE는 다른 업스트림 Tika 구성 요소도 설명하지만 이것이 해당 구성 요소가 제품과 함께 제공된다는 뜻은 아닙니다. 원본 MIME 데이터와 Qt 변환 검증은 [해당 소스 자료](source-delivery.md)를 참조하세요.

<a name="xdzip-file-and-component-association"></a>

## 파일과 구성 요소 연결

1. 검증 루트 내 파일 해시를 다시 계산하고 deployment-audit.json 이후 수정된 파일을 거부합니다.
2. macOS SDK 모드는 실제 해석된 Qt SDK 상대 경로와 바이트 동일성을 사용하며 독립 배포를 주장하지 않습니다.
3. Linux 배포는 원본 SDK 경로/SHA-256과 patchelf 이후 산출물 경로/SHA-256을 기록합니다. 이후 검사는 양방향으로 기록을 검증하고 오래되거나 변경된 복사 증거를 거부합니다.
4. Windows는 SDK 파일 바이트 동일성을 우선합니다. 이름은 유일하지만 바이트가 다른 후보는 filename-only-unverified-rewrite로만 표시하고 검증된 소스가 아닌 누락 항목으로 등록합니다.
5. SPDX CONTAINS로 파일 소유자를 찾은 다음 Qt 저장소 간 참조와 내장된 제3자를 포함하여 DEPENDS_ON을 재귀적으로 해석합니다. 누락된 패키지 참조는 누락 항목으로 남깁니다. 저장소 전체의 CONTAINS로 모든 도구를 런타임에 잘못 포함하지 않습니다.
6. 파일별 선언이 없는 비네이티브 QML과 번역은 해당 소스 저장소 범위에만 연결하며 개별 라이선스는 추론하지 않습니다. SPDX 파일 기록이 없는 SDK 네이티브 라이브러리, Qt 범위 밖 네이티브 파일, NOASSERTION 라이선스, 누락된 텍스트는 별도로 기록합니다.

실제 SDK 파일 바이트는 나중의 설치 처리나 서명 등으로 업스트림 빌드 SPDX의 SHA-1과 다를 수 있습니다. 도구는 원인을 추론하지 않고 예상 값, 비교 결과, 누락 항목을 보존합니다. SDK 바이트 연결, 배포 재작성 기록, 업스트림 원본 해시는 서로 다른 세 가지 증거입니다.

전용 macOS 패키징은 최종 서명된 앱이 존재하는 동안 배포 연결을 생성합니다. 정확한 리소스 바이트 또는 arm64 섹션, UUID, 가져오기 버전으로 SDK 입력과 일치시키고 보고서를 실제 파일 SBOM에 추가합니다. 비교에서는 서명, 로드 명령 레이아웃, LINKEDIT를 제외합니다. 범위와 전용 고지는 [네이티브 런타임 출처와 고지](native-runtime-materials.md)를 참조하세요. 일반 SDK 모드가 이를 통해 배포 결론을 입증하는 것은 아닙니다.

QML 디렉터리에서 `Contents/PlugIns/quick/`으로 이동한 플러그인의 경우 배포 연결에 패키지 내 감사된 QML 심볼릭 링크를 사용하여 유일한 SDK 입력을 찾을 수 있습니다. 기록 생성과 사용 시 모두 실제 링크, 감사 대상, 파일 해시를 다시 확인하고 같은 바이트 또는 Mach-O 비교를 수행합니다. 출처가 없으면 미검증으로 남기며 서로 다른 여러 소스나 변경된 링크는 거부합니다. 파일 이름 일치만으로 귀속 증거를 강화하거나 업스트림 SDK 해시 차이와 라이선스 검토 누락을 없앨 수 없습니다.

Rust는 전체 호스트 빌드 그래프를 보존하고 데스크톱 `xdzip-qt-bridge`, 형식 워커 `xdzip-codec-bridge`, Windows/Linux의 실제 `xdzip-update-helper`에서 일반 의존성을 따라 런타임 후보를 생성합니다. macOS에는 별도의 업데이트 도우미 루트가 없습니다. 빌드, 개발, proc-macro 도구와 그 의존성은 제외합니다. 이는 잠재적 런타임 입력이지 각 패키지의 기계 코드가 링크 후 남았다는 증거가 아닙니다. 링커 제거, 네이티브 C/C++, 컴파일러 런타임은 추가 검토가 필요합니다.

슬래시로 구분된 단순한 이전 Cargo 라이선스 대안은 소스 CycloneDX에서 SPDX `OR`로 변환하고 업스트림 원문은 `xdzip:cargo_license_raw`에 보존합니다. 우선순위가 모호한 슬래시 표현식은 거부합니다. 원본 빌드 목록과 고지는 다시 쓰지 않으며 표준 `AND`, `OR`, `WITH`는 변경하지 않습니다. 이는 형식 정규화이지 라이선스 분기 선택이나 릴리스 승인이 아닙니다.

<a name="xdzip-execution-and-delivery"></a>

## 실행과 제공

알려진 패치 적용 QtNetwork 파생물은 파일 기록에 `network_security_variant`를 추가하며 보고서는 완전하고 다시 검증된 빌드 증빙도 보존합니다. 업스트림 SDK 다이제스트 차이는 남기고 릴리스 검토에서 고정된 소스, 전체 패치, 빌드 증빙, 배포 변환 증거로 설명합니다. 릴리스 준비에는 최종 Network 귀속을 해당 바이트에 연결해야 하며 파일 이름 일치만으로 통과하지 않습니다. 소스와 산출물 SBOM은 파생물의 식별 정보를 각각 표시하지만 전체 미완료 상태를 라이선스 승인으로 바꾸지 않습니다.

```sh
python3 scripts/prepare-runtime-notices.py --qt-prefix /path/to/Qt/6.8.2/gcc_64 \
  --output build/review/notices/Qt-runtime
python3 scripts/runtime_attribution.py --audit build/review/deployment-audit.json \
  --root build/review/linux --qt-prefix /path/to/Qt/6.8.2/gcc_64 \
  --deployment build/review/linux-deployment.json --output build/review/runtime-attribution.json
python3 scripts/sbom.py --audit build/review/deployment-audit.json \
  --source build/review/source-build.cdx.json --attribution build/review/runtime-attribution.json \
  --output build/review/artifact-files.cdx.json
```

Linux는 배포 재작성 기록을 전달하며 다른 플랫폼은 --deployment를 생략합니다. 자체 목록의 재귀를 피하도록 보고서는 검증 루트 밖에 씁니다. 주문형 Qt와 Release 워크플로는 배포와 최종 해시 전에 라이선스 자료를 준비합니다. Windows와 Linux는 실제 런타임에 고지를 넣고 Linux tar.gz에도 제공합니다. macOS 주문형 Qt는 SDK를 읽고 로그와 JSON만 기록합니다. 명시적으로 승인된 전용 Release 워크플로는 [릴리스 프로토콜](release-protocol.md)에 따라 실제 릴리스 자료를 별도로 배포, ad-hoc 서명, 패키징, 검사하고 배포 범위와 해시를 별도로 보존합니다. SDK 기록은 최종 앱 번들의 귀속을 대신하지 않습니다.

runtime-attribution.json에는 파일, 구성 요소, 관계, 업스트림 선언, 저작권, 텍스트 출처, 누락이 포함됩니다. artifact-files.cdx.json은 같은 구성 요소와 알려진 소유 관계를 추가하고 보고서를 동일한 실제 감사 해시 및 빌드 식별 정보에 연결합니다. 공식 CycloneDX 스키마와 참조 검사는 계속되며 compositions는 미완료 상태입니다.

기본 검사는 명시적으로 기록된 개발 검토 누락을 허용하지만 이를 조용히 통과로 표시하지 않습니다. 식별 정보, 경로, 바이트, 버전, 복사 기록, 라이선스 텍스트 해시 오류는 즉시 실패합니다. --require-complete는 릴리스를 차단하는 검사이며 현재 귀속이나 릴리스 자료가 미완료이면 실패합니다. 테스트는 재작성된 파일, 오래된 복사 기록, 경로 이탈, 잘못된 버전, 문서 간 누락, 부분 SBOM, Rust 도구 제외를 다룹니다.

<a name="xdzip-unresolved-release-gates"></a>

## 해결되지 않은 릴리스 조건

[소스 제공 수집기](source-delivery.md)는 실제 프로젝트 소스, Rust vendor, 실행용 Qt 모듈 아카이브 네 개, 빌드 모듈 아카이브 두 개, 재빌드 및 동적 교체 지침을 보존합니다. 항목별 SDK와 업스트림 빌드 동등성을 증명하지는 않습니다. 각 릴리스에서는 정확한 정적 링크 귀속, OS 및 재배포 런타임, 모든 내장 리소스의 개별 라이선스, 오픈소스 라이선스 분기 선택과 의무 이행을 계속 검토해야 합니다. [로컬 Mac 배포 검증](macos-deployment.md)의 메타데이터 변경과 교체, [Qt 소스의 실제 빌드·로드·교체](../../../developers/engineering/candidate-verification.md#qt-source-and-compatible-replacement)는 증거가 분리된 별개의 절차입니다. 어느 쪽도 임의 SDK의 바이트 단위 빌드 동등성을 입증하지 않습니다. 소스 목록과 SPDX 필드는 라이선스 의무를 대신하지 않습니다. 과거 `v0.1.0` 후보의 출처와 라이선스는 [#4](https://github.com/xiaodaozhi/xdzip/issues/4)와 [#53](https://github.com/xiaodaozhi/xdzip/issues/53)에 있으며 이후 후보는 별도 검토가 필요합니다. 유료 서명 식별 정보는 필수 조건이 아닙니다. 기존 데스크톱 수용은 [#5](https://github.com/xiaodaozhi/xdzip/issues/5)에 남아 있습니다.

[Qt SDK 대응 검증](qt-sdk-correspondence.md)은 고정된 아카이브 커밋, SDK 소스 선언의 파일별 해시, 설치된 구성, 알려진 AGL 소비자 패치도 기록합니다. 선언된 범위의 일치가 SDK 전체 빌드 동등성을 증명하지 않으며 기존 릴리스 차단 조건도 유지됩니다.

자동 마무리는 실제 링커 맵과 정적 라이브러리 멤버, 런타임 리소스 해시, 신뢰할 수 있는 네이티브 의존성 버전, 정확히 되돌릴 수 있는 바이너리 차이 분석을 추가합니다. [전체 계약](../../../developers/engineering/candidate-verification.md)을 참조하세요. 재구성한 바이트 해시가 업스트림 선언과 정확히 일치할 때만 변환이 설명되었다고 표시합니다. 원래 SDK 차이를 삭제하거나 문서화되지 않은 게시자 이력을 추측하지 않습니다.
