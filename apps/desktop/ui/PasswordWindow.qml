pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

OperationWindow {
    id: root
    objectName: "passwordWindow"
    required property var controller
    property var prompt: controller.passwordPrompt || ({})
    property real availableClientHeight: Screen.desktopAvailableHeight - Theme.passwordScreenMargin
    property bool uiReady: false
    property bool pending: false
    property bool rejected: false
    property bool showPassword: false
    property string shownTaskId: ""
    property string shownPromptId: ""
    property bool dismissing: false
    readonly property bool tooLong: utf8Size(password.text) > 1024
    readonly property alias headingItem: heading
    readonly property alias bodyItem: scroll
    readonly property alias footerItem: footer
    readonly property alias fieldItem: password
    title: qsTrId("password.title")
    modality: Qt.ApplicationModal
    width: Theme.passwordLogicalWidth
    requestedHeight: Math.min(Math.ceil(content.implicitHeight + footer.implicitHeight),
                              Theme.passwordMaximumHeight, availableClientHeight)
    onActiveFocusItemChanged: if (visible) Qt.callLater(() => root.revealFocus(scroll))
    onPromptChanged: if (uiReady) syncPrompt()
    Component.onCompleted: { uiReady = true; syncPrompt(); }
    onClosing: {
        const task = shownTaskId;
        const token = shownPromptId;
        clearSecret();
        if (!dismissing && task && token && String(prompt.taskId || "") === task
                && String(prompt.promptId || "") === token)
            controller.cancelTask(task);
    }
    function utf8Size(value: string): int {
        let size = 0;
        for (let i = 0; i < value.length; ++i) {
            const unit = value.charCodeAt(i);
            if (unit < 0x80) ++size;
            else if (unit < 0x800) size += 2;
            else if (unit >= 0xd800 && unit <= 0xdbff && i + 1 < value.length
                     && value.charCodeAt(i + 1) >= 0xdc00 && value.charCodeAt(i + 1) <= 0xdfff) {
                size += 4;
                ++i;
            } else size += 3;
        }
        return size;
    }
    function clearSecret() {
        password.text = "";
        showPassword = false;
    }
    function revealPrompt() {
        syncPrompt();
        if (shownTaskId && shownPromptId) present();
    }
    function syncPrompt() {
        const task = String(prompt.taskId || "");
        const token = String(prompt.promptId || "");
        if (!task || !token) {
            clearSecret();
            pending = false;
            rejected = false;
            shownTaskId = "";
            shownPromptId = "";
            dismissing = true;
            close();
            dismissing = false;
            return;
        }
        if (task === shownTaskId && token === shownPromptId) return;
        clearSecret();
        pending = false;
        rejected = false;
        shownTaskId = task;
        shownPromptId = token;
        Qt.callLater(() => {
            if (root.shownTaskId === task && root.shownPromptId === token) {
                root.present();
                password.forceActiveFocus();
            }
        });
    }
    function submit() {
        if (pending || tooLong || !shownTaskId || !shownPromptId
                || shownTaskId !== String(prompt.taskId || "")
                || shownPromptId !== String(prompt.promptId || "")) return;
        // A synchronous acknowledgement never transfers this secret through a task snapshot.
        pending = true;
        rejected = false;
        const task = shownTaskId;
        const token = shownPromptId;
        if (controller.submitPassword(task, token, password.text)) {
            clearSecret();
        } else if (shownTaskId === task && shownPromptId === token) {
            pending = false;
            rejected = true;
        }
    }
    ColumnLayout {
        width: root.width
        height: root.height
        spacing: 0
        ScrollView {
            id: scroll
            objectName: "passwordScroll"
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.minimumHeight: 0
            contentWidth: availableWidth
            clip: true
            ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
            ColumnLayout {
                id: content
                width: scroll.availableWidth
                spacing: 0
                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.margins: Theme.passwordContentInset
                    spacing: Theme.passwordSectionGap
                    Text {
                        id: heading
                        objectName: "passwordHeading"
                        Layout.fillWidth: true
                        text: qsTrId("password.heading")
                        textFormat: Text.PlainText
                        font.pixelSize: Math.round(Theme.passwordTitleSize)
                        font.bold: true
                        lineHeight: Math.round(font.pixelSize * Theme.passwordTitleLineFactor)
                        lineHeightMode: Text.FixedHeight
                        Layout.preferredHeight: Math.max(1, lineCount) * lineHeight
                        color: Theme.text
                        wrapMode: Text.Wrap
                    }
                    Text {
                        objectName: "passwordDescription"
                        Layout.fillWidth: true
                        text: qsTrId("password.description")
                        textFormat: Text.PlainText
                        font.pixelSize: Math.round(Theme.passwordBodySize)
                        lineHeight: Math.round(font.pixelSize * Theme.passwordBodyLineFactor)
                        lineHeightMode: Text.FixedHeight
                        Layout.preferredHeight: Math.max(1, lineCount) * lineHeight
                        color: Theme.textSecondary
                        wrapMode: Text.Wrap
                    }
                    Text {
                        objectName: "passwordFileName"
                        Layout.fillWidth: true
                        text: root.prompt.name || ""
                        textFormat: Text.PlainText
                        font.pixelSize: Math.round(Theme.passwordBodySize)
                        lineHeight: Math.round(font.pixelSize * Theme.passwordBodyLineFactor)
                        lineHeightMode: Text.FixedHeight
                        Layout.preferredHeight: Math.max(1, lineCount) * lineHeight
                        color: Theme.text
                        wrapMode: Text.WrapAnywhere
                    }
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: Theme.passwordFieldGap
                        Text {
                            objectName: "passwordLabel"
                            Layout.fillWidth: true
                            text: qsTrId("password.label")
                            font.pixelSize: Math.round(Theme.passwordBodySize)
                            font.weight: Font.DemiBold
                            lineHeight: Math.round(font.pixelSize * Theme.passwordBodyLineFactor)
                            lineHeightMode: Text.FixedHeight
                            Layout.preferredHeight: Math.max(1, lineCount) * lineHeight
                            color: Theme.text
                        }
                        RowLayout {
                            Layout.fillWidth: true
                            spacing: Theme.passwordFieldGap
                            OperationField {
                                id: password
                                objectName: "extractionPassword"
                                Layout.fillWidth: true
                                implicitHeight: Math.max(Theme.passwordFieldHeight,
                                                         contentHeight + Theme.passwordFieldVerticalInset * 2)
                                font.pixelSize: Math.round(Theme.passwordBodySize)
                                echoMode: root.showPassword ? TextInput.Normal : TextInput.Password
                                inputMethodHints: Qt.ImhSensitiveData | Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
                                maximumLength: 1024
                                enabled: !root.pending
                                placeholderText: qsTrId("password.placeholder")
                                Accessible.name: qsTrId("password.label")
                                onAccepted: root.submit()
                                onTextEdited: root.rejected = false
                            }
                            OperationButton {
                                objectName: "passwordVisibility"
                                text: root.showPassword ? qsTrId("password.hide") : qsTrId("password.show")
                                Accessible.name: text
                                enabled: !root.pending
                                onClicked: root.showPassword = !root.showPassword
                            }
                        }
                    }
                    Text {
                        objectName: "passwordError"
                        Layout.fillWidth: true
                        visible: root.tooLong || root.rejected || !!root.prompt.retry
                        text: root.tooLong ? qsTrId("password.limit") : root.rejected ? qsTrId("password.submitRejected") : qsTrId("password.retry")
                        textFormat: Text.PlainText
                        font.pixelSize: Math.round(Theme.passwordBodySize)
                        lineHeight: Math.round(font.pixelSize * Theme.passwordBodyLineFactor)
                        lineHeightMode: Text.FixedHeight
                        Layout.preferredHeight: Math.max(1, lineCount) * lineHeight
                        color: Theme.menuDanger
                        wrapMode: Text.Wrap
                        Accessible.role: Accessible.StaticText
                    }
                    Text {
                        objectName: "passwordPrivacy"
                        Layout.fillWidth: true
                        text: qsTrId("password.privacy")
                        textFormat: Text.PlainText
                        font.pixelSize: Math.round(Theme.passwordNoteSize)
                        lineHeight: Math.round(font.pixelSize * Theme.passwordBodyLineFactor)
                        lineHeightMode: Text.FixedHeight
                        Layout.preferredHeight: Math.max(1, lineCount) * lineHeight
                        color: Theme.textSecondary
                        wrapMode: Text.Wrap
                    }
                }
            }
        }
        OperationFooter {
            id: footer
            objectName: "passwordFooter"
            Layout.fillWidth: true
            OperationFooterButton {
                objectName: "passwordCancel"
                text: qsTrId("ui.537d17f1c531")
                onClicked: root.close()
            }
            OperationFooterButton {
                objectName: "passwordSubmit"
                text: root.pending ? qsTrId("password.submitting") : qsTrId("password.continue")
                primary: true
                enabled: !root.pending && !root.tooLong && root.shownPromptId.length > 0
                onClicked: root.submit()
            }
        }
    }
}
