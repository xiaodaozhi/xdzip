import QtQuick

Item {
    // Shared nine-slice pixels are ready with the menu's first frame.
    // Stretch only the straight edges, preserving the authored corner blur.
    BorderImage {
        id: shadow
        objectName: "menuShadowLayer-0"
        readonly property real inset: Theme.menuShadowRadius * 2
        readonly property int fontPercent: Math.round(Theme.fontScale * 100)
        x: -inset; y: -inset
        width: parent.width + inset * 2
        height: parent.height + inset * 2
        source: "qrc:/xdzip/assets/menu/" + (Theme.dark ? "dark" : "light") + "-" + fontPercent + ".png"
        border.left: inset + Math.round(Theme.menuRadius)
        border.top: border.left
        border.right: border.left
        border.bottom: border.left
        horizontalTileMode: BorderImage.Stretch
        verticalTileMode: BorderImage.Stretch
        asynchronous: false
        cache: true
    }
    Rectangle {
        anchors.fill: parent
        radius: Theme.menuRadius
        color: Theme.menuSurface
        border.color: Theme.menuBorder
    }
}
