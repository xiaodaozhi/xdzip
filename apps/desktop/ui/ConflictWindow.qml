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
    readonly property bool archiveCreation: task.creating === true && task.kind !== "move" && task.kind !== "update"
    readonly property real density: Theme.operationExtractDensity
    function unit(value: real): real { return value * density; }
    width: Theme.operationConflictWidth
    requestedHeight: Math.min(Math.ceil(conflictContent.implicitHeight + footer.implicitHeight), Theme.operationConflictClientHeight, Screen.desktopAvailableHeight - 80)
    title: qsTrId("ui.bbd2a22f3a54")
    ColumnLayout {
        anchors.fill: parent; spacing: root.unit(0)
        ScrollView {
            id: scroll
            Layout.fillWidth: true; Layout.fillHeight: true
            contentWidth: availableWidth; clip: true
            ColumnLayout {
                id: conflictContent
                width: scroll.availableWidth; spacing: Theme.operationHeaderBodyGap
                OperationHeader {
                    Layout.fillWidth: true
                    source: "qrc:/xdzip/assets/conflict-document.png"
                    title: root.archiveCreation ? qsTrId("ui.a3ea7a6ceaff") : qsTrId("ui.5673940d4393")
                    subtitle: qsTrId("ui.25dad65a6ae3")
                }
                ColumnLayout {
                    objectName: "operationHeaderBody"
                    Layout.fillWidth: true; Layout.margins: root.unit(30); Layout.topMargin: 0; spacing: root.unit(16)

                    RowLayout {
                        Layout.fillWidth: true; spacing: root.unit(12)
                        Image { Layout.preferredWidth: root.unit(36); Layout.preferredHeight: root.unit(36); source: AssetLibrary.fileIcon(root.task.promptName || "", false, false) }
                        Text { Layout.fillWidth: true; text: root.task.promptName || ""; textFormat: Text.PlainText; font.pixelSize: 14 * Theme.fontScale; color: Theme.text; wrapMode: Text.WrapAnywhere }
                    }
                    GridLayout {
                        Layout.fillWidth: true; columns: Theme.fontScale > 1.35 ? 1 : 2; columnSpacing: root.unit(16); rowSpacing: root.unit(12)
                        Repeater {
                            model: root.archiveCreation ? [qsTrId("ui.6760b8a159bb"), qsTrId("ui.d1ea7b4cf107")] : [qsTrId("ui.b20339d20b27"), qsTrId("ui.ad893b1fee61")]
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
                                    Text { Layout.fillWidth: true; text: qsTrId("ui.2cbced881b2d"); color: Theme.textSecondary; font.pixelSize: Theme.operationFooterNoteSize; horizontalAlignment: Text.AlignHCenter }
                                    Text { Layout.fillWidth: true; text: comparisonCard.index === 0 ? root.task.existingModified || "—" : root.task.incomingModified || "—"; font.pixelSize: Theme.operationFooterNoteSize; color: Theme.text; horizontalAlignment: Text.AlignHCenter }
                                }
                            }
                        }
                    }
                    InfoRow { objectName: "conflictNotice"; Layout.fillWidth: true; text: root.task.overwriteAllowed ? qsTrId("ui.6878e39a8fb2") : qsTrId("ui.baaef55f9a78"); labelSize: Theme.operationFooterNoteSize; iconSize: 20 }
                    OperationCheck { density: root.density; id: remaining; objectName: "conflictApplyAll"; Layout.fillWidth: true; visible: !root.archiveCreation; text: qsTrId("ui.5a8648b2a130") }
                }
            }
        }
        OperationFooter {
            id: footer
            Layout.fillWidth: true
            contentInset: root.unit(30); verticalInset: root.unit(17); minimumFooterHeight: root.unit(74)
            note: qsTrId("ui.df37976cd784"); noteGap: 4; noteIcon: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "shield.png"
            OperationFooterButton { objectName: "conflictSkip"; text: root.archiveCreation ? qsTrId("ui.f6e498e6a16d") : qsTrId("ui.fc50a99caae0"); onClicked: root.decision(1, remaining.checked) }
            OperationFooterButton { objectName: "conflictRename"; text: qsTrId("ui.7d590c4f89be"); primary: true; onClicked: root.decision(2, remaining.checked) }
            OperationFooterButton { objectName: "conflictOverwrite"; text: root.archiveCreation ? qsTrId("ui.992f443cf975") : qsTrId("ui.e4b715a60c85"); dangerous: true; enabled: Boolean(root.task.overwriteAllowed); onClicked: root.decision(3, remaining.checked) }
        }
    }
}
