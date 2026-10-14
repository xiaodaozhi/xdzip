pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts

OperationWindow {
    id: root
    objectName: "quitWindow"
    property bool stopping: false
    signal waitRequested()
    signal cancelAndQuitRequested()
    width: Theme.operationDefaultWidth
    requestedHeight: Theme.fontScale > 1.35 ? 540 : 270
    title: qsTr("退出 xdzip")
    ColumnLayout {
        anchors.fill: parent; spacing: 0
        ColumnLayout {
            Layout.fillWidth: true; Layout.fillHeight: true; Layout.margins: 24; spacing: 16
            Text { text: root.stopping ? qsTr("正在停止任务") : qsTr("仍有任务正在进行"); font.pixelSize: Theme.operationTitleSize; font.bold: true; color: Theme.text }
            Text { Layout.fillWidth: true; text: root.stopping ? qsTr("等待当前文件提交与暂存清理完成后退出。") : qsTr("可以等待任务完成，或取消未完成的工作后退出。已提交的文件会保留。"); font.pixelSize: Theme.operationBodySize; color: Theme.textSecondary; wrapMode: Text.Wrap }
        }
        OperationFooter {
            Layout.fillWidth: true
            OperationFooterButton { text: qsTr("返回应用"); enabled: !root.stopping; onClicked: root.close() }
            OperationFooterButton { text: qsTr("等待完成"); enabled: !root.stopping; onClicked: root.waitRequested() }
            OperationFooterButton { text: qsTr("取消并退出"); enabled: !root.stopping; primary: true; onClicked: root.cancelAndQuitRequested() }
        }
    }
}
