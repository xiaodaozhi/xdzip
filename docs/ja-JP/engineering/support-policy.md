<a name="xdzip-system-support-and-maintenance-policy"></a>
# システムのサポートとメンテナンスポリシー

[English](../../engineering/support-policy.md) | [简体中文](../../zh-CN/engineering/support-policy.md) | [繁體中文](../../zh-TW/engineering/support-policy.md) | 日本語 | [한국어](../../ko-KR/engineering/support-policy.md) | [Français](../../fr-FR/engineering/support-policy.md) | [Deutsch](../../de-DE/engineering/support-policy.md) | [Español](../../es-ES/engineering/support-policy.md) | [Português (Brasil)](../../pt-BR/engineering/support-policy.md) | [Русский](../../ru-RU/engineering/support-policy.md)

XDZip がサポートするシステムと保守方針を示します。対応するパッケージは[最新の安定版](https://github.com/xiaodaozhi/xdzip/releases/latest)からダウンロードでき、既知の制限はリリースノートで確認できます。

<a name="xdzip-initial-support-scope"></a>
## 初期のサポート範囲

| プラットフォーム | 初期のサポート範囲 | 検証内容と制限 |
| --- | --- | --- |
| macOS | macOS 14 以降、Apple Silicon（arm64）、v0.3.2 以降は Intel（x86_64） | デプロイメントターゲットは 14.0 です。GitHub ではアーキテクチャごとに DMG とアプリの ZIP を配布します。各アーキテクチャの両パッケージには、整合性のために ad-hoc 署名を施した同じアプリが含まれます。 |
| Windows | Microsoft が引き続きメンテナンスしている Windows 11 x86_64 | MSVC ビルド、ユーザー単位のインストーラー、CI での実際のインストール、アップグレード、切り戻し、アンインストール。Windows Server の CI はすべての Windows 11 バージョンを対象にしていません。ARM64 パッケージはありません。 |
| Linux | Ubuntu 24.04 LTS x86_64、X11、glibc 2.39 以降 | 現行の .deb と Qt SDK を備えていない隔離環境での実行を検証しています。他のディストリビューション、古い glibc、Wayland、ARM64 は対象外です。 |

macOS の最小バージョンは現行のデプロイメントターゲットに従います。以前の設計に記載されたバージョン 13 は未検証です。Windows 10 の標準サポートは 2025-10-14 に終了しました。Windows 10 は初期サポートの対象外であり、LTSC や ESU も自動的には対象になりません。[Microsoft のライフサイクルに関する発表](https://learn.microsoft.com/en-us/lifecycle/announcements/windows-10-end-of-support)を参照してください。[Qt 6.8 の対応プラットフォーム](https://doc.qt.io/qt-6.8/supported-platforms.html)は依存ライブラリの対応範囲です。xdzip の動作は個別に検証します。

<a name="xdzip-maintenance"></a>
## メンテナンス

この個人プロジェクトは、対応可能な範囲でメンテナンスします。有償サービス、固定の応答時間、実施していないセキュリティ上の対応は約束しません。最新の公開安定版を優先します。開発プレビューでは互換性を変更することがあります。既知の問題と切り戻し方法はリリースノートに記載します。セキュリティ上の問題が判明した場合は、影響する配布ファイルを取り下げることがあります。デスクトップアプリでは「xdzip について」から更新の確認とインストールの開始ができます。バックグラウンドでの更新確認や無人インストールは行いません。現在のソースには CLI の更新コマンドもあります。[コマンドライン仕様](command-line.md)に対象となるインストールと公開済みの配布ファイルとの違いを示します。

候補版では Qt 6.8.2 と既存の Rust、Cargo のバージョンを固定し、同じ条件で検証できるようにします。古いパッチに脆弱性がないことや、無料の LTS 更新が恒久的に提供されることは保証しません。公開前に、メンテナーは Qt の公式セキュリティアドバイザリと利用可能なオープンソースの更新を確認し、保守中の公開版を優先して評価します。依存関係に未修正のセキュリティ問題があり、xdzip に影響する場合は、該当するリリースを保留します。Qt を更新するときは、ソース、モジュールライセンス、Windows の非公開インターフェース、動的な置き換え、3 プラットフォームのビルド、実際のメニューとウィンドウの動作を再検証します。有償メンテナンスの購入は任意です。

<a name="xdzip-approved-scope-and-changes"></a>
## 承認済みの範囲と変更

アーキテクチャ、システム、配布経路を追加するときは、対応を実装して検証したうえでこのページを更新します。対象プラットフォームの根拠は [ADR-011](../design/architecture-decisions.md#xdzip-adr-011-initial-support-scope-and-personal-project-maintenance-policy) を参照してください。

脆弱性の報告は[セキュリティポリシー](../../../SECURITY.md)に従ってください。
