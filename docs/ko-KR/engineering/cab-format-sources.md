# CAB 형식 자료와 구현 근거

[English](../../engineering/cab-format-sources.md) | [简体中文](../../zh-CN/engineering/cab-format-sources.md) | [繁體中文](../../zh-TW/engineering/cab-format-sources.md) | [日本語](../../ja-JP/engineering/cab-format-sources.md) | 한국어 | [Français](../../fr-FR/engineering/cab-format-sources.md) | [Deutsch](../../de-DE/engineering/cab-format-sources.md) | [Español](../../es-ES/engineering/cab-format-sources.md) | [Português (Brasil)](../../pt-BR/engineering/cab-format-sources.md)

이 문서는 CAB 컨테이너, MSZIP, LZX와 Quantum의 자료, 범위와 독립 구현 제약을 기록합니다. 제품 기능, 리소스 관리와 형식별 교체 규칙은 각각 [격리된 형식 작업 프로세스 계약](native-codecs.md), [자체 Rust 계약](../../engineering/independent-rust-codecs.md), [분할 볼륨 계약](archive-opening.md)에 정의되어 있습니다. 자료 기록은 상호 운용성 승인이나 릴리스 라이선스 검토를 대체하지 않습니다.

## Microsoft 기술 문서

Microsoft 형식 문서를 우선합니다. 표에는 실제로 읽은 사본을 고정합니다. 날짜는 문서 버전을 식별하며 URL이 언제나 같은 바이트를 반환한다는 뜻은 아닙니다. 다운로드한 사본은 SHA-256으로 검증하세요. 제목이 같다는 것만으로 버전이 동일하다고 볼 수 없습니다.

