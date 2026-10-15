pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: root
    objectName: "welcomePage"
    property bool canOpen: true
    property bool canCreate: true
    property bool busy: false
    property string message: ""
    property bool dropSupported: false
    property string dropMessage: ""
    signal openRequested
    signal createRequested
    signal cancelRequested
    signal dragEntered(var urls)
    signal archiveDropped(var urls)
    function focusCreate() {
        createAction.forceActiveFocus(Qt.TabFocusReason);
    }
    function revealAction(action) {
        const point = action.mapToItem(scroller.contentItem, 0, 0);
        if (point.y < scroller.contentY)
            scroller.contentY = Math.max(0, point.y - 6);
        else if (point.y + action.height > scroller.contentY + scroller.height)
            scroller.contentY = Math.min(scroller.contentHeight - scroller.height, point.y + action.height + 6 - scroller.height);
    }
    function focusOpen() {
        openAction.forceActiveFocus(Qt.TabFocusReason);
    }
    color: Theme.surface
    ScrollView {
        anchors.fill: parent
        clip: true
        ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
        Flickable {
            id: scroller
            contentWidth: width
            contentHeight: Math.max(height, contentColumn.implicitHeight + 56)
            ColumnLayout {
                id: contentColumn
                height: implicitHeight
                width: Math.min(Theme.welcomeContentWidth, root.width - 64)
                x: (root.width - width) / 2
                y: Math.max(28, (root.height - implicitHeight) / 2)
                spacing: 0
                RowLayout {
                    Layout.alignment: Qt.AlignHCenter
                    spacing: 10
                    Image {
                        source: "qrc:/xdzip/assets/app-icon.png"
                        Layout.preferredWidth: 26
                        Layout.preferredHeight: 26
                        fillMode: Image.PreserveAspectFit
                    }
                    Text {
                        text: "xdzip"
                        font.pixelSize: 14 * Theme.fontScale
                        font.letterSpacing: 1
                        color: Theme.textSecondary
                    }
                }
                Text {
                    objectName: "welcomeHeadline"
                    Layout.topMargin: root.width < 900 ? 12 : 17
                    Layout.fillWidth: true
                    text: qsTrId("ui.431ebe9b36de")
                    font.pixelSize: (root.width < 900 ? Theme.welcomeNarrowHeadlineSize : Theme.welcomeHeadlineSize) * Theme.fontScale
                    font.weight: Font.DemiBold
                    lineHeightMode: Text.FixedHeight
                    lineHeight: font.pixelSize * 1.45
                    horizontalAlignment: Text.AlignHCenter
                    color: Theme.text
                    wrapMode: Text.Wrap
                }
                Text {
                    Layout.topMargin: 8
                    Layout.fillWidth: true
                    text: qsTrId("ui.785a4505c2ba")
                    font.pixelSize: Theme.welcomeMainBodySize * Theme.fontScale
                    lineHeightMode: Text.FixedHeight
                    lineHeight: 22 * Theme.fontScale
                    horizontalAlignment: Text.AlignHCenter
                    color: Theme.textSecondary
                    wrapMode: Text.Wrap
                }
                RowLayout {
                    Layout.topMargin: root.width < 900 ? 24 : 30
                    Layout.fillWidth: true
                    spacing: Theme.welcomeActionGap
                    WelcomeAction {
                        id: openAction
                        objectName: "welcomeOpen"
                        Layout.fillWidth: true
                        Layout.preferredWidth: (Theme.welcomeContentWidth - Theme.welcomeActionGap) / 2
                        Layout.fillHeight: true
                        title: qsTrId("ui.80c2aec8adfc")
                        description: qsTrId("ui.0e801031324c")
                        shortcutText: (Qt.platform.os === "osx" ? "⌘ " : "Ctrl+") + "O"
                        asset: "open"
                        primary: true
                        busy: root.busy
                        enabled: root.canOpen
                        onClicked: root.openRequested()
                        onActiveFocusChanged: if (activeFocus)
                            root.revealAction(openAction)
                    }
                    WelcomeAction {
                        id: createAction
                        objectName: "welcomeCreate"
                        Layout.fillWidth: true
                        Layout.preferredWidth: (Theme.welcomeContentWidth - Theme.welcomeActionGap) / 2
                        Layout.fillHeight: true
                        title: qsTrId("ui.fe189b616c1d")
                        description: qsTrId("ui.b0d96542deee")
                        shortcutText: (Qt.platform.os === "osx" ? "⌘ " : "Ctrl+") + "N"
                        asset: "new"
                        enabled: root.canCreate
                        onClicked: root.createRequested()
                        onActiveFocusChanged: if (activeFocus)
                            root.revealAction(createAction)
                    }
                }
                Rectangle {
                    id: dropZone
                    objectName: "welcomeDropZone"
                    Layout.topMargin: Theme.welcomeDropGap
                    Layout.fillWidth: true
                    implicitHeight: Math.max(Theme.welcomeDropHeight, dropText.implicitHeight + 24)
                    radius: 8
                    color: dropArea.containsDrag && root.dropSupported ? Theme.selection : Theme.canvas
                    border.color: dropArea.containsDrag && root.dropSupported ? Theme.accentInk : Theme.operationFieldBorder
                    border.width: dropArea.containsDrag && root.dropSupported ? 2 : 0
                    Canvas {
                        anchors.fill: parent
                        visible: !dropArea.containsDrag || !root.dropSupported
                        property color strokeColor: Theme.operationFieldBorder
                        onStrokeColorChanged: requestPaint()
                        onWidthChanged: requestPaint()
                        onHeightChanged: requestPaint()
                        onPaint: {
                            const ctx = getContext("2d");
                            ctx.reset();
                            ctx.strokeStyle = strokeColor;
                            ctx.lineWidth = 1;
                            ctx.setLineDash([4, 3]);
                            const r = 8, x = .5, y = .5, w = width - 1, h = height - 1;
                            ctx.beginPath();
                            ctx.moveTo(x + r, y);
                            ctx.lineTo(x + w - r, y);
                            ctx.quadraticCurveTo(x + w, y, x + w, y + r);
                            ctx.lineTo(x + w, y + h - r);
                            ctx.quadraticCurveTo(x + w, y + h, x + w - r, y + h);
                            ctx.lineTo(x + r, y + h);
                            ctx.quadraticCurveTo(x, y + h, x, y + h - r);
                            ctx.lineTo(x, y + r);
                            ctx.quadraticCurveTo(x, y, x + r, y);
                            ctx.closePath();
                            ctx.stroke();
                        }
                    }
                    RowLayout {
                        anchors.centerIn: parent
                        width: Math.min(implicitWidth, parent.width - 24)
                        spacing: 10
                        Image {
                            source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "archive-grey.png"
                            Layout.preferredWidth: 20
                            Layout.preferredHeight: 20
                            fillMode: Image.PreserveAspectFit
                        }
                        Text {
                            id: dropText
                            Layout.fillWidth: true
                            text: dropArea.containsDrag ? root.dropMessage : qsTrId("ui.4a11171118b0")
                            font.pixelSize: Theme.welcomeMainBodySize * Theme.fontScale
                            color: dropArea.containsDrag && root.dropSupported ? Theme.accentInk : Theme.textSecondary
                            wrapMode: Text.Wrap
                        }
                    }
                }
                RowLayout {
                    Layout.topMargin: root.width < 900 ? 18 : 22
                    Layout.alignment: Qt.AlignHCenter
                    spacing: 8
                    Text {
                        text: qsTrId("ui.68e2f0d4ff9a")
                        font.pixelSize: 11 * Theme.fontScale
                        color: Theme.textSecondary
                    }
                    Repeater {
                        model: ["ZIP", "TAR", "TAR.GZ"]
                        Rectangle {
                            required property string modelData
                            implicitWidth: formatLabel.implicitWidth + 14
                            implicitHeight: formatLabel.implicitHeight + 6
                            color: Theme.canvas
                            border.color: Theme.border
                            radius: 4
                            Text {
                                id: formatLabel
                                anchors.centerIn: parent
                                text: parent.modelData
                                font.pixelSize: 11 * Theme.fontScale
                                color: Theme.textSecondary
                            }
                        }
                    }
                }
                Text {
                    objectName: "welcomeFeedback"
                    Layout.topMargin: visible ? 10 : 0
                    Layout.fillWidth: true
                    visible: text.length > 0
                    text: root.message
                    textFormat: Text.PlainText
                    font.pixelSize: 12 * Theme.fontScale
                    horizontalAlignment: Text.AlignHCenter
                    wrapMode: Text.Wrap
                    color: Theme.textSecondary
                    Accessible.role: Accessible.AlertMessage
                }
                OperationButton {
                    Layout.topMargin: visible ? 8 : 0
                    Layout.alignment: Qt.AlignHCenter
                    visible: root.busy
                    text: qsTrId("ui.caf06adff4f1")
                    onClicked: root.cancelRequested()
                }
            }
        }
    }

    DropArea {
        id: dropArea
        anchors.fill: parent
        enabled: root.canOpen
        onEntered: drag => {
            drag.accepted = drag.hasUrls;
            root.dragEntered(drag.urls);
        }
        onDropped: drop => {
            if (drop.hasUrls) {
                root.archiveDropped(drop.urls);
                drop.acceptProposedAction();
            }
        }
    }
}
