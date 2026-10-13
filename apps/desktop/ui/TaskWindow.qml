pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

OperationWindow {
    id: root
    objectName: "taskWindow"
    width: Theme.operationProgressWidth
    requestedHeight: Math.min(Theme.operationProgressClientHeight, Screen.desktopAvailableHeight - 80)
    property bool sampleData: false
    title: sampleData ? qsTr("解压任务 · 示例数据 — xdzip") : qsTr("解压任务 — xdzip")
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
    readonly property bool terminal: ["succeeded", "failed", "cancelled"].includes(taskStatus)
    readonly property bool largeType: Theme.fontScale > 1.35
    signal cancelRequested()
    signal openOutputRequested()
    signal conflictRequested()
    readonly property alias timingHint: timingHint
    readonly property alias closingHint: footer.hint
    readonly property string heading: taskStatus === "succeeded" ? qsTr("解压完成")
        : taskStatus === "failed" ? qsTr("解压未完成") : taskStatus === "cancelled" ? qsTr("任务已取消")
        : taskStatus === "waiting" ? qsTr("等待处理重名文件") : stopping ? qsTr("正在停止")
        : taskStatus === "queued" ? qsTr("任务已排队") : qsTr("正在解压")
    ColumnLayout {
        anchors.fill: parent; spacing: 0
        ScrollView {
            id: scroll
            objectName: "taskScroll"
            Layout.fillWidth: true; Layout.fillHeight: true
            contentWidth: availableWidth; clip: true
            ColumnLayout {
                width: scroll.availableWidth; spacing: 0
                RowLayout {
                    Layout.fillWidth: true
                    Layout.leftMargin: Theme.operationProgressContentInset; Layout.rightMargin: Theme.operationProgressContentInset
                    Layout.topMargin: 20; Layout.bottomMargin: 16; spacing: 20
                    OperationHero {
                        source: "qrc:/xdzip/assets/" + (root.terminal ? root.taskStatus === "succeeded" ? "success-large" : root.taskStatus === "cancelled" ? "cancelled-large" : "partial-warning" : "document-download") + ".png"
                        Layout.preferredWidth: canvasWidth; Layout.preferredHeight: canvasHeight
                    }
                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 7
                        Text { Layout.fillWidth: true; text: root.heading; font.pixelSize: Theme.operationTitleSize; lineHeight: Math.round(Theme.operationTitleSize * 1.35); lineHeightMode: Text.FixedHeight; font.bold: true; color: Theme.text; wrapMode: Text.Wrap }
                        Text { Layout.fillWidth: true; text: root.archiveName; textFormat: Text.PlainText; font.pixelSize: 17 * Theme.fontScale; lineHeight: Math.round(17 * Theme.fontScale * 1.5); lineHeightMode: Text.FixedHeight; color: Theme.textSecondary; elide: Text.ElideMiddle }
                    }
                }
                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.leftMargin: Theme.operationProgressContentInset; Layout.rightMargin: Theme.operationProgressContentInset
                    Layout.bottomMargin: 12; spacing: 12
                    TaskPathCard { Layout.fillWidth: true; heading: qsTr("解压到"); value: root.destination; iconSource: "qrc:/xdzip/assets/folder.png" }
                    Rectangle {
                        Layout.fillWidth: true; implicitHeight: progressColumn.implicitHeight + 38
                        color: Theme.taskPanel; border.color: Theme.taskBorder; radius: Theme.controlRadius
                        ColumnLayout {
                            id: progressColumn
                            anchors.fill: parent; anchors.margins: 19; spacing: 11
                            RowLayout {
                                Layout.fillWidth: true; spacing: 17
                                ProgressBar {
                                    id: progress
                                    Layout.fillWidth: true
                                    value: Number.isFinite(root.fraction) ? Math.max(0, Math.min(1, root.fraction)) : 0
                                    indeterminate: !root.totalKnown
                                    Accessible.name: qsTr("解压进度")
                                    background: Rectangle { implicitHeight: Theme.operationProgressTrackHeight; radius: height / 2; color: Theme.taskTrack }
                                    contentItem: Item {
                                        implicitHeight: Theme.operationProgressTrackHeight
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
                                Text { objectName: "progressLabel"; text: root.totalKnown ? Math.floor(progress.value * 100) + "%" : qsTr("总量未知"); font.pixelSize: Theme.operationTitleSize; font.bold: true; color: Theme.accentInk }
                            }
                            GridLayout {
                                Layout.fillWidth: true; columns: root.largeType ? 1 : 2; rowSpacing: 8
                                Text { Layout.fillWidth: true; objectName: "processedLabel"; text: root.processedSummary; textFormat: Text.PlainText; font.pixelSize: Theme.operationBodySize; color: Theme.textSecondary; wrapMode: Text.Wrap }
                                Text { text: root.fileCountSummary; textFormat: Text.PlainText; font.pixelSize: Theme.operationBodySize; color: Theme.textSecondary }
                            }
                            Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Theme.taskBorder }
                            GridLayout {
                                Layout.fillWidth: true; columns: root.largeType ? 1 : 5; columnSpacing: 18; rowSpacing: 12
                                TaskMetric { Layout.fillWidth: true; Layout.preferredWidth: 1; heading: qsTr("平均写入"); value: root.stopping ? "—" : root.speedSummary; labelSize: Theme.operationMetricLabelSize; valueSize: Theme.operationMetricValueSize }
                                Rectangle { visible: !root.largeType; Layout.fillHeight: true; implicitWidth: 1; color: Theme.taskBorder }
                                TaskMetric { Layout.fillWidth: true; Layout.preferredWidth: 1; heading: qsTr("预计剩余"); value: root.stopping || !root.totalKnown ? "—" : root.remainingSummary; labelSize: Theme.operationMetricLabelSize; valueSize: Theme.operationMetricValueSize }
                                Rectangle { visible: !root.largeType; Layout.fillHeight: true; implicitWidth: 1; color: Theme.taskBorder }
                                TaskMetric { Layout.fillWidth: true; Layout.preferredWidth: 1; heading: qsTr("当前阶段"); value: root.stopping ? qsTr("停止与清理") : root.phaseSummary; labelSize: Theme.operationMetricLabelSize; valueSize: Theme.operationMetricValueSize }
                            }
                        }
                    }
                    TaskPathCard {
                        Layout.fillWidth: true; objectName: "currentFileCard"
                        heading: root.terminal ? qsTr("结果") : qsTr("当前文件")
                        value: root.outcomeUncertain ? qsTr("请检查目标目录中的已提交文件。") : root.terminal ? root.resultSummary : root.currentFile
                        iconSource: root.terminal ? "qrc:/xdzip/assets/" + (root.taskStatus === "succeeded" ? "success-large" : root.taskStatus === "cancelled" ? "cancelled-large" : "partial-warning") + ".png" : AssetLibrary.fileIcon(root.currentFile, root.currentFile.endsWith("/"), false)
                    }
                    Text { Layout.fillWidth: true; visible: !!root.errorSummary; text: root.errorSummary; textFormat: Text.PlainText; color: Theme.menuDanger; wrapMode: Text.Wrap; font.pixelSize: Theme.operationNoteSize }
                    Text { Layout.fillWidth: true; visible: root.cleanupWarning; text: qsTr("部分暂存数据未能清理，已保留恢复材料，请检查目标目录。"); color: Theme.menuDanger; wrapMode: Text.Wrap; font.pixelSize: Theme.operationNoteSize }
                    InfoRow { id: timingHint; Layout.fillWidth: true; text: qsTr("实际耗时取决于文件与磁盘状态。"); labelSize: Theme.operationMetricLabelSize; lineFactor: 1.4; iconSize: Theme.operationFooterIconSize }
                }
            }
        }
        OperationFooter {
            id: footer
            Layout.fillWidth: true
            note: root.terminal ? qsTr("已提交的文件保留在目标目录。") : qsTr("关闭此窗口后，任务仍会继续。")
            AppButton { text: root.terminal ? qsTr("关闭") : qsTr("返回浏览"); labelSize: Theme.operationFooterFontSize; baseHeight: 38; cornerRadius: 6; horizontalInset: 17; minimumWidth: 119; onClicked: root.close() }
            AppButton { visible: root.taskStatus === "waiting"; text: qsTr("处理重名文件"); labelSize: Theme.operationFooterFontSize; baseHeight: 38; cornerRadius: 6; horizontalInset: 17; minimumWidth: 119; primary: true; onClicked: root.conflictRequested() }
            AppButton { visible: root.terminal; text: qsTr("打开目标目录"); labelSize: Theme.operationFooterFontSize; baseHeight: 38; cornerRadius: 6; horizontalInset: 17; minimumWidth: 119; onClicked: root.openOutputRequested() }
            AppButton { objectName: "cancelButton"; visible: !root.terminal; text: root.stopping ? qsTr("正在停止…") : qsTr("取消任务"); labelSize: Theme.operationFooterFontSize; baseHeight: 38; cornerRadius: 6; horizontalInset: 17; minimumWidth: 119; enabled: !root.stopping; onClicked: root.cancelRequested() }
        }
    }
}
