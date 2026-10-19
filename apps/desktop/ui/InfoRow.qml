import QtQuick
import QtQuick.Layouts

RowLayout {
    id: root
    property url iconSource: Theme.dark ? "qrc:/xdzip/assets/info-dark.png" : "qrc:/xdzip/assets/info-light.png"
    property real lineFactor: 1.5
    property real labelSize: Theme.secondarySize
    property real iconSize: Theme.iconSize
    required property string text
    readonly property alias iconItem: icon
    readonly property alias labelItem: label
    spacing: Theme.gap

    Image {
        id: icon
        Layout.preferredWidth: root.iconSize
        Layout.preferredHeight: root.iconSize
        Layout.alignment: Qt.AlignVCenter
        source: root.iconSource
        sourceSize: Qt.size(root.iconSize * Screen.devicePixelRatio, root.iconSize * Screen.devicePixelRatio)
        Accessible.ignored: true
    }
    Text {
        id: label
        Layout.fillWidth: true
        text: root.text
        textFormat: Text.PlainText
        wrapMode: Text.Wrap
        color: Theme.textSecondary
        font.pixelSize: root.labelSize
        lineHeight: Math.round(root.labelSize * root.lineFactor)
        lineHeightMode: Text.FixedHeight
        Accessible.role: Accessible.StaticText
        Accessible.name: text
    }
}
