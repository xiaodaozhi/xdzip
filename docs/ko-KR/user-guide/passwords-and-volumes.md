# 암호 및 분할 볼륨

[English](../../user-guide/passwords-and-volumes.md) | [简体中文](../../zh-CN/user-guide/passwords-and-volumes.md) | [繁體中文](../../zh-TW/user-guide/passwords-and-volumes.md) | [日本語](../../ja-JP/user-guide/passwords-and-volumes.md) | 한국어 | [Français](../../fr-FR/user-guide/passwords-and-volumes.md) | [Deutsch](../../de-DE/user-guide/passwords-and-volumes.md) | [Español](../../es-ES/user-guide/passwords-and-volumes.md) | [Português (Brasil)](../../pt-BR/user-guide/passwords-and-volumes.md) | [Русский](../../ru-RU/user-guide/passwords-and-volumes.md)

<a name="xdzip-passwords"></a>

## 암호

AES Crypt와 파일 이름을 암호화한 7z 같은 압축 파일은 파일 목록을 표시하기 전에 암호가 필요합니다. 다른 압축 파일은 압축을 풀거나 검사하거나 미리 볼 때 암호를 요구할 수 있습니다. 암호를 입력할 때 공백과 대소문자를 그대로 유지하세요. ZIP AES-256은 지원하는 프로필에서 사용할 수 있습니다. ZIP AES-128/192와 기존 ZipCrypto는 지원하지 않습니다. 암호가 맞아도 작업이 실패하면 지원하는 암호화인지, 파일이 손상되지 않았는지 확인하세요. [지원되는 암호화](../engineering/password-extraction.md)

암호로 보호하는 압축 파일을 만들려면 암호화를 지원하는 형식을 선택하세요. ZIP AES-256은 파일 내용을 보호하며 파일 이름은 그대로 표시합니다. 파일 목록도 보호해야 하면 7z를 선택하고 파일 이름 암호화를 켜세요. 암호는 압축 파일과 따로 보관하세요.

<a name="xdzip-split-volumes"></a>

## 분할 볼륨

1. 같은 압축 파일의 모든 볼륨을 하나의 폴더에 넣고 원래 파일 이름을 유지하세요.
2. 지원되는 볼륨을 여세요. XDZip은 같은 폴더에서 나머지를 찾습니다. 번호가 붙은 ZIP 및 7z, PKZIP, 지원되는 RAR 및 CAB 세트는 이름 규칙이 서로 다릅니다.
3. 세트가 불완전하면 빠진 파일을 구한 다음 압축 파일을 다시 여세요. 수동으로 합치거나 이름을 바꾸면 읽을 수 없게 될 수 있습니다.

분할 압축 파일을 만들려면 ZIP 또는 7z를 선택하고 만들기 창에서 볼륨 크기를 지정하세요. 세트 전체를 공유하세요. 압축 파일을 분할해도 XDZip에서 편집할 수 있게 되지는 않습니다. [볼륨 이름과 지원 프로필](../engineering/archive-opening.md)
