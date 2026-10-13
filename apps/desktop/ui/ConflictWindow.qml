pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

OperationWindow {
    id: root
    objectName: "conflictWindow"
    required property var task
    required property var sizeFormatter
    signal decision(int policy, bool all)
    width: Theme.operationDefaultWidth
    height: Math.min(583, Screen.desktopAvailableHeight - 80)
    title: qsTr("文件已存在 — xdzip")
    ColumnLayout {
        anchors.fill: parent; spacing: 0
        ScrollView {
            id: scroll
            Layout.fillWidth: true; Layout.fillHeight: true
            contentWidth: availableWidth; clip: true
            ColumnLayout {
                width: scroll.availableWidth; spacing: 16
                ColumnLayout {
                    Layout.fillWidth: true; Layout.margins: 30; spacing: 16
                    RowLayout {
                        Layout.fillWidth: true; spacing: 20
                        OperationHero { source: "qrc:/xdzip/assets/conflict-document.png"; canvasWidth: 60; canvasHeight: 68; artworkWidth: 56; artworkHeight: 65; Layout.preferredWidth: canvasWidth; Layout.preferredHeight: canvasHeight }
                        ColumnLayout {
                            Layout.fillWidth: true
                            Text { text: qsTr("目标位置已有同名文件"); font.pixelSize: Theme.operationTitleSize; font.bold: true; color: Theme.text }
                            Text { Layout.fillWidth: true; text: qsTr("请选择要保留的版本"); textFormat: Text.PlainText; wrapMode: Text.Wrap; color: Theme.textSecondary; font.pixelSize: 15 * Theme.fontScale }
                        }
                    }
                    RowLayout {
                        Layout.fillWidth: true; spacing: 12
                        Image { Layout.preferredWidth: 36; Layout.preferredHeight: 36; source: AssetLibrary.fileIcon(root.task.promptName || "", false, false) }
                        Text { Layout.fillWidth: true; text: root.task.promptName || ""; textFormat: Text.PlainText; font.pixelSize: 18 * Theme.fontScale; color: Theme.text; wrapMode: Text.WrapAnywhere }
                    }
                    GridLayout {
                        Layout.fillWidth: true; columns: Theme.fontScale > 1.35 ? 1 : 2; columnSpacing: 16; rowSpacing: 12
                        Repeater {
                            model: [qsTr("已有文件"), qsTr("归档中的文件")]
                            Rectangle {
                                id: comparisonCard
                                required property string modelData
                                required property int index
                                Layout.fillWidth: true; implicitHeight: Math.max(190, comparisonContent.implicitHeight + 32)
                                color: Theme.taskPanel; border.color: Theme.taskBorder; radius: Theme.controlRadius
                                ColumnLayout {
                                    id: comparisonContent
                                    anchors.fill: parent; anchors.margins: 16; spacing: 12
                                    Image { Layout.alignment: Qt.AlignHCenter; Layout.preferredWidth: 60; Layout.preferredHeight: 60; source: AssetLibrary.fileIcon(root.task.promptName || "", false, true) }
                                    Text { Layout.fillWidth: true; text: comparisonCard.modelData; font.pixelSize: Theme.operationBodySize; font.bold: true; color: Theme.text; horizontalAlignment: Text.AlignHCenter }
                                    Text { Layout.fillWidth: true; text: root.sizeFormatter(comparisonCard.index === 0 ? root.task.existingBytes || "0" : root.task.incomingBytes || "0"); font.pixelSize: Theme.operationBodySize; color: Theme.textSecondary; horizontalAlignment: Text.AlignHCenter }
                                    Text { Layout.fillWidth: true; text: qsTr("修改日期"); color: Theme.textSecondary; font.pixelSize: Theme.operationNoteSize; horizontalAlignment: Text.AlignHCenter }
                                    Text { Layout.fillWidth: true; text: comparisonCard.index === 0 ? root.task.existingModified || "—" : root.task.incomingModified || "—"; font.pixelSize: Theme.operationNoteSize; color: Theme.text; horizontalAlignment: Text.AlignHCenter }
                                }
                            }
                        }
                    }
                    InfoRow { Layout.fillWidth: true; text: root.task.overwriteAllowed ? qsTr("覆盖会替换已有文件，原文件不会进入废纸篓。") : qsTr("这个文件是源归档，只能跳过或保留两个版本。"); labelSize: Theme.operationNoteSize; iconSize: 20 }
                    OperationCheck { id: remaining; objectName: "conflictApplyAll"; Layout.fillWidth: true; text: qsTr("对此任务后续的普通文件冲突使用相同操作") }
                }
            }
        }
        OperationFooter {
            Layout.fillWidth: true
            contentInset: 30; verticalInset: 17; minimumFooterHeight: 74
            note: qsTr("等待你的决定"); noteGap: 4; noteIcon: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "shield.png"
            AppButton { objectName: "conflictSkip"; text: qsTr("跳过"); labelSize: Theme.operationFooterFontSize; baseHeight: 38; cornerRadius: 6; horizontalInset: 17; onClicked: root.decision(1, remaining.checked) }
            AppButton { objectName: "conflictRename"; text: qsTr("自动改名"); labelSize: Theme.operationFooterFontSize; baseHeight: 38; cornerRadius: 6; horizontalInset: 17; primary: true; onClicked: root.decision(2, remaining.checked) }
            AppButton { objectName: "conflictOverwrite"; text: qsTr("覆盖文件"); labelSize: Theme.operationFooterFontSize; baseHeight: 38; cornerRadius: 6; horizontalInset: 17; dangerous: true; enabled: Boolean(root.task.overwriteAllowed); onClicked: root.decision(3, remaining.checked) }
        }
    }
}
