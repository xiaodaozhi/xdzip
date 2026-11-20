<a name="xdzip-license-decision-and-third-party-materials"></a>

# 라이선스 결정과 타사 자료

[English](../../engineering/licensing.md) | [简体中文](../../zh-CN/engineering/licensing.md) | [繁體中文](../../zh-TW/engineering/licensing.md) | [日本語](../../ja-JP/engineering/licensing.md) | 한국어 | [Français](../../fr-FR/engineering/licensing.md) | [Deutsch](../../de-DE/engineering/licensing.md) | [Español](../../es-ES/engineering/licensing.md) | [Português (Brasil)](../../pt-BR/engineering/licensing.md) | [Русский](../../ru-RU/engineering/licensing.md)

상태: 채택됨. 프로젝트는 2026-10-11에 별도로 명시된 경우를 제외하고 프로젝트의 독창적인 코드, 문서와 기타 독창적 콘텐츠에 **Apache-2.0**을 적용하기로 했습니다. 이전 실험 문서의 MIT OR Apache-2.0 선택지는 이 저장소에 적용되지 않습니다.

루트의 [LICENSE](../../../LICENSE)는 Apache License 2.0 표준 전문을 포함하며 SPDX 식별자는 `Apache-2.0`입니다. 기여 지침과 릴리스 설계도 갱신했습니다. 라이선스 선택은 저장소 공개 상태를 바꾸지 않습니다. 2026-10-19의 읽기 전용 검사에서 공개 저장소임을 확인했습니다. 릴리스 허용에는 실제 자료의 별도 검토가 여전히 필요합니다.

Qt 동적 링크는 모듈별 LGPL/GPL/상용 조건을 확인해야 합니다. 백엔드, 아이콘, 글꼴, 도구와 정적 Rust 의존성은 각각 검토합니다. 라이선스 호환성이 코드에 보안 결함이 없다는 뜻은 아닙니다. 필요한 고지, 소스 및 재링크 자료와 설치 정보는 릴리스 산출물에 함께 제공합니다.

7z, RAR, CAB와 ISO/UDF는 [독립적인 Rust 구현](independent-rust-codecs.md)을 사용하며 공식 UnRAR과 7-Zip 소스는 현재 제품 빌드나 배포에 들어가지 않습니다. 고정 라우팅과 제어된 I/O는 [격리 백엔드 계약](native-codecs.md)을 참고하세요. 분석 소스, 개발 참조 도구와 과거 공식 백엔드 전달물은 각각 원래 라이선스를 유지합니다. 독립 구현은 엄격한 클린룸이나 법적 위험이 없다는 주장이 아닙니다. 공식 7-Zip 또는 UnRAR을 포함한 과거 전달물에는 원래 조건, 고지와 수정 자료 의무가 계속 적용되며 현재 의존성을 제거했다고 Apache-2.0으로 다시 표시할 수 없습니다. 현재 제품 소스 목록이 비어 있어도 과거 전달물이나 독립 참조 도구의 의무가 없어지지 않습니다. 실험 빌드는 공식 라이선스 검토를 대신하지 않습니다.

RAR의 유한한 와이어 형식 매핑과 공개 테스트 자료는 각각 출처, 원본 고지와 라이선스 기록을 유지합니다. [RAR 구현 근거](rar-format-sources.md)를 참고하세요. 제품 엔진을 제거해도 이 자료가 프로젝트 고유 코드로 재라이선스되지는 않습니다.

유한한 매핑에는 [고정된 원본 참조 텍스트와 출처](../../../third_party/rar-wire-mappings/README.md)도 유지합니다. 여기에는 상위 프로젝트 내부 버전 UnRAR 7.30 beta1의 완전한 원본 라이선스, 다운로드 아카이브 `unrarsrc-7.3.1.tar.gz`, 원문 SHA-256이 포함됩니다. 고지 수집기는 이 제한된 자료를 실제 고지 해시에 포함하고 대응 소스 전달물도 별도 사본을 유지합니다. 이는 공식 엔진을 다시 도입하거나 런타임 의존성으로 선언하지 않으며 조건의 적용 여부를 자동으로 결정하거나 이 후보의 릴리스를 승인하지도 않습니다.

