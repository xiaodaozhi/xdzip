<a name="xdzip-auxiliary-algorithm-work-accounting"></a>

# 알고리즘 보조 작업량 집계

[English](../../engineering/auxiliary-decoding-work.md) | [简体中文](../../zh-CN/engineering/auxiliary-decoding-work.md) | [繁體中文](../../zh-TW/engineering/auxiliary-decoding-work.md) | [日本語](../../ja-JP/engineering/auxiliary-decoding-work.md) | 한국어 | [Français](../../fr-FR/engineering/auxiliary-decoding-work.md) | [Deutsch](../../de-DE/engineering/auxiliary-decoding-work.md) | [Español](../../es-ES/engineering/auxiliary-decoding-work.md) | [Português (Brasil)](../../pt-BR/engineering/auxiliary-decoding-work.md) | [Русский](../../ru-RU/engineering/auxiliary-decoding-work.md)

압축 풀기와 검사에서는 디코딩한 파일 바이트, 실제 출력 및 알고리즘 보조 작업량을 따로 누적합니다. PPM 모델 초기화, 용량 한도 때문에 다시 시작하는 작업, 제어 정보나 매개변수 디코딩은 파일 내용에 포함되지 않습니다. 이 작업이 `decoded_bytes`, `written_bytes`, 파일 진행 상태나 압축 해제 비율을 늘리거나 미리 집계한 출력의 허용량을 만들면 안 됩니다. 작업 스냅샷에는 사용자에게 표시하는 보조 카운터를 추가하지 않습니다.

`DecodedSink::charge_auxiliary(bytes)`는 이미 수행한 보조 작업만 보고합니다. 기본 구현은 파일 카운터를 변경하지 않습니다. 알고리즘은 먼저 남은 한도로 예정된 작업을 허용하고 실행한 뒤 실제 작업량을 집계합니다. 실패 또는 취소 전 완료한 작업도 포함합니다. 실행하지 않은 예약은 실제 작업으로 보고하지 않습니다. 오류, 모델 초기화 또는 암호 재시도 시 이미 수행한 작업량을 돌려주지 않습니다. RAR의 내부 총 작업 한도는 파일 디코딩과 보조 작업의 합계를 계속 제한합니다.

<a name="xdzip-isolation-boundary"></a>

## 격리 경계

- `AUX_WORK=24`는 리틀 엔디언 `u64` 개수 하나만 전달하며 내용은 담지 않습니다. 각 패킷에는 1부터 65,536까지의 값이 들어가고 프레임은 그 위치에서 정확히 끝나야 합니다. 집계가 성공하면 호스트는 빈 `ACK`를 반환합니다. 0, 잘린 데이터, 뒤따르는 필드, 초과 값 및 잘못된 작업 단계를 거부합니다.
- 이 메시지는 RAR 데이터 디코딩에만 사용할 수 있습니다. 메타데이터 검색, 만들기, 수정 또는 다른 형식에서는 사용하지 않습니다. CXX는 개수를 동기적으로 전달하며 Rust 포인터를 유지하지 않습니다. 입력과 출력 슬라이스는 호출 중에만 빌립니다. C++ 예외는 콜백 안에서 안정적인 오류로 변환합니다. Rust의 스택 해제는 CXX 경계를 넘지 않으며 비정상 중단이나 프로세스 종료 시 미완료 출력을 실패로 처리합니다.
- 네이티브 디코딩 인스턴스는 보조 작업을 최대 20 GiB까지 누적하며 항목, 프로세스 재시작과 암호 변경 사이에서도 유지합니다. 호스트 수신자를 호출하기 전에 완료한 작업량을 기록합니다. 수신자가 거부해도 나중에 재시도할 때 허용량을 반환하지 않습니다. `AUX_WORK`는 `WorkerCredit`을 늘리지 않습니다.
- 런타임의 압축 풀기와 검사 작업은 독립적으로 보조 작업을 최대 20 GiB까지 누적합니다. 집계 객체는 항목과 암호 재시도 루프 밖에 있습니다. 실제 파일 바이트 및 출력 한도와 별도로 검사합니다. 한도를 넘거나 취소하면 실패를 반환하며 미완료 파일은 확정하지 않습니다. 공개 카운터의 변경으로 한도를 우회할 수 없습니다.
- `WORK=17`은 계속 디코딩된 파일 바이트를 뜻합니다. 수락에 성공했을 때만 동일한 유효 프로세스에 `PRECHARGED_DATA=23` 전달용 일회성 허용량을 부여합니다. 배달 전에 차감하며 재시작하면 지웁니다. RAR, 7z, CAB에서 사용하며 검색과 만들기에서는 사용하지 않습니다. 보조 작업량을 파일 출력 허용량으로 바꿀 수 없습니다.

높은 압축률의 허용 여부는 고정한 선언 크기, 압축된 크기 또는 전체 압축 파일 크기를 사용합니다. 런타임의 파일 디코딩 및 출력 한도는 독립적으로 적용합니다. 87바이트 파일이 모델 초기화에 수 MiB의 작업을 사용해도 파일 카운터에는 실제 87바이트만 반영합니다.

<a name="xdzip-required-acceptance-conditions"></a>

## 필수 인수 조건

- 일반 작은 파일은 64 KiB를 넘는 보조 작업 및 모델 초기화 후에도 압축 풀기와 검사를 마칠 수 있습니다. 파일 집계는 정확하며 최종 콘텐츠 검증은 성공합니다.
- 네이티브 계층과 런타임에서 반복된 실제 보조 패킷이 20 GiB에 도달할 때 각각 거부되는지 검증합니다. 암호 재시도와 프로세스 교체는 누적 작업량을 초기화하지 않습니다. 미완료 파일은 확정하지 않습니다.
- 모의 프로세스의 다른 형식과 메타데이터 단계 요청, 빈 패킷이나 너무 큰 패킷, 뒤따르는 필드, 보조 작업으로만 허용량을 얻으려는 사전 집계 출력은 모두 실패합니다.
- 일반 `WORK`는 동일한 프로세스의 사전 집계 RAR 출력에 허용량을 부여할 수 있습니다. 한 번만 사용하며 오류 또는 재시작 뒤에 재사용할 수 없습니다.
- 프로토콜 테스트 데이터는 상태 머신과 집계만 검증합니다. 완전한 Rust RAR 프로세스, 실제 런타임 업무 경로, 세 플랫폼 및 데스크톱 인수에는 별도 실행 증거가 필요합니다.

프로토콜용 테스트 구성은 프로젝트의 `tools/native-codec/main.cpp`와 해당 스텁만 컴파일합니다. 압축 알고리즘은 포함하지 않습니다.

```sh
cmake -S crates/xdzip-codec-native/tests -B build/auxiliary-protocol
cmake --build build/auxiliary-protocol --config Release
```

`XDZIP_NATIVE_CODEC_WORKER`가 테스트 구성을 가리키도록 하고 `XDZIP_AUXILIARY_PROTOCOL_FIXTURE=1`을 설정한 뒤 `cargo test -p xdzip-codec-native auxiliary_protocol -- --ignored`를 명시적으로 실행합니다. 일반적인 대상 한정 회귀 검사는 `cargo test -p xdzip-codec-native -p xdzip-runtime auxiliary`를 사용합니다. 코덱 API 테스트는 기본 수신자의 호환성을 확인합니다. 생성된 증거는 build 또는 CI에 두고 결과와 남은 항목은 [마이그레이션 작업 #51](https://github.com/xiaodaozhi/xdzip/issues/51)에 기록합니다.
