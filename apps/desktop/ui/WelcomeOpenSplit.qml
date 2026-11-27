pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import Xdzip.Settings 1.0

FocusScope {
    id: root
    property alias busy: openButton.busy
    signal clicked
    signal recentSelected(string path)

    readonly property int disclosureWidth: Math.round(Theme.welcomeRecentDisclosureWidth * Theme.fontScale)
    readonly property bool splitVisible: enabled && !busy
        && (openButton.hovered || disclosure.hovered || disclosure.visualFocus || recentMenu.visible)

    implicitWidth: openButton.implicitWidth
    implicitHeight: openButton.implicitHeight

    function focusOpen() {
        openButton.forceActiveFocus(Qt.OtherFocusReason);
    }
    onVisibleChanged: if (!visible) recentMenu.close()
    onEnabledChanged: if (!enabled) recentMenu.close()

    function showRecent() {
        disclosure.forceActiveFocus(Qt.OtherFocusReason);
        recentMenu.paths = AppSettings.recentArchives();
        recentMenu.open();
    }

    WelcomeAction {
        id: openButton
        objectName: "welcomeOpen"
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.right: disclosure.left
        title: qsTrId("ui.80c2aec8adfc")
        description: qsTrId("ui.0e801031324c")
        shortcutText: (Qt.platform.os === "osx" ? "⌘ " : "Ctrl+") + "O"
        asset: "open"
        primary: true
        onClicked: root.clicked()
        Keys.onDownPressed: event => {
            if (event.modifiers & Qt.AltModifier) {
                root.showRecent();
                event.accepted = true;
            }
        }
        background: Rectangle {
            objectName: "welcomeOpenBackground"
            width: root.splitVisible ? openButton.width : root.width
            height: openButton.height
            radius: Theme.welcomeActionRadius
            topRightRadius: root.splitVisible ? 0 : radius
            bottomRightRadius: topRightRadius
            color: !openButton.enabled ? Theme.accentDisabled
                : openButton.down ? Theme.accentPressed
                : openButton.hovered ? Theme.accentHover : Theme.accent
            Rectangle {
                anchors.fill: parent
                anchors.margins: 3
                visible: openButton.visualFocus
                color: "transparent"
                border.color: Theme.accentText
                radius: Math.max(0, Theme.welcomeActionRadius - 3)
            }
        }
    }

    Button {
        id: disclosure
        objectName: "welcomeRecentToggle"
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: root.disclosureWidth
        enabled: !root.busy
        hoverEnabled: true
        focusPolicy: Qt.StrongFocus
        Accessible.name: qsTrId("desktop.recent")
        Accessible.description: "Alt+↓"
        onClicked: root.showRecent()
        Keys.onReturnPressed: root.showRecent()
        Keys.onDownPressed: root.showRecent()

        background: Rectangle {
            objectName: "welcomeRecentBackground"
            visible: root.splitVisible
            radius: Theme.welcomeActionRadius
            topLeftRadius: 0
            bottomLeftRadius: 0
            color: disclosure.down || recentMenu.visible ? Theme.accentPressed
                : disclosure.hovered ? Theme.accentHover : Theme.accent
            Rectangle {
                width: 1
                height: parent.height
                color: Theme.accentText
                opacity: Theme.welcomeRecentDividerOpacity
            }
            Rectangle {
                anchors.fill: parent
                anchors.margins: 3
                visible: disclosure.visualFocus
                color: "transparent"
                border.color: Theme.accentText
                radius: Math.max(0, Theme.welcomeActionRadius - 3)
            }
        }
        contentItem: Item {
            Canvas {
                anchors.centerIn: parent
                width: Theme.welcomeRecentDisclosureIconSize
                height: width
                visible: root.splitVisible
                onPaint: {
                    const context = getContext("2d");
                    context.reset();
                    context.strokeStyle = Theme.accentText;
                    context.lineWidth = 1.5;
                    context.lineCap = "round";
                    context.lineJoin = "round";
                    context.beginPath();
                    context.moveTo(3, 5);
                    context.lineTo(7, 9);
                    context.lineTo(11, 5);
                    context.stroke();
                }
            }
        }
        padding: (width - Theme.welcomeRecentDisclosureIconSize) / 2
    }

    WelcomeRecentPopup {
        id: recentMenu
        parent: root
        focusReturn: disclosure
        onRecentSelected: path => root.recentSelected(path)
        onClearRequested: AppSettings.clearRecentArchives()
    }
}
