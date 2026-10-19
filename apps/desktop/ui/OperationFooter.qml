pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts

Rectangle {
    id: root
    default property alias actions: buttons.data
    property real contentInset: Theme.operationProgressContentInset
    property real verticalInset: 12
    property real minimumFooterHeight: Theme.operationFooterHeight
    property url noteIcon: Theme.dark ? "qrc:/xdzip/assets/info-dark.png" : "qrc:/xdzip/assets/info-light.png"
    property real noteGap: 12
    property string note: ""
    readonly property alias hint: hint
    implicitHeight: Math.max(root.minimumFooterHeight, footer.implicitHeight + root.verticalInset * 2)
    color: Theme.surface
    Rectangle { width: parent.width; height: 1; color: Theme.taskBorder }
    GridLayout {
        id: footer
        anchors.fill: parent
        anchors.leftMargin: root.contentInset
        anchors.rightMargin: root.contentInset
        anchors.topMargin: root.verticalInset; anchors.bottomMargin: root.verticalInset
        columns: !hint.visible || width >= hint.implicitWidth + buttons.implicitWidth + columnSpacing ? 2 : 1
        columnSpacing: 12; rowSpacing: 10
        InfoRow {
            id: hint
            visible: root.note.length > 0
            Layout.fillWidth: true
            text: root.note
            iconSource: root.noteIcon
            spacing: root.noteGap
            labelSize: Theme.operationFooterNoteSize
            iconSize: Theme.operationFooterIconSize
        }
        RowLayout {
            id: buttons
            objectName: "operationFooterActions"
            spacing: 10
            Layout.alignment: Qt.AlignRight
            Layout.fillWidth: false
        }
    }
}
