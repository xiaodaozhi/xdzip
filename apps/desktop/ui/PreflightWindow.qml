pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

OperationWindow {
    id: root
    objectName: "preflightWindow"
    title: qsTr("xdzip · 解压预检")
    property var report: ({})
    width: 700
    height: Math.min(620, Screen.desktopAvailableHeight - 80)
    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 18
        Text {
            text: root.report.error ? qsTr("预检未完成") : root.report.findingCount !== "0" ? qsTr("发现需要处理的条目") : qsTr("基础预检完成")
            font.pixelSize: Theme.panelTitleSize; font.bold: true; color: Theme.text
        }
        Text {
            Layout.fillWidth: true
            text: qsTr("检查本页选择，包含所选目录中的全部后代条目。不会创建或修改文件。")
            wrapMode: Text.Wrap; color: Theme.textSecondary; font.pixelSize: Theme.bodySize
        }
        Rectangle {
            Layout.fillWidth: true; implicitHeight: 92
            color: Theme.taskPanel; radius: 12; border.color: Theme.taskBorder
            RowLayout {
                anchors.fill: parent; anchors.margins: 18; spacing: 24
                Repeater {
                    model: [qsTr("展开后的记录"), qsTr("声明字节总量"), qsTr("发现的问题")]
                    ColumnLayout {
                        required property int index
                        required property string modelData
                        Layout.fillWidth: true
                        Text { text: parent.modelData; color: Theme.textSecondary; font.pixelSize: Theme.secondarySize }
                        Text {
                            text: [root.report.entries || "—", root.report.bytes === "unknown" ? qsTr("无法计算") : root.report.bytes || "—", root.report.findingCount || "—"][parent.index]
                            color: Theme.text; font.pixelSize: 18; font.bold: true
                        }
                    }
                }
            }
        }
        ScrollView {
            Layout.fillWidth: true; Layout.fillHeight: true
            contentWidth: availableWidth
            ColumnLayout {
                width: parent.width; spacing: 12
                Text {
                    Layout.fillWidth: true
                    visible: !!root.report.error
                    text: root.report.error || ""
                    wrapMode: Text.Wrap; color: Theme.text; font.pixelSize: Theme.bodySize
                }
                Repeater {
                    model: root.report.findings || []
                    Text {
                        required property string modelData
                        Layout.fillWidth: true
                        text: modelData; textFormat: Text.PlainText; wrapMode: Text.Wrap
                        color: Theme.text; font.pixelSize: Theme.bodySize
                    }
                }
                Text {
                    Layout.fillWidth: true
                    text: root.report.findingCount === "0" ? qsTr("未发现本阶段规则覆盖的路径或容量问题。") : qsTr("最多显示前 64 条问题；不会自动覆盖、改名或跳过。")
                    color: Theme.textSecondary; wrapMode: Text.Wrap; font.pixelSize: Theme.bodySize
                    visible: !root.report.error
                }
                Text {
                    Layout.fillWidth: true
                    visible: !!root.report.unicodeReview
                    text: qsTr("包含 Unicode 名称，仍需在目标文件系统检查名称等价与碰撞。")
                    wrapMode: Text.Wrap; color: Theme.textSecondary; font.pixelSize: Theme.bodySize
                }
            }
        }
        Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Theme.border }
        Text {
            Layout.fillWidth: true
            text: qsTr("尚未校验文件内容，也未检查目标磁盘。安全写入与解码链路完成前，不能开始解压。")
            wrapMode: Text.Wrap; color: Theme.textSecondary; font.pixelSize: Theme.bodySize
        }
        AppButton { Layout.alignment: Qt.AlignRight; text: qsTr("返回浏览"); primary: true; onClicked: root.close() }
    }
}
