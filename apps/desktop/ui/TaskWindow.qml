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
    title: operationKind === "move" ? (terminal ? qsTrId("move.result") : qsTrId("move.progress")) : operationKind === "update" ? (terminal ? qsTrId("update.result") : qsTrId("update.progress")) : operationKind === "export" || operationKind === "export-copy" ? (terminal ? qsTrId("export.result") : qsTrId("export.preparing")) : checking ? sampleData ? qsTrId("ui.09fb2202dff9") : qsTrId("ui.06840dd516fe") : sampleData ? qsTrId("ui.cbd8ddd9b2b3") : terminal ? creating ? qsTrId("ui.ecabd6e05c84") : qsTrId("ui.7525c4c57b28") : creating ? qsTrId("ui.7d33a8f142f9") : qsTrId("ui.60483c3431b0")
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
    property string processedSummary: qsTrId("ui.730a0e3c817a")
    property string fileCountSummary: qsTrId("ui.0c98ac32e82f")
    property string speedSummary: qsTrId("ui.f4199761edc9")
    property string remainingSummary: qsTrId("ui.f4199761edc9")
    property string phaseSummary: qsTrId("ui.b53be09930f4")
    property var resultTask: ({})
    readonly property bool terminal: ["succeeded", "failed", "cancelled"].includes(taskStatus)
    readonly property bool largeType: Theme.fontScale > 1.35
    signal cancelRequested()
    signal openOutputRequested()
    signal conflictRequested()
    signal passwordRequested()
    Connections {
        target: root
        function onActiveFocusItemChanged() { root.revealFocus(scroll); }
    }
    readonly property alias timingHint: timingHint
    readonly property alias closingHint: footer.hint
    readonly property string heading: operationKind === "move" ? (terminal ? qsTrId("move.result") : qsTrId("move.progress")) : operationKind === "update" ? (terminal ? qsTrId("update.result") : qsTrId("update.progress")) : operationKind === "export" || operationKind === "export-copy" ? (terminal ? qsTrId("export.result") : qsTrId("export.preparing")) : checking ? taskStatus === "succeeded" ? qsTrId("ui.67ec0b18537f") : taskStatus === "failed" ? qsTrId("ui.b327e25dbc82") : taskStatus === "cancelled" ? qsTrId("ui.ce9bc029cc7e") : stopping ? qsTrId("ui.619dd70cb582") : taskStatus === "waiting_password" ? qsTrId("password.waiting") : taskStatus === "queued" ? qsTrId("ui.56bbef5868e7") : qsTrId("ui.2ef948f6a64e") : taskStatus === "succeeded" ? (creating ? qsTrId("ui.b58663721975") : qsTrId("ui.49e8128e1a96"))
        : taskStatus === "failed" ? (creating ? qsTrId("ui.b5926b78a037") : Number(resultTask.committedFiles || 0) > 0 ? qsTrId("ui.cf06124d3ffd") : qsTrId("ui.5bef0b169e84")) : taskStatus === "cancelled" ? qsTrId("ui.6f96c2add214")
        : taskStatus === "waiting" ? qsTrId("ui.00b405cb9dd9") : taskStatus === "waiting_password" ? qsTrId("password.waiting") : stopping ? qsTrId("ui.af48159f380f")
        : taskStatus === "queued" ? qsTrId("ui.402ce5b3351f") : taskStatus === "verifying" ? qsTrId("ui.ceecf5089636") : (creating ? qsTrId("ui.34ee864936c6") : qsTrId("ui.09eb6850c88c"))
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
                        Text { Layout.fillWidth: true; Layout.preferredHeight: root.terminal ? implicitHeight : lineHeight; text: root.checking ? root.archiveName : root.terminal ? root.creating ? root.taskStatus === "succeeded" ? qsTrId("ui.d7776832f17e") : root.outcomeUncertain ? qsTrId("ui.c96310851765") : qsTrId("ui.bd737a972601") : qsTrId("ui.d78e5fd4e099") : root.archiveName; textFormat: Text.PlainText; font.pixelSize: Math.round(root.unit(17 * Theme.fontScale)); lineHeight: Math.round(Math.round(root.unit(17 * Theme.fontScale)) * 1.5); lineHeightMode: Text.FixedHeight; color: Theme.textSecondary; elide: Text.ElideMiddle }
                    }
                }
                ColumnLayout {
                    visible: !root.terminal || root.checking
                    Layout.fillWidth: true
                    Layout.leftMargin: root.unit(Theme.operationProgressContentInset); Layout.rightMargin: root.unit(Theme.operationProgressContentInset)
                    Layout.bottomMargin: root.unit(12); spacing: root.unit(12)
                    TaskPathCard { density: root.density; Layout.fillWidth: true; heading: root.checking ? qsTrId("ui.c0dbe7153a05") : root.creating ? qsTrId("ui.e31cccd7bba0") : qsTrId("ui.8d7e2c47b34e"); value: root.destination; iconSource: "qrc:/xdzip/assets/folder.png" }
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
                                    Accessible.name: root.checking ? qsTrId("ui.928c432129ac") : root.creating ? qsTrId("ui.317cb00c1275") : qsTrId("ui.fcef101b1132")
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
                                Text { objectName: "progressLabel"; Layout.preferredHeight: lineHeight; lineHeight: Math.round(root.unit(Theme.operationProgressPercentageLineHeight) * Theme.fontScale); lineHeightMode: Text.FixedHeight; verticalAlignment: Text.AlignVCenter; text: root.totalKnown ? Math.floor(progress.value * 100) + "%" : qsTrId("ui.d80cf5192559"); font.pixelSize: Math.round(root.unit(Theme.operationTitleSize)); font.bold: true; color: Theme.accentInk }
                            }
                            GridLayout {
                                Layout.fillWidth: true; columns: root.largeType ? 1 : 2; rowSpacing: root.unit(8)
                                Text { Layout.fillWidth: true; Layout.preferredHeight: Math.max(1, lineCount) * lineHeight; lineHeight: Math.round(root.unit(Theme.operationProgressBodyLineHeight) * Theme.fontScale); lineHeightMode: Text.FixedHeight; objectName: "processedLabel"; text: root.processedSummary; textFormat: Text.PlainText; font.pixelSize: Math.round(root.unit(Theme.operationBodySize)); color: Theme.textSecondary; wrapMode: Text.Wrap }
                                Text { Layout.preferredHeight: lineHeight; lineHeight: Math.round(root.unit(Theme.operationProgressBodyLineHeight) * Theme.fontScale); lineHeightMode: Text.FixedHeight; text: root.fileCountSummary; textFormat: Text.PlainText; font.pixelSize: Math.round(root.unit(Theme.operationBodySize)); color: Theme.textSecondary }
                            }
                            Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Theme.taskBorder }
                            GridLayout {
                                Layout.fillWidth: true; columns: root.largeType ? 1 : 5; columnSpacing: root.unit(18); rowSpacing: root.unit(12)
                                TaskMetric { density: root.density; Layout.fillWidth: true; Layout.preferredWidth: 1; heading: root.checking ? qsTrId("ui.21e080cef726") : qsTrId("ui.725c2dd76eaa"); value: root.stopping ? "—" : root.speedSummary; labelSize: Math.round(root.unit(Theme.operationMetricLabelSize)); valueSize: Math.round(root.unit(Theme.operationMetricValueSize)) }
                                Rectangle { visible: !root.largeType; Layout.fillHeight: true; implicitWidth: 1; color: Theme.taskBorder }
                                TaskMetric { density: root.density; Layout.fillWidth: true; Layout.preferredWidth: 1; heading: qsTrId("ui.8db1c6ec1423"); value: root.stopping || !root.totalKnown ? "—" : root.remainingSummary; labelSize: Math.round(root.unit(Theme.operationMetricLabelSize)); valueSize: Math.round(root.unit(Theme.operationMetricValueSize)) }
                                Rectangle { visible: !root.largeType; Layout.fillHeight: true; implicitWidth: 1; color: Theme.taskBorder }
                                TaskMetric { density: root.density; Layout.fillWidth: true; Layout.preferredWidth: 1; heading: qsTrId("ui.30b7a5d7edaa"); value: root.stopping ? qsTrId("ui.684398ae6c51") : root.phaseSummary; labelSize: Math.round(root.unit(Theme.operationMetricLabelSize)); valueSize: Math.round(root.unit(Theme.operationMetricValueSize)) }
                            }
                        }
                    }
                    TaskPathCard { density: root.density;
                        Layout.fillWidth: true; objectName: "currentFileCard"
                        heading: root.terminal ? root.checking ? qsTrId("ui.7c5569c38a3c") : qsTrId("ui.bb7ef73495dd") : qsTrId("ui.33e19add5200")
                        value: root.checking && root.terminal ? root.resultSummary : root.outcomeUncertain ? qsTrId("ui.e43731762133") : root.terminal ? root.resultSummary : root.currentFile
                        iconSource: root.terminal ? "qrc:/xdzip/assets/" + (root.taskStatus === "succeeded" ? "success-large" : root.taskStatus === "cancelled" ? "cancelled-large" : "document-warning") + ".png" : AssetLibrary.fileIcon(root.currentFile, root.currentFile.endsWith("/"), false)
                    }
                    Text { objectName: "checkFailureEntry"; Layout.fillWidth: true; visible: root.checking && root.terminal && root.taskStatus !== "succeeded" && !!root.currentFile; text: qsTrId("integrity.stoppedAt").arg(root.currentFile); textFormat: Text.PlainText; color: Theme.textSecondary; wrapMode: Text.Wrap; font.pixelSize: Math.round(root.unit(Theme.operationNoteSize)) }
                    Text { Layout.fillWidth: true; visible: !!root.errorSummary; text: root.errorSummary; textFormat: Text.PlainText; color: Theme.menuDanger; wrapMode: Text.Wrap; font.pixelSize: Math.round(root.unit(Theme.operationNoteSize)) }
                    Text { Layout.fillWidth: true; visible: root.cleanupWarning; text: qsTrId("ui.6739af981bf5"); color: Theme.menuDanger; wrapMode: Text.Wrap; font.pixelSize: Math.round(root.unit(Theme.operationNoteSize)) }
                    InfoRow { id: timingHint; Layout.fillWidth: true; text: qsTrId("ui.77ec1e1ef2e8"); labelSize: Math.round(root.unit(Theme.operationMetricLabelSize)); lineFactor: 1.4; iconSize: Theme.operationFooterIconSize }
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
            note: root.checking ? root.sampleData ? qsTrId("ui.13c5ccd127db") : root.resultTask.format === "TAR" ? qsTrId("integrity.tarNote") : qsTrId("integrity.readOnlyNote") : root.creating && root.terminal ? (root.taskStatus === "succeeded" ? qsTrId("ui.dcb94c06dbcd") : root.outcomeUncertain ? qsTrId("ui.c96310851765") : root.cleanupWarning ? qsTrId("ui.3ac0720d9fdb") : qsTrId("ui.f49c93812dd8")) : root.terminal ? qsTrId("ui.3e97ae919078") : qsTrId("ui.188180d3d3d3")
            OperationFooterButton { text: root.terminal ? qsTrId("ui.3fd47edce45b") : qsTrId("ui.26867436321f"); onClicked: root.close() }
            OperationFooterButton { visible: root.taskStatus === "waiting"; text: qsTrId("ui.26bd480adc53"); primary: true; onClicked: root.conflictRequested() }
            OperationFooterButton { visible: root.taskStatus === "waiting_password"; text: qsTrId("password.open"); primary: true; onClicked: root.passwordRequested() }
            OperationFooterButton { objectName: "openOutputButton"; visible: root.terminal && !root.checking; enabled: !root.resultTask.openPath || root.taskStatus === "succeeded"; text: root.resultTask.openPath ? qsTrId("action.openExtractedFile") : qsTrId("ui.aee023c587c7"); onClicked: root.openOutputRequested() }
            OperationFooterButton { objectName: "cancelButton"; visible: !root.terminal; text: root.stopping ? qsTrId("ui.8a99be840844") : root.checking ? qsTrId("ui.4d761bdef85f") : qsTrId("ui.537d17f1c531"); enabled: !root.stopping; onClicked: root.cancelRequested() }
        }
    }
}
