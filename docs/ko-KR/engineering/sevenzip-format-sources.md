<a name="xdzip-7z-format-sources-and-independent-implementation"></a>

# 7z 형식 출처와 독립 구현

[English](../../engineering/sevenzip-format-sources.md) | [简体中文](../../zh-CN/engineering/sevenzip-format-sources.md) | [繁體中文](../../zh-TW/engineering/sevenzip-format-sources.md) | [日本語](../../ja-JP/engineering/sevenzip-format-sources.md) | 한국어 | [Français](../../fr-FR/engineering/sevenzip-format-sources.md) | [Deutsch](../../de-DE/engineering/sevenzip-format-sources.md) | [Español](../../es-ES/engineering/sevenzip-format-sources.md) | [Português (Brasil)](../../pt-BR/engineering/sevenzip-format-sources.md) | [Русский](../../ru-RU/engineering/sevenzip-format-sources.md)

이 문서는 재사용 가능한 출처와 구현 경계를 기록합니다. 마이그레이션 승인과 남은 기능은 [issue #50](https://github.com/xiaodaozhi/xdzip/issues/50)에서 추적합니다. Rust 경로 고정과 제품의 공식 의존성 제거가 플랫폼 간, 수동 또는 릴리스 승인을 자동으로 입증하지는 않습니다. 일반적인 대체 규칙은 [독립 구현 계약](independent-rust-codecs.md)에 있습니다.

<a name="xdzip-specifications-and-reference-identity"></a>

## 사양과 참조 식별 정보

컨테이너 필드, 가변 길이 정수, 메서드 그래프, 속성과 파일 매핑은 공식 `DOC/7zFormat.txt`와 `DOC/Methods.txt`를 따릅니다. 고정된 참조는 [7-Zip 26.05 공식 소스 아카이브](https://github.com/ip7z/7zip/releases/download/26.05/7z2605-src.tar.xz), SHA-256 `0b6080b8c3e7a579161d2d495224c0e51c467576cb4c7048b0ab485ef44cc22e`입니다. 두 문서의 SHA-256은 각각 `8ec976b73ca88710286be1e991e49dbab485d9c0bf2410a80fa87e657850f82b`, `b16ea5fdc50345fa2cb031f06fe410cea0b089f74a4138ac8141b9e2d816ffc7`입니다.

LZMA 비트스트림은 Igor Pavlov의 2015-06-14 [공식 사양 패키지](https://www.7-zip.org/a/lzma-specification.7z)도 참조합니다. 아카이브 SHA-256은 `4529aca392d6b9598f72086e7d136b1e05d2863bf7a757836d3fd4f49e623b58`, 사양 텍스트의 SHA-256은 `6a4d441c2fa1d8d37b0189c7147a4b5424d5cdb122a7fc1fbebd022864ed54fb`입니다. 형식 설명, 사양의 수학과 의사 코드만 읽으며 패키지의 참조 디코더를 Rust 포팅 입력으로 사용하지 않습니다. 문서나 샘플을 다운로드해도 새로운 재배포 라이선스가 부여되지 않습니다.

일반 Deflate는 RFC 1951을 따르고 ZIP, GZIP 및 MSZIP과 한 구현을 공유합니다. Deflate64는 그 구현에서 64 KiB 기록, 거리 코드 30/31, 길이 코드 285의 확장 규칙을 명시적으로 선택하며 일반 Deflate/MSZIP은 바뀌지 않습니다. [PKWARE APPNOTE](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT)의 향상된 Deflate 절은 개요에 불과합니다. 구체적인 비트스트림 차이는 고정 참조의 사실 분석과 독립 읽기 검증으로 보완하며 개요를 완전한 사양으로 제시해서는 안 됩니다. AES-CBC 수학과 독립 벡터는 [NIST SP 800-38A](https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf)를 따릅니다. 7z 속성, UTF-16LE 암호 파생과 0 채움은 추가 형식 규칙이며 ZIP AES나 PKCS#7에서 추론할 수 없습니다. BZip2 설명서와 PPMd 형식 이름은 과거 비트스트림, 모델 재시작 또는 종료를 완전히 명시하지 않으므로 설명서만 읽어 이 구현을 완성할 수 있다고 서술해서는 안 됩니다.

<a name="xdzip-reference-analysis-and-original-coding"></a>

## 참조 분석과 독창적 코드 작성

분석자는 고정된 원본 아카이브의 다음 그룹을 선택적으로 읽어 생략된 필드, 수학 상태, 용량 재시작, 종료와 오류 관계를 보완합니다. 인계에는 사실과 식별 요구 사항만 포함하며 참조 코드, 줄별 재작성, 클래스나 배열 배치, 함수 분해, 복사한 제어 흐름, 번역된 주석은 포함하지 않습니다. Rust 알고리즘은 해당 참조 구현을 읽지 않은 구현자가 독립적으로 구성하며 분석자는 자신이 분석한 알고리즘을 구현하지 않습니다. 컨테이너 작성자도 공식 컨테이너 구현을 읽지 않고 문서와 사실 인계에 따라 독립적으로 코드를 작성합니다.

| 분석 주제 | 고정 아카이브의 참조 범위 | 라이선스 구분 |
| --- | --- | --- |
| LZMA1/LZMA2 | `C/LzmaDec.c/.h`, `C/Lzma2Dec.c/.h`, 인코더의 range, chunk/reset, 속성 부분과 C++ Lzma/Lzma2 래퍼 | 이 C 파일은 퍼블릭 도메인을 선언하며 C++ 래퍼는 아카이브 기본 조건을 사용 |
| 컨테이너 상세 | `CPP/7zip/Archive/7z/7zIn.cpp`의 외부 속성, 하위 스트림 체크섬 상속과 파일 속성 | 아카이브 기본 LGPL-2.1-or-later |
| 7z AES | `CPP/7zip/Crypto/7zAes.cpp/.h`, MyAes, FilterCoder와 7z 코덱 래퍼의 블록 길이, 암호, 패딩, 완료 부분 | 아카이브 기본 LGPL-2.1-or-later. 기반 AES 구현은 읽거나 포팅하지 않음 |
| BZip2 | BZip2Decoder, Const, Crc 및 일반 7z와 독립 bz2 래퍼의 오류 처리 차이 | 아카이브 기본 LGPL-2.1-or-later. 형식 이름으로 다른 구현의 라이선스를 부여하지 않음 |
| PPMd7 | `C/Ppmd7.c/.h`, `Ppmd7Dec.c`, `Ppmd7Enc.c`의 수학 상태, 가상 용량과 재시작 사실. C++ Ppmd 래퍼의 속성 및 종료 제약 | 이 C 파일은 퍼블릭 도메인을 선언하며 C++ 래퍼는 기본 조건을 사용. 컨텍스트 배치, 할당자 구현과 제어 흐름은 복사하지 않음 |
| Deflate64 | `CPP/7zip/Compress/DeflateConst.h`와 DeflateDecoder의 64비트 모드 부분, Deflate64Register와 7z 속성 허용 | 아카이브 기본 LGPL-2.1-or-later. 일반 Deflate와의 형식 차이만 인계 |
| ISA, Delta, Swap | `C/Bra.c`, `Bra86.c`, `Bra.h`, `Delta.c`, `SwapBytes.c`의 스칼라 의미와 C++ 메서드 등록, 속성 및 끝부분 래퍼 | 이 C 파일은 퍼블릭 도메인을 선언하며 C++ 래퍼는 별도로 기본 조건을 사용. SIMD 전략은 형식 규칙이 아님 |
| BCJ2 | `C/Bcj2.c/.h`의 네 스트림과 수학 상태, C++ Bcj2Coder의 디코더 완료와 크기 래퍼 | C 파일은 퍼블릭 도메인을 선언하며 C++ 래퍼는 기본 조건을 사용. 인코더 본체는 분석하거나 복사하지 않음 |

고정 아카이브와 파일의 식별 정보, 실제 읽은 범위와 사실 인계 요약을 개발 증거와 함께 보관합니다. 참조를 업그레이드할 때 다시 확인하며 오래된 줄 번호로 새 버전의 동작을 입증하지 않습니다. 사양 상수, 식별자와 수학 관계는 형식 구현에 사용할 수 있지만 내부 최적화와 과거에 정의되지 않은 동작은 제품에 권한을 부여하지 않습니다. 퍼블릭 도메인 선언은 전체 아카이브, 문서, C++ 래퍼나 다른 샘플로 확장되지 않습니다.

이는 엄격한 클린룸이나 법적 위험이 없다는 주장이 아닙니다. 제3자 출처, 공식 백엔드를 포함한 과거 배포와 독립 테스트 도구는 각자의 조건을 따릅니다. 현재 제품은 공식 백엔드를 빌드하지 않습니다. [라이선스](licensing.md)와 [대응 소스](source-delivery.md)를 참조하세요.

<a name="xdzip-interoperability-boundaries"></a>

## 상호 운용 경계

자체 작성기와 리더 사이의 성공은 독립 참조를 대신하지 못합니다. 테스트는 독립 압축기의 알려진 출력, 자체 압축 스트림의 공식 읽기, 전체 입력 소비, 종료, CRC, 손상, 할당량, 취소와 소스 오류를 별도로 검증합니다. 구성 요소 검증과 호스트, 프로세스, 플랫폼 승인은 별도로 기록합니다.

과거 호환성 허용은 각각 확인해야 합니다. 예를 들어 BZip2 7z 노드는 마지막 바이트의 0이 아닌 미사용 비트와 블록 끝에서 추가 횟수 바이트 없이 정확히 네 번 반복된 바이트를 허용할 수 있지만 올바른 CRC, 선언된 크기와 완전한 스트림 종료는 여전히 요구합니다. 이것이 추가 후행 바이트나 연결된 스트림을 허용하지는 않습니다. 마지막 바이트의 비트와 추가 바이트는 같은 종류의 패딩이 아닙니다.

PPMd7은 외부 길이가 알려진 경우와 알려지지 않은 경우의 종료 프로파일을 구분합니다. 정확한 출력 후 길이를 아는 7z 노드는 추가 END 기호를 소비하지 않고 range 종료 상태와 압축 범위를 확인하며 길이를 모르는 공통 리더는 END를 요구합니다. 형식이 정한 모델 용량 재시작과 호스트 메모리 허용은 별개입니다. 메모리 부족으로 가짜 형식 재시작을 일으켜서는 안 됩니다. 독립 비교 입력에는 고정된 공식 C 인코더가 생성한 용량 재시작 스트림이 포함됩니다. 생성기와 원본 참조 소스는 로컬 테스트 도구일 뿐 제품 의존성이 아닙니다.
