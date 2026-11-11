pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Xdzip.Settings 1.0

Rectangle {
    id: panel
    required property var detail
    required property var sizeFormatter
    required property var iconForFile
    property var preview: ({})
    property bool canPreview: false
    readonly property bool imageProviderReady: PreviewImages.ready()
    signal previewRequested(string id)
    signal previewCancelled
    property int selectionCount: 0
    property bool showCloseAction: true
    property bool canExtract: false
    signal extractRequested(string id)
    signal closeRequested
    color: Theme.inspector
    Rectangle {
        width: 1
        height: parent.height
        color: Theme.border
    }
    PaneHeader {
        id: header
        objectName: "inspectorPaneHeader"
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: implicitHeight
        leftInset: Theme.mainInspectorLeftInset
        rightInset: Theme.mainInspectorRightInset
        actionInset: panel.showCloseAction ? Theme.mainIconButtonSize + Theme.mainUnits(10) : 0
        Text {
            objectName: "inspectorPaneTitle"
            Layout.fillWidth: true
            Layout.fillHeight: true
            text: qsTrId("ui.f3c5fe4dd2da")
            font.pixelSize: Theme.mainPaneTitleSize
            font.weight: Font.DemiBold
            color: Theme.text
            elide: Text.ElideRight
            verticalAlignment: Text.AlignVCenter
        }
        ArchiveAction {
            objectName: "inspectorClose"
            parent: header
            anchors.right: parent.right
            anchors.rightMargin: header.rightInset
            anchors.verticalCenter: parent.verticalCenter
            width: Theme.mainIconButtonSize
            height: Theme.mainIconButtonSize
            visible: panel.showCloseAction
            leftPadding: 0
            rightPadding: 0
            asset: "close"
            Accessible.name: qsTrId("ui.0b4ff2491b1d")
            explanation: Accessible.name
            onClicked: panel.closeRequested()
        }
    }
    ScrollView {
        id: scroll
        anchors.top: header.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.leftMargin: Theme.mainInspectorLeftInset
        anchors.rightMargin: Theme.mainInspectorRightInset
        anchors.bottomMargin: Theme.mainUnits(20)
        contentWidth: availableWidth
        ColumnLayout {
            width: scroll.availableWidth
            spacing: 0
            Rectangle {
                objectName: "previewContent"
                Layout.fillWidth: true
                implicitHeight: Math.round(Theme.mainPreviewHeight)
                radius: Theme.mainPreviewRadius
                color: Theme.canvas
                border.color: Theme.border
                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: Theme.contentPreviewInset
                    spacing: Theme.contentPreviewGap
                    Item {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        clip: true
                        ColumnLayout {
                            anchors.centerIn: parent
                            width: parent.width
                            spacing: Theme.contentPreviewGap
                            visible: !panel.preview.state || panel.preview.state === "loading" || panel.preview.state === "password" || panel.preview.state === "error"
                            Image {
                                visible: !panel.preview.state
                                Layout.alignment: Qt.AlignHCenter
                                Layout.preferredWidth: Theme.contentPreviewIconSize
                                Layout.preferredHeight: Theme.contentPreviewIconSize
                                source: panel.iconForFile(panel.detail.name || "", panel.detail.directory || false)
                                fillMode: Image.PreserveAspectFit
                            }
                            BusyIndicator {
                                Layout.alignment: Qt.AlignHCenter
                                visible: panel.preview.state === "loading"
                                running: visible
                                implicitWidth: Theme.contentPreviewSpinnerSize
                                implicitHeight: implicitWidth
                                Accessible.name: qsTrId("preview.loading")
                            }
                            Text {
                                Layout.fillWidth: true
                                text: panel.preview.message || (panel.preview.state === "loading" ? qsTrId("preview.loading") : panel.preview.state === "password" ? qsTrId("preview.passwordDescription") : panel.detail.id ? qsTrId("preview.hint") : qsTrId("ui.3d997d4452fc"))
                                textFormat: Text.PlainText
                                font.pixelSize: Theme.mainPropertySize
                                color: panel.preview.state === "error" ? Theme.menuDanger : Theme.textSecondary
                                horizontalAlignment: Text.AlignHCenter
                                wrapMode: Text.Wrap
                            }
                        }
                        ScrollView {
                            objectName: "contentPreviewTextScroll"
                            anchors.fill: parent
                            visible: panel.preview.state === "text"
                            contentWidth: availableWidth
                            TextArea {
                                objectName: "contentPreviewText"
                                text: visible ? panel.preview.text || "" : ""
                                textFormat: TextEdit.PlainText
                                readOnly: true
                                selectByMouse: true
                                wrapMode: TextEdit.Wrap
                                color: Theme.text
                                font.family: Theme.userDataFontFamily
                                font.pixelSize: Theme.mainPropertySize
                                padding: 0
                                background: null
                                Accessible.name: qsTrId("preview.action")
                            }
                        }
                        Image {
                            id: contentImage
                            objectName: "contentPreviewImage"
                            anchors.fill: parent
                            visible: panel.preview.state === "image"
                            source: visible && panel.imageProviderReady ? panel.preview.image || "" : ""
                            sourceSize: Qt.size(Theme.contentPreviewImageSample, Theme.contentPreviewImageSample)
                            asynchronous: true
                            cache: false
                            fillMode: Image.PreserveAspectFit
                            Accessible.name: panel.detail.name || qsTrId("preview.action")
                        }
                        Text {
                            anchors.fill: parent
                            visible: panel.preview.state === "image" && contentImage.status === Image.Error
                            text: qsTrId("preview.imageFailed")
                            textFormat: Text.PlainText
                            wrapMode: Text.Wrap
                            horizontalAlignment: Text.AlignHCenter
                            verticalAlignment: Text.AlignVCenter
                            color: Theme.menuDanger
                            font.pixelSize: Theme.mainPropertySize
                        }
                    }
                    ArchiveAction {
                        objectName: "contentPreviewAction"
                        Layout.alignment: Qt.AlignHCenter
                        visible: Boolean(panel.detail.id) && !Boolean(panel.detail.directory)
                        text: panel.preview.state === "loading" || panel.preview.state === "password" ? qsTrId("common.cancel") : panel.preview.state ? qsTrId("preview.again") : qsTrId("preview.action")
                        enabled: panel.canPreview || panel.preview.state === "loading" || panel.preview.state === "password"
                        onClicked: panel.preview.state === "loading" || panel.preview.state === "password" ? panel.previewCancelled() : panel.previewRequested(String(panel.detail.id))
                    }
                }
            }
            Text {
                Layout.fillWidth: true
                Layout.topMargin: Theme.mainUnits(14)
                text: panel.selectionCount > 1 ? qsTrId("ui.aeb126abb448", Number(panel.selectionCount)) : panel.detail.name || qsTrId("ui.ce2a58ab4ccf")
                textFormat: Text.PlainText
                wrapMode: Text.WrapAnywhere
                font.family: panel.selectionCount <= 1 && panel.detail.name ? Theme.userDataFontFamily : Theme.uiFontFamily
                font.pixelSize: Theme.mainDetailTitleSize
                font.weight: Font.DemiBold
                color: Theme.text
            }
            Text {
                Layout.fillWidth: true
                Layout.topMargin: Theme.mainUnits(6)
                text: panel.selectionCount > 1 ? qsTrId("ui.43850dc99c15") : panel.detail.type ? panel.detail.type + (panel.detail.directory ? "" : qsTrId("ui.8915e82b0e8f")) : qsTrId("ui.a853163f3d9c")
                color: Theme.textSecondary
                font.pixelSize: Theme.mainDetailTypeSize
                wrapMode: Text.Wrap
            }
            Rectangle {
                Layout.fillWidth: true
                Layout.topMargin: Theme.mainUnits(16)
                Layout.bottomMargin: Theme.mainUnits(10)
                implicitHeight: 1
                color: Theme.border
            }
            Text {
                visible: panel.detail.attention || false
                Layout.fillWidth: true
                Layout.bottomMargin: visible ? Theme.mainUnits(8) : 0
                text: panel.detail.status || ""
                textFormat: Text.PlainText
                wrapMode: Text.Wrap
                font.pixelSize: Theme.mainPropertySize
                color: Theme.menuDanger
            }
            Repeater {
                model: [
                    {
                        label: qsTrId("ui.301a69c74dfc"),
                        value: panel.detail.size ? panel.sizeFormatter(panel.detail.size) : "—"
                    },
                    {
                        label: qsTrId("ui.2582e6a2c9e2"),
                        value: panel.detail.packed ? panel.sizeFormatter(panel.detail.packed) : "—"
                    },
                    {
                        label: qsTrId("ui.2cbced881b2d"),
                        value: panel.detail.modified || "—"
                    },
                    {
                        label: qsTrId("ui.9e9d1bfc0746"),
                        value: panel.detail.method || "—"
                    }
                ]
                GridLayout {
                    required property var modelData
                    columns: Theme.fontScale > 1.3 ? 1 : 2
                    rowSpacing: Theme.mainUnits(4)
                    columnSpacing: Theme.mainUnits(10)
                    Layout.fillWidth: true
                    visible: panel.selectionCount > 0
                    Layout.minimumHeight: Math.max(Theme.contentPreviewSpinnerSize, Theme.mainPropertySize * 1.6 + Theme.mainUnits(8))
                    Text {
                        text: parent.modelData.label
                        color: Theme.textSecondary
                        font.pixelSize: Theme.mainPropertySize
                    }
                    Text {
                        Layout.fillWidth: true
                        text: parent.modelData.value
                        textFormat: Text.PlainText
                        wrapMode: Text.WrapAnywhere
                        horizontalAlignment: Text.AlignRight
                        color: Theme.text
                        font.pixelSize: Theme.mainPropertySize
                    }
                }
            }
            ArchiveAction {
                id: extractAction
                objectName: "inspectorExtract"
                visible: panel.selectionCount > 0
                Layout.fillWidth: true
                Layout.topMargin: Theme.mainUnits(12)
                implicitHeight: Math.max(Theme.mainUnits(47), Theme.mainHeaderSize * 1.6 + Theme.mainUnits(16))
                asset: "external-open"
                text: qsTrId("ui.236df7227118")
                enabled: panel.canExtract && Boolean(panel.detail.id)
                explanation: qsTrId("ui.9abb78dabe17")
                onClicked: panel.extractRequested(String(panel.detail.id))
                background: Rectangle {
                    objectName: "inspectorExtractBackground"
                    radius: Theme.mainUnits(6)
                    color: !panel.canExtract ? Theme.surface : extractAction.down ? Theme.controlPressed : extractAction.hovered ? Theme.controlHover : Theme.surface
                    border.color: Theme.border
                    border.width: 1
                }
            }
        }
    }
}
