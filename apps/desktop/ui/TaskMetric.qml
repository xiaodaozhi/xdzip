import QtQuick
import QtQuick.Layouts

Item {
    id: root
    property real density: 1
    property real labelSize: Theme.secondarySize
    property real valueSize: Theme.panelTitleSize * .9
    required property string heading
    required property string value
    Layout.alignment: Qt.AlignTop
    implicitHeight: metric.y + metric.height
    Text {
        id: headingLabel
        width: parent.width
        text: root.heading
        font.pixelSize: root.labelSize
        lineHeight: Math.round(root.labelSize * 1.33)
        lineHeightMode: Text.FixedHeight
        height: Math.max(1, lineCount) * lineHeight
        color: Theme.textSecondary
        wrapMode: Text.Wrap
    }
    Text {
        id: metric
        width: parent.width
        y: headingLabel.height + 5 * root.density
        text: root.value
        textFormat: Text.PlainText
        font.pixelSize: root.valueSize
        lineHeight: Math.round(root.valueSize * 1.26)
        lineHeightMode: Text.FixedHeight
        height: Math.max(1, lineCount) * lineHeight
        font.weight: Font.Bold
        color: Theme.text
        wrapMode: Text.Wrap
    }
}
