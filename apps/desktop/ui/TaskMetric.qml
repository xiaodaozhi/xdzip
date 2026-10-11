import QtQuick
import QtQuick.Layouts

Item {
    id: root
    required property string heading
    required property string value
    Layout.alignment: Qt.AlignTop
    implicitHeight: metric.y + metric.implicitHeight
    Text {
        width: parent.width
        text: root.heading
        font.pixelSize: Theme.secondarySize
        color: Theme.textSecondary
        wrapMode: Text.Wrap
    }
    Text {
        id: metric
        width: parent.width
        // Use the same first baseline for Latin and CJK fallback fonts.
        y: Math.ceil(Theme.secondarySize * 1.6) + 8 + Math.ceil(font.pixelSize * 1.2) - baselineOffset
        text: root.value
        textFormat: Text.PlainText
        font.pixelSize: Theme.panelTitleSize * 0.9
        font.weight: Font.DemiBold
        color: Theme.text
        wrapMode: Text.Wrap
    }
}
