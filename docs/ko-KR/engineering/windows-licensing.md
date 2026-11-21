<a name="xdzip-windows-development-tool-and-runtime-licensing"></a>

# Windows 개발 도구와 런타임 라이선스

[English](../../engineering/windows-licensing.md) | [简体中文](../../zh-CN/engineering/windows-licensing.md) | [繁體中文](../../zh-TW/engineering/windows-licensing.md) | [日本語](../../ja-JP/engineering/windows-licensing.md) | 한국어 | [Français](../../fr-FR/engineering/windows-licensing.md) | [Deutsch](../../de-DE/engineering/windows-licensing.md) | [Español](../../es-ES/engineering/windows-licensing.md) | [Português (Brasil)](../../pt-BR/engineering/windows-licensing.md) | [Русский](../../ru-RU/engineering/windows-licensing.md)

이 페이지는 Windows 빌드와 재배포에 적용되는 경로를 정의하며 [네이티브 런타임 자료](native-runtime-materials.md), [개발자 빌드 진입점](../../engineering/developer-build.md), [설치 계약](installers-associations.md)과 함께 사용합니다. 구체적인 환경, 검증 커밋, 자료 해시와 승인 결론은 issue와 로컬 또는 CI 산출물에 기록하며 이 페이지는 특정 기기의 라이선스 증명을 보관하지 않습니다.

<a name="xdzip-free-community-path-and-local-choice"></a>

## 무료 Community 경로와 로컬 선택

프로젝트의 원본 콘텐츠는 OSI가 승인한 오픈 소스 라이선스 Apache-2.0으로 공개됩니다. Visual Studio Community 2022의 개인 약관은 개인이 자신의 애플리케이션을 개발할 수 있게 하며 조직 약관도 사용자가 OSI 승인 라이선스로 공개되는 애플리케이션을 개발하고 테스트할 수 있게 합니다. 조직의 오픈 소스 사용과 다른 상업적 내부 사용은 별도로 평가하며 이 프로젝트의 적격성을 다른 프로젝트에 확장하지 않습니다.

