# パスワードと分割ボリューム

[English](../../user-guide/passwords-and-volumes.md) | [简体中文](../../zh-CN/user-guide/passwords-and-volumes.md) | [繁體中文（台灣）](../../zh-TW/user-guide/passwords-and-volumes.md) | 日本語 | [한국어](../../ko-KR/user-guide/passwords-and-volumes.md) | [Français](../../fr-FR/user-guide/passwords-and-volumes.md) | [Deutsch](../../de-DE/user-guide/passwords-and-volumes.md) | [Español](../../es-ES/user-guide/passwords-and-volumes.md) | [Português (Brasil)](../../pt-BR/user-guide/passwords-and-volumes.md) | [Русский](../../ru-RU/user-guide/passwords-and-volumes.md)

<a name="xdzip-passwords"></a>

## パスワード

AES Crypt や、ファイル名を暗号化した 7z などのアーカイブでは、ファイル一覧を表示する前にパスワードが必要です。形式によっては、展開、検証、プレビュー中に入力を求められます。パスワードを入力するときは、空白と大文字・小文字を元のままにしてください。ZIP AES-256 は対応プロファイルで使えます。ZIP AES-128/192 と従来の ZipCrypto には対応していません。パスワードが合っていても開けない場合は、暗号化方式への対応とファイルの破損を確認してください。[対応する暗号化方式](../engineering/password-extraction.md)

パスワード付きアーカイブを作るには、暗号化に対応する形式を選びます。ZIP AES-256 はファイルの内容を保護し、ファイル名は表示されます。ファイル一覧も保護するには、7z を選び、ファイル名の暗号化を有効にしてください。パスワードはアーカイブとは別に保管します。

<a name="xdzip-split-volumes"></a>

## 分割ボリューム

1. 同じアーカイブのボリュームをすべて 1 つのフォルダーに置き、元のファイル名を保持します。
2. 対応するボリュームを開きます。XDZip は同じフォルダーから残りのパートを見つけます。連番の ZIP と 7z、PKZIP、対応する RAR と CAB のセットではファイル名の規則が異なります。
3. パートが足りない場合は、不足分を入手してアーカイブを開き直します。手作業で連結したり名前を変更したりすると、読み込めなくなることがあります。

分割アーカイブを作るには、作成ウィンドウで ZIP または 7z を選んでボリュームのサイズを指定します。共有するときはすべてのパートを渡してください。分割したアーカイブは XDZip で編集できるようにはなりません。[ボリュームの名称と対応プロファイル](../engineering/archive-opening.md)
