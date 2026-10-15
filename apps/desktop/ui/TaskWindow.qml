pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

OperationWindow {
    id: root
    objectName: "taskWindow"
    readonly property real density: Theme.operationProgressDensity
    function unit(value: real): real { return value * density; }
    width: Theme.operationProgressWidth
    requestedHeight: Math.min(Math.ceil(taskContent.implicitHeight + footer.implicitHeight), Theme.operationResultClientHeight, Screen.desktopAvailableHeight - 80)
    property bool creating: false
    property string operationKind: creating ? "create" : "extract"
    readonly property bool checking: operationKind === "check"
    property bool sampleData: false
    title: checking ? sampleData ? qsTr("完整性检查，界面预览") : qsTr("完整性检查") : sampleData ? qsTr("任务，示例数据") : terminal ? creating ? qsTr("压缩结果") : qsTr("解压结果") : creating ? qsTr("压缩任务") : qsTr("解压任务")
    allowMinimize: true
    required property string archiveName
    required property string destination
    required property string currentFile
    required property real fraction
    required property bool totalKnown
    required property bool stopping
    property string taskStatus: stopping ? "cancelling" : "running"
    property string errorSummary: ""
    property bool cleanupWarning: false
    property bool outcomeUncertain: false
    property string resultSummary: ""
    property string processedSummary: qsTr("等待任务数据")
    property string fileCountSummary: qsTr("数量未知")
    property string speedSummary: qsTr("等待估算")
    property string remainingSummary: qsTr("等待估算")
    property string phaseSummary: qsTr("准备中")
    property var resultTask: ({})
    readonly property bool terminal: ["succeeded", "failed", "cancelled"].includes(taskStatus)
    readonly property bool largeType: Theme.fontScale > 1.35
    signal cancelRequested()
    signal openOutputRequested()
    signal conflictRequested()
    Connections {
        target: root
        function onActiveFocusItemChanged() { root.revealFocus(scroll); }
    }
    readonly property alias timingHint: timingHint
    readonly property alias closingHint: footer.hint
    readonly property string heading: checking ? taskStatus === "succeeded" ? qsTr("检查完成") : taskStatus === "failed" ? qsTr("完整性检查未完成") : taskStatus === "cancelled" ? qsTr("检查已取消") : stopping ? qsTr("正在停止检查") : taskStatus === "queued" ? qsTr("检查已排队") : qsTr("正在检查归档") : taskStatus === "succeeded" ? (creating ? qsTr("压缩完成") : qsTr("解压完成"))
        : taskStatus === "failed" ? (creating ? qsTr("压缩未完成") : Number(resultTask.committedFiles || 0) > 0 ? qsTr("部分文件未完成") : qsTr("解压未完成")) : taskStatus === "cancelled" ? qsTr("任务已取消")
        : taskStatus === "waiting" ? qsTr("等待处理重名文件") : stopping ? qsTr("正在停止")
        : taskStatus === "queued" ? qsTr("任务已排队") : taskStatus === "verifying" ? qsTr("正在校验归档") : (creating ? qsTr("正在压缩") : qsTr("正在解压"))
    ColumnLayout {
        anchors.fill: parent; spacing: 0
        ScrollView {
            id: scroll
            objectName: "taskScroll"
            Layout.fillWidth: true; Layout.fillHeight: true
            contentWidth: availableWidth; clip: true
            ColumnLayout {
                id: taskContent
                objectName: "taskBodyContent"
                width: scroll.availableWidth; spacing: 0
                RowLayout {
                    Layout.fillWidth: true
                    Layout.leftMargin: root.unit(Theme.operationProgressContentInset); Layout.rightMargin: root.unit(Theme.operationProgressContentInset)
                    Layout.topMargin: root.terminal ? 20 : root.unit(20); Layout.bottomMargin: root.terminal ? 16 : root.unit(16); spacing: root.terminal ? 20 : root.unit(20)
                    OperationHero {
                        density: root.density
                        artworkWidth: root.terminal ? 72 : 66 * root.density
                        artworkHeight: root.terminal ? 72 : 74 * root.density
                        source: "qrc:/xdzip/assets/" + (root.terminal ? root.taskStatus === "succeeded" ? "success-large" : root.taskStatus === "cancelled" ? "cancelled-large" : "document-warning" : root.checking ? (Theme.dark ? "dark/document-check" : "document-check") : root.creating ? (Theme.dark ? "dark/archive-add-large" : "archive-add-large") : "document-download") + ".png"
                        Layout.preferredWidth: canvasWidth; Layout.preferredHeight: canvasHeight
                    }
                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 7
                        Text { Layout.fillWidth: true; Layout.preferredHeight: root.terminal ? implicitHeight : lineHeight; text: root.heading; font.pixelSize: Math.round(root.unit(Theme.operationTitleSize)); lineHeight: Math.round(Math.round(root.unit(Theme.operationTitleSize)) * 1.35); lineHeightMode: Text.FixedHeight; font.bold: true; color: Theme.text; wrapMode: Text.Wrap }
                        Text { Layout.fillWidth: true; Layout.preferredHeight: root.terminal ? implicitHeight : lineHeight; text: root.checking ? root.archiveName : root.terminal ? root.creating ? root.taskStatus === "succeeded" ? qsTr("归档已完成校验并保存。") : root.outcomeUncertain ? qsTr("请检查目标目录及恢复材料。") : qsTr("临时归档未发布。") : qsTr("已保留成功解压的文件。") : root.archiveName; textFormat: Text.PlainText; font.pixelSize: Math.round(root.unit(17 * Theme.fontScale)); lineHeight: Math.round(Math.round(root.unit(17 * Theme.fontScale)) * 1.5); lineHeightMode: Text.FixedHeight; color: Theme.textSecondary; elide: Text.ElideMiddle }
                    }
                }
                ColumnLayout {
                    visible: !root.terminal || root.checking
                    Layout.fillWidth: true
                    Layout.leftMargin: root.unit(Theme.operationProgressContentInset); Layout.rightMargin: root.unit(Theme.operationProgressContentInset)
                    Layout.bottomMargin: root.unit(12); spacing: root.unit(12)
                    TaskPathCard { density: root.density; Layout.fillWidth: true; heading: root.checking ? qsTr("检查对象") : root.creating ? qsTr("保存到") : qsTr("解压到"); value: root.destination; iconSource: "qrc:/xdzip/assets/folder.png" }
                    Rectangle {
                        Layout.fillWidth: true; implicitHeight: progressColumn.implicitHeight + root.unit(38)
                        color: Theme.taskPanel; border.color: Theme.taskBorder; radius: Theme.controlRadius
                        ColumnLayout {
                            id: progressColumn
                            anchors.fill: parent; anchors.margins: root.unit(19); spacing: root.unit(11)
                            RowLayout {
                                Layout.fillWidth: true; spacing: root.unit(17)
                                ProgressBar {
                                    id: progress
                                    Layout.fillWidth: true
                                    value: Number.isFinite(root.fraction) ? Math.max(0, Math.min(1, root.fraction)) : 0
                                    indeterminate: !root.totalKnown
                                    Accessible.name: root.checking ? qsTr("完整性检查进度") : root.creating ? qsTr("压缩进度") : qsTr("解压进度")
                                    background: Rectangle { implicitHeight: root.unit(Theme.operationProgressTrackHeight); radius: height / 2; color: Theme.taskTrack }
                                    contentItem: Item {
                                        implicitHeight: root.unit(Theme.operationProgressTrackHeight)
                                        Rectangle {
                                            objectName: "progressFill"
                                            width: progress.indeterminate ? 0 : parent.width * progress.visualPosition
                                            height: parent.height; radius: height / 2
                                            gradient: Gradient {
                                                orientation: Gradient.Horizontal
                                                GradientStop { position: 0; color: Theme.progressStart }
                                                GradientStop { position: 1; color: Theme.progressEnd }
                                            }
                                        }
                                    }
                                }
                                Text { objectName: "progressLabel"; Layout.preferredHeight: lineHeight; lineHeight: Math.round(root.unit(Theme.operationProgressPercentageLineHeight) * Theme.fontScale); lineHeightMode: Text.FixedHeight; verticalAlignment: Text.AlignVCenter; text: root.totalKnown ? Math.floor(progress.value * 100) + "%" : qsTr("总量未知"); font.pixelSize: Math.round(root.unit(Theme.operationTitleSize)); font.bold: true; color: Theme.accentInk }
                            }
                            GridLayout {
                                Layout.fillWidth: true; columns: root.largeType ? 1 : 2; rowSpacing: root.unit(8)
                                Text { Layout.fillWidth: true; Layout.preferredHeight: Math.max(1, lineCount) * lineHeight; lineHeight: Math.round(root.unit(Theme.operationProgressBodyLineHeight) * Theme.fontScale); lineHeightMode: Text.FixedHeight; objectName: "processedLabel"; text: root.processedSummary; textFormat: Text.PlainText; font.pixelSize: Math.round(root.unit(Theme.operationBodySize)); color: Theme.textSecondary; wrapMode: Text.Wrap }
                                Text { Layout.preferredHeight: lineHeight; lineHeight: Math.round(root.unit(Theme.operationProgressBodyLineHeight) * Theme.fontScale); lineHeightMode: Text.FixedHeight; text: root.fileCountSummary; textFormat: Text.PlainText; font.pixelSize: Math.round(root.unit(Theme.operationBodySize)); color: Theme.textSecondary }
                            }
                            Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Theme.taskBorder }
                            GridLayout {
                                Layout.fillWidth: true; columns: root.largeType ? 1 : 5; columnSpacing: root.unit(18); rowSpacing: root.unit(12)
                                TaskMetric { density: root.density; Layout.fillWidth: true; Layout.preferredWidth: 1; heading: root.checking ? qsTr("平均读取") : qsTr("平均写入"); value: root.stopping ? "—" : root.speedSummary; labelSize: Math.round(root.unit(Theme.operationMetricLabelSize)); valueSize: Math.round(root.unit(Theme.operationMetricValueSize)) }
                                Rectangle { visible: !root.largeType; Layout.fillHeight: true; implicitWidth: 1; color: Theme.taskBorder }
                                TaskMetric { density: root.density; Layout.fillWidth: true; Layout.preferredWidth: 1; heading: qsTr("预计剩余"); value: root.stopping || !root.totalKnown ? "—" : root.remainingSummary; labelSize: Math.round(root.unit(Theme.operationMetricLabelSize)); valueSize: Math.round(root.unit(Theme.operationMetricValueSize)) }
                                Rectangle { visible: !root.largeType; Layout.fillHeight: true; implicitWidth: 1; color: Theme.taskBorder }
                                TaskMetric { density: root.density; Layout.fillWidth: true; Layout.preferredWidth: 1; heading: qsTr("当前阶段"); value: root.stopping ? qsTr("停止与清理") : root.phaseSummary; labelSize: Math.round(root.unit(Theme.operationMetricLabelSize)); valueSize: Math.round(root.unit(Theme.operationMetricValueSize)) }
                            }
                        }
                    }
                    TaskPathCard { density: root.density;
                        Layout.fillWidth: true; objectName: "currentFileCard"
                        heading: root.terminal ? root.checking ? qsTr("检查结果") : qsTr("结果") : qsTr("当前文件")
                        value: root.checking && root.terminal ? root.resultSummary : root.outcomeUncertain ? qsTr("请检查目标目录中的已提交文件。") : root.terminal ? root.resultSummary : root.currentFile
                        iconSource: root.terminal ? "qrc:/xdzip/assets/" + (root.taskStatus === "succeeded" ? "success-large" : root.taskStatus === "cancelled" ? "cancelled-large" : "document-warning") + ".png" : AssetLibrary.fileIcon(root.currentFile, root.currentFile.endsWith("/"), false)
                    }
                    Text { Layout.fillWidth: true; visible: !!root.errorSummary; text: root.errorSummary; textFormat: Text.PlainText; color: Theme.menuDanger; wrapMode: Text.Wrap; font.pixelSize: Math.round(root.unit(Theme.operationNoteSize)) }
                    Text { Layout.fillWidth: true; visible: root.cleanupWarning; text: qsTr("部分暂存数据未能清理，已保留恢复材料，请检查目标目录。"); color: Theme.menuDanger; wrapMode: Text.Wrap; font.pixelSize: Math.round(root.unit(Theme.operationNoteSize)) }
                    InfoRow { id: timingHint; Layout.fillWidth: true; text: qsTr("实际耗时取决于文件与磁盘状态。"); labelSize: Math.round(root.unit(Theme.operationMetricLabelSize)); lineFactor: 1.4; iconSize: Theme.operationFooterIconSize }
                }
                OperationResultPanel {
                    objectName: "taskResultPanel"
                    visible: root.terminal && !root.checking
                    Layout.fillWidth: true
                    Layout.leftMargin: root.unit(Theme.operationProgressContentInset)
                    Layout.rightMargin: root.unit(Theme.operationProgressContentInset)
                    Layout.bottomMargin: 12
                    task: root.resultTask
                }
            }
        }
        OperationFooter {
            id: footer
            objectName: "taskFooter"
            contentInset: root.unit(Theme.operationProgressContentInset)
            verticalInset: root.unit(12); minimumFooterHeight: root.unit(Theme.operationFooterHeight)
            Layout.fillWidth: true
            note: root.checking ? root.sampleData ? qsTr("界面预览，不执行实际校验。") : root.terminal ? qsTr("检查已结束，未写出文件。") : qsTr("关闭此窗口后，任务仍会继续。") : root.creating && root.terminal ? (root.taskStatus === "succeeded" ? qsTr("归档已完整保存并校验。") : root.outcomeUncertain ? qsTr("请检查目标目录及恢复材料。") : root.cleanupWarning ? qsTr("暂存清理未完成，恢复材料已保留。") : qsTr("已有归档保留，暂存数据已处理。")) : root.terminal ? qsTr("已提交的文件保留在目标目录。") : qsTr("关闭此窗口后，任务仍会继续。")
            OperationFooterButton { text: root.terminal ? qsTr("关闭") : qsTr("返回浏览"); onClicked: root.close() }
            OperationFooterButton { visible: root.taskStatus === "waiting"; text: qsTr("处理重名文件"); primary: true; onClicked: root.conflictRequested() }
            OperationFooterButton { visible: root.terminal && !root.checking; text: qsTr("打开目标文件夹"); onClicked: root.openOutputRequested() }
            OperationFooterButton { objectName: "cancelButton"; visible: !root.terminal; text: root.stopping ? qsTr("正在停止…") : root.checking ? qsTr("取消检查") : qsTr("取消任务"); enabled: !root.stopping; onClicked: root.cancelRequested() }
        }
    }
}
