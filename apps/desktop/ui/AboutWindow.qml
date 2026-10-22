pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Xdzip.Settings 1.0

OperationWindow {
    id: root
    objectName: "aboutWindow"
    title: qsTrId("about.title")
    property bool copied: false
    property var updater: typeof UpdateService === "undefined" ? null : UpdateService
    readonly property string updateState: updater ? updater.state : "unavailable"
    function updateMessage() {
        switch (updateState) {
        case "checking": return qsTrId("update.checking");
        case "up_to_date": return qsTrId("update.current");
        case "updated": return qsTrId("update.updated");
        case "available": return qsTrId("update.available").arg(updater.candidateVersion);
        case "downloading": return qsTrId("update.downloading");
        case "verifying": return qsTrId("update.verifying");
        case "downloaded": return qsTrId("update.downloaded");
        case "waiting_tasks": return qsTrId("update.waitingTasks");
        case "preparing": return qsTrId("update.preparing");
        case "installing": return qsTrId("update.installing");
        case "recovery_required": return qsTrId("update.recovery");
        case "error": {
            const code = updater.errorCode;
            if (code === "rate_limited") return qsTrId("update.rateLimited");
            if (code === "tls") return qsTrId("update.tls");
            if (code === "network" || code === "download") return qsTrId("update.network");
            if (code === "timeout" || code === "authorization_timeout") return qsTrId("update.timeout");
            if (code === "authorization_cancelled") return qsTrId("update.authorizationCancelled");
            if (code.startsWith("authorization_") || code === "helper_exit") return qsTrId("update.authorizationFailed");
            if (code === "disk" || code === "cache") return qsTrId("update.disk");
            if (code === "minimum_system") return qsTrId("update.minimumSystem");
            if (code.includes("signature") || code.includes("hash") || code === "download_size") return qsTrId("update.verificationFailed");
            return qsTrId("update.failed");
        }
        case "unavailable": return qsTrId("update.preview");
        default: return qsTrId("update.idle");
        }
    }
    width: Theme.aboutLogicalWidth
    requestedHeight: Math.min(Theme.aboutMaximumHeight, Screen.desktopAvailableHeight - 80,
                              body.implicitHeight + footer.implicitHeight)
    onVisibleChanged: if (visible) copied = false
    onActiveFocusItemChanged: if (visible) Qt.callLater(() => root.revealFocus(scroll))
    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        ScrollView {
            id: scroll
            objectName: "aboutScroll"
            onHeightChanged: if (root.visible) Qt.callLater(() => root.revealFocus(scroll))
            onContentHeightChanged: if (root.visible) Qt.callLater(() => root.revealFocus(scroll))
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            contentWidth: availableWidth
            ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
            ColumnLayout {
                id: body
                width: scroll.availableWidth
                spacing: Theme.aboutSectionGap
                Item { Layout.preferredHeight: Theme.aboutContentInset - Theme.aboutSectionGap }
                Image {
                    objectName: "aboutArtwork"
                    source: Qt.platform.os === "osx" ? "qrc:/xdzip/assets/app-icon-macos.png" : "qrc:/xdzip/assets/app-icon.png"
                    Layout.alignment: Qt.AlignHCenter
                    Layout.preferredWidth: Theme.aboutArtworkSize
                    Layout.preferredHeight: Theme.aboutArtworkSize
                    fillMode: Image.PreserveAspectFit
                    Accessible.ignored: true
                }
                Text { text: "xdzip"; Layout.alignment: Qt.AlignHCenter; font.pixelSize: Theme.aboutTitleSize * Theme.fontScale; font.bold: true; color: Theme.text }
                Text {
                    Layout.leftMargin: Theme.aboutContentInset; Layout.rightMargin: Theme.aboutContentInset; Layout.fillWidth: true
                    text: qsTrId("about.description"); horizontalAlignment: Text.AlignHCenter
                    font.pixelSize: Theme.aboutBodySize * Theme.fontScale; color: Theme.textSecondary; wrapMode: Text.Wrap
                }
                GridLayout {
                    Layout.leftMargin: Theme.aboutContentInset; Layout.rightMargin: Theme.aboutContentInset; Layout.fillWidth: true
                    columns: 2; columnSpacing: Theme.aboutSectionGap; rowSpacing: Theme.aboutRowGap
                    Repeater {
                        model: [qsTrId("about.version"), BuildInfo.version,
                                qsTrId("about.build"), String(BuildInfo.buildNumber),
                                qsTrId("about.revision"), BuildInfo.revision === "unknown" ? qsTrId("about.unknown") : BuildInfo.revision,
                                qsTrId("about.runtime"), "Qt " + BuildInfo.qtVersion + ", " + BuildInfo.platform + ", " + BuildInfo.architecture]
                        delegate: Text {
                            required property string modelData
                            required property int index
                            objectName: "aboutValue-" + index
                            Layout.fillWidth: index % 2 === 1
                            Layout.alignment: Qt.AlignTop
                            text: modelData
                            font.pixelSize: Theme.aboutBodySize * Theme.fontScale
                            color: index % 2 === 0 ? Theme.textSecondary : Theme.text
                            wrapMode: Text.WrapAnywhere
                        }
                    }
                }
                Text {
                    Layout.leftMargin: Theme.aboutContentInset; Layout.rightMargin: Theme.aboutContentInset; Layout.fillWidth: true
                    visible: BuildInfo.channel !== "stable" || BuildInfo.dirty
                    text: (BuildInfo.channel !== "stable" ? qsTrId("about.development") : "") + (BuildInfo.dirty ? "\n" + qsTrId("about.modified") : "")
                    color: Theme.textSecondary; font.pixelSize: Theme.aboutNoteSize * Theme.fontScale; wrapMode: Text.Wrap
                }
                Text {
                    Layout.leftMargin: Theme.aboutContentInset; Layout.rightMargin: Theme.aboutContentInset; Layout.fillWidth: true
                    text: qsTrId("about.license"); color: Theme.textSecondary
                    font.pixelSize: Theme.aboutNoteSize * Theme.fontScale; wrapMode: Text.Wrap
                }
                OperationButton {
                    objectName: "aboutHomepage"
                    Layout.alignment: Qt.AlignHCenter
                    text: qsTrId("about.homepage")
                    onClicked: Qt.openUrlExternally("https://github.com/xiaodaozhi/xdzip")
                }
                ColumnLayout {
                    objectName: "aboutUpdateSection"
                    Layout.leftMargin: Theme.aboutContentInset
                    Layout.rightMargin: Theme.aboutContentInset
                    Layout.fillWidth: true
                    spacing: Theme.aboutRowGap
                    Text {
                        objectName: "aboutUpdateStatus"
                        Layout.fillWidth: true
                        text: root.updateMessage()
                        textFormat: Text.PlainText
                        horizontalAlignment: Text.AlignHCenter
                        font.pixelSize: Theme.aboutBodySize * Theme.fontScale
                        color: root.updateState === "error" || root.updateState === "recovery_required" ? Theme.menuDanger : Theme.textSecondary
                        wrapMode: Text.Wrap
                        Accessible.name: text
                    }
                    ProgressBar {
                        objectName: "aboutUpdateProgress"
                        Layout.fillWidth: true
                        Layout.preferredHeight: Theme.aboutUpdateProgressHeight
                        visible: root.updater && ["downloading", "verifying", "preparing"].includes(root.updateState)
                        indeterminate: !root.updater || root.updater.progress < 0 || root.updateState !== "downloading"
                        value: root.updater ? Math.max(0, root.updater.progress) : 0
                        Accessible.name: qsTrId("update.progress")
                    }
                    Text {
                        Layout.fillWidth: true
                        visible: root.updater && ["available", "downloaded"].includes(root.updateState) && !root.updater.canInstall
                        text: root.updater && !root.updater.trusted ? qsTrId("update.unsigned") : qsTrId("update.manual")
                        font.pixelSize: Theme.aboutNoteSize * Theme.fontScale
                        color: Theme.textSecondary
                        horizontalAlignment: Text.AlignHCenter
                        wrapMode: Text.Wrap
                    }
                    RowLayout {
                        Layout.alignment: Qt.AlignHCenter
                        spacing: Theme.aboutRowGap
                        OperationButton {
                            objectName: "aboutCheckUpdate"
                            text: qsTrId("update.check")
                            enabled: root.updater && !root.updater.busy
                            onClicked: root.updater.check()
                        }
                        OperationButton {
                            objectName: "aboutCancelUpdate"
                            visible: root.updater && root.updater.busy && root.updateState !== "installing"
                            enabled: root.updater && root.updateState !== "installing"
                            text: qsTrId("update.cancel")
                            onClicked: root.updater.cancel()
                        }
                    }
                    OperationButton {
                        objectName: "aboutInstallUpdate"
                        Layout.alignment: Qt.AlignHCenter
                        visible: root.updater && ["available", "downloaded"].includes(root.updateState)
                        text: root.updater && root.updater.canInstall ? qsTrId("update.install") : root.updater && root.updater.trusted ? qsTrId("update.download") : qsTrId("update.release")
                        onClicked: {
                            if (root.updater.canInstall) root.updater.downloadAndInstall();
                            else if (root.updater.trusted) root.updater.download();
                            else root.updater.openReleasePage();
                        }
                    }
                    OperationButton {
                        objectName: "aboutUpdateRecovery"
                        Layout.alignment: Qt.AlignHCenter
                        visible: root.updater && root.updateState === "recovery_required"
                        text: qsTrId("update.release")
                        onClicked: root.updater.openReleasePage()
                    }
                    OperationButton {
                        objectName: "aboutOpenUpdate"
                        Layout.alignment: Qt.AlignHCenter
                        visible: root.updater && root.updater.downloaded
                        text: qsTrId("update.openDownload")
                        onClicked: root.updater.openDownloadFolder()
                    }
                }
                Item { Layout.preferredHeight: Theme.aboutContentInset - Theme.aboutSectionGap }
            }
        }
        OperationFooter {
            id: footer
            objectName: "aboutFooter"
            Layout.fillWidth: true
            OperationFooterButton {
                objectName: "aboutCopy"
                text: root.copied ? qsTrId("about.copied") : qsTrId("about.copy")
                onClicked: { BuildInfo.copyVersionInfo(); root.copied = true; }
            }
            OperationFooterButton { objectName: "aboutDone"; text: qsTrId("about.done"); onClicked: root.close() }
        }
    }
}
