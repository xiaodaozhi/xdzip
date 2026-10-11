import QtQuick
import QtQuick.Layouts

ColumnLayout {
    id: root
    required property string heading
    required property string value
    spacing: 6
    Text {
        Layout.fillWidth: true
        text: root.heading
        font.pixelSize: Theme.secondarySize
        color: Theme.textSecondary
        wrapMode: Text.Wrap
    }
    Text {
        Layout.fillWidth: true
        text: root.value
        textFormat: Text.PlainText
        font.pixelSize: Theme.panelTitleSize * 0.9
        font.weight: Font.DemiBold
        color: Theme.text
        wrapMode: Text.Wrap
    }
}
