import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

OperationWindow {
    id: root
    title: qsTr("解压进度 · 组件预览")
    allowMinimize: true
    // All values are supplied by the preview owner. No archive work runs here.
    required property string archiveName
    required property string destination
    required property string currentFile
    required property real fraction
    required property bool totalKnown
    required property bool stopping
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
                spacing: Theme.gutter
                RowLayout {
                    spacing: Theme.gutter
                    Image {
                        source: "qrc:/xdzip/assets/document-download.png"
                        Layout.preferredWidth: 56
                        Layout.preferredHeight: 64
                        sourceSize: Qt.size(112, 128)
                        fillMode: Image.PreserveAspectFit
                        Accessible.ignored: true
                    }
                    ColumnLayout {
                        Layout.fillWidth: true
                        Text {
                            Layout.fillWidth: true
                            text: root.stopping ? qsTr("正在停止") : qsTr("正在解压")
                            font.pixelSize: Theme.panelTitleSize * 1.4
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
                }
                InfoRow {
                    Layout.fillWidth: true
                    text: qsTr("示例数据 · 仅验证组件与窗口行为，不执行文件操作。")
                }
                Rectangle {
                    Layout.fillWidth: true
                    implicitHeight: destinationColumn.implicitHeight + Theme.gutter * 2
                    color: Theme.canvas
                    border.color: Theme.border
                    radius: Theme.panelRadius
                    ColumnLayout {
                        id: destinationColumn
                        anchors.fill: parent
                        anchors.margins: Theme.gutter
                        spacing: Theme.gap
                        Text {
                            text: qsTr("解压到")
                            font.pixelSize: Theme.bodySize
                            font.bold: true
                            color: Theme.text
                        }
                        Text {
                            Layout.fillWidth: true
                            text: root.destination
                            textFormat: Text.PlainText
                            font.pixelSize: Theme.bodySize
                            color: Theme.textSecondary
                            wrapMode: Text.WrapAnywhere
                        }
                    }
                }
                Rectangle {
                    Layout.fillWidth: true
                    implicitHeight: progressColumn.implicitHeight + Theme.gutter * 2
                    color: Theme.canvas
                    border.color: Theme.border
                    radius: Theme.panelRadius
                    ColumnLayout {
                        id: progressColumn
                        anchors.fill: parent
                        anchors.margins: Theme.gutter
                        spacing: Theme.gap
                        RowLayout {
                            Layout.fillWidth: true
                            ProgressBar {
                                id: progress
                                Layout.fillWidth: true
                                value: Math.max(0, Math.min(1, root.fraction))
                                indeterminate: !root.totalKnown
                                Accessible.name: qsTr("解压进度")
                                background: Rectangle {
                                    implicitHeight: 14
                                    radius: 7
                                    color: Theme.border
                                }
                                contentItem: Item {
                                    implicitHeight: 14
                                    Rectangle {
                                        width: progress.indeterminate ? 0 : parent.width * progress.visualPosition
                                        height: parent.height
                                        radius: 7
                                        color: Theme.accent
                                    }
                                }
                            }
                            Text {
                                objectName: "progressLabel"
                                text: root.totalKnown ? Math.floor(progress.value * 100) + "%" : qsTr("总量未知")
                                font.pixelSize: Theme.panelTitleSize
                                font.bold: true
                                color: Theme.accentInk
                            }
                        }
                        Text {
                            Layout.fillWidth: true
                            text: root.stopping ? qsTr("正在等待安全停止与清理") : qsTr("当前阶段：写入文件")
                            wrapMode: Text.Wrap
                            font.pixelSize: Theme.bodySize
                            color: Theme.textSecondary
                        }
                        Text {
                            Layout.fillWidth: true
                            text: qsTr("速度与剩余时间：等待真实任务数据")
                            wrapMode: Text.Wrap
                            font.pixelSize: Theme.secondarySize
                            color: Theme.textSecondary
                        }
                    }
                }
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: Theme.gap
                    Text {
                        text: qsTr("当前文件")
                        font.pixelSize: Theme.bodySize
                        font.bold: true
                        color: Theme.text
                    }
                    Text {
                        Layout.fillWidth: true
                        text: root.currentFile
                        textFormat: Text.PlainText
                        wrapMode: Text.WrapAnywhere
                        font.pixelSize: Theme.bodySize
                        color: Theme.textSecondary
                    }
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
                color: Theme.border
            }
            ColumnLayout {
                Layout.fillWidth: true
                Layout.margins: Theme.gutter
                spacing: Theme.gap
                InfoRow {
                    id: closingHint
                    Layout.fillWidth: true
                    text: qsTr("关闭此窗口后，任务仍会继续。")
                }
                RowLayout {
                    Layout.alignment: Qt.AlignRight
                    spacing: Theme.gap
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
