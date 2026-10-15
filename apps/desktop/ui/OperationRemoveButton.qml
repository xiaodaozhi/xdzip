import QtQuick
import QtQuick.Controls

Button {
    id: root
    property string entryName: ""
    implicitWidth: Theme.operationControlRemoveHitSize
    implicitHeight: implicitWidth
    padding: 0
    hoverEnabled: true
    focusPolicy: Qt.StrongFocus
    Accessible.name: qsTr("移除 %1").arg(entryName)
    background: Rectangle {
        objectName: root.objectName + "Background"
        anchors.centerIn: parent
        width: Theme.operationControlRemoveArtworkSize * 18 / 24
        height: width
        radius: width / 2
        color: root.down ? Theme.controlPressed : root.hovered ? Theme.controlHover : Theme.surface
        border.color: "transparent"
        border.width: 0
    }
    contentItem: Item {
        Image {
            objectName: root.objectName + "Icon"
            anchors.centerIn: parent
            width: Theme.operationControlRemoveArtworkSize
            height: width
            source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "remove.png"
            opacity: root.enabled ? 1 : .45
        }
    }
}
