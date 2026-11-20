<a name="xdzip-cxx-bridge-dependency-rationale"></a>

# cxx 브리지 의존성을 선택한 이유

[English](../../engineering/cxx-admission.md) | [简体中文](../../zh-CN/engineering/cxx-admission.md) | [繁體中文（台灣）](../../zh-TW/engineering/cxx-admission.md) | [日本語](../../ja-JP/engineering/cxx-admission.md) | 한국어 | [Français](../../fr-FR/engineering/cxx-admission.md) | [Deutsch](../../de-DE/engineering/cxx-admission.md) | [Español](../../es-ES/engineering/cxx-admission.md) | [Português (Brasil)](../../pt-BR/engineering/cxx-admission.md) | [Русский](../../ru-RU/engineering/cxx-admission.md)

Qt와 Rust는 `cxx`가 생성한 타입과 소유권이 명시된 브리지로 연결됩니다. C++는 Qt 객체를 보유하고 Rust는 도메인 및 실행 환경 로직을 보유합니다. [Cargo.lock](../../../Cargo.lock)과 [도구 정책](toolchain.md)에 따라 `cxx`와 `cxx-build`의 버전은 정확히 일치해야 합니다. cxx는 타사 형식 확장을 위한 안정적인 ABI가 아닙니다. 직접 작성한 C ABI는 의존성을 줄이는 대신 문자열, 자유 함수 및 소유권 불변식의 유지보수를 늘립니다.

<a name="xdzip-source-and-licensing"></a>

## 소스와 라이선스

소스는 crates.io에서 배포하며 업스트림은 [dtolnay/cxx](https://github.com/dtolnay/cxx)입니다. 해시, 버전, 기능, 빌드 스크립트 및 라이선스는 잠금 파일과 [브리지 의존성 목록](../../../specs/engineering/bridge-dependencies.json)에서 확인합니다. 릴리스 전에 실제 적용되는 고지를 다시 확인하고 모으세요. 잠금 파일에는 대상 조건을 충족하지 않는 의존성도 들어갈 수 있습니다. 실제 링크된 바이너리, 컴파일러, Qt 및 시스템 라이브러리는 별도로 확인합니다.

<a name="xdzip-build-and-ownership-boundaries"></a>

## 빌드와 소유권 경계

cxx-build는 소스와 헤더를 생성하고 C++ 컴파일 도구를 호출합니다. cxx는 함께 제공되는 브리지를 컴파일하고 link-cplusplus는 C++ 표준 라이브러리를 링크합니다. Cargo는 잠금 파일에 따라 가져온 의존성을 검증합니다. 새로운 빌드 스크립트나 선택 기능을 추가하면 네트워크 접근과 실행 동작을 다시 검토합니다. 이전 검사는 새 버전의 보안 검사를 대신하지 않습니다.

std/alloc과 제품에 필요한 기능을 유지하세요. 기능을 줄이려면 UTF-8, 오류 전파, 고유 핸들, 역압, 종료 및 대기 작업 비우기를 다시 검증해야 합니다. 실제 인터페이스는 [실행 환경 계약](product-runtime.md)과 [브리지 설계](../design/qt-rust-bridge.md)에서 정의합니다. 라이선스는 [릴리스 검토 절차](../../../developers/qa/release-review.md)를 따릅니다.

검사, CI 실행 및 인수 결과는 [WP-01 이슈](https://github.com/xiaodaozhi/xdzip/issues/1)와 Actions 결과물에 기록합니다.
