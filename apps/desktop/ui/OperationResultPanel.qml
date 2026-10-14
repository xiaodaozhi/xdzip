pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ColumnLayout {
    id: root
    required property var task
    property bool expanded: false
    readonly property var entries: task.details || []
    readonly property var failures: entries.filter(e => e.status === "unfinished")
    readonly property int remaining: Math.max(0, Number(task.files || 0) - Number(task.committedFiles || 0) - Number(task.skippedFiles || 0))
    readonly property string savedCaption: {
        const total = Number(task.creating ? task.outputParts || 0 : task.committedFiles || 0);
        const saved = entries.filter(e => e.status === "completed");
        if (!saved.length) return qsTr("尚未保存文件");
        const names = saved.slice(0, 3).map(e => e.name.replace(/.*\//, "")).join("、");
        const bytes = Number(task.creating ? task.writtenBytes || 0 : task.committedBytes || 0);
        const amount = bytes >= 1000000 ? (bytes / 1000000).toFixed(1) + " MB" : bytes >= 1000 ? (bytes / 1000).toFixed(1) + " KB" : bytes + " B";
        return qsTr("已保存：%1%2（%3）").arg(names).arg(total > 3 ? qsTr(" 等 %1 个文件").arg(total) : "").arg(amount);
    }
    spacing: Theme.operationResultCardGap
    Rectangle {
        objectName: "resultCounts"
        Layout.fillWidth: true; implicitHeight: Math.max(80, counts.implicitHeight + 24)
        color: Theme.operationCard; border.color: Theme.operationCardBorder; radius: Theme.operationResultCardRadius
        GridLayout {
            id: counts
            columns: Theme.fontScale > 1.35 ? 1 : 3
            columnSpacing: 0; rowSpacing: Theme.operationResultCardGap
            anchors.fill: parent; anchors.margins: Theme.operationResultCardInset
            Repeater {
                model: [
                    {label: root.task.creating ? qsTr("已保存归档／分卷") : qsTr("成功"), value: root.task.creating ? root.task.outputParts || 0 : root.task.committedFiles || 0, icon: "status-partial-success"},
                    {label: qsTr("跳过"), value: root.task.skippedFiles || 0, icon: "status-skipped"},
                    {label: qsTr("未完成"), value: root.task.creating ? root.task.status === "succeeded" ? 0 : 1 : root.remaining, icon: "status-warning"}
                ]
                delegate: Item {
                    id: metric
                    required property var modelData
                    required property int index
                    Layout.fillWidth: true; implicitHeight: stat.implicitHeight
                    ColumnLayout {
                        id: stat
                        anchors.horizontalCenter: parent.horizontalCenter; spacing: 5
                        RowLayout {
                            Layout.alignment: Qt.AlignHCenter; spacing: 8
                            Image { source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + metric.modelData.icon + ".png"; Layout.preferredWidth: 24; Layout.preferredHeight: 24 }
                            Text { text: metric.modelData.label; Layout.maximumWidth: Math.max(60, metric.width - 40); wrapMode: Text.Wrap; font.pixelSize: Math.round(Theme.operationResultBodySize); color: Theme.textSecondary }
                        }
                        Text { Layout.alignment: Qt.AlignHCenter; Layout.preferredHeight: Math.round(Theme.operationResultCountSize); text: metric.modelData.value; font.pixelSize: Math.round(Theme.operationResultCountSize); verticalAlignment: Text.AlignVCenter; lineHeight: Math.round(Theme.operationResultCountSize); lineHeightMode: Text.FixedHeight; font.bold: true; color: Theme.text }
                    }
                    Rectangle { visible: Theme.fontScale <= 1.35 && metric.index < 2; anchors.right: parent.right; height: parent.height; width: 1; color: Theme.operationCardBorder }
                }
            }
        }
    }
    Rectangle {
        objectName: "resultFailures"
        visible: root.failures.length > 0 || Boolean(root.task.error)
        Layout.fillWidth: true; implicitHeight: failed.implicitHeight + 24
        color: Theme.operationCard; border.color: Theme.operationCardBorder; radius: Theme.operationResultCardRadius
        ColumnLayout {
            id: failed
            anchors.fill: parent; anchors.margins: Theme.operationResultCardInset; spacing: 8
            Text { text: qsTr("未完成的文件"); font.pixelSize: Math.round(Theme.operationResultSectionSize); font.bold: true; color: Theme.text }
            Repeater { model: root.failures; delegate: ResultFileRow { required property var modelData; Layout.fillWidth: true; entry: modelData; showReason: true } }
            Rectangle {
                visible: Boolean(root.task.error); Layout.fillWidth: true
                implicitHeight: message.implicitHeight + 16
                color: Theme.resultWarningSurface; radius: 5
                InfoRow {
                    id: message
                    anchors.left: parent.left; anchors.right: parent.right; anchors.margins: 8; anchors.verticalCenter: parent.verticalCenter
                    iconSource: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "status-warning.png"
                    text: root.task.error || ""; labelSize: Math.round(Theme.operationResultNoteSize); iconSize: 20
                }
            }
        }
    }
    Rectangle {
        objectName: "resultDestination"
        Layout.fillWidth: true; implicitHeight: destination.implicitHeight + 24
        color: Theme.operationCard; border.color: Theme.operationCardBorder; radius: Theme.operationResultCardRadius
        ColumnLayout {
            id: destination
            anchors.fill: parent; anchors.margins: Theme.operationResultCardInset; spacing: 8
            Text { text: qsTr("目标位置"); font.pixelSize: Math.round(Theme.operationResultSectionSize); font.bold: true; color: Theme.text }
            Rectangle {
                Layout.fillWidth: true; implicitHeight: location.implicitHeight + 20
                color: Theme.surface; border.color: Theme.operationCardBorder; radius: 5
                RowLayout {
                    id: location
                    anchors.fill: parent; anchors.margins: 10; spacing: 12
                    Image { source: AssetLibrary.fileIcon("", true, false); Layout.preferredWidth: 28; Layout.preferredHeight: 28 }
                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 3
                        Text { Layout.fillWidth: true; text: root.task.target || ""; textFormat: Text.PlainText; elide: Text.ElideMiddle; font.pixelSize: Math.round(Theme.operationResultBodySize); color: Theme.text }
                        Text {
                            Layout.fillWidth: true; wrapMode: Text.Wrap
                            text: root.task.externalReader ? root.savedCaption + "\n" + qsTr("AES 或分卷文件可使用 7-Zip 等兼容工具打开。") : root.savedCaption
                            font.pixelSize: Math.round(Theme.operationResultNoteSize); color: Theme.textSecondary
                        }
                    }
                }
            }
        }
    }
    Rectangle {
        objectName: "resultDetails"
        Layout.fillWidth: true; implicitHeight: detailColumn.implicitHeight + 16
        color: Theme.operationCard; border.color: Theme.operationCardBorder; radius: Theme.operationResultCardRadius
        ColumnLayout {
            id: detailColumn
            anchors.fill: parent; anchors.margins: 8; spacing: 4
            ResultDisclosureButton {
                objectName: "resultDetailsToggle"
                Layout.fillWidth: true
                recordCount: root.entries.length
                checked: root.expanded
                onToggled: root.expanded = checked
            }
            ScrollView {
                id: detailsScroll
                visible: root.expanded; Layout.fillWidth: true
                Layout.preferredHeight: Math.min(Theme.operationResultDetailsHeight, rows.implicitHeight)
                contentWidth: availableWidth; clip: true
                ColumnLayout {
                    id: rows
                    width: detailsScroll.availableWidth; spacing: 0
                    Repeater { model: root.entries; delegate: ResultFileRow { required property var modelData; Layout.fillWidth: true; entry: modelData } }
                }
            }
            Text {
                visible: root.expanded && (!root.entries.length || Number(root.task.omittedDetails) > 0)
                Layout.fillWidth: true; wrapMode: Text.Wrap
                text: root.entries.length ? qsTr("显示最近 %1 条记录，较早的 %2 条未保留；统计包含全部文件。").arg(root.entries.length).arg(root.task.omittedDetails) : qsTr("尚未产生文件处理记录。")
                font.pixelSize: Math.round(Theme.operationResultNoteSize); color: Theme.textSecondary
            }
        }
    }
    Text {
        visible: Boolean(root.task.cleanupWarning || root.task.outcomeUncertain)
        Layout.fillWidth: true; wrapMode: Text.Wrap
        text: root.task.outcomeUncertain ? qsTr("无法确认全部提交结果，请检查目标目录及恢复材料。") : qsTr("部分暂存数据未能清理，恢复材料已保留。")
        font.pixelSize: Math.round(Theme.operationResultNoteSize); color: Theme.menuDanger
    }
}
