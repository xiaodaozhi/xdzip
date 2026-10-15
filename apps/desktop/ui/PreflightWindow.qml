pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

OperationWindow {
    id: root
    objectName: "preflightWindow"
    title: qsTrId("ui.ff6f3ae7c4fb")
    property var report: ({})
    width: Theme.preflightLogicalWidth
    requestedHeight: Math.min(Math.ceil(content.implicitHeight + footer.implicitHeight), Theme.preflightMaximumHeight, Screen.desktopAvailableHeight - 80)
    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        ScrollView {
            id: body
            objectName: "preflightBody"
            Layout.fillWidth: true
            Layout.fillHeight: true
            contentWidth: availableWidth
            clip: true
            ColumnLayout {
                id: content
                width: body.availableWidth
                spacing: 0
                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.margins: Theme.preflightContentInset
                    spacing: Theme.preflightSectionGap
                    Text {
                        objectName: "preflightHeading"
                        Layout.fillWidth: true
                        text: root.report.error ? qsTrId("ui.716a0b6591ca") : root.report.findingCount !== "0" ? qsTrId("ui.c668803dc295") : qsTrId("ui.9cb79c4bee66")
                        font.pixelSize: Theme.preflightTitleSize
                        font.bold: true
                        wrapMode: Text.Wrap
                        color: Theme.text
                    }
                    Text {
                        Layout.fillWidth: true
                        text: qsTrId("ui.db661e297081")
                        wrapMode: Text.Wrap
                        color: Theme.textSecondary
                        font.pixelSize: Theme.preflightBodySize
                    }
                    Rectangle {
                        objectName: "preflightSummary"
                        Layout.fillWidth: true
                        implicitHeight: stats.implicitHeight + Theme.preflightCardInset * 2
                        color: Theme.operationCard
                        radius: Theme.preflightCardRadius
                        border.color: Theme.operationCardBorder
                        RowLayout {
                            id: stats
                            anchors.fill: parent
                            anchors.margins: Theme.preflightCardInset
                            spacing: Theme.preflightSectionGap
                            Repeater {
                                model: [qsTrId("ui.0790c26b1e36"), qsTrId("ui.377532e93480"), qsTrId("ui.9d959257ce7e")]
                                ColumnLayout {
                                    id: statistic
                                    required property int index
                                    required property string modelData
                                    Layout.fillWidth: true
                                    Layout.preferredWidth: 1
                                    Text {
                                        Layout.fillWidth: true
                                        text: statistic.modelData
                                        wrapMode: Text.Wrap
                                        color: Theme.textSecondary
                                        font.pixelSize: Theme.preflightNoteSize
                                    }
                                    Text {
                                        Layout.fillWidth: true
                                        text: [root.report.entries || "—", root.report.bytes === "unknown" ? qsTrId("ui.aa524057ccca") : root.report.bytes || "—", root.report.findingCount || "—"][statistic.index]
                                        textFormat: Text.PlainText
                                        elide: Text.ElideRight
                                        color: Theme.text
                                        font.pixelSize: Theme.preflightCountSize
                                        font.bold: true
                                    }
                                }
                            }
                        }
                    }
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: Theme.preflightSectionGap
                        Text {
                            Layout.fillWidth: true
                            visible: !!root.report.error
                            text: root.report.error || ""
                            textFormat: Text.PlainText
                            wrapMode: Text.Wrap
                            color: Theme.text
                            font.pixelSize: Theme.preflightBodySize
                        }
                        Repeater {
                            model: root.report.findings || []
                            Text {
                                required property string modelData
                                Layout.fillWidth: true
                                text: modelData
                                textFormat: Text.PlainText
                                wrapMode: Text.Wrap
                                color: Theme.text
                                font.pixelSize: Theme.preflightBodySize
                            }
                        }
                        Text {
                            Layout.fillWidth: true
                            visible: !root.report.error
                            text: root.report.findingCount === "0" ? qsTrId("ui.f644f6ec6fad") : qsTrId("ui.780102188429")
                            color: Theme.textSecondary
                            wrapMode: Text.Wrap
                            font.pixelSize: Theme.preflightBodySize
                        }
                        Text {
                            Layout.fillWidth: true
                            visible: !!root.report.unicodeReview
                            text: qsTrId("ui.b8b0671912e5")
                            wrapMode: Text.Wrap
                            color: Theme.textSecondary
                            font.pixelSize: Theme.preflightBodySize
                        }
                    }
                }
            }
        }
        OperationFooter {
            id: footer
            objectName: "preflightFooter"
            Layout.fillWidth: true
            note: qsTrId("ui.0d92f3d4cae3")
            OperationFooterButton {
                objectName: "preflightClose"
                text: qsTrId("ui.26867436321f")
                primary: true
                onClicked: root.close()
            }
        }
    }
}
