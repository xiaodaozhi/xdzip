pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// Decision windows share geometry, typography, scrolling and footer placement.
OperationWindow {
    id: root
    property string heading: ""
    property string message: ""
    property real availableClientHeight: Screen.desktopAvailableHeight - Theme.confirmationScreenMargin
    default property alias actions: footer.actions
    readonly property alias headingItem: headingLabel
    readonly property alias messageItem: messageLabel
    readonly property alias bodyItem: scroll
    readonly property alias footerItem: footer
    modality: Qt.ApplicationModal
    width: Theme.confirmationLogicalWidth
    requestedHeight: Math.min(Math.ceil(content.implicitHeight + footer.implicitHeight),
                              Theme.confirmationMaximumHeight, availableClientHeight)
    onActiveFocusItemChanged: if (visible) Qt.callLater(() => root.revealFocus(scroll))
    ColumnLayout {
        // Native resize notifications can lag behind the reconciled client height.
        width: root.width
        height: root.height
        spacing: 0
        ScrollView {
            id: scroll
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.minimumHeight: 0
            clip: true
            contentWidth: availableWidth
            ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
            onHeightChanged: if (root.visible) Qt.callLater(() => root.revealFocus(scroll))
            ColumnLayout {
                id: content
                width: scroll.availableWidth
                spacing: 0
                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.margins: Theme.confirmationContentInset
                    spacing: Theme.confirmationSectionGap
                    Text {
                        id: headingLabel
                        Layout.fillWidth: true
                        text: root.heading
                        textFormat: Text.PlainText
                        font.pixelSize: Theme.confirmationTitleSize
                        font.bold: true
                        color: Theme.text
                        wrapMode: Text.Wrap
                    }
                    Text {
                        id: messageLabel
                        visible: text.length > 0
                        Layout.fillWidth: true
                        text: root.message
                        textFormat: Text.PlainText
                        font.pixelSize: Theme.confirmationBodySize
                        color: Theme.textSecondary
                        wrapMode: Text.Wrap
                    }
                }
            }
        }
        OperationFooter {
            id: footer
            objectName: root.objectName + "Footer"
            Layout.fillWidth: true
        }
    }
}
