import QtQuick
import QtQuick.Controls

MenuSeparator {
    property bool commandVisible: true
    visible: commandVisible
    objectName: "commandSeparator"
    width: parent ? parent.width : implicitWidth
    implicitWidth: Theme.menuWidth - Theme.menuPadding * 2
    implicitHeight: commandVisible ? Theme.menuPadding * 2 + 1 : 0
    padding: Theme.menuPadding
    contentItem: Rectangle {
        implicitHeight: 1
        color: Theme.menuBorder
    }
}
