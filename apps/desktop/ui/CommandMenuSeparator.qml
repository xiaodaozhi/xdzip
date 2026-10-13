import QtQuick
import QtQuick.Controls

MenuSeparator {
    objectName: "commandSeparator"
    width: parent ? parent.width : implicitWidth
    implicitWidth: Theme.menuWidth - Theme.menuPadding * 2
    implicitHeight: Theme.menuPadding * 2 + 1
    padding: Theme.menuPadding
    contentItem: Rectangle {
        implicitHeight: 1
        color: Theme.menuBorder
    }
}