| 문서와 진입점 | 고정 버전 | 범위와 제한 |
| --- | --- | --- |
| [MS-CAB 공식 보관 자료 진입점](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxprotlp/5641c1b2-35ed-4542-9058-a07ccab5b0a1), [PDF](https://officeprotocoldocs-f5hpbjgea6b8gneq.b02.azurefd.net/files/Archive_Exchange/%5BMS-CAB%5D.pdf) | 바닥글 `v20110304`, 2011-03-04 릴리스 | §2 CFHEADER, CFFOLDER, CFFILE, CFDATA, §3.1 체크섬 방법. Microsoft는 이 문서를 보관 자료로 분류하고 보관 자료에 의존하지 말라고 명시적으로 경고합니다. 현재 유지되는 사양이라고 부르지 마세요. 압축 알고리즘 내부 비트스트림은 정의하지 않습니다. |
| [MS-MCI](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-mci/27f0a9bf-9567-4e40-ad66-6ae9ab9d2786), [PDF](https://officeprotocoldocs-f5hpbjgea6b8gneq.b02.azurefd.net/files/MS-MCI/%5bMS-MCI%5d.pdf) | 2016-10-17, 개정 7.3 | §2 MSZIP 구조, CK 마커, 완전한 Deflate 스트림과 블록 크기. Deflate 비트스트림은 [RFC 1951](https://www.rfc-editor.org/rfc/rfc1951.html)도 따릅니다. |
| [MS-PATCH](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-patch/cc78752a-b4af-4eee-88cb-01f4d8a4c2bf), [PDF](https://officeprotocoldocs-f5hpbjgea6b8gneq.b02.azurefd.net/files/MS-PATCH/%5bMS-PATCH%5d.pdf) | 2025-08-19, 개정 14.1 | §2.1 비트스트림, 윈도와 거리, §2.2 헤더와 E8, §2.3 블록, §2.4 Huffman 트리. 공통 메커니즘과 차이를 비교할 목적으로만 사용합니다. LZX Delta를 설명하며 CAB LZX의 대체 사양이 아닙니다. |

| 고정 PDF | SHA-256 |
| --- | --- |
| MS-CAB | `09f796a493547697ed7d4bc36222ac132bf3ca1bd6b1da76fbdc3b38b5307709` |
| MS-MCI | `cb097c434e1cfccb368f0d7e9084ab4d47a65809a2eea94bbf41f9722c1a0343` |
| MS-PATCH | `ea17466b97c9a9d4e2c815538387bf442fb2393ed29632c6f58b2c47ef1f29fe` |

Microsoft Open Specifications 문서에는 구현에 필요한 문서 라이선스와 특허 고지를 포함하여 자체 저작권 및 지식재산 조건이 있습니다. 그렇다고 이 프로젝트의 Apache-2.0 자료가 되는 것은 아니며 다운로드할 수 있다는 사실이 모든 특허 또는 재배포 문제가 해결되었음을 뜻하지 않습니다.

## 1997년 CAB SDK 문서

원래 제목은 Microsoft Cabinet File Format, Microsoft MSZIP Data Compression Format, Microsoft LZX Data Compression Format입니다. 읽은 사본에는 Microsoft의 1997년 저작권 고지가 있고 제3자가 호스팅합니다.

| 미러된 원본 | 읽은 범위 | SHA-256 |
| --- | --- | --- |
| [CABFMT.DOC](https://am.net/lib/tools/Microsoft/cab-sdk/DOCS/CABFMT.DOC) | §2 컨테이너 배치와 볼륨 간 의미, §3 고정 컨테이너 예제, §4.1 체크섬 정의 | `5a220658b76f8a66e9013e342b4eb236f057b597a8e6f3939c86f05efc51e83a` |
| [MSZIPFMT.DOC](https://am.net/lib/tools/Microsoft/cab-sdk/DOCS/MSZIPFMT.DOC) | 소개와 구현 세부 사항, 특히 독립 Deflate 블록과 보존된 기록 | `046d283fa3b697e46c35b6a329bb90c86cbc9f4dedbed24c3ae43bd9b8ccf239` |
| [LZXFMT.DOC](https://am.net/lib/tools/Microsoft/cab-sdk/DOCS/LZXFMT.DOC) | 개념, LZX 압축 데이터 형식, 트리 코딩, 거리·길이와 E8 전처리 | `68ba4df86e08bece3259574133f1c61606d143d85f749b388f5f1e8ea46f8bef` |

이 `.DOC` 파일은 실제로 RTF입니다. 제목, 저작권 고지와 CAB 설명은 Microsoft 작성 문서의 사본으로 사용하는 근거가 되지만 호스트는 Microsoft가 아니며 Microsoft가 공개한 CAB SDK 아카이브와 바이트 단위 비교를 하지 않았습니다. [Microsoft KB 310618(보관 자료)](https://mskb.pkisolutions.com/kb/310618)은 원래 SDK 내용을 열거하고 오래된 다운로드를 사용할 수 없다고 기록합니다. 보관 항목이 미러 파일의 진위를 인증하지는 않습니다. 읽은 범위는 형식 문서이며 SDK의 FCI/FDI 구현이나 예제 코드는 아닙니다.

공개 TCOMP 매개변수는 Windows SDK 10.0.19041.0의 `um/fdi_fci_types.h` 상수 선언 및 [TCOMPfromLZXWindow 문서](https://learn.microsoft.com/en-us/windows/win32/api/fdi_fci_types/nf-fdi_fci_types-tcompfromlzxwindow)와도 비교했습니다. 헤더 SHA-256은 `27a1f6f7728f5db64d11e434d101819d646ca49270930606bb76c7989ebe0254`입니다. 공개 매개변수 범위와 비트 정의만 사용했으며 이를 근거로 FCI/FDI 알고리즘을 호출하거나 분석하지 않았습니다.

LZXFMT 개요, 표와 본문 일부는 서로 충돌합니다. MS-PATCH는 공통 메커니즘 식별에 도움이 되지만 참조 데이터, 확장된 긴 일치, 128 KiB부터 32 MiB까지의 윈도와 추가 블록 헤더를 CAB에 직접 적용할 수 없습니다. CAB LZX 윈도는 15~21비트이며 원문, 참조 동작과 구별 벡터를 통한 독립 검사가 필요합니다.

## Quantum 자료 범위

확보한 원래 알고리즘 공급업체 자료는 [Cinematronics QUANTUM 사용자 설명서 미러](https://carrero.es/wp-content/uploads/2025/07/QUANTUM.pdf)이며 copyright Cinematronics 1993, 1994, 1995, all rights reserved입니다. PDF SHA-256은 `a2d6ed51e47db407ea5849b42004750f2f36c82a380f263b983c5282838a8212`입니다. 6절은 독립적인 `.Q` 아카이브 컨테이너를 설명하지만 CAB Quantum에 필요한 산술 비트스트림, 확률 모델 갱신 또는 프레임 종료를 정의하지 않습니다. `.Q`의 전체 스트림 상태나 파일 검사를 CAB 규칙으로 옮기지 마세요.

[Matthew Russotto의 Quantum 연구 페이지](http://www.russotto.net/quantumcomp.html)와 과거 아카이브에서는 검증 가능한 원문을 얻지 못했습니다. 따라서 내용을 읽은 증거로 취급하지 않았습니다. 원문이 알고리즘을 완전히 정의하는지 판단하거나 라이선스 조건을 추론할 수 없습니다.

추가로 읽은 자료는 [David Carrero의 UnQuantum README](https://github.com/dcarrero/unquantum/blob/main/README.md)뿐이며 사본 SHA-256은 `187af81f8231cfe4cc31aec0c76476b1e33179fe517a365c820fc49c1b8106e0`입니다. 해당 프로젝트의 디코더 소스는 읽지 않았습니다. 이는 `.Q` 프로젝트의 비공식 개요이며 Russotto, libmspack과 바이너리 연구를 참조한다고 밝힙니다. Russotto의 원문이 아니고 CAB 비트스트림 사양을 완전히 대체할 수 없습니다. 프로젝트의 MIT 표시는 모든 참조와 알고리즘 소스가 MIT 라이선스임을 증명하지 않습니다.

이러한 공백은 고정된 공식 참조 구현의 동작 분석으로 보완합니다. 비공식 자료를 일괄 배제하지 않으며 참조 구현을 Microsoft 형식 사양이라고 부르지도 않습니다.

## 고정 참조 소스와 분석 범위

참조는 [공식 7-Zip 26.05](https://github.com/ip7z/7zip/releases/tag/26.05), 구체적으로 [7z2605-src.tar.xz](https://github.com/ip7z/7zip/releases/download/26.05/7z2605-src.tar.xz)이며 아카이브 SHA-256은 `0b6080b8c3e7a579161d2d495224c0e51c467576cb4c7048b0ab485ef44cc22e`입니다. 빌드 소스 목록은 [native-codec-sources.json](../../../scripts/native-codec-sources.json)을 참조하세요. 해당 목록은 전체 공식 소스를 설명하고 아래 기록은 CAB 참조 분석에 실제 사용한 파일만 식별합니다.

아카이브의 `DOC/License.txt`에 따르면 다음 디코더와 공통 파일에는 기본 LGPL-2.1-or-later가 적용되며 이 파일에서 다른 라이선스 고지는 발견되지 않았습니다. 아카이브에는 다른 파일용 BSD 조건도 있습니다. 아카이브 전체를 예외 없는 단일 라이선스로 단순화하거나 이 디코더 파일을 잘못 BSD라고 표시하지 마세요.

| 아카이브 파일 | 실제 분석 범위 | SHA-256 |
| --- | --- | --- |
| `CPP/7zip/Compress/QuantumDecoder.cpp` | 산술 비트스트림, 모델, 토큰, 프레임 종료, 사전 상태 | `1a22429d792493573f542782a45a7f702b6e636e5aee1d47bdb3d1cf75ef11e2` |
| `CPP/7zip/Compress/QuantumDecoder.h` | 모델 알파벳과 선택자 상수 | `7571ee0360a7abe8f297f4cd4e34df1863003f252c6fab66c61f8d1bd25a21b4` |
| `CPP/7zip/Compress/Lzx.h` | LZX 비트스트림 상수 | `779b32ecaecfc5b8c135e0051394001786d680fca4cfda57a480884342500e99` |
| `CPP/7zip/Compress/LzxDecoder.h` | 프레임 간 유지 상태의 경계 | `18141528bc8587ade4f7b1e330ee35dc8e31d6c6ab34d86a2dff59d362d0d6c9` |
| `CPP/7zip/Compress/LzxDecoder.cpp` | CAB 모드 트리, 원시 블록, 패딩, 반복 거리, E8, 기록과 종료. 최적화 코드와 메모리 배치는 구현 설계가 아닙니다 | `e78bf50f27f21e04966785fafe7f94fddc533a9e6103f79b81c4e7d7613d1aab` |
| `CPP/7zip/Compress/HuffmanDecoder.h` | 완전한 트리·빈 트리 허용만 분석, 고정 사본의 약 109~164행과 430~497행 | `22f93e29b2408ce7e1c1bca5f5f807c1383a8d1634fe3ae6cfcab15f084109c0` |
| `CPP/7zip/Archive/Cab/CabHandler.cpp` | 압축 방법, CFDATA 프레임과 기록 이어짐만 분석, 고정 사본의 약 1020~1252행 | `77b82387eea40eda02929b90d3006877c08765089d6b96c5c12b9cfd3f56b8f7` |
| `C/7zVersion.h` | 고정 버전 식별 | `f7dc58b3a3c01569e061eda9cfe633ad8983667eb0fa10ab94a6c364ddd7e699` |
| `DOC/License.txt` | 기본 파일 라이선스 규칙과 예외 | `9ac2b4a97ab5d523965534d8b2d5868e511b39096d51fff458ab72c38b80fccc` |
| `DOC/copying.txt` | LGPL 문서 식별 다이제스트이며 법률 자문으로 해석하지 않음 | `1e7e6bae5a5bde32f1ae5a7c37a082d1ab03cf89354f7f936ac40be9e39a6531` |

행 번호는 그 고정 사본의 읽기 범위만 제한하며 파일 식별은 다이제스트로 확립합니다. 참조 버전을 올릴 때 차이를 다시 기록하세요. 같은 행 번호를 사용한다고 동작이 바뀌지 않았음을 증명하지 않습니다.

## 분석과 독립 구현의 분리

초기 준비에서는 형식 문서와 프로젝트 소유 인터페이스만 읽었습니다. 이후 지정된 참조 분석자가 위 소스 파일을 명시적으로 읽고 비트스트림, 수학적 갱신, 재설정, 종료와 오류의 누락되거나 모순된 조건을 코드 표현이 없는 동작 사양으로 바꾸어 해당 소스를 읽지 않은 LZX와 Quantum 구현자에게 전달했습니다.

각 알고리즘의 Rust 구현은 구현자가 참조 소스를 읽지 않고 상태, 타입, 제어 흐름과 테스트까지 독립적으로 구성했습니다. 분석자는 두 알고리즘 중 어느 것도 Rust로 구현하지 않았으며 CAB 컨테이너를 별도로 구현했습니다. 이는 CAB 작업에 관여한 아무도 참조 소스를 보지 않았다는 뜻이 아니므로 엄격한 클린룸이라고 부를 수 없습니다. 동작 사양에는 참조 코드, 행별 재작성, 이식한 구조 또는 번역한 주석을 넣지 않으며 사양 이름, 상수와 수학적 관계는 출처를 유지합니다.

참조 읽기, 독립 코딩과 블랙박스 상호 운용성은 서로 다른 질문에 답합니다. 자체 인코더와 디코더가 함께 성공해도 독립 호환성을 확립하지 않습니다. 참조 도구의 성공이 모든 관대한 동작이 이 프로젝트의 무결성 계약에 맞는다는 것을 증명하지 않습니다. 참조 소스, 문서, 도구와 샘플은 각자의 라이선스를 유지합니다. Rust로 다시 작성했다고 법적 위험이 제거되었다는 뜻은 아닙니다. 라이선스와 릴리스 요구 사항은 [라이선스 계약](licensing.md)과 [대응 소스 계약](../../engineering/source-delivery.md)을 참조하세요.

## 유지해야 할 기술적 차이

다음 동작 경계는 형식 문서와 범위를 제한한 참조 분석이 함께 제약하며 Rust 내부 구조나 최적화 방법을 규정하지 않습니다.

- CAB 폴더는 압축 기록 단위이며 파일은 그 안의 출력 범위입니다. 파일 경계, CFDATA 경계와 볼륨 간 계속은 기록을 자동으로 재설정하지 않고 새 논리 폴더만 재설정합니다. `cbUncomp=0`인 볼륨 간 압축 조각은 하나의 논리 프레임으로 연결하되 각 물리 CFDATA 체크섬을 별도로 검증합니다. 이 체크섬은 바이트·워드 XOR이며 예약 데이터를 포함하고 CRC나 연결 후 단일 체크섬으로 대체할 수 없습니다. 메타데이터, 길이, 볼륨 관계와 페이로드 경계는 별도로 검사합니다.
- 각 MSZIP 프레임에는 CK와 완전한 원시 Deflate 스트림이 들어 있습니다. 트리와 비트스트림은 프레임마다 재설정하고 폴더마다 최대 32 KiB 기록을 유지합니다. 프레임 제한은 압축 32,780바이트와 출력 32,768바이트입니다. 프로젝트의 공유 Deflate 사전 기록 인터페이스를 재사용하고 두 번째 Deflate 구현을 만들지 마세요. 잘린 스트림을 0으로 채우거나 강제로 패딩해도 엄격한 종료 검사를 대신하지 못합니다.
- CAB LZX는 리틀엔디언 16비트 워드를 사용하며 각 워드 안에서는 최상위 비트부터 처리합니다. 15~21비트 윈도의 위치 슬롯 수는 각각 30, 32, 34, 36, 38, 42, 50입니다. 오래된 개요의 큰 윈도 수는 전체 거리 표와 비교하여 수정해야 합니다. 트리 길이 델타는 modulo 17로 뺍니다. 정렬 트리는 주 트리와 길이 트리보다 앞에 옵니다. 주 트리, 사전 트리와 정렬 트리는 완전해야 합니다. 길이 트리는 완전히 비어 있어도 되지만 그 빈 트리를 요청하면 실패해야 합니다.
- LZX 원시 블록도 공통 블록 길이를 가집니다. 원시 데이터 전에 1~16개의 0비트를 건너뛰며 이미 워드에 정렬된 경우 전체 워드 하나를 포함합니다. 홀수 원시 블록의 0 패드 바이트는 다음 프레임 시작에서 소비할 수 있으며 생략은 다음 블록이 없는 최종 폴더 끝에서만 허용합니다. 일반 압축 프레임은 0~15개의 0 패딩 비트만 허용하고 추가 전체 워드나 임의의 후행 바이트는 허용하지 않습니다. LZX 블록은 CFDATA 프레임에 걸칠 수 있습니다. 최종 `finish`는 선언된 출력이 아직 끝나지 않은 블록을 거부해야 합니다. 이전 CAB 통합의 관대한 종료 검사 누락을 유지하지 마세요.
- LZX LZ 기록은 E8 역변환 전 바이트를 저장합니다. 새 폴더에는 참조할 수 있는 유효한 0 기록이 없습니다. E8은 각 프레임의 마지막 10바이트를 제외하고 검사합니다. 변환 크기는 부호 없는 완전한 32비트 값으로 해석하고 위치는 폴더 전체에서 누적합니다. 마지막 6바이트에 대한 오래된 설명과 양의 부호 있는 크기에서만 유효한 비교를 직접 적용할 수 없습니다.
- Quantum은 매 프레임 16비트 폐구간 산술 부호 상태를 재설정합니다. 9개의 확률 모델과 실제 사전 기록은 폴더 전체에서 유지됩니다. 선택자 값 7개가 모델도 7개뿐이라는 뜻은 아닙니다. 심볼 빈도는 매번 8씩 증가하며 총 빈도가 엄격히 3800보다 클 때만 다음 사용 전에 크기를 조정합니다. 모델별 누적 빈도 절반 처리, 네 번째와 이후 매 50번째 재정렬, 동일 빈도의 순서는 비트스트림 동작이며 언어의 기본 정렬로 임의 대체할 수 없습니다. 사전 윈도는 10~21비트이고 레벨은 1~7이며 레벨은 디코딩 의미를 바꾸지 않습니다.
- Quantum에는 별도 종료 토큰이 없고 컨테이너가 정확한 출력 길이를 제공합니다. 같은 압축 바이트가 서로 다른 합법적인 선언 길이를 만족할 수 있습니다. 예상 출력에 도달한 후 최소 2개의 0비트와 바이트 경계까지 필요한 최소 비트를 소비하여 총 2~9비트를 처리하고 압축 입력을 정확히 소진합니다. 덧붙인 0바이트, 0이 아닌 종료 비트, 잘림과 프레임 간 일치는 성공할 수 없습니다. 새 폴더는 아직 생성되지 않은 윈도 내용을 참조할 수 없습니다.

호스트는 계속해서 볼륨 집합과 소스 식별 정보의 고정, 이름 파싱, 항목 식별, 경로 사전 검사, 작업 예산, 취소, 준비와 확정을 담당합니다. 형식 계층은 호스트가 승인한 스트림만 읽고 CAB 내부 이름으로 파일을 독립적으로 열지 않습니다. 원래 이름과 UTF-8 선언은 별도로 보존합니다. 일반적인 중복 이름 레코드는 병합하지 않고 사양이 명시적으로 정의한 이어진 파일만 결합합니다. 알고리즘 오류가 호스트 취소, 할당량 또는 I/O 오류를 가려서는 안 됩니다. 공유 계약이 인터페이스와 리소스 제한을 관리하며 참조 버퍼 크기나 HRESULT 분류로 새 제품 권한을 도출하지 마세요.
