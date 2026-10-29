pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts

// Common result-detail row. Status is a backend outcome, never inferred from text.
Rectangle {
    id: root
    required property var entry
    property bool showReason: false
    readonly property bool incomplete: entry.status === "unfinished"
    readonly property string reason: ({disk_full: qsTrId("ui.1eeed25bf9dd"), source_changed: qsTrId("ui.89c590068e9d"), integrity_error: qsTrId("ui.a096c1176981"), cancelled: qsTrId("ui.a37778f17c5f"), output_io: qsTrId("ui.55039130fcb8")})[entry.errorCode] || qsTrId("ui.6707de42c29d")
    implicitHeight: Math.max(Theme.operationResultRowHeight, labels.implicitHeight + 16, statusLabel.implicitHeight + 20)
    color: incomplete && !showReason ? (Theme.resultWarningSurface) : Theme.surface
    RowLayout {
        anchors.fill: parent
        anchors.margins: 10
        spacing: 12
        Image {
            source: AssetLibrary.fileIcon(root.entry.name, false, false)
            Layout.preferredWidth: 28; Layout.preferredHeight: 32
            fillMode: Image.PreserveAspectFit
        }
        ColumnLayout {
            id: labels
            Layout.fillWidth: true; spacing: 2
            Text {
                Layout.fillWidth: true; text: root.entry.name
                font.family: Theme.userDataFontFamily
                textFormat: Text.PlainText; elide: Text.ElideMiddle
                font.pixelSize: Math.round(Theme.operationResultBodySize)
                lineHeight: Math.round(18 * Theme.fontScale); lineHeightMode: Text.FixedHeight
                font.bold: true; color: Theme.text
            }
            Text {
                text: Number(root.entry.size) >= 1000000 ? (Number(root.entry.size) / 1000000).toFixed(1) + " MB" : Number(root.entry.size) >= 1000 ? (Number(root.entry.size) / 1000).toFixed(1) + " KB" : root.entry.size + " B"
                font.pixelSize: Math.round(Theme.operationResultNoteSize); color: Theme.textSecondary
                lineHeight: Math.round(16 * Theme.fontScale); lineHeightMode: Text.FixedHeight
            }
        }
        Image {
            source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + (root.incomplete ? "status-warning" : root.entry.status === "skipped" ? "status-skipped" : "status-partial-success") + ".png"
            Layout.preferredWidth: 20; Layout.preferredHeight: 20
        }
        Text {
            id: statusLabel
            text: root.showReason ? root.reason : root.incomplete ? qsTrId("ui.6707de42c29d") : root.entry.status === "skipped" ? qsTrId("ui.71e95b289aad") : qsTrId("ui.f28461bb49c8")
            textFormat: Text.PlainText; wrapMode: Text.Wrap
            Layout.maximumWidth: Math.max(88, root.width * 0.3)
            font.pixelSize: Math.round(Theme.operationResultBodySize)
            color: root.incomplete ? (Theme.resultWarningInk) : root.entry.status === "skipped" ? Theme.textSecondary : Theme.accentInk
        }
    }
    Rectangle { anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: parent.bottom; height: 1; color: Theme.operationCardBorder }
}
