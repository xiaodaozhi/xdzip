pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

FocusScope {
    id: root
    property string namePrefix: "welcomeOpening"
    property bool busy: false
    property bool cancelling: false
    property bool cancellationPending: false
    property string archiveName: ""
    property string titleLabel: qsTrId("ui.821ffb8cd6a2")
    property string noteLabel: qsTrId("welcome.opening.note")
    property string progressLabel: qsTrId("welcome.opening.progress")
    property string cancelLabel: qsTrId("welcome.opening.cancel")
    property color backdropColor: Qt.rgba(Theme.canvas.r, Theme.canvas.g, Theme.canvas.b, Theme.welcomeOpeningDimOpacity)
    signal cancelRequested()
    signal dismissed()
    objectName: namePrefix + "Overlay"
    visible: root.busy
    z: 10
    onVisibleChanged: {
        if (visible) openingCancel.forceActiveFocus(Qt.OtherFocusReason);
        else { root.cancelling = false; root.dismissed(); }
    }
    Rectangle {
        anchors.fill: parent
        color: root.backdropColor
    }
    MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons }
    Rectangle {
        id: openingCard
        objectName: root.namePrefix + "Card"
        anchors.centerIn: parent
        width: Math.min(Theme.welcomeOpeningCardWidth, parent.width - 32)
        height: Math.min(openingContent.implicitHeight + openingFooter.implicitHeight + Theme.welcomeOpeningGap + Theme.welcomeOpeningInset * 2, parent.height - 16)
        radius: Theme.welcomeOpeningRadius
        color: Theme.surface
        border.color: Theme.border
        Accessible.role: Accessible.Dialog
        Accessible.name: root.titleLabel
        ColumnLayout {
            anchors.fill: parent
            anchors.margins: Theme.welcomeOpeningInset
            spacing: Theme.welcomeOpeningGap
            ScrollView {
                id: openingScroll
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.minimumHeight: 0
                contentWidth: availableWidth
                contentHeight: openingContent.implicitHeight
                ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
                clip: true
                ColumnLayout {
                    id: openingContent
                    width: openingScroll.availableWidth
                    height: implicitHeight
                    spacing: Theme.welcomeOpeningGap
                    Text {
                        objectName: root.namePrefix + "Title"
                        Layout.fillWidth: true
                        text: root.titleLabel
                        color: Theme.text
                        font.pixelSize: Theme.welcomeOpeningTitleSize * Theme.fontScale
                        font.bold: true
                        wrapMode: Text.Wrap
                    }
                    Text {
                        Layout.fillWidth: true
                        text: root.archiveName
                        font.family: Theme.userDataFontFamily
                        textFormat: Text.PlainText
                        color: Theme.textSecondary
                        font.pixelSize: Theme.welcomeMainBodySize * Theme.fontScale
                        elide: Text.ElideMiddle
                    }
                    ProgressBar {
                        id: readingProgress
                        objectName: root.namePrefix + "Progress"
                        Layout.fillWidth: true
                        indeterminate: true
                        palette.highlight: Theme.accent
                        Accessible.name: root.progressLabel
                        background: Rectangle { implicitHeight: 4; color: Theme.taskTrack; radius: 2 }
                        contentItem: Item {
                            implicitHeight: 4
                            Rectangle {
                                id: readingBeam
                                width: parent.width * .32
                                height: parent.height
                                radius: 2
                                color: Theme.accent
                                SequentialAnimation on x {
                                    running: root.busy
                                    loops: Animation.Infinite
                                    NumberAnimation { from: 0; to: Math.max(0, readingProgress.availableWidth - readingBeam.width); duration: 1000; easing.type: Easing.InOutSine }
                                    NumberAnimation { to: 0; duration: 1000; easing.type: Easing.InOutSine }
                                }
                            }
                        }
                    }
                    Text {
                        Layout.fillWidth: true
                        text: root.noteLabel
                        color: Theme.textSecondary
                        font.pixelSize: Theme.welcomeMainBodySize * Theme.fontScale
                        wrapMode: Text.Wrap
                    }
                }
            }
            RowLayout {
                id: openingFooter
                objectName: root.namePrefix + "Footer"
                Layout.fillWidth: true
                Item { Layout.fillWidth: true }
                OperationFooterButton {
                    id: openingCancel
                    objectName: root.namePrefix + "Cancel"
                    enabled: !root.cancelling && !root.cancellationPending
                    text: root.cancelling || root.cancellationPending ? qsTrId("welcome.opening.cancelling") : root.cancelLabel
                    onClicked: { root.cancelling = true; root.cancelRequested(); }
                }
            }
        }
    }
}
