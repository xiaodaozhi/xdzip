pragma ComponentBehavior: Bound
import QtQuick
import "ArchiveFormats.js" as ArchiveFormats
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: root
    objectName: "welcomePage"
    property bool canOpen: true
    property bool canCreate: true
    property bool busy: false
    property bool waitingForUpdate: false
    property alias cancellingOpening: openingOverlay.cancelling
    property bool externalOpening: false
    property string message: ""
    property string openingName: ""
    property bool dropSupported: false
    property string dropMessage: ""
    readonly property int readableFormatCount: new Set(ArchiveFormats.formatGroups
        .filter(group => group.id !== "installer-metadata")
        .reduce((labels, group) => labels.concat(group.labels), [])).size
    signal openRequested
    signal recentArchiveRequested(string path)
    signal createRequested
    signal formatsRequested
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
        openAction.focusOpen();
    }
    component FormatTextButton: OperationButton {
        id: control
        horizontalInset: 0
        leftPadding: 0
        rightPadding: 0
        background: Rectangle {
            color: "transparent"
            border.color: Theme.accentInk
            border.width: control.visualFocus ? 1 : 0
            radius: control.cornerRadius
        }
    }
    color: Theme.surface
    ScrollView {
        objectName: "welcomeContent"
        visible: !root.externalOpening
        enabled: !root.busy
        anchors.fill: parent
        clip: true
        ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
        Flickable {
            id: scroller
            contentWidth: width
            contentHeight: Math.max(height, contentColumn.y + contentColumn.implicitHeight + Theme.welcomeContentTopInset)
            ColumnLayout {
                id: contentColumn
                height: implicitHeight
                width: Math.min(Theme.welcomeContentWidth, root.width - 64)
                x: (root.width - width) / 2
                y: Theme.welcomeContentTopInset
                spacing: 0
                BrandWordmark {
                    objectName: "welcomeBrandWordmark"
                    Layout.alignment: Qt.AlignHCenter
                    iconHeight: Theme.welcomeWordmarkHeight
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
                    WelcomeOpenSplit {
                        objectName: "welcomeOpenCard"
                        id: openAction
                        Layout.fillWidth: true
                        Layout.preferredWidth: (Theme.welcomeContentWidth - Theme.welcomeActionGap) / 2
                        Layout.fillHeight: true
                        busy: root.busy
                        enabled: root.canOpen
                        onClicked: root.openRequested()
                        onRecentSelected: path => root.recentArchiveRequested(path)
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
                    objectName: "welcomeFormats"
                    Layout.topMargin: root.width < 900 ? 18 : 22
                    Layout.fillWidth: true
                    spacing: Theme.welcomeFormatGap
                    Text {
                        objectName: "welcomeFormatSummary"
                        Layout.fillWidth: true
                        font.pixelSize: Math.round(Theme.welcomeFormatTitleSize * Theme.fontScale)
                        color: Theme.textSecondary
                        wrapMode: Text.WordWrap
                        textFormat: Text.PlainText
                        text: qsTrId("welcome.formatSummary", root.readableFormatCount)
                    }
                    FormatTextButton {
                        id: formatsSettingsButton
                        objectName: "welcomeFormatsSettings"
                        text: qsTrId("welcome.formatsOpenSettings")
                        labelSize: Theme.welcomeFormatFontSize * Theme.fontScale
                        horizontalInset: Theme.welcomeFormatLinkInset
                        leftPadding: Theme.welcomeFormatLinkInset
                        rightPadding: Theme.welcomeFormatLinkInset
                        background: Rectangle {
                            objectName: "welcomeFormatsSettingsBackground"
                            radius: formatsSettingsButton.cornerRadius
                            color: !formatsSettingsButton.enabled ? "transparent"
                                : formatsSettingsButton.down ? Theme.controlPressed
                                : formatsSettingsButton.hovered ? Qt.rgba(Theme.controlHover.r, Theme.controlHover.g, Theme.controlHover.b, 0.5) : "transparent"
                        }
                        onClicked: root.formatsRequested()
                        onActiveFocusChanged: if (activeFocus)
                            root.revealAction(formatsSettingsButton)
                        contentItem: RowLayout {
                            spacing: Theme.welcomeFormatLinkGap
                            Text {
                                text: formatsSettingsButton.text
                                font.pixelSize: Math.round(formatsSettingsButton.labelPixelSize)
                                color: Theme.accentInk
                                Accessible.ignored: true
                            }
                            Image {
                                Layout.preferredWidth: Theme.welcomeFormatTitleSize
                                Layout.preferredHeight: Theme.welcomeFormatTitleSize
                                source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "chevron-right.png"
                            }
                        }
                    }
                }
                RowLayout {
                    objectName: "welcomeFeedbackRow"
                    Layout.topMargin: visible ? Theme.welcomeFeedbackTopGap : 0
                    Layout.fillWidth: true
                    visible: root.message.length > 0
                    spacing: Theme.welcomeFeedbackGap
                    Image {
                        objectName: "welcomeFeedbackIcon"
                        Layout.preferredWidth: Theme.welcomeFeedbackIconSize
                        Layout.preferredHeight: Theme.welcomeFeedbackIconSize
                        Layout.alignment: Qt.AlignTop
                        source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "status-warning.png"
                        fillMode: Image.PreserveAspectFit
                        Accessible.ignored: true
                    }
                    Text {
                        objectName: "welcomeFeedback"
                        Layout.fillWidth: true
                        text: root.message
                        textFormat: Text.PlainText
                        font.pixelSize: 12 * Theme.fontScale
                        wrapMode: Text.Wrap
                        color: Theme.menuDanger
                        Accessible.role: Accessible.AlertMessage
                    }
                }

            }
        }
    }

    BrowserBusyOverlay {
        id: openingOverlay
        anchors.fill: parent
        busy: root.busy
        archiveName: root.openingName
        titleLabel: root.waitingForUpdate ? qsTrId("sessions.waitForArchive") : qsTrId("ui.821ffb8cd6a2")
        noteLabel: root.waitingForUpdate ? qsTrId("sessions.waitForArchiveNote") : qsTrId("welcome.opening.note")
        progressLabel: root.waitingForUpdate ? qsTrId("sessions.waitForArchive") : qsTrId("welcome.opening.progress")
        onCancelRequested: root.cancelRequested()
        onDismissed: Qt.callLater(function () { if (root.visible && root.canOpen) root.focusOpen(); })
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
