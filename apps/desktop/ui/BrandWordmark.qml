import QtQuick

Item {
    id: root
    required property real iconHeight
    implicitHeight: iconHeight * Theme.fontScale
    implicitWidth: implicitHeight * Theme.brandWordmarkAspect
    readonly property alias source: artwork.source
    readonly property alias status: artwork.status
    Accessible.role: Accessible.StaticText
    Accessible.name: "xdzip"

    Image {
        id: artwork
        anchors.fill: parent
        source: Theme.dark ? "qrc:/xdzip/assets/brand-wordmark-dark.svg"
                           : "qrc:/xdzip/assets/brand-wordmark-light.svg"
        sourceSize: Qt.size(Math.ceil(width * Screen.devicePixelRatio),
                            Math.ceil(height * Screen.devicePixelRatio))
        fillMode: Image.PreserveAspectFit
        Accessible.ignored: true
    }
}
