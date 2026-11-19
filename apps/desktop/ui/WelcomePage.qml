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
    property string expandedFormatGroup: "archives"
    readonly property int expandedFormatIndex: ArchiveFormats.formatGroups.findIndex(group => group.id === expandedFormatGroup)
    signal openRequested
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
        openAction.forceActiveFocus(Qt.TabFocusReason);
    }
    function toggleFormatGroup(groupId) {
        expandedFormatGroup = expandedFormatGroup === groupId ? "" : groupId;
        if (expandedFormatGroup)
            Qt.callLater(() => { if (formatPreview.visible) root.revealAction(formatPreview); });
    }
    function formatGroupLabel(groupId) {
        switch (groupId) {
        case "archives": return qsTrId("format.group.archives");
        case "compressed-tar": return qsTrId("format.group.compressed-tar");
        case "compressed-streams": return qsTrId("format.group.compressed-streams");
        case "text-envelopes": return qsTrId("format.group.text-envelopes");
        case "installer-metadata": return qsTrId("format.group.installer-metadata");
        default: return groupId;
        }
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
                ColumnLayout {
                    objectName: "welcomeFormats"
                    Layout.topMargin: root.width < 900 ? 18 : 22
                    Layout.fillWidth: true
                    spacing: Theme.welcomeFormatGap
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: Theme.welcomeFormatGap
                        Text {
                            text: qsTrId("ui.68e2f0d4ff9a")
                            Layout.fillWidth: true
                            font.pixelSize: Math.round(Theme.welcomeFormatTitleSize * Theme.fontScale)
                            font.weight: Font.DemiBold
                            color: Theme.text
                        }
                        FormatTextButton {
                            id: formatsSettingsButton
                            objectName: "welcomeFormatsSettings"
                            text: qsTrId("welcome.formatsOpenSettings")
                            labelSize: Theme.welcomeFormatFontSize * Theme.fontScale
                            onClicked: root.formatsRequested()
                            onActiveFocusChanged: if (activeFocus)
                                root.revealAction(formatsSettingsButton)
                            contentItem: RowLayout {
                                spacing: Theme.welcomeFormatChipGap
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
                    Flow {
                        Layout.fillWidth: true
                        spacing: Theme.welcomeFormatGroupGap
                        Repeater {
                            model: ArchiveFormats.formatGroups
                            delegate: FormatTextButton {
                                id: badge
                                required property var modelData
                                objectName: "welcomeFormatGroup-" + modelData.id
                                readonly property bool expanded: root.expandedFormatGroup === modelData.id
                                text: root.formatGroupLabel(modelData.id)
                                Accessible.name: qsTrId("welcome.formatGroupCount", modelData.suffixes.length).arg(text)
                                Accessible.checkable: true
                                Accessible.checked: expanded
                                Accessible.onPressAction: if (enabled) click()
                                Accessible.onToggleAction: if (enabled) click()
                                onClicked: root.toggleFormatGroup(modelData.id)
                                onActiveFocusChanged: if (activeFocus) root.revealAction(badge)
                                baseHeight: Theme.welcomeFormatGroupHeight
                                labelSize: Theme.welcomeFormatTitleSize * Theme.fontScale
                                contentItem: RowLayout {
                                    spacing: Theme.welcomeFormatChipGap
                                    Text {
                                        text: badge.text
                                        font.pixelSize: Math.round(badge.labelPixelSize)
                                        font.weight: badge.expanded ? Font.DemiBold : Font.Normal
                                        color: badge.expanded || badge.hovered ? Theme.accentInk : Theme.text
                                        Accessible.ignored: true
                                    }
                                    Text {
                                        text: String(badge.modelData.suffixes.length)
                                        font.pixelSize: Math.round(Theme.welcomeFormatCountSize * Theme.fontScale)
                                        color: badge.expanded ? Theme.accentInk : Theme.textSecondary
                                        Accessible.ignored: true
                                    }
                                }
                            }
                        }
                    }
                    Item {
                        id: formatPreview
                        objectName: "welcomeFormatPreview"
                        readonly property var suffixes: root.expandedFormatIndex < 0 ? []
                            : ArchiveFormats.formatGroups[root.expandedFormatIndex].suffixes
                        readonly property bool truncated: extensionRow.implicitWidth > width
                        Layout.fillWidth: true
                        implicitHeight: extensionRow.implicitHeight
                        visible: root.expandedFormatIndex >= 0
                        Accessible.role: Accessible.StaticText
                        Accessible.name: suffixes.join(", ")
                        TextMetrics {
                            id: lowercaseMetrics
                            text: "x"
                            font.family: Theme.uiFontFamily
                            font.weight: Font.Normal
                            font.pixelSize: Math.round(Theme.welcomeFormatFontSize * Theme.fontScale)
                        }
                        Item {
                            id: extensionViewport
                            anchors.left: parent.left
                            anchors.verticalCenter: parent.verticalCenter
                            width: Math.max(0, parent.width - (formatPreview.truncated
                                ? overflowMark.implicitWidth + Theme.welcomeFormatChipGap : 0))
                            height: extensionRow.implicitHeight
                            clip: true
                            Row {
                                id: extensionRow
                                spacing: Theme.welcomeFormatChipGap
                                Repeater {
                                    model: formatPreview.suffixes
                                    delegate: Rectangle {
                                        id: chip
                                        required property string modelData
                                        // Keep the row's measured width even if a chip would be clipped.
                                        opacity: x + width <= extensionViewport.width ? 1 : 0
                                        implicitWidth: suffix.implicitWidth + 2 * Theme.welcomeFormatChipInset
                                        implicitHeight: suffix.implicitHeight + 2 * Theme.welcomeFormatChipVerticalInset
                                        color: Theme.operationButtonSurface
                                        border.color: Theme.operationFieldBorder
                                        border.width: 1
                                        radius: Theme.welcomeFormatChipRadius
                                        antialiasing: true
                                        Text {
                                            id: suffix
                                            anchors.horizontalCenter: parent.horizontalCenter
                                            // Center the lowercase body on a shared baseline.
                                            y: (chip.height - lowercaseMetrics.tightBoundingRect.height) / 2
                                                - baselineOffset - lowercaseMetrics.tightBoundingRect.y
                                            text: chip.modelData
                                            color: Theme.textSecondary
                                            font: lowercaseMetrics.font
                                            Accessible.ignored: true
                                        }
                                    }
                                }
                            }
                        }
                        Text {
                            id: overflowMark
                            anchors.right: parent.right
                            anchors.verticalCenter: parent.verticalCenter
                            visible: formatPreview.truncated
                            text: "…"
                            font.pixelSize: Math.round(Theme.welcomeFormatFontSize * Theme.fontScale)
                            color: Theme.textSecondary
                            Accessible.ignored: true
                        }
                        HoverHandler { id: formatHover }
                        CommandTooltip {
                            requested: formatHover.hovered && formatPreview.truncated
                            text: formatPreview.suffixes.join(", ")
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
