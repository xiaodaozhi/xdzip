import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ApplicationWindow {
    id: root
    width: 1040
    height: 720
    minimumWidth: 800
    minimumHeight: 600
    visible: true
    title: qsTr("xdzip · 组件开发预览")
    color: Theme.canvas
    property bool stopping: false
    property bool totalKnown: true

    ColumnLayout {
        anchors.centerIn: parent
        width: Math.min(parent.width - Theme.gutter * 4, 560)
        spacing: Theme.gutter
        Text {
            text: "xdzip"
            font.pixelSize: Theme.panelTitleSize * 2
            font.bold: true
            color: Theme.text
        }
        Text {
            Layout.fillWidth: true
            text: qsTr("组件开发预览")
            font.pixelSize: Theme.panelTitleSize
            color: Theme.text
        }
        Text {
            Layout.fillWidth: true
            text: qsTr("当前版本用于检查独立任务窗口、主题与文字缩放。示例不会打开归档或读写文件。")
            wrapMode: Text.Wrap
            font.pixelSize: Theme.bodySize
            color: Theme.textSecondary
        }
        AppButton {
            text: qsTr("打开任务窗口")
            primary: true
            onClicked: task.present()
        }
        AppButton {
            text: Theme.dark ? qsTr("切换浅色") : qsTr("切换深色")
            onClicked: Theme.dark = !Theme.dark
        }
        AppButton {
            text: Theme.fontScale === 1 ? qsTr("文字放大至 200%") : qsTr("恢复文字大小")
            onClicked: Theme.fontScale = Theme.fontScale === 1 ? 2 : 1
        }
        AppButton {
            text: root.totalKnown ? qsTr("检查未知总量") : qsTr("恢复已知总量")
            onClicked: root.totalKnown = !root.totalKnown
        }
        AppButton {
            text: qsTr("重置示例状态")
            onClicked: root.stopping = false
        }
    }
    TaskWindow {
        id: task
        owner: root
        archiveName: qsTr("北境旅行.zip")
        destination: qsTr("下载 / 北境旅行")
        currentFile: qsTr("旅途瞬间-08.jpg")
        fraction: 0.6
        totalKnown: root.totalKnown
        stopping: root.stopping
        onCancelRequested: root.stopping = true
    }
}
