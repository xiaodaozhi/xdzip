pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts

Item {
    // Both panes reserve the same font-scaled row, including its vertical inset.
    // The header remains outside scrolling content so long details cannot move it.
    implicitHeight: Math.ceil(32 + Math.max(36, Theme.bodySize * 1.6 + 12))
    default property alias content: row.data

    RowLayout {
        id: row
        anchors.fill: parent
        anchors.margins: 16
        spacing: 12
    }
}
