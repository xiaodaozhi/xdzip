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
    readonly property real density: Theme.operationExtractDensity
    function unit(value: real): real { return value * density; }
    width: Theme.operationConflictWidth
    requestedHeight: Math.min(Math.ceil(conflictContent.implicitHeight + footer.implicitHeight), Theme.operationConflictClientHeight, Screen.desktopAvailableHeight - 80)
    title: qsTr("文件已存在 — xdzip")
    ColumnLayout {
        anchors.fill: parent; spacing: root.unit(0)
        ScrollView {
            id: scroll
            Layout.fillWidth: true; Layout.fillHeight: true
            contentWidth: availableWidth; clip: true
            ColumnLayout {
                id: conflictContent
                width: scroll.availableWidth; spacing: root.unit(16)
                ColumnLayout {
                    Layout.fillWidth: true; Layout.margins: root.unit(30); spacing: root.unit(16)
                    RowLayout {
                        Layout.fillWidth: true; spacing: root.unit(20)
                        OperationHero { source: "qrc:/xdzip/assets/conflict-document.png"; canvasWidth: root.unit(60); canvasHeight: root.unit(68); artworkWidth: root.unit(56); artworkHeight: root.unit(65); Layout.preferredWidth: canvasWidth; Layout.preferredHeight: canvasHeight }
                        ColumnLayout {
                            Layout.fillWidth: true
                            Text { text: root.task.creating ? qsTr("目标位置已有同名归档") : qsTr("目标位置已有同名文件"); font.pixelSize: Math.round(root.unit(Theme.operationTitleSize)); font.bold: true; color: Theme.text }
                            Text { Layout.fillWidth: true; text: qsTr("请选择要保留的版本"); textFormat: Text.PlainText; wrapMode: Text.Wrap; color: Theme.textSecondary; font.pixelSize: 13 * Theme.fontScale }
                        }
                    }
                    RowLayout {
                        Layout.fillWidth: true; spacing: root.unit(12)
                        Image { Layout.preferredWidth: root.unit(36); Layout.preferredHeight: root.unit(36); source: AssetLibrary.fileIcon(root.task.promptName || "", false, false) }
                        Text { Layout.fillWidth: true; text: root.task.promptName || ""; textFormat: Text.PlainText; font.pixelSize: 14 * Theme.fontScale; color: Theme.text; wrapMode: Text.WrapAnywhere }
                    }
                    GridLayout {
                        Layout.fillWidth: true; columns: Theme.fontScale > 1.35 ? 1 : 2; columnSpacing: root.unit(16); rowSpacing: root.unit(12)
                        Repeater {
                            model: root.task.creating ? [qsTr("已有归档"), qsTr("新建归档")] : [qsTr("已有文件"), qsTr("归档中的文件")]
                            Rectangle {
                                id: comparisonCard
                                required property string modelData
                                required property int index
                                Layout.fillWidth: true; implicitHeight: Math.max(root.unit(190), comparisonContent.implicitHeight + root.unit(32))
                                color: Theme.taskPanel; border.color: Theme.taskBorder; radius: Theme.controlRadius
                                ColumnLayout {
                                    id: comparisonContent
                                    anchors.fill: parent; anchors.margins: root.unit(16); spacing: root.unit(12)
                                    Image { Layout.alignment: Qt.AlignHCenter; Layout.preferredWidth: root.unit(60); Layout.preferredHeight: root.unit(60); source: AssetLibrary.fileIcon(root.task.promptName || "", false, true) }
                                    Text { Layout.fillWidth: true; text: comparisonCard.modelData; font.pixelSize: Math.round(root.unit(Theme.operationBodySize)); font.bold: true; color: Theme.text; horizontalAlignment: Text.AlignHCenter }
                                    Text { Layout.fillWidth: true; text: root.sizeFormatter(comparisonCard.index === 0 ? root.task.existingBytes || "0" : root.task.incomingBytes || "0"); font.pixelSize: Math.round(root.unit(Theme.operationBodySize)); color: Theme.textSecondary; horizontalAlignment: Text.AlignHCenter }
                                    Text { Layout.fillWidth: true; text: qsTr("修改日期"); color: Theme.textSecondary; font.pixelSize: Theme.operationFooterNoteSize; horizontalAlignment: Text.AlignHCenter }
                                    Text { Layout.fillWidth: true; text: comparisonCard.index === 0 ? root.task.existingModified || "—" : root.task.incomingModified || "—"; font.pixelSize: Theme.operationFooterNoteSize; color: Theme.text; horizontalAlignment: Text.AlignHCenter }
                                }
                            }
                        }
                    }
                    InfoRow { objectName: "conflictNotice"; Layout.fillWidth: true; text: root.task.overwriteAllowed ? qsTr("覆盖会替换已有文件，原文件不会进入废纸篓。") : qsTr("这个文件是源归档，只能跳过或保留两个版本。"); labelSize: Theme.operationFooterNoteSize; iconSize: 20 }
                    OperationCheck { density: root.density; id: remaining; objectName: "conflictApplyAll"; Layout.fillWidth: true; visible: !root.task.creating; text: qsTr("对此任务后续的普通文件冲突使用相同操作") }
                }
            }
        }
        OperationFooter {
            id: footer
            Layout.fillWidth: true
            contentInset: root.unit(30); verticalInset: root.unit(17); minimumFooterHeight: root.unit(74)
            note: qsTr("等待你的决定"); noteGap: 4; noteIcon: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "shield.png"
            OperationFooterButton { objectName: "conflictSkip"; text: root.task.creating ? qsTr("取消创建") : qsTr("跳过"); onClicked: root.decision(1, remaining.checked) }
            OperationFooterButton { objectName: "conflictRename"; text: qsTr("自动改名"); primary: true; onClicked: root.decision(2, remaining.checked) }
            OperationFooterButton { objectName: "conflictOverwrite"; text: root.task.creating ? qsTr("替换归档") : qsTr("覆盖文件"); dangerous: true; enabled: Boolean(root.task.overwriteAllowed); onClicked: root.decision(3, remaining.checked) }
        }
    }
}
