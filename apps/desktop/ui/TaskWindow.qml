import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

OperationWindow {
    id: root
    title: qsTr("解压进度 · 示例数据")
    height: Math.min(620, Screen.desktopAvailableHeight - 80)
    allowMinimize: true
    // Presentation values belong to the owner. This preview performs no archive I/O.
    required property string archiveName
    required property string destination
    required property string currentFile
    required property real fraction
    required property bool totalKnown
    required property bool stopping
    property string processedSummary: qsTr("等待任务数据")
    property string fileCountSummary: qsTr("数量未知")
    property string speedSummary: qsTr("等待估算")
    property string remainingSummary: qsTr("等待估算")
    property string phaseSummary: qsTr("准备中")
    readonly property bool largeType: Theme.fontScale > 1.35
    signal cancelRequested()
    readonly property alias timingHint: timingHint
    readonly property alias closingHint: closingHint

    ScrollView {
        id: scroll
        objectName: "taskScroll"
        anchors.fill: parent
        contentWidth: availableWidth
        clip: true
        ColumnLayout {
            width: scroll.availableWidth
            spacing: 0
            ColumnLayout {
                Layout.fillWidth: true
                Layout.margins: Theme.gutter
                spacing: 12
                RowLayout {
                    Layout.fillWidth: true
                    Layout.bottomMargin: 4
                    spacing: 18
                    Image {
                        source: "qrc:/xdzip/assets/document-download.png"
                        Layout.preferredWidth: 66
                        Layout.preferredHeight: 66
                        fillMode: Image.PreserveAspectFit
                        Accessible.ignored: true
                    }
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 4
                        Text {
                            Layout.fillWidth: true
                            text: root.stopping ? qsTr("正在停止") : qsTr("正在解压")
                            font.pixelSize: Theme.panelTitleSize * 1.25
                            font.bold: true
                            color: Theme.text
                            wrapMode: Text.Wrap
                        }
                        Text {
                            Layout.fillWidth: true
                            text: root.archiveName
                            textFormat: Text.PlainText
                            font.pixelSize: Theme.bodySize
                            color: Theme.textSecondary
                            elide: Text.ElideMiddle
                        }
                    }
                    Text {
                        Layout.alignment: Qt.AlignTop
                        text: qsTr("示例数据")
                        color: Theme.textSecondary
                        font.pixelSize: Theme.secondarySize
                    }
                }
                TaskPathCard {
                    Layout.fillWidth: true
                    heading: qsTr("解压到")
                    value: root.destination
                    iconSource: "qrc:/xdzip/assets/folder.png"
                }
                Rectangle {
                    Layout.fillWidth: true
                    implicitHeight: progressColumn.implicitHeight + 32
                    color: Theme.taskPanel
                    border.color: Theme.taskBorder
                    radius: Theme.controlRadius
                    ColumnLayout {
                        id: progressColumn
                        anchors.fill: parent
                        anchors.margins: 16
                        spacing: 12
                        RowLayout {
                            Layout.fillWidth: true
                            spacing: 16
                            ProgressBar {
                                id: progress
                                Layout.fillWidth: true
                                value: Number.isFinite(root.fraction) ? Math.max(0, Math.min(1, root.fraction)) : 0
                                indeterminate: !root.totalKnown
                                Accessible.name: qsTr("解压进度")
                                background: Rectangle {
                                    implicitHeight: 18
                                    radius: 9
                                    color: Theme.taskTrack
                                }
                                contentItem: Item {
                                    implicitHeight: 18
                                    Rectangle {
                                        objectName: "progressFill"
                                        width: progress.indeterminate ? 0 : parent.width * progress.visualPosition
                                        height: parent.height
                                        radius: 9
                                        gradient: Gradient {
                                            orientation: Gradient.Horizontal
                                            GradientStop { position: 0; color: Theme.progressStart }
                                            GradientStop { position: 1; color: Theme.progressEnd }
                                        }
                                    }
                                }
                            }
                            Text {
                                objectName: "progressLabel"
                                text: root.totalKnown ? Math.floor(progress.value * 100) + "%" : qsTr("总量未知")
                                font.pixelSize: Theme.panelTitleSize * 1.15
                                font.bold: true
                                color: Theme.accentInk
                            }
                        }
                        GridLayout {
                            Layout.fillWidth: true
                            columns: root.largeType ? 1 : 2
                            rowSpacing: 8
                            Text {
                                Layout.fillWidth: true
                                objectName: "processedLabel"
                                text: root.processedSummary
                                textFormat: Text.PlainText
                                font.pixelSize: Theme.bodySize
                                color: Theme.textSecondary
                                wrapMode: Text.Wrap
                            }
                            Text {
                                text: root.totalKnown ? root.fileCountSummary : qsTr("文件总数未知")
                                font.pixelSize: Theme.bodySize
                                color: Theme.textSecondary
                            }
                        }
                        Rectangle {
                            Layout.fillWidth: true
                            implicitHeight: 1
                            color: Theme.taskBorder
                        }
                        GridLayout {
                            Layout.fillWidth: true
                            columns: root.largeType ? 1 : 5
                            columnSpacing: 18
                            rowSpacing: 12
                            TaskMetric {
                                Layout.fillWidth: true
                                Layout.preferredWidth: 1
                                heading: qsTr("速度")
                                value: root.stopping ? "—" : root.speedSummary
                            }
                            Rectangle {
                                visible: !root.largeType
                                Layout.fillHeight: true
                                implicitWidth: 1
                                color: Theme.taskBorder
                            }
                            TaskMetric {
                                Layout.fillWidth: true
                                Layout.preferredWidth: 1
                                heading: qsTr("预计剩余")
                                value: root.stopping || !root.totalKnown ? "—" : root.remainingSummary
                            }
                            Rectangle {
                                visible: !root.largeType
                                Layout.fillHeight: true
                                implicitWidth: 1
                                color: Theme.taskBorder
                            }
                            TaskMetric {
                                Layout.fillWidth: true
                                Layout.preferredWidth: 1
                                heading: qsTr("当前阶段")
                                value: root.stopping ? qsTr("停止与清理") : root.phaseSummary
                            }
                        }
                    }
                }
                TaskPathCard {
                    Layout.fillWidth: true
                    objectName: "currentFileCard"
                    heading: qsTr("当前文件")
                    value: root.currentFile
                    iconSource: "qrc:/xdzip/assets/image-file.png"
                }
                InfoRow {
                    id: timingHint
                    Layout.fillWidth: true
                    text: qsTr("实际耗时取决于文件与磁盘状态。")
                }
            }
            Rectangle {
                Layout.fillWidth: true
                implicitHeight: 1
                color: Theme.taskBorder
            }
            GridLayout {
                Layout.fillWidth: true
                Layout.leftMargin: Theme.gutter
                Layout.rightMargin: Theme.gutter
                Layout.topMargin: 12
                Layout.bottomMargin: 12
                columns: root.largeType ? 1 : 2
                columnSpacing: 16
                rowSpacing: 12
                InfoRow {
                    id: closingHint
                    Layout.fillWidth: true
                    text: qsTr("关闭此窗口后，任务仍会继续。")
                }
                RowLayout {
                    Layout.alignment: Qt.AlignRight
                    spacing: 10
                    AppButton {
                        text: qsTr("返回浏览")
                        onClicked: root.close()
                    }
                    AppButton {
                        objectName: "cancelButton"
                        text: root.stopping ? qsTr("正在停止…") : qsTr("取消任务")
                        enabled: !root.stopping
                        onClicked: root.cancelRequested()
                    }
                }
            }
        }
    }
}
