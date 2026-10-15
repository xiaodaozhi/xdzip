pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

AbstractButton {
    id: root
    focusPolicy: Qt.StrongFocus
    property int recordCount: 0
    implicitHeight: Math.max(28, contentItem.implicitHeight + 8)
    checkable: true
    hoverEnabled: true
    padding: 4
    Accessible.name: qsTr("查看处理明细")
    background: Rectangle {
        radius: Theme.operationControlRadius
        color: root.hovered ? Theme.selection : "transparent"
        border.width: 0
        border.color: Theme.accent
    }
    contentItem: RowLayout {
        spacing: 8
        Image {
            source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + (root.checked ? "chevron-down" : "chevron-right") + ".png"
            Layout.preferredWidth: 18; Layout.preferredHeight: 18
        }
        Text { text: qsTr("查看处理明细"); font.pixelSize: Math.round(Theme.operationResultBodySize); font.bold: true; color: Theme.text }
        Item { Layout.fillWidth: true }
        Text { text: qsTr("%1 条记录").arg(root.recordCount); font.pixelSize: Math.round(Theme.operationResultNoteSize); color: Theme.textSecondary }
    }
}
