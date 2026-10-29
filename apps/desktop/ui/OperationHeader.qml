pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: root
    objectName: "operationHeader"
    required property url source
    required property string title
    property string subtitle: ""
    property bool subtitleUserData: false
    property string titleObjectName: "operationHeaderTitle"
    property string iconObjectName: "operationHeaderIcon"
    readonly property alias titleItem: heading
    readonly property alias subtitleItem: description
    readonly property alias iconItem: artwork
    implicitHeight: Math.ceil(Math.max(Theme.operationHeaderNormalHeight,
        Theme.operationHeaderIconSize + Theme.operationHeaderVerticalInset * 2,
        labels.implicitHeight + Theme.operationHeaderVerticalInset * 2))
    Image {
        id: artwork
        objectName: root.iconObjectName
        x: Theme.operationHeaderHorizontalInset
        anchors.verticalCenter: parent.verticalCenter
        width: Theme.operationHeaderIconSize
        height: Theme.operationHeaderIconSize
        source: root.source
        fillMode: Image.PreserveAspectFit
        Accessible.ignored: true
    }
    Column {
        id: labels
        x: Theme.operationHeaderHorizontalInset + Theme.operationHeaderIconSize + Theme.operationHeaderGap
        anchors.verticalCenter: parent.verticalCenter
        width: Math.max(0, root.width - x - Theme.operationHeaderHorizontalInset)
        spacing: Theme.operationHeaderTextGap
        Text {
            id: heading
            objectName: root.titleObjectName
            width: parent.width
            text: root.title
            textFormat: Text.PlainText
            font.pixelSize: Math.round(Theme.operationHeaderTitleSize)
            font.bold: true
            lineHeight: Math.round(font.pixelSize * Theme.operationHeaderTitleLineFactor)
            lineHeightMode: Text.FixedHeight
            wrapMode: Text.Wrap
            color: Theme.text
        }
        Text {
            id: description
            objectName: "operationHeaderSubtitle"
            width: parent.width
            visible: text.length > 0
            text: root.subtitle
            textFormat: Text.PlainText
            font.pixelSize: Math.round(Theme.operationHeaderSubtitleSize)
            font.family: root.subtitleUserData ? Theme.userDataFontFamily : Theme.uiFontFamily
            lineHeight: Math.round(font.pixelSize * Theme.operationHeaderSubtitleLineFactor)
            lineHeightMode: Text.FixedHeight
            wrapMode: Text.WrapAnywhere
            color: Theme.textSecondary
        }
    }
}
