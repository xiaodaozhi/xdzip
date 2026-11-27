pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

OperationWindow {
    id: root
    objectName: "archiveCommentWindow"
    required property var controller
    required property DiscardArchiveCommentWindow discardWindow
    property string archiveName: ""
    property string session: ""
    property string original: ""
    property string error: ""
    property bool editable: false
    property bool unavailable: false
    property bool loading: false
    property bool saving: false
    property bool recovering: false
    property bool reloadAfterFailure: false
    property bool cancelling: false
    property var task: ({})
    signal dismissed()

    readonly property bool dirty: editor.text !== original
    readonly property bool escapeClosesFocusedWindow: true
    readonly property int byteCount: editable ? controller.archiveCommentBytes(editor.text) : 0
    readonly property bool valid: byteCount <= 65535 && controller.validArchiveComment(editor.text)
    readonly property bool canSave: editable && dirty && valid && !loading && !saving
        && controller.ready && !controller.busy && controller.sessionId === session
    readonly property bool canCancelSave: saving && !cancelling
        && !["committing", "cleaning"].includes(task.status)
    title: qsTrId("archive.comment.title")
    clientWidth: Theme.confirmationLogicalWidth
    requestedHeight: Math.min(Math.ceil(content.implicitHeight + footer.implicitHeight),
                              Theme.confirmationMaximumHeight,
                              Screen.desktopAvailableHeight - Theme.confirmationScreenMargin)
    modality: Qt.WindowModal

    function openForCurrentArchive() {
        if (visible) {
            present();
            return;
        }
        archiveName = controller.title;
        session = controller.sessionId;
        original = "";
        editor.text = "";
        error = "";
        editable = false;
        unavailable = false;
        recovering = false;
        reloadAfterFailure = false;
        loading = true;
        if (!controller.requestArchiveComment()) {
            loading = false;
            error = qsTrId("archive.comment.readFailed");
        }
        present();
        closeButton.forceActiveFocus();
    }
    function dismissOwner() {
        discardWindow.close();
        reloadAfterFailure = false;
        loading = false;
        hide();
    }
    function discardDraft() {
        editor.text = original;
        close();
    }
    function recover() {
        if (!visible || !reloadAfterFailure || !controller.ready || controller.busy
                || controller.sessionId === session)
            return;
        reloadAfterFailure = false;
        recovering = true;
        loading = true;
        if (!controller.requestArchiveComment()) {
            loading = false;
            recovering = false;
        }
    }
    function save() {
        if (!canSave)
            return;
        saving = true;
        error = "";
        task = ({});
        cancelling = false;
        if (!controller.saveArchiveComment(session, editor.text)) {
            saving = false;
            error = qsTrId("archive.comment.saveFailed");
        }
    }
    onClosing: event => {
        if (saving || dirty) {
            event.accepted = false;
            if (!saving)
                discardWindow.confirm();
        } else if (loading) {
            loading = false;
            controller.cancel();
        }
    }
    onVisibleChanged: if (!visible) dismissed()
    onActiveFocusItemChanged: if (visible) Qt.callLater(() => root.revealFocus(bodyScroll))

    Connections {
        target: root.controller
        function onChanged() {
            if (!root.visible)
                return;
            if (!root.controller.runtimeAvailable && (root.saving || root.loading)) {
                root.error = root.saving ? qsTrId("archive.comment.saveFailed") : qsTrId("archive.comment.readFailed");
                root.saving = false;
                root.loading = false;
                root.editable = false;
                root.reloadAfterFailure = false;
                root.recovering = false;
            } else if (root.reloadAfterFailure) {
                Qt.callLater(root.recover);
            } else if (root.editable && !root.saving && !root.loading && root.controller.ready
                       && root.controller.sessionId !== root.session) {
                root.editable = false;
                root.error = qsTrId("archive.comment.sourceChanged");
            }
        }
        function onArchiveCommentLoaded(value) {
            if (!root.loading)
                return;
            root.loading = false;
            if (root.recovering) {
                root.recovering = false;
                if (value.editable && value.text === root.original) {
                    root.session = value.session;
                    root.editable = true;
                } else {
                    root.error = value.error || qsTrId("archive.comment.sourceChanged");
                    root.editable = false;
                }
                return;
            }
            root.error = value.error || "";
            root.unavailable = Boolean(value.unavailable);
            root.editable = Boolean(value.editable);
            root.original = value.text || "";
            editor.text = root.original;
            editor.cursorPosition = 0;
            if (!root.error && !root.unavailable)
                editor.forceActiveFocus();
        }
        function onArchiveCommentSaveChanged(value) {
            if (!root.saving || (value.session && value.session !== root.session))
                return;
            root.task = value;
            if (value.status === "succeeded") {
                root.saving = false;
                root.original = editor.text;
                root.close();
            } else if (value.status === "failed" || value.status === "cancelled") {
                root.saving = false;
                root.cancelling = false;
                root.error = value.error || (value.status === "cancelled"
                    ? qsTrId("ui.6f96c2add214") : qsTrId("archive.comment.saveFailed"));
                if (value.errorCode === "source_changed" || value.errorCode === "stale_session") {
                    root.editable = false;
                } else if (value.id) {
                    root.editable = false;
                    root.reloadAfterFailure = true;
                    Qt.callLater(root.recover);
                }
            }
        }
    }
    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        ScrollView {
            id: bodyScroll
            objectName: "archiveCommentBody"
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.minimumHeight: 0
            clip: true
            contentWidth: availableWidth
            ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
            ColumnLayout {
                id: content
                width: bodyScroll.availableWidth
                spacing: 0
                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.margins: Theme.confirmationContentInset
                    spacing: Theme.confirmationSectionGap
                    Text {
                        Layout.fillWidth: true
                        text: root.title
                        textFormat: Text.PlainText
                        color: Theme.text
                        font.pixelSize: Theme.confirmationTitleSize
                        font.weight: Font.DemiBold
                        wrapMode: Text.Wrap
                    }
                    Text {
                        Layout.fillWidth: true
                        text: root.archiveName
                        textFormat: Text.PlainText
                        color: Theme.textSecondary
                        font.family: Theme.userDataFontFamily
                        font.pixelSize: Theme.confirmationBodySize
                        wrapMode: Text.Wrap
                    }
                    OperationFrame {
                        Layout.fillWidth: true
                        Layout.preferredHeight: Theme.archiveCommentEditorHeight
                        control: editor
                        color: editor.readOnly ? Theme.operationCard : Theme.surface
                        border.color: editor.activeFocus && !editor.readOnly ? Theme.accent
                                                                            : Theme.operationFieldBorder
                        ScrollView {
                            id: editorScroll
                            objectName: "archiveCommentTextScroll"
                            anchors.fill: parent
                            anchors.margins: 1
                            contentWidth: availableWidth
                            clip: true
                            ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
                            TextArea {
                                id: editor
                                objectName: "archiveCommentText"
                                font.family: Theme.userDataFontFamily
                                font.pixelSize: Theme.confirmationBodySize
                                readOnly: !root.editable || root.loading || root.saving
                                selectByMouse: true
                                padding: Theme.archiveCommentEditorInset
                                background: null
                                color: Theme.text
                                selectionColor: Theme.accent
                                selectedTextColor: Theme.accentText
                                placeholderTextColor: Theme.textSecondary
                                placeholderText: root.loading || root.unavailable || root.error
                                    ? "" : root.editable ? qsTrId("archive.comment.enter")
                                                         : qsTrId("archive.comment.empty")
                                textFormat: TextEdit.PlainText
                                wrapMode: TextEdit.Wrap
                                Accessible.name: root.title
                                // Leave Tab for keyboard navigation between controls.
                                Keys.onTabPressed: event => {
                                    nextItemInFocusChain(true).forceActiveFocus(Qt.TabFocusReason);
                                    event.accepted = true;
                                }
                                Keys.onBacktabPressed: event => {
                                    nextItemInFocusChain(false).forceActiveFocus(Qt.BacktabFocusReason);
                                    event.accepted = true;
                                }
                            }
                        }
                    }
                    Text {
                        Layout.fillWidth: true
                        visible: text.length > 0
                        textFormat: Text.PlainText
                        text: root.saving ? (root.cancelling ? qsTrId("welcome.opening.cancelling")
                                                            : qsTrId("archive.comment.saving"))
                            : root.loading ? qsTrId("archive.comment.loading")
                            : root.error ? root.error
                            : root.unavailable ? qsTrId("archive.comment.unavailable")
                            : root.editable ? (root.dirty && !root.valid
                                ? qsTrId("archive.comment.invalid") : "") : qsTrId("archive.comment.readOnly")
                        color: root.error || (root.editable && root.dirty && !root.valid && !root.saving)
                            ? Theme.menuDanger : Theme.textSecondary
                        font.pixelSize: Theme.confirmationBodySize
                        wrapMode: Text.Wrap
                        Accessible.role: Accessible.StaticText
                    }
                    Text {
                        Layout.fillWidth: true
                        visible: root.editable && !root.saving
                        text: qsTrId("archive.comment.bytes")
                            .arg(root.byteCount.toLocaleString(Qt.locale(), "f", 0))
                            .arg((65535).toLocaleString(Qt.locale(), "f", 0))
                        horizontalAlignment: Text.AlignRight
                        textFormat: Text.PlainText
                        color: root.byteCount > 65535 ? Theme.menuDanger : Theme.textSecondary
                        font.pixelSize: Theme.operationFooterNoteSize
                    }
                }
            }
        }
        OperationFooter {
            id: footer
            Layout.fillWidth: true
            OperationFooterButton {
                id: closeButton
                objectName: "archiveCommentClose"
                text: root.saving ? qsTrId("archive.comment.cancelSave")
                    : root.editable ? qsTrId("common.cancel") : qsTrId("tabs.closeCommand")
                enabled: !root.saving || root.canCancelSave
                onClicked: {
                    if (root.saving) {
                        root.cancelling = true;
                        root.controller.cancel();
                    } else {
                        root.close();
                    }
                }
            }
            OperationFooterButton {
                objectName: "archiveCommentSave"
                text: qsTrId("archive.comment.save")
                primary: true
                visible: root.editable
                enabled: root.canSave
                onClicked: root.save()
            }
        }
    }
}
