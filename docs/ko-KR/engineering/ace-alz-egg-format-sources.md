<a name="xdzip-ace-alz-and-egg-source-and-licensing-gates"></a>

# ACE, ALZ, EGG의 출처와 라이선스 요건

[English](../../engineering/ace-alz-egg-format-sources.md) | [简体中文](../../zh-CN/engineering/ace-alz-egg-format-sources.md) | [繁體中文（台灣）](../../zh-TW/engineering/ace-alz-egg-format-sources.md) | [日本語](../../ja-JP/engineering/ace-alz-egg-format-sources.md) | 한국어 | [Français](../../fr-FR/engineering/ace-alz-egg-format-sources.md) | [Deutsch](../../de-DE/engineering/ace-alz-egg-format-sources.md) | [Español](../../es-ES/engineering/ace-alz-egg-format-sources.md) | [Português (Brasil)](../../pt-BR/engineering/ace-alz-egg-format-sources.md) | [Русский](../../ru-RU/engineering/ace-alz-egg-format-sources.md)

xdzip의 ACE, ALZ, EGG 지원 및 관련 결정을 뒷받침하는 공개 기술 자료와 라이선스 제한을 설명합니다. 참조 파일은 `build/toolchain/references`에 보관하며 제품 종속성에 포함하지 않습니다.

<a name="xdzip-ace"></a>

## ACE

- Marcel Lemke의 기술 자료 *Technical information of the archiver ACE v1.2*. `wdlkmpx/unace`에서 가져왔으며 SHA-256은 `F645C91348D5DCD560A19CA21DEF849D25C304EFFAACF65B03EFBB0A893E51F9`입니다.
- 독립 구현 `droe/acefile`, 커밋 `11ed076ae19e8228148731cfbc05f019a395b3b9`. BSD-2-Clause 구현 라이선스의 SHA-256은 `AD376BD9695F0E355F5C523443ABF822BB896999A3252AAA614D313CC4F1F5B5`, 구현의 SHA-256은 `BC701569AD8521964C62256472AB44976AC57FB46E52B147A21018BC96348F1F`입니다.
- 이 구현은 LZ77, DELTA, EXE, PIC, SOUND, 암호 및 볼륨을 포함하여 ACE 1.0과 2.0을 읽을 수 있다고 명시합니다. 라이선스는 Python 구현에만 적용되며 ACE 기술에 관한 권리를 주장하지 않는다고 명시합니다.

기술 자료는 블록 헤더, CRC, 압축 없이 저장한 항목과 이전 LZ77 계열을 정의하기에 충분합니다. ACE 기술의 상업적 사용 허가에는 해당하지 않습니다. 구현이 WinAce와 unace를 언급하는 것만으로는 독립적인 상호 운용성 자료가 되지 않습니다. 제품에서 ACE를 영구히 제외합니다. xdzip에서 ACE를 구현하거나 등록하지 않으며 확장자, 감지기, MIME 유형 또는 파일 연결을 추가하지 않습니다.

<a name="xdzip-alz"></a>

## ALZ

- 형식 사양 `alkegi/docs`, 커밋 `36230c34d3056604cc63519e9855a79e894496b7`. CC0 라이선스의 SHA-256은 `F4E7F373B9B996950337E8D41A4A2939C2D90B7725E9BAF3D5084A22717AD328`입니다.
- `alz.md`의 SHA-256은 `CBD35E77AB53CFD08C0FF192FA2793438150E4935522608936B1A0942560E237`입니다.
- 문서는 ALZ와 로컬 파일 서명, 가변 폭 크기, CRC-32, UTF-8/CP949 이름, Store, RFC 1951 원시 Deflate 및 ALZ에서 변경한 BZip2 스트림을 규정합니다. 전통적인 PKWARE 암호화와 분할 볼륨 구조도 설명합니다.

xdzip은 제한된 읽기 전용 ALZ를 지원합니다. 일반적인 단일 볼륨에서 암호화 없는 Store와 원시 Deflate를 지원하고 CRC, 크기 및 종료를 정확히 검사합니다. 코덱, 런타임, CLI와 파일 연결을 자동 테스트합니다. 변형 BZip2, ZipCrypto 및 분할 볼륨은 독립적인 완전한 샘플과 별도의 리소스 및 안전성 검사가 필요합니다. 문서의 CC0 라이선스만으로 모든 구현 세부 사항에 타사 특허나 상표 의무가 없다고 판단할 수 없습니다.

<a name="xdzip-egg"></a>

## EGG

- 공개 형식 문서 `alkegi/docs`, 위와 같은 커밋 `36230c34d3056604cc63519e9855a79e894496b7` 및 CC0 라이선스.
- `egg.md`의 SHA-256은 `4CA2CB6E8FA9A4F40C30EDE05A5E977BE113A96E1527E28A7B1ED0836D8A43A3`입니다.
- 참조 모듈 `dterracino/UnEgg`, 커밋 `f21e653b02f7ab467bbc16af879e68a48dabcb2b`. EGG 사양 PDF의 SHA-256은 `0BB7E6F174CAFB9F6726D70B7DEE8DBF33C034717058B1B0EB2B372D9A8FBB37`입니다.
- UnEGG 소스 라이선스의 SHA-256은 소스 패키지의 경우 `9B012928756D9243CC549FA0A7D7B5742ED8F69B00A2BE1C964F42EF21F1D8D6`, 모듈 패키지의 경우 `30F2E45148D43D7A77574403807C13AA359EE9ACE841A00B6A5348AD961423F0`입니다.

UnEGG 라이선스는 상업적 사용에 ESTsoft 승인을 요구하며 비상업적 재배포만 허용합니다. 이 모듈을 사용하여 EGG 압축기를 개발하는 것도 금지합니다. 모든 권리는 ESTsoft가 보유합니다. 따라서 모듈은 테스트 참조에만 사용하며 복사하거나 링크하거나 제품의 종속성으로 사용할 수 없습니다.

형식 자체는 제한된 모듈과 독립적으로 구현할 수 있습니다. xdzip은 단일 볼륨 Store와 원시 Deflate의 독립 구현을 암호화 및 솔리드 압축이 없는 범위에서 허용합니다. 코덱, 런타임, CLI 및 파일 연결 테스트에서 정확한 블록 크기와 CRC-32를 확인합니다. 표준 BZip2와 문서의 LZMA 프레임에는 별도로 완전한 샘플과 검토가 필요합니다. AZO, AES/LEA, ZipCrypto, 분할 및 솔리드 압축 파일은 현재 지원 범위에 포함되지 않습니다.

<a name="xdzip-required-implementation-gates"></a>

## 구현을 포함하기 위한 요건

1. 참조 자료를 제품 트리 밖에 두고 사용하기 전에 고정 해시를 검증합니다.
2. 각 허용 방식에 독립적인 완전한 샘플을 준비합니다. 헤더만 있거나 Store만 포함한 샘플로는 방식의 상호 운용성을 입증할 수 없습니다.
3. 컨테이너, 런타임, CLI 및 데스크톱 진입점을 함께 추가합니다. 알 수 없는 방식은 표시하되 데이터 처리 권한을 부여하지 않습니다.
4. 암호, 볼륨, 솔리드 상태와 특수 개체는 별도의 출처, 안전성 및 취소 검사를 통과할 때까지 비활성화합니다.
5. 공개 기능 레지스트리를 바꾸기 전에 라이선스 결정과 샘플 식별 정보를 이슈에 기록합니다.
