pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

OperationWindow {
    id: root
    objectName: "extractWindow"
    required property var controller
    property var intent: controller.extractionIntent
    property var targetState: controller.targetCheck
    property bool allFiles: true
    width: Theme.operationExtractWidth
    requestedHeight: Math.min(Theme.operationExtractClientHeight, Screen.desktopAvailableHeight - 80)
    title: qsTr("解压文件 — xdzip")
    function configure() {
        allFiles = Boolean(intent.all);
        target.text = intent.target || "";
        sameFolder.checked = true;
        conflict.currentIndex = 0;
        highRatio.checked = false;
        openAfter.checked = true;
        present();
        controller.checkTarget(target.text);
    }
    readonly property bool targetReady: Boolean(targetState.valid) && controller.targetMatches(target.text)
    ColumnLayout {
        anchors.fill: parent; spacing: 0
        ScrollView {
            id: body
            Layout.fillWidth: true; Layout.fillHeight: true
            contentWidth: availableWidth; clip: true
            ColumnLayout {
                width: body.availableWidth; spacing: 0
                RowLayout {
                    Layout.fillWidth: true
                    Layout.leftMargin: Theme.operationGutter; Layout.rightMargin: Theme.operationGutter
                    Layout.topMargin: 21; Layout.bottomMargin: 16; spacing: 20
                    OperationHero {
                        source: "qrc:/xdzip/assets/document-download.png"
                        canvasWidth: 70; canvasHeight: 81; artworkWidth: 66; artworkHeight: 79
                        Layout.preferredWidth: canvasWidth; Layout.preferredHeight: canvasHeight
                    }
                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 7
                        Text { text: qsTr("解压文件"); font.pixelSize: Theme.operationTitleSize; lineHeight: Math.round(Theme.operationTitleSize * 1.35); lineHeightMode: Text.FixedHeight; font.bold: true; color: Theme.text }
                        Text {
                            Layout.fillWidth: true
                            text: root.intent.archive || ""; textFormat: Text.PlainText
                            font.pixelSize: Theme.operationBodySize; color: Theme.textSecondary; elide: Text.ElideMiddle
                        }
                    }
                }
                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.leftMargin: Theme.operationExtractContentInset; Layout.rightMargin: Theme.operationExtractContentInset
                    Layout.bottomMargin: 20; spacing: Theme.operationCardGap
                    Rectangle {
                        Layout.fillWidth: true; implicitHeight: targetContent.implicitHeight + 32
                        color: Theme.taskPanel; radius: Theme.operationFieldRadius; border.color: Theme.taskBorder
                        ColumnLayout {
                            id: targetContent
                            anchors.fill: parent; anchors.margins: Theme.operationCardInset; spacing: 10
                            Text { text: qsTr("解压到"); font.pixelSize: Theme.operationBodySize; font.bold: true; color: Theme.text }
                            RowLayout {
                                Layout.fillWidth: true; spacing: 10
                                OperationField { id: target; objectName: "extractTarget"; Layout.fillWidth: true; Accessible.name: qsTr("目标目录") }
                                AppButton {
                                    text: qsTr("浏览…"); labelSize: Theme.operationBodySize; baseHeight: 40; cornerRadius: 6; horizontalInset: 17
                                    enabled: !root.controller.busy
                                    onClicked: { const path = root.controller.chooseTarget(target.text); if (path) { target.text = path; root.controller.checkTarget(path); } }
                                }
                            }
                            OperationCheck { id: sameFolder; objectName: "extractSameFolder"; Layout.fillWidth: true; text: qsTr("创建同名文件夹"); checked: true }
                            RowLayout {
                                Layout.fillWidth: true
                                Text {
                                    Layout.fillWidth: true
                                    text: root.targetState.checking ? qsTr("正在检查目标目录…") : root.targetReady ? qsTr("目标目录可写") : root.targetState.error || qsTr("修改目录后请重新检查")
                                    textFormat: Text.PlainText; wrapMode: Text.Wrap
                                    color: root.targetReady ? Theme.accentInk : Theme.textSecondary; font.pixelSize: Theme.operationNoteSize
                                }
                                AppButton { objectName: "extractCheckTarget"; text: qsTr("检查目标"); labelSize: Theme.operationNoteSize; enabled: !root.controller.busy; onClicked: root.controller.checkTarget(target.text) }
                            }
                            Text {
                                visible: Number(root.targetState.leftovers || 0) > 0
                                Layout.fillWidth: true
                                text: qsTr("发现 %1 个可能的暂存目录，将保留供检查。").arg(root.targetState.leftovers)
                                wrapMode: Text.Wrap; font.pixelSize: Theme.operationNoteSize; color: Theme.textSecondary
                            }
                        }
                    }
                    Rectangle {
                        Layout.fillWidth: true; implicitHeight: scopeContent.implicitHeight + 32
                        color: Theme.taskPanel; radius: Theme.operationFieldRadius; border.color: Theme.taskBorder
                        ColumnLayout {
                            id: scopeContent
                            anchors.fill: parent; anchors.margins: Theme.operationCardInset; spacing: 10
                            Text { text: qsTr("解压范围"); font.pixelSize: Theme.operationBodySize; font.bold: true; color: Theme.text }
                            OperationCheck {
                                objectName: "extractSelectedScope"
                                visible: Number(root.intent.selectedCount || 0) > 0
                                Layout.fillWidth: true; radioStyle: true
                                text: qsTr("所选文件和目录 · %1 项").arg(root.intent.selectedCount || 0)
                                checked: !root.allFiles
                                onClicked: root.allFiles = false
                            }
                            Rectangle {
                                visible: !root.allFiles
                                Layout.fillWidth: true; Layout.leftMargin: 28
                                implicitHeight: sampleContent.implicitHeight
                                color: Theme.surface; radius: 6; border.color: Theme.taskBorder
                                ColumnLayout {
                                    id: sampleContent
                                    anchors.fill: parent; spacing: 0
                                    Repeater {
                                        model: root.intent.samples || []
                                        RowLayout {
                                            id: sampleRow
                                            required property var modelData
                                            Layout.fillWidth: true; Layout.minimumHeight: 45
                                            Layout.leftMargin: 10; Layout.rightMargin: 10; spacing: 10
                                            Image { source: AssetLibrary.fileIcon(String(sampleRow.modelData.name), Boolean(sampleRow.modelData.directory), false); Layout.preferredWidth: 30; Layout.preferredHeight: 30 }
                                            Text { Layout.fillWidth: true; text: sampleRow.modelData.name; textFormat: Text.PlainText; elide: Text.ElideMiddle; color: Theme.text; font.pixelSize: Theme.operationBodySize }
                                        }
                                    }
                                }
                            }
                            OperationCheck {
                                objectName: "extractAllScope"
                                Layout.fillWidth: true; radioStyle: true
                                text: qsTr("全部归档 · %1 条记录").arg(root.intent.archiveCount || 0)
                                checked: root.allFiles
                                onClicked: root.allFiles = true
                            }
                        }
                    }
                    Rectangle {
                        Layout.fillWidth: true; implicitHeight: policyContent.implicitHeight + 32
                        color: Theme.taskPanel; radius: Theme.operationFieldRadius; border.color: Theme.taskBorder
                        ColumnLayout {
                            id: policyContent
                            anchors.fill: parent; anchors.margins: Theme.operationCardInset; spacing: 10
                            Text { text: qsTr("遇到重名文件时"); font.pixelSize: Theme.operationBodySize; font.bold: true; color: Theme.text }
                            OperationSelect {
                                id: conflict; objectName: "extractConflictPolicy"
                                Layout.fillWidth: true
                                model: [qsTr("询问我如何处理"), qsTr("跳过已有文件"), qsTr("自动改名，保留两个版本"), qsTr("覆盖已有文件")]
                            }
                            Text { Layout.fillWidth: true; text: qsTr("只处理普通文件；不会覆盖源归档或跟随链接。"); wrapMode: Text.Wrap; font.pixelSize: Theme.operationNoteSize; color: Theme.textSecondary }
                        }
                    }
                    OperationCheck { id: openAfter; objectName: "extractOpenAfter"; Layout.fillWidth: true; text: qsTr("完成后打开目标文件夹") }
                    OperationCheck { id: highRatio; objectName: "extractHighRatio"; Layout.fillWidth: true; text: qsTr("允许高压缩比文件（仍受字节限额约束）") }
                    Text {
                        Layout.fillWidth: true
                        visible: Boolean(root.intent.error)
                        text: root.intent.error || ""; textFormat: Text.PlainText
                        font.pixelSize: Theme.operationNoteSize; color: Theme.menuDanger; wrapMode: Text.Wrap
                    }
                }
            }
        }
        OperationFooter {
            Layout.fillWidth: true
            contentInset: 24; verticalInset: 13; minimumFooterHeight: 67
            noteGap: 4; noteIcon: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "shield.png"
            note: root.targetReady ? qsTr("准备就绪") : qsTr("请检查目标目录")
            AppButton { text: qsTr("取消"); labelSize: Theme.operationFooterFontSize; baseHeight: 40; cornerRadius: 6; horizontalInset: 17; minimumWidth: 96; onClicked: root.close() }
            AppButton {
                objectName: "startExtract"
                text: qsTr("开始解压"); labelSize: Theme.operationFooterFontSize; baseHeight: 40; cornerRadius: 6; horizontalInset: 17; minimumWidth: 96; primary: true
                enabled: root.targetReady && !root.controller.busy && root.intent.session === root.controller.sessionId
                onClicked: root.controller.startExtraction(target.text, root.allFiles, sameFolder.checked, conflict.currentIndex, highRatio.checked, openAfter.checked)
            }
        }
    }
}
