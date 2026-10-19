import QtQuick
import QtQuick.Layouts

Rectangle {
    id: root
    property real density: 1
    readonly property alias valueItem: valueLabel
    property real labelSize: Math.round(Theme.operationBodySize * density)
    property real valueSize: Math.round(Theme.operationPathFontSize * density)
    required property string heading
    required property string value
    required property url iconSource
    implicitHeight: content.implicitHeight + 22 * root.density
    color: Theme.taskPanel
    border.color: Theme.taskBorder
    radius: Theme.controlRadius * root.density

    ColumnLayout {
        id: content
        anchors.fill: parent
        anchors.leftMargin: 16 * root.density
        anchors.rightMargin: 16 * root.density
        anchors.topMargin: 11 * root.density
        anchors.bottomMargin: 11 * root.density
        spacing: 9 * root.density
        Text {
            // Qt's implicit text box includes font leading beyond the fixed
            // line height; the layout must use the authored line slot.
            Layout.preferredHeight: lineHeight
            text: root.heading
            font.pixelSize: root.labelSize
            lineHeight: Math.round(root.labelSize * 1.25)
            lineHeightMode: Text.FixedHeight
            font.weight: Font.DemiBold
            color: Theme.text
        }
        Rectangle {
            Layout.fillWidth: true
            implicitHeight: Math.max(Theme.operationPathRowHeight * root.density, row.implicitHeight + 14 * root.density)
            radius: (Theme.controlRadius - 2) * root.density
            color: Theme.surface
            border.color: Theme.taskBorder
            RowLayout {
                id: row
                anchors.fill: parent
                anchors.margins: 7 * root.density
                spacing: Theme.gap * root.density
                Image {
                    Layout.preferredWidth: 32 * root.density
                    Layout.preferredHeight: 32 * root.density
                    source: root.iconSource
                    fillMode: Image.PreserveAspectFit
                    Accessible.ignored: true
                }
                Text {
                    id: valueLabel
                    Layout.fillWidth: true
                    text: root.value
                    textFormat: Text.PlainText
                    wrapMode: Text.WrapAnywhere
                    font.pixelSize: root.valueSize
                    color: Theme.text
                }
            }
        }
    }
}
