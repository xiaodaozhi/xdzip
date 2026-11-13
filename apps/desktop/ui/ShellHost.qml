pragma ComponentBehavior: Bound
import QtQuick
import Xdzip.Native
import Xdzip.Settings 1.0

Item {
    id: root
    required property var sessionManager
    property string sessionIdentity: ""
    property bool userHidden: false
    property bool globalQuit: false
    readonly property var controller: batch.reader
    readonly property var batchController: batch
    readonly property bool active: batch.active
    readonly property bool trayAvailable: sessionManager.shellTrayAvailable
    signal dismissed()
    Component.onCompleted: { Theme.dark = AppSettings.dark; Theme.fontScale = AppSettings.textScale; }
    function showTask(id: string) { root.userHidden = false; if (batch.finished) operations.showBatchResult(true); else operations.showTask(id); }
    function presentQuit() { globalQuit = true; quitWindow.present(); }
    function closeWindowDialogs() {
        panel.hide(); createWindow.hide(); extractWindow.hide();
        operations.taskWindow.hide(); operations.conflictWindow.hide(); operations.passwordWindow.hide();
    }
    function visibleWindows(): bool {
        return panel.visible || createWindow.visible || extractWindow.visible || operations.taskWindow.visible || operations.passwordWindow.visible || quitWindow.visible;
    }
    function showCurrent() {
        userHidden = false;
        if (batch.finished && batch.results.length) operations.showBatchResult(true);
        else if (operations.activeTaskId.length) operations.showTask(operations.activeTaskId);
        else panel.present();
        if (batch.reader.passwordPrompt.promptId) operations.passwordWindow.revealPrompt();
    }
    function begin() { panel.present(); }
    Connections {
        target: AppSettings
        function onAppearanceChanged() { Theme.dark = AppSettings.dark; Theme.fontScale = AppSettings.textScale; }
    }
    ShellBatchController {
        id: batch
        onConfigureCreate: (inputs, target, name, sevenZip) => {
            panel.hide();
            createWindow.configureSelection(inputs, target, name, sevenZip);
        }
        onConfigureExtract: { panel.hide(); extractWindow.configure(); }
        onCreationRejected: {
            if (!root.userHidden && !root.sessionManager.quitDecisionPending) {
                panel.hide(); createWindow.present();
            }
        }
        onShowTask: id => {
            panel.hide(); createWindow.close(); extractWindow.close();
            if (!root.userHidden && !root.sessionManager.quitDecisionPending) operations.showTask(id);
        }
        onShowSummary: {
            panel.hide();
            operations.showBatchResult(!root.userHidden && !root.sessionManager.quitDecisionPending && !operations.taskWindow.visible);
            root.dismissed();
        }
    }
    ShellActionsWindow {
        id: panel
        centerOnScreen: true
        batch: root.batchController
        onCancelRequested: batch.cancel()
        onDismissed: {
            if (batch.active && !root.trayAvailable) { quitWindow.present(); panel.show(); }
            else { root.userHidden = true; Qt.callLater(() => root.dismissed()); }
        }
    }
    CreateWindow {
        id: createWindow
        centerOnScreen: true
        controller: batch.reader
        onSubmitting: batch.configuringCreation()
        onClosing: Qt.callLater(() => root.dismissed())
    }
    ExtractWindow {
        id: extractWindow
        centerOnScreen: true
        controller: batch.reader
        shellMode: true
        shellArchiveCount: batch.archiveCount
        shellSubmit: (target, layout, conflict, highRatio, openAfter) => {
            extractWindow.close();
            batch.configureExtraction(target, layout, conflict, highRatio, openAfter);
        }
        onClosing: Qt.callLater(() => root.dismissed())
    }
    OperationTasks {
        id: operations
        centerOnScreen: true
        shellBatch: batch
        controller: batch.reader
        automaticPresentation: !root.userHidden
        presentationBlocked: root.sessionManager.quitDecisionPending
        preventOrphanedClose: batch.active && !root.trayAvailable
        cancelHandler: () => batch.cancel()
        onOrphanedClose: { root.globalQuit = false; quitWindow.present(); }
        Connections {
            target: operations.taskWindow
            function onVisibleChanged() {
                if (!operations.taskWindow.visible && batch.active) root.userHidden = true;
                Qt.callLater(() => root.dismissed());
            }
        }
    }
    QuitWindow {
        id: quitWindow
        centerOnScreen: true
        owner: operations.taskWindow.visible ? operations.taskWindow : panel.visible ? panel : null
        onWaitRequested: { root.userHidden = false; if (root.globalQuit) root.sessionManager.confirmQuit(true); root.globalQuit = false; close(); }
        onCancelAndQuitRequested: { if (root.globalQuit) root.sessionManager.confirmQuit(false); else batch.cancel(); root.globalQuit = false; close(); }
        onClosing: { if (root.sessionManager.quitDecisionPending) root.sessionManager.cancelQuit(); root.globalQuit = false; }
    }
}
