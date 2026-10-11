import QtQuick
import QtQuick.Layouts

RowLayout {
    id: root
    required property string text
    readonly property alias iconItem: icon
    readonly property alias labelItem: label
    spacing: Theme.gap

    Image {
        id: icon
        Layout.preferredWidth: Theme.iconSize
        Layout.preferredHeight: Theme.iconSize
        Layout.alignment: Qt.AlignVCenter
        source: Theme.dark ? "qrc:/xdzip/assets/info-dark.png" : "qrc:/xdzip/assets/info-light.png"
        sourceSize: Qt.size(Theme.iconSize * Screen.devicePixelRatio, Theme.iconSize * Screen.devicePixelRatio)
        Accessible.ignored: true
    }
    Text {
        id: label
        Layout.fillWidth: true
        text: root.text
        textFormat: Text.PlainText
        wrapMode: Text.Wrap
        color: Theme.textSecondary
        font.pixelSize: Theme.secondarySize
        Accessible.role: Accessible.StaticText
        Accessible.name: text
    }
}
