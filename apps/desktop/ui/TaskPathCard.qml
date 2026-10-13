import QtQuick
import QtQuick.Layouts

Rectangle {
    id: root
    readonly property alias valueItem: valueLabel
    property real labelSize: Theme.operationBodySize
    property real valueSize: Theme.operationPathFontSize
    required property string heading
    required property string value
    required property url iconSource
    implicitHeight: content.implicitHeight + 22
    color: Theme.taskPanel
    border.color: Theme.taskBorder
    radius: Theme.controlRadius

    ColumnLayout {
        id: content
        anchors.fill: parent
        anchors.leftMargin: 16
        anchors.rightMargin: 16
        anchors.topMargin: 11
        anchors.bottomMargin: 11
        spacing: 9
        Text {
            text: root.heading
            font.pixelSize: root.labelSize
            lineHeight: Math.round(root.labelSize * 1.25)
            lineHeightMode: Text.FixedHeight
            font.weight: Font.DemiBold
            color: Theme.text
        }
        Rectangle {
            Layout.fillWidth: true
            implicitHeight: Math.max(Theme.operationPathRowHeight, row.implicitHeight + 14)
            radius: Theme.controlRadius - 2
            color: Theme.surface
            border.color: Theme.taskBorder
            RowLayout {
                id: row
                anchors.fill: parent
                anchors.margins: 7
                spacing: Theme.gap
                Image {
                    Layout.preferredWidth: 32
                    Layout.preferredHeight: 32
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