고정된 원문은 공백을 정규화한 후 SPDX `UnRAR` 라이선스 텍스트와 단어별로 일치합니다. 참조 매핑을 보수적으로 처리하기 위해 RAR crate와 그 소스 SBOM은 `Apache-2.0 AND UnRAR`을 선언하며 선택형 이중 라이선스가 아니라 둘 다 요구합니다. 이 crate는 crates.io에 게시하지 않습니다. `UnRAR`은 SPDX 3.28 이후의 OSI 미승인 식별자이며 오래된 도구는 인식하지 못할 수 있습니다. 원본 라이선스와 제한을 자료와 함께 보존합니다. 이 선언은 매핑의 저작권 상태를 결정하거나 원래 alpha.8 목록을 다시 쓰지 않습니다.

플랫폼 런타임 고지, 공급업체 입력과 라이선스 자격은 [네이티브 런타임 출처와 고지](native-runtime-materials.md)를 따릅니다. 일반 라이선스 텍스트는 공급업체 저작권 고지를 대신하지 않습니다.

자체 개발 환경이 필요한 Windows 유지 관리자는 조건을 충족하면 무료 Visual Studio Community 경로를 사용할 수 있습니다. GitHub 표준 호스팅 러너에 사전 설치된 Enterprise에는 별도의 호스팅 사용 권한이 있으며 CI 빌드와 배포에 로컬 Community IDE가 추가로 필요하지 않습니다. 적용 가능성, Microsoft CRT/SDK 재배포 조건과 하위 사용자 설명은 [Windows 개발 도구 및 런타임 라이선스](windows-licensing.md)를 참고하세요. 이 Microsoft 구성 요소에는 프로젝트의 Apache-2.0이 적용되지 않으며 일반 사용자는 Visual Studio를 설치할 필요가 없습니다.

[실제 구성 요소 귀속 절차](runtime-attribution.md)는 고정된 SDK SPDX에서 모듈, 타사, 저작권과 상위 라이선스 표현을 추출하고 고정 소스의 라이선스 텍스트를 Windows/Linux 런타임 디렉터리와 함께 전달합니다. 상위 licenseConcluded는 프로젝트가 해당 라이선스 선택지를 선택하거나 승인했다는 뜻이 아닙니다. 각 후보는 실제 구성 요소의 소스, 고지, 대응 소스와 교체 자료를 확인해야 합니다. 식별된 적용 의무나 필수 자료가 해결되지 않으면 해당 후보의 릴리스가 차단됩니다. 상위 NOASSERTION 또는 incomplete로 표시된 기계 보고서만으로 라이선스 충돌이 입증되거나 면제가 허용되는 것은 아닙니다.

시각 이미지 세 장은 생성형 디자인 도구로 제작했으며 Bandizip 스크린샷이 아닙니다. 참조 링크는 연구 근거만 기록합니다. 생성 자산의 독점성은 보장하지 않습니다. 릴리스 아이콘, 예제 이미지와 글꼴에는 별도 출처 기록이 필요하며 디자인 원본에서 잘라내는 것으로 자산 감사가 완료되지는 않습니다.

최초 ZIP 메타데이터 백엔드는 MIT 라이선스 rawzip을 사용했습니다. 버전과 고지는 [과거 허용 기록](zip-metadata-admission.md)을 참고하세요. 현재 ZIP, TAR과 GZIP/Deflate는 [독립 Rust 구현](independent-rust-codecs.md)을 사용하고 형식 참조 라이브러리는 개발 테스트에만 사용합니다. 과거 전달물과 테스트 참조의 라이선스 의무는 유지됩니다. 범용 암호 기반의 고지는 [의존성 계약](dependencies.md)에 따라 수집합니다. 프로젝트 고유 콘텐츠의 라이선스는 계속 Apache-2.0입니다.

HTML 디자인은 공식 Noto Sans SC의 파생 부분 집합을 사용합니다. Qt 고정 UI 텍스트는 언어에 따라 Noto Sans TC, JP, KR과 Noto Sans의 파생 부분 집합도 사용합니다. 이 글꼴에는 프로젝트의 Apache-2.0이 아닌 OFL-1.1이 적용됩니다. 글꼴 계열 이름, 고정된 출처, 버전과 해시는 [글꼴 목록](../../../specs/dependencies/ui-font.json)과 [네이티브 TTF 내보내기](../../../design/assets/fonts/README.en.md)를 참고하세요. 원본 라이선스 전문은 [CJK OFL](../../../design/assets/fonts/OFL.txt)과 [Noto Sans OFL](../../../design/assets/fonts/NotoSans-OFL.txt)입니다. 오프라인 HTML에는 해당 글꼴 바이너리와 완전한 고지가 포함되고 Qt 배포 고지에는 해당 글꼴 라이선스 전문이 포함됩니다. 제품에는 상위 CJK 글꼴 전체가 포함되지 않습니다.
