import QtQuick
import QtQuick.Layouts

Rectangle {
    id: root
    readonly property alias valueItem: valueLabel
    required property string heading
    required property string value
    required property url iconSource
    implicitHeight: content.implicitHeight + 28
    color: Theme.taskPanel
    border.color: Theme.taskBorder
    radius: Theme.controlRadius

    ColumnLayout {
        id: content
        anchors.fill: parent
        anchors.margins: 14
        spacing: 10
        Text {
            text: root.heading
            font.pixelSize: Theme.bodySize
            font.weight: Font.DemiBold
            color: Theme.text
        }
        Rectangle {
            Layout.fillWidth: true
            implicitHeight: row.implicitHeight + 16
            radius: Theme.controlRadius - 2
            color: Theme.surface
            border.color: Theme.taskBorder
            RowLayout {
                id: row
                anchors.fill: parent
                anchors.margins: 8
                spacing: Theme.gap
                Image {
                    Layout.preferredWidth: 26
                    Layout.preferredHeight: 26
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
                    font.pixelSize: Theme.bodySize
                    color: Theme.text
                }
            }
        }
    }
}
