<a name="xdzip-bandizip-format-inventory-and-xdzip-feasibility"></a>

# Bandizip 형식 목록과 xdzip 구현 가능성

[English](../../engineering/bandizip-format-assessment.md) | [简体中文](../../zh-CN/engineering/bandizip-format-assessment.md) | [繁體中文](../../zh-TW/engineering/bandizip-format-assessment.md) | [日本語](../../ja-JP/engineering/bandizip-format-assessment.md) | 한국어 | [Français](../../fr-FR/engineering/bandizip-format-assessment.md) | [Deutsch](../../de-DE/engineering/bandizip-format-assessment.md) | [Español](../../es-ES/engineering/bandizip-format-assessment.md) | [Português (Brasil)](../../pt-BR/engineering/bandizip-format-assessment.md) | [Русский](../../ru-RU/engineering/bandizip-format-assessment.md)

이 평가는 Bandizip이 나열한 형식을 xdzip 구현 범위와 비교하고 소스, 테스트 자료, 자동 검증이 필요한 기능을 식별합니다. 목록은 Bandizip 제품 페이지와 형식 도움말에서 가져왔으며 2026-11-15에 확인했습니다.

- [Bandizip 제품 페이지](https://www.bandisoft.com/bandizip/)에는 `7Z`, `ACE`, `AES`, `ALZ`, `ARJ`, `BH`, `BIN`, `BR`, `BZ`, `BZ2`, `CAB`, `Compound(MSI)`, `DAA(1.0)`, `DEB`, `EGG`, `GZ`, `IMG`, `ISO`, `ISZ`, `LHA`, `LZ`, `LZH`, `LZMA`, `PMA`, `RAR`, `RAR5`, `SFX(EXE)`, `TAR`, `TBZ/TBZ2`, `TGZ`, `TLZ`, `TXZ`, `UDF`, `WIM`, `XPI`, `XZ`, `Z`, `ZIP`, `ZIPX`, `ZPAQ`, `PEA`, `UU`, `UUE`, `XXE`, `ASAR`, `ZSTD`, `NSIS`, `LZ4`가 나열됩니다.
- 같은 페이지는 `ZIP`, `7Z`, `ZIPX`, `EXE (SFX)`, `TAR`, `TGZ`, `LZH`, `ISO`, `GZ`, `XZ`를 생성할 수 있다고 설명합니다. 더 긴 목록은 주로 추출 목록입니다.
- [Bandizip ALZ 및 EGG 도움말](https://www.bandisoft.com/bandizip/help/how-to-open-alz-and-egg-files/)은 ALZ의 Deflate, 난독화 Deflate, BZip2와 ZipCrypto, EGG의 Deflate, BZip2, LZMA, AZO, AES/LEA 암호화, 솔리드 블록, 분할 압축 파일을 설명합니다.
- [Bandizip ZPAQ 도움말](https://www.bandisoft.com/bandizip/help/what-is-a-zpaq/)은 ZPAQ를 독자적인 저널형 압축 형식으로 설명하고 Bandizip이 처리할 수 있는 압축 방식을 나열합니다.
- [디스크 이미지 및 기타 압축 형식 소스 안내서](disk-image-archive-format-sources.md)는 DAA, ISZ, ZPAQ, PEA, BOND, J2J의 공개 소스, 라이선스, 코퍼스, 클린룸 검사를 기록합니다. BOND와 J2J는 현재 공식 Bandizip 압축 형식으로 확인되지 않아 과거 조사 단서로만 남습니다.

<a name="xdzip-current-xdzip-disposition"></a>

## xdzip의 현재 상태

표의 의미는 다음과 같습니다. **허용**은 현재 레지스트리, runtime, CLI, 데스크톱 경로에 명시된 프로필에 대한 자동화 증거가 있음을 뜻합니다. **별칭/프로필**은 콘텐츠 검증 후에만 기존 컨테이너를 통해 확장자를 허용한다는 뜻입니다. **후보**는 향후 진입점으로 유용하지만 디코더를 주장하지 않는다는 뜻입니다. **차단**은 공개되고 독립적으로 시험 가능한 데이터 형식 명세와 호환 구현 또는 자체 알고리즘이 준비되기 전에는 책임 있게 허용할 수 없다는 뜻입니다. **제외**는 프로젝트가 지원하지 않기로 명시적으로 결정한 형식이며 제품 결정을 재검토하지 않는 한 레지스트리나 탐지기를 추가하지 않습니다.

| Bandizip 이름 | xdzip 상태 | 현재 설명 가능한 범위 또는 이유 |
| --- | --- | --- |
| 7Z | 허용 | 선언된 암호, 솔리드, 분할 프로필을 포함하는 독립 Rust 7z 백엔드. |
| ACE | 제외 | 제품 결정으로 ACE 지원을 영구 포기하며 디코더, 탐지기, 확장자, MIME 또는 연결 항목을 추가하지 않음. |
| AES | 허용 | AES Crypt v0–v3, 암호 및 HMAC 프로필. 임의의 암호화 압축 파일이 아닌 `.aes` 엔벨로프. |
| ALZ | 허용 | 일반 단일 볼륨 Store와 raw Deflate를 읽기 전용으로 지원하고 정확한 크기와 CRC-32 검증. 변경된 BZip2, ZipCrypto, 분할 볼륨은 제외. |
| ARJ | 허용 | 전체 헤더, 크기, 멤버 CRC-32를 검증하는 읽기 전용 Store와 방식 1–4. |
| BH | 허용 | 일반 `BH 05 07` 멤버, 방식 8 raw Deflate, 정확한 크기와 CRC-32를 읽기 전용으로 지원. Store, 암호화, 분할과 다른 방식은 제외. |
| BIN | 후보 | 디스크, 실행 파일, 데이터에 걸쳐 모호한 확장자. 검증된 내부 형식만 탐지하며 일반 BIN을 압축 디코더로 등록하지 않음. |
| BR | 허용 | 제한된 Brotli 스트림과 TAR.BR 프로필. |
| BZ / BZ2 | 허용 | BZip2 단일 파일 스트림과 선언된 TAR.BZ2 별칭. |
| CAB | 허용 | Store, MSZip, Quantum, LZX 및 승인된 캐비닛 세트를 포함하는 독립 Rust CAB 백엔드. |
| Compound(MSI) | 허용 | 콘텐츠가 검증된 `.msi`의 읽기 전용 CFB 구조, 스토리지/스트림 이름과 제한된 원시 스트림 복사. MSI 테이블, 포함된 CAB 해석, 변환, 사용자 지정 동작과 설치는 제외. |
| DAA(1.0) | 제외 | 제품 결정에 따라 영구 미지원. 탐지기, 코덱, 레지스트리 항목, 판독기, 기록기를 허용하지 않음. |
| DEB | 허용 | 일반 `ar` 엔벨로프, control/data 보기와 허용된 압축 멤버. |
| EGG | 허용 | 암호화되지 않고 솔리드가 아닌 일반 단일 볼륨 Store와 raw Deflate를 읽기 전용으로 지원하고 정확한 블록 크기와 CRC-32 검증. AZO, BZip2, LZMA, 암호화, 분할 및 다른 프로필은 제외. |
| GZ | 허용 | 연결된 스트림과 TAR.GZ를 포함한 GZIP 멤버. |
| IMG | 후보 | 모호한 디스크 이미지 확장자. 콘텐츠 기반 ISO/UDF 허용이 일반 IMG 지원을 뜻하지 않음. |
| ISO | 허용 | ISO9660/Joliet/Rock Ridge 읽기와 일반 ISO9660 + Joliet 데이터 이미지 생성. |
| ISZ | 제외 | 제품 결정으로 ISZ 지원을 포기하며 탐지기, 레지스트리 항목, 키 처리를 추가하지 않음. |
| LHA / LZH | 허용 | LHA 레벨 0–3, Store, LH1–LH7, LZS/LZ5 및 LH5 생성. |
| LZ | 허용 | Lzip v1과 TAR.LZ. |
| LZMA | 허용 | LZMA Alone과 TAR.LZMA. |
| PMA | 허용 | CP/M 프레이밍과 CRC 검사를 사용하는 PM0/PM1/PM2 읽기 전용 프로필. |
| RAR / RAR5 | 허용 | 선언된 범위 내 구형, RAR5, RAR7 확장, 암호, 솔리드와 분할 프로필을 포함하는 독립 Rust RAR 백엔드. |
| SFX(EXE) | 별칭/프로필 | ZIP 판독기가 ZIP SFX 접두사를 검증함. 일반 실행 파일의 추출이나 실행은 절대 허용하지 않음. 자체 Windows x64 Store/Deflate ZIP 생성은 명시적 `.sfx.exe` 프로필로 제공하며 자동 실행하지 않음. |
| TAR | 허용 | 레지스트리에 명시된 TAR과 필터 적용 TAR.GZ/TAR.BZ2/TAR.LZMA/TAR.LZ/TAR.XZ/TAR.Z/TAR.BR/TAR.LZ4/TAR.ZST 프로필. |
| TBZ / TBZ2 | 별칭/프로필 | BZip2 필터 TAR 별칭. |
| TGZ | 별칭/프로필 | GZIP 필터 TAR 별칭. |
| TLZ | 별칭/프로필 | LZMA 필터 TAR 별칭. |
| TXZ | 별칭/프로필 | XZ 필터 TAR 별칭. |
| UDF | 허용 | 하이브리드 이미지 네임스페이스 규칙을 포함한 읽기 전용 UDF 이미지. |
| WIM | 제외 | 제품 결정으로 WIM에 필요한 이미지 전용 판독기, 리소스 모델, 코퍼스를 추진하지 않음. |
| XPI | 별칭/프로필 | XPI 확장자의 ZIP 콘텐츠이며 콘텐츠 검증이 기준. |
| XZ | 허용 | LZMA2, 선언된 검사와 허용된 필터를 사용하는 XZ와 TAR.XZ. |
| Z | 허용 | Unix compress LZW와 TAR.Z/TAZ 별칭. |
| ZIP | 허용 | ZIP/ZIP64, AES-256, 지원되는 분할, 방식, 재작성 프로필. |
| ZIPX | 별칭/프로필 | XZ 같은 검증된 ZIP 방식에 대해 허용하며 별도 컨테이너 구현이 아님. 현재 ZIPX 생성은 제공하지 않음. |
| ZPAQ | 제외 | 제품 결정으로 이 솔리드 저널형 형식에 필요한 ZPAQL 가상 머신을 구현하지 않음. |
| PEA | 허용 | PEA 1.x 단일 볼륨, UTF-8, 암호화되지 않은 PCOMPRESS0 저장 파일/디렉터리 읽기 전용 프로필. 검사, 암호화, 압축, 분할, 생성과 수정은 제외. |
| UU / UUE | 허용 | 물리적 멤버와 줄 검증을 포함하는 제한된 UU/UUE 엔벨로프. |
| XXE | 허용 | 제한된 XXE 엔벨로프 프로필. |
| ASAR | 허용 | 순서가 있는 메타데이터, 저장 범위, 선언된 SHA-256 검사. |
| ZSTD | 허용 | 표준 Zstandard 프레임과 TAR.ZST/TZST 프로필. |
| NSIS | 허용 | 정적 `firstheader` 메타데이터만 지원. 콘텐츠 탐지로 제한된 PE overlay를 검증하며 `.exe`는 끌어다 놓기나 셸 연결 항목이 아니고 PE, 스크립트, 플러그인, 페이로드 실행을 허용하지 않음. |
| LZ4 | 허용 | 표준 LZ4 프레임과 TAR.LZ4. |

따라서 이 목록은 `BIN`, `IMG`, `SFX(EXE)`의 일반 처리기 추가를 정당화하지 않습니다. 이 이름은 실행 파일, 디스크 또는 설치 프로그램 데이터를 나타낼 수 있으며 확장자만으로 압축 기능을 확정할 수 없습니다. NSIS에는 콘텐츠를 검증하는 더 좁은 정적 메타데이터 판독기가 있으며 실행이나 페이로드 권한을 부여하지 않고 `.exe` 항목을 끌어다 놓기 및 셸 연결에서 숨깁니다. Compound(MSI)는 콘텐츠 검증 CFB 판독기를 갖지만 `.msi` 확장자도 실행 권한을 부여하지 않습니다.

DAA, ISZ, ZPAQ, PEA, BOND, J2J는 [디스크 이미지와 압축 형식 소스](disk-image-archive-format-sources.md)를 참조하세요. PEA는 문서화된 보수적 읽기 전용 프로필로 제공됩니다. ACE, DAA, ISZ, ZPAQ는 제품 결정으로 제외됩니다. BOND와 J2J는 Bandizip 압축 형식이라는 확인이 없어 제외됩니다.

<a name="xdzip-delivery-plan"></a>

## 제공 계획

1. **목록과 레지스트리 조건.** 허용된 프로필마다 `specs/engineering/archive-formats.json`에 기계가 읽을 수 있는 항목 하나를 유지합니다. Rust, C++, JavaScript, 연결 소비자를 생성하고 두 레지스트리 검사를 실행합니다. 새 형식마다 형식 계약과 대응 수용 항목을 추가합니다.
2. **코드보다 먼저 형식 계약.** 공개 소스 리비전과 SHA-256을 고정하고 헤더 필드, 이름 인코딩, 물리적 종료, 방식, 체크섬, 중복 식별 정보와 리소스 제한을 문서화합니다. 참조 도구는 완전한 자체 샘플을 생산할 때만 증거이며 충돌이나 단일 Store 샘플은 제한으로 기록합니다.
3. **원시 알고리즘 테스트.** Qt와 독립적인 제한 판독기를 구현합니다. 빈 페이로드와 경계, 허용된 모든 방식, 정확한 입력/출력/작업/메모리 예산, 취소, 타입 지정 I/O, 원본 변경, 잘림, 후행 바이트, CRC와 오류 후 종료 상태를 다룹니다. 알 수 없는 방식은 페이로드 권한 없이 계속 표시합니다.
4. **컨테이너와 제품 허용.** 코덱 crate, runtime 약한/강한 탐지, CLI list/test/extract, 데스크톱 레이블과 연결 테스트 자료를 추가합니다. 고정 메타데이터와 물리적 중복 선택을 검증하고 관련 자동 Qt와 레이아웃 검사를 실행합니다. 이 연구 및 형식 부분에는 수동 시각 수용이 필요하지 않습니다.
5. **우선순위.** 보수적 ALZ Store/raw Deflate, 암호화되지 않고 솔리드가 아닌 EGG Store/raw Deflate, CFB/Compound(MSI) 메타데이터, 보수적 PEA 저장 프로필, NSIS 정적 `firstheader` 메타데이터가 허용되었습니다. ACE, DAA, WIM, ISZ, ZPAQ, BOND, J2J는 제품 결정으로 영구 제외됩니다. ZIPX 생성은 현재 기록기 범위에서 제외됩니다. [자체 SFX 기록기](sfx-creation.md)는 별도로 구현되었으며 읽기 별칭 지원에서 자동으로 따라오지 않습니다.
6. **릴리스 경계.** 코덱, runtime, CLI, 데스크톱 증거가 완성될 때까지 후보 형식을 공개 기능 표에서 제외합니다. 지원하지 않는 방식과 참조 도구의 제한은 issue에 기록하고 Store나 다른 컨테이너로 조용히 폴백하지 않습니다.

현재 BH 부분은 이 계획을 따릅니다. 읽기 전용이며 암호나 분할 경로가 없고 독립적으로 제한된 Deflate 판독기, 필수 CRC-32, 정확한 물리적 종료를 사용합니다. Compound(MSI)도 같은 범위를 따릅니다. CFB FAT/DIFAT, 디렉터리 트리, mini-stream, 순환 검증은 Qt와 독립적이며 불변 메타데이터 스냅샷이 일치한 뒤에만 원시 스트림을 복사합니다. NSIS는 고정 firstheader 메타데이터 기록까지만 처리하고 페이로드 접근에는 미지원을 반환합니다. MSI 또는 NSIS 설치 프로그램 동작을 실행하지 않습니다.
