pragma ComponentBehavior: Bound
import QtQuick
import Xdzip.Settings 1.0

Item {
    id: root
    required property var controller
    property Window owner: null
    property bool automaticPresentation: true
    property bool presentationBlocked: false
    property bool preventOrphanedClose: false
    property var cancelHandler: null
    property string activeTaskId: ""
    property var taskSnapshot: ({})
    readonly property alias taskWindow: taskWindow
    readonly property alias conflictWindow: conflictWindow
    readonly property alias passwordWindow: passwordWindow
    signal orphanedClose()
    function cancelCurrent() {
        if (cancelHandler) cancelHandler();
        else controller.cancelTask(activeTaskId);
    }
    Connections {
        target: root.controller
        function onTasksChanged() {
            const previous = root.taskSnapshot;
            root.taskSnapshot = root.controller.tasks.find(t => t.id === root.activeTaskId) || {};
            if (root.activeTaskId.length && !root.taskSnapshot.id) { taskWindow.close(); root.activeTaskId = ""; }
            if (root.taskSnapshot.openAccepted && (!previous.openAccepted || previous.id !== root.taskSnapshot.id)) taskWindow.close();
            if (root.automaticPresentation && !root.presentationBlocked && root.controller.runtimeAvailable && !root.taskSnapshot.cancelRequested && root.taskSnapshot.status === "waiting" && root.taskSnapshot.promptId && !conflictWindow.visible)
                conflictWindow.presentPrompt(false);
            if (!root.controller.runtimeAvailable || root.taskSnapshot.cancelRequested || root.taskSnapshot.status !== "waiting" || !root.taskSnapshot.promptId) conflictWindow.close();
        }
    }
    function phaseLabel(status: string): string {
        return ({
                queued: qsTrId("ui.03f07c12d5fa"),
                preparing: qsTrId("ui.a1b9ba49c8a2"),
                running: taskSnapshot.creating ? qsTrId("ui.f066ff44699e") : qsTrId("ui.8865f5103610"),
                verifying: qsTrId("ui.b87ef7da7959"),
                waiting: qsTrId("ui.9a6cfa6abc53"),
                waiting_password: qsTrId("password.waiting"),
                committing: qsTrId("ui.f5312f1e339a"),
                cancelling: qsTrId("ui.684398ae6c51"),
                cleaning: qsTrId("ui.b4966391542e"),
                succeeded: qsTrId("ui.c0b3fbff51cc"),
                failed: qsTrId("ui.28384d7afd2e"),
                cancelled: qsTrId("ui.a37778f17c5f")
            })[status] || qsTrId("ui.b53be09930f4");
    }
    function taskStateLabel(status: string): string {
        return ({
                queued: qsTrId("ui.d6f766f2adf5"),
                preparing: qsTrId("ui.b53be09930f4"),
                running: qsTrId("ui.dc9591e56d50"),
                verifying: qsTrId("ui.784f9b54ae2b"),
                waiting: qsTrId("ui.8fb3a3261b43"),
                waiting_password: qsTrId("password.waiting"),
                committing: qsTrId("ui.a1a1720d3eb5"),
                cancelling: qsTrId("ui.72eea71385de"),
                cleaning: qsTrId("ui.718b6372d7fb"),
                succeeded: qsTrId("ui.f28461bb49c8"),
                failed: qsTrId("ui.6707de42c29d"),
                cancelled: qsTrId("ui.a37778f17c5f")
            })[status] || qsTrId("ui.b53be09930f4");
    }
    function taskStatus(task: var): string {
        return task.cancelRequested && task.status !== "cleaning" ? "cancelling" : task.status || "queued";
    }
    function taskLabel(task: var): string {
        const label = task.kind === "check" ? qsTrId("tasks.check")
            : task.kind === "move" ? qsTrId("move.task")
            : task.kind === "update" ? qsTrId("tasks.add")
            : ["export", "export-copy"].includes(task.kind) ? qsTrId("tasks.export")
            : task.creating ? qsTrId("ui.9a63e2cb4339") : qsTrId("ui.fb4b66729449");
        return label.arg(task.backendId || task.id);
    }
    function showTask(id: string) {
        const task = root.controller.tasks.find(t => t.id === id);
        if (!task) return;
        if (id !== activeTaskId) conflictWindow.close();
        activeTaskId = id;
        taskSnapshot = task;
        taskWindow.present();
        if (root.controller.runtimeAvailable && !taskSnapshot.cancelRequested && taskSnapshot.status === "waiting" && taskSnapshot.promptId)
            conflictWindow.presentPrompt(true);
        if (taskSnapshot.status === "waiting_password" && taskSnapshot.passwordPromptId)
            passwordWindow.revealPrompt();
    }
    function taskFraction(): real {
        const t = taskSnapshot;
        if (t.status === "succeeded")
            return 1;
        const bytes = Number(t.declaredBytes || 0);
        const files = Number(t.files || 0);
        if (t.kind === "check")
            return Math.min(.99, bytes ? Number(t.verifiedBytes || 0) / bytes : files ? Number(t.verifiedFiles || 0) / files : 0);
        return Math.min(.99, bytes ? Number(t.decodedBytes || 0) / bytes : files ? (Number(t.committedFiles || 0) + Number(t.skippedFiles || 0)) / files : 0);
    }
    function averageSpeed(): string {
        const seconds = Number(taskSnapshot.elapsedMs || 0) / 1000;
        if (taskSnapshot.terminal && seconds < 2)
            return "—";
        const processed = Number(taskSnapshot.kind === "check" ? taskSnapshot.readBytes || 0 : taskSnapshot.writtenBytes || 0);
        return seconds >= 2 && processed > 0 ? formatSize(String(Math.round(processed / seconds))) + "/s" : qsTrId("ui.f4199761edc9");
    }
    function remainingTime(): string {
        const t = taskSnapshot;
        const seconds = Number(t.elapsedMs || 0) / 1000;
        if (t.kind === "check" && !t.totalKnown)
            return "—";
        if (t.kind === "check") {
            const verified = Number(t.verifiedBytes || 0);
            return seconds >= 2 && verified > 0 && t.status === "running" ? qsTrId("ui.d6c40aaea130").arg(Math.ceil(Math.max(0, Number(t.declaredBytes || 0) - verified) / (verified / seconds))) : "—";
        }
        const written = Number(t.creating ? t.decodedBytes || 0 : t.writtenBytes || 0);
        if (seconds < 2 || !written || t.status !== "running")
            return "—";
        return qsTrId("ui.d6c40aaea130").arg(Math.ceil(Math.max(0, Number(t.declaredBytes || 0) - Number(t.decodedBytes || 0)) / (written / seconds)));
    }
    function formatSize(value: string): string {
        if (!/^\d+$/.test(value))
            return "—";
        const size = Number(value);
        if (size < 1024)
            return value + " B";
        if (size < 1048576)
            return (size / 1024).toLocaleString(Qt.locale(AppSettings.activeLanguage), "f", 1) + " KiB";
        if (size < 1073741824)
            return (size / 1048576).toLocaleString(Qt.locale(AppSettings.activeLanguage), "f", 1) + " MiB";
        return (size / 1073741824).toLocaleString(Qt.locale(AppSettings.activeLanguage), "f", 1) + " GiB";
    }
    TaskWindow {
        id: taskWindow
        onClosing: close => {
            if (root.preventOrphanedClose && !terminal) { close.accepted = false; root.orphanedClose(); }
        }
        owner: root.owner
        archiveName: root.taskSnapshot.archive || root.controller.extractionIntent.archive || ""
        destination: taskWindow.checking ? root.taskSnapshot.source || "" : root.taskSnapshot.target || root.controller.extractionIntent.target || ""
        currentFile: root.taskSnapshot.current || qsTrId("ui.f07335163e36")
        currentFileUserData: Boolean(root.taskSnapshot.current)
        fraction: root.taskFraction()
        creating: root.taskSnapshot.creating || false
        operationKind: root.taskSnapshot.kind || (creating ? "create" : "extract")
        totalKnown: root.taskSnapshot.totalKnown || false
        stopping: root.taskSnapshot.cancelRequested === true || ["cancelling", "cleaning"].includes(root.taskSnapshot.status)
        taskStatus: root.taskStatus(root.taskSnapshot)
        phaseSummary: taskWindow.checking && taskStatus === "running" ? qsTrId("integrity.running") : root.phaseLabel(taskStatus)
        errorSummary: root.taskSnapshot.error || ""
        cleanupWarning: root.taskSnapshot.cleanupWarning || false
        outcomeUncertain: root.taskSnapshot.outcomeUncertain || false
        processedSummary: taskWindow.checking ? qsTrId("integrity.processed").arg(root.formatSize(root.taskSnapshot.readBytes || "0")).arg(root.formatSize(root.taskSnapshot.verifiedBytes || "0")) : taskWindow.creating ? qsTrId("ui.a1c9c21cfbef").arg(root.formatSize(root.taskSnapshot.decodedBytes || "0")).arg(root.formatSize(root.taskSnapshot.writtenBytes || "0")) : qsTrId("ui.8bce13df4816").arg(root.formatSize(root.taskSnapshot.writtenBytes || "0")).arg(root.formatSize(root.taskSnapshot.committedBytes || "0"))
        fileCountSummary: taskWindow.checking ? qsTrId("integrity.files").arg(root.taskSnapshot.verifiedFiles || "0").arg(root.taskSnapshot.files || "0") : taskWindow.creating ? qsTrId("ui.1f43814c240b").arg(root.taskSnapshot.files || "0").arg(root.taskSnapshot.terminal ? qsTrId("ui.b61825c80ed6") : qsTrId("ui.483f8dd01321")) : qsTrId("ui.ed52971e9256").arg(root.taskSnapshot.committedFiles || "0").arg(root.taskSnapshot.files || "0").arg(root.taskSnapshot.skippedFiles || "0")
        resultSummary: taskWindow.checking ? qsTrId("integrity.result").arg(root.taskSnapshot.verifiedFiles || "0").arg(Math.max(0, Number(root.taskSnapshot.files || 0) - Number(root.taskSnapshot.verifiedFiles || 0))) : qsTrId("ui.51fe7c80c764").arg(root.taskSnapshot.committedFiles || "0").arg(root.taskSnapshot.skippedFiles || "0").arg(root.taskSnapshot.createdDirectories || "0")
        resultTask: root.taskSnapshot
        speedSummary: root.averageSpeed()
        remainingSummary: root.remainingTime()
        onCancelRequested: root.cancelCurrent()
        onOpenOutputRequested: root.controller.openOutput(root.activeTaskId)
        onConflictRequested: conflictWindow.presentPrompt(true)
        onPasswordRequested: passwordWindow.revealPrompt()
    }
    ConflictWindow {
        id: conflictWindow
        owner: root.owner || (taskWindow.visible ? taskWindow : null)
        task: root.taskSnapshot
        decisionsAvailable: root.controller.runtimeAvailable
        sizeFormatter: root.formatSize
        onDecision: (taskId, promptId, policy, all) => {
            if (root.cancelHandler && root.taskSnapshot.creating && policy === 1) { root.cancelCurrent(); conflictWindow.close(); return; }
            if (root.controller.decideTask(taskId, promptId, policy, all))
                conflictWindow.close();
        }
    }
    PasswordWindow {
        id: passwordWindow
        owner: root.owner || (taskWindow.visible ? taskWindow : null)
        controller: root.controller
        cancelHandler: root.cancelHandler
        automaticPresentation: root.automaticPresentation
        presentationBlocked: root.presentationBlocked
    }
}
