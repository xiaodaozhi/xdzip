import QtQuick

Item {
    id: root
    required property url source
    property real canvasWidth: 75
    property real canvasHeight: 72
    property real artworkWidth: 66
    property real artworkHeight: 74
    implicitWidth: canvasWidth
    implicitHeight: canvasHeight
    Image {
        anchors.centerIn: parent
        width: root.artworkWidth
        height: root.artworkHeight
        source: root.source
        fillMode: Image.PreserveAspectFit
        Accessible.ignored: true
    }
}
