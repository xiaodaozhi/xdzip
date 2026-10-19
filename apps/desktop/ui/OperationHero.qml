import QtQuick

Item {
    id: root
    property real density: 1
    required property url source
    property real canvasWidth: 75 * density
    property real canvasHeight: 72 * density
    property real artworkWidth: 66 * density
    property real artworkHeight: 74 * density
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
