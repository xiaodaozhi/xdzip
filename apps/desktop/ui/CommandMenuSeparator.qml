import QtQuick
import QtQuick.Controls

MenuSeparator {
    implicitHeight: Theme.menuPadding * 2 + 1
    padding: Theme.menuPadding
    contentItem: Rectangle {
        implicitHeight: 1
        color: Theme.menuBorder
    }
}
