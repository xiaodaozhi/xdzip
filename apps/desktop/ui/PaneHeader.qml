pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts

Item {
    id: root
    // Both panes reserve the same font-scaled row, including its vertical inset.
    // The header remains outside scrolling content so long details cannot move it.
    implicitHeight: Math.round(Math.max(Theme.mainPaneHeaderHeight, Theme.mainPaneTitleSize * 1.6 + 24))
    property real leftInset: Theme.mainPaneInset
    property real rightInset: Theme.mainPaneInset
    default property alias content: row.data

    RowLayout {
        id: row
        anchors.fill: parent
        anchors.leftMargin: root.leftInset
        anchors.rightMargin: root.rightInset
        anchors.topMargin: 0
        anchors.bottomMargin: 0
        spacing: 10
    }
}