관리자가 자신의 Community 개발 환경을 필요로 하면 [Microsoft 공식 설치 프로그램](https://learn.microsoft.com/en-us/visualstudio/install/use-command-line-parameters-to-install-visual-studio?view=vs-2022)에서 Community 2022를 얻고 [해당 버전의 원본 약관](https://visualstudio.microsoft.com/license-terms/vs2022-ga-community/)을 읽고 수락한 뒤 그 범위에서 사용합니다. C++ 빌드에는 `Microsoft.VisualStudio.Component.VC.Tools.x86.x64`와 Windows SDK를 선택하며 모든 워크로드를 설치할 필요는 없습니다. Professional/Enterprise 구매나 프로젝트별 런타임 배포 인증서의 별도 신청은 요구되지 않습니다.

라이선스 검토 때문에 대형 IDE를 로컬에 상시 설치할 필요는 없습니다. 아래 표준 GitHub 호스팅 환경에서만 빌드하고 게시하는 경우 적용 근거는 호스팅된 Visual Studio 사용 허가와 REDIST 약관이며 호스팅 CI를 위해 로컬 Community 설치나 개인 Enterprise 구매가 필요하지 않습니다. Community는 관리자가 자신의 환경을 원할 때의 무료 선택지이며 호스팅 빌드의 적법성에 대한 추가 로컬 설치 조건이 아닙니다.

Community 제3절은 조건에 맞는 빌드 장치에서 소프트웨어 파일과 Build Tools를 허용하며 제4절은 기능, 다운스트림 조건과 제한을 전제로 목록의 배포 가능 코드에 대한 배포권을 부여합니다. VC_redist 다운로드, Build Tools 설치 또는 라이선스 문구 동봉만으로 개발 도구 사용 자격이 입증되지는 않습니다. Visual Studio 라이선스가 없을 때 Build Tools 예외는 자신의 애플리케이션을 빌드하는 데 합리적으로 필요한, 제3자가 공개한 OSI 승인 오픈 소스 C++ 구성 요소의 컴파일과 빌드를 포함합니다. 그 의존성의 개발이나 테스트는 컴파일과 빌드에 필요한 경미한 수정 외에는 허용하지 않으며 자신의 모든 C++ 프로그램 개발에 대한 일반적인 예외가 아닙니다. VS Code를 사용해도 MSVC 라이선스가 자동으로 부여되지 않습니다. [Microsoft 라이선스 안내](https://www.microsoft.com/licensing/guidance/Visual-Studio)와 [Build Tools 약관](https://visualstudio.microsoft.com/license-terms/vs2022-ga-diagnosticbuildtools/)을 참조하세요.

로컬에서 `vswhere`가 보고하는 제품, 전체 버전, 미리 보기 아님 상태와 C++ 구성 요소를 확인하고 실제 인스턴스의 개발 환경에 들어간 뒤 `cl`, `link`, SDK를 검사합니다. 설치된 2019 Build Tools는 Community 2022가 아니며 현재 MSVC 2022 도구 체인 검증을 대체하지 않습니다. 일반 `desktop.py prepare`는 시스템 IDE나 SDK를 몰래 설치하지 않습니다. 최초 설치에는 Microsoft 설치 프로그램을 사용하고 기존 도구를 보존하며 자동 재시작을 금지합니다.

2022 선택은 현재 릴리스 도구 체인 결정이며 무료 Community 라이선스가 2022만 허용한다는 주장이 아닙니다. [Qt 6.8 지원 표](https://doc.qt.io/qt-6.8/supported-platforms.html)는 Windows MSVC 2022를 나열하며 이 저장소는 해당 `msvc2022_64` SDK를 사용합니다. 2019 도구는 로컬 개발용으로 유지할 수 있지만 현재 SDK와 혼합할 때 버전 간 ABI 호환성만으로 완전한 지원을 입증할 수 없습니다. [Microsoft 호환성 규칙](https://learn.microsoft.com/en-us/cpp/porting/binary-compat-2015-2017?view=msvc-170)은 일반적으로 최종 링커가 입력 라이브러리에 사용된 도구보다 오래되지 않도록 요구하며 런타임 라이브러리와 `/GL`/`/LTCG`를 별도로 제한합니다. 대체 구성에는 별도 라이선스와 검증이 필요하며 이로 인해 로컬에 전체 2022 IDE를 설치해야 하는 것은 아닙니다.

<a name="xdzip-github-hosted-builds"></a>

## GitHub 호스팅 빌드

프로젝트의 Windows 워크플로는 표준 GitHub 호스팅 `windows-2022` 러너를 사용합니다. 사전 설치된 제품은 Community가 아닌 Visual Studio Enterprise 2022입니다. runner-images 관리자는 GitHub와 Microsoft의 합의에 따라 사전 설치 Enterprise를 공개 및 비공개 GitHub Actions/Azure DevOps 호스팅 러너에서 별도 개인 Enterprise 라이선스 없이 사용할 수 있다고 명확히 [확인했습니다](https://github.com/actions/runner-images/issues/12945#issuecomment-3348002951). 프로젝트는 이 호스팅 빌드 환경에 의존하며 공개 설명은 전체 합의를 대체하거나 공개한다고 주장하지 않습니다.

[GitHub 호스팅 러너 문서](https://docs.github.com/en/actions/concepts/runners/github-hosted-runners)에 따르면 각 Windows 작업은 독립 VM을 받고 `Set up job`은 해당 실행의 이미지와 소프트웨어 목록을 가리킵니다. 워크플로는 [Actions 추가 약관](https://docs.github.com/en/site-policy/github-terms/github-terms-for-additional-products-and-features#actions) 범위에서 이 저장소만 컴파일, 테스트, 배포 및 릴리스합니다. 공개 저장소의 무료 컴퓨팅 한도와 Microsoft 구성 요소 재배포권은 별개입니다.

CI는 실제 VS 제품과 버전, 컴파일러 상대 위치, 도구 버전과 SDK 버전을 `windows-toolchain.json`에 기록하고 제품 키나 개발자의 개인 경로 없이 같은 커밋의 패키지 및 자료와 보관합니다. 이는 도구 식별이지 라이선스 자격의 자동 증명이 아닙니다. 자체 호스팅 러너, 이미지에서 로컬로 복사한 도구, 다른 CI 서비스와 일반 공유 서버는 호스팅 라이선스를 상속하지 않으므로 Community/Build Tools 약관과 장치 조건을 다시 확인합니다.

<a name="xdzip-microsoft-component-redistribution"></a>

## Microsoft 구성 요소 재배포

[VS 2022 REDIST 목록](https://learn.microsoft.com/en-us/visualstudio/releases/2022/redistribution)은 유효하게 라이선스된 Community 2022를 명시적으로 포함합니다. 현재 빌드는 [windows-runtime.json](../../../scripts/windows-runtime.json)에 고정된 릴리스 VC_redist x64를 사용하고 변경 없이 추출하여 CRT DLL 8개와 원본 런타임 라이선스를 각각 확인합니다. 컴파일러, 디버거 또는 `debug_nonredist`는 패키지에 넣지 않습니다. 무료 Community 개발 경로나 호스팅 CI의 Enterprise 신원이 Microsoft DLL을 Apache-2.0으로 재라이선스하지는 않습니다.

새 개발 패키지에는 Windows SDK의 `d3dcompiler_47.dll`, `dxcompiler.dll`, `dxil.dll`이나 Mesa 소프트웨어 OpenGL 라이브러리가 포함되지 않습니다. 과거 패키지에서 이 세 파일은 명시된 SDK의 `Redist/D3D/x64`에서 왔습니다. [SDK REDIST 목록](https://learn.microsoft.com/en-us/legal/windows-sdk/redist)은 유효한 SDK 라이선스 아래 변경 없이 Classic Windows 애플리케이션과 배포하는 것을 허용하고 UWP 애플리케이션과의 배포를 금지합니다. xdzip은 Classic Windows 데스크톱 앱입니다. 향후 패키지에서 파일을 제거해도 과거 SDK 라이선스, 고지와 배포 조건은 별도로 계속 적용됩니다.

동봉된 `share/notices/Windows-runtime-NOTICE.txt`는 각 구성 요소의 라이선스 경계를 두 언어로 설명합니다. 원본 Microsoft CRT 약관은 `Microsoft-VC-runtime/license.rtf`에 있습니다. 설치 프로그램은 원본 CRT 계약을 표시하고 수락을 요구한 뒤 구성 요소 고지를 표시합니다. 포터블 사용자도 동봉된 Microsoft 약관을 읽고 준수해야 합니다. 두 패키지는 원본 라이선스와 출처를 보존합니다. 하위 재배포자는 자격과 배포 조건을 독립적으로 충족해야 하며 프로젝트의 Apache-2.0 라이선스만으로 Microsoft 구성 요소를 재배포할 수 없습니다.

SDK 원문을 표시하거나 동봉하는 것만으로 외부 배포자와 최종 사용자가 SDK에서 요구하는 보호 조건에 동의했음을 입증할 수 없으며 게시자의 유효한 라이선스와 책임 평가를 대체하지 않습니다. 현재 설치 프로그램의 수락 페이지는 CRT에만 적용됩니다. 향후 SDK 그래픽 DLL을 다시 배포한다면 설치 프로그램과 포터블 패키지의 적용 약관 및 수락을 각각 평가하고 실제 증거를 릴리스 기록에 보관합니다. 과거 [#135](https://github.com/xiaodaozhi/xdzip/issues/135)를 참조하세요.

설치 프로그램을 컴파일하기 전에 CRT 계약과 이중 언어 고지의 고정 해시를 확인하며 자료에는 같은 원본 CRT 텍스트와 출처 기록이 포함됩니다. 과거 SDK 약관은 `Windows-SDK-graphics/sdk_license.rtf`에 있습니다. 라이선스 수정은 이전 공개 자산이나 과거 빌드 신원을 다시 쓰지 않습니다. 수정 패키지는 [불변 릴리스 절차](release-protocol.md)에 따라 새 버전으로 발행합니다.
