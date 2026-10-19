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
    title: qsTrId("ui.c19dff3add03")
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
            text: qsTrId("ui.c19dff3add03")
            font.pixelSize: Theme.panelTitleSize
            color: Theme.text
        }
        Text {
            Layout.fillWidth: true
            text: qsTrId("ui.e2cb3aefdf46")
            wrapMode: Text.Wrap
            font.pixelSize: Theme.bodySize
            color: Theme.textSecondary
        }
        AppButton {
            text: qsTrId("ui.1ff29e2c4f9e")
            primary: true
            onClicked: task.present()
        }
        AppButton {
            text: qsTrId("ui.110c344f1bfb")
            onClicked: {
                integrity.taskStatus = "running";
                integrity.present();
            }
        }
        AppButton {
            text: Theme.dark ? qsTrId("ui.a9c0aba4a8d9") : qsTrId("ui.0aa457c7157c")
            onClicked: Theme.dark = !Theme.dark
        }
        AppButton {
            text: Theme.fontScale === 1 ? qsTrId("ui.654163d57759") : qsTrId("ui.e4c6ed7edc0e")
            onClicked: Theme.fontScale = Theme.fontScale === 1 ? 2 : 1
        }
        AppButton {
            text: root.totalKnown ? qsTrId("ui.46494508e8c5") : qsTrId("ui.36811ad33361")
            onClicked: root.totalKnown = !root.totalKnown
        }
        AppButton {
            text: qsTrId("ui.85a67ad77e63")
            onClicked: root.stopping = false
        }
    }
    TaskWindow {
        id: task
        sampleData: true
        owner: root
        archiveName: qsTrId("ui.520b9959055c")
        destination: qsTrId("ui.ff8d59dd14c0")
        currentFile: qsTrId("ui.256a90e1f32a")
        processedSummary: task.totalKnown ? qsTrId("ui.2b2e4496585d") : qsTrId("ui.2e94836b4139")
        fileCountSummary: qsTrId("ui.513fdd900786")
        speedSummary: "6.2 MB/s"
        remainingSummary: qsTrId("ui.59edd20c37ae")
        phaseSummary: qsTrId("ui.7ed80e44eaf9")
        fraction: 0.6
        totalKnown: root.totalKnown
        stopping: root.stopping
        onCancelRequested: root.stopping = true
    }
    IntegrityWindow {
        id: integrity
        owner: root
        sampleData: true
        archiveName: qsTrId("ui.520b9959055c")
        destination: qsTrId("ui.5d0e624f9cbb")
        currentFile: qsTrId("ui.256a90e1f32a")
        processedSummary: qsTrId("ui.8ef695dbdd2d")
        fileCountSummary: qsTrId("ui.513fdd900786")
        speedSummary: "6.2 MB/s"
        remainingSummary: qsTrId("ui.59edd20c37ae")
        phaseSummary: qsTrId("ui.33e13bf4ef00")
        resultSummary: qsTrId("ui.d0343e6846c6")
        fraction: 0.6
        totalKnown: true
        stopping: false
        onCancelRequested: taskStatus = "cancelled"
    }
}
