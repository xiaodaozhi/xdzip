pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

OperationWindow {
    id: root
    objectName: "createWindow"
    required property var controller
    property alias inputs: inputPanel.inputs
    readonly property real density: Theme.operationCreateDensity
    function unit(value: real): real {
        return value * density;
    }
    width: Theme.operationCreateWidth
    requestedHeight: Math.min(Theme.operationCreateClientHeight, Screen.desktopAvailableHeight - 60)
    title: qsTrId("ui.fe189b616c1d")
    readonly property bool targetReady: Boolean(controller.targetCheck.valid) && controller.targetMatches(target.text)
    readonly property bool sevenZip: formatGroup.currentIndex === 3
    readonly property bool supportsPasswordAndVolumes: formatGroup.currentIndex === 0 || sevenZip
    readonly property string formatExtension: [".zip", ".tar", ".tar.gz", ".7z"][formatGroup.currentIndex]
    readonly property bool validName: fileName.text.trim().toLowerCase().endsWith(formatExtension) && fileName.text.trim().length > formatExtension.length && !/[\\/:*?"<>|\x00-\x1f]/.test(fileName.text)
    readonly property real volumeBytes: {
        if (!supportsPasswordAndVolumes) return 0;
        if (volume.currentIndex !== 13)
            return [0, 3000000, 4000000, 5000000, 10000000, 20000000, 25000000, 100000000, 300000000, 650000000, 700000000, 4000000000, 4700000000][volume.currentIndex];
        if (!/^[0-9]+$/.test(volume.editText)) return -1;
        const bytes = Number(volume.editText) * [1, 1000, 1000000, 1000000000][volumeUnit.currentIndex];
        return Number.isSafeInteger(bytes) && bytes > 0 && bytes <= 25769803776 ? bytes : -1;
    }
    function passwordBytes(text: string): int {
        try { return unescape(encodeURIComponent(text)).length; } catch (_) { return -1; }
    }
    readonly property bool passwordValid: !supportsPasswordAndVolumes || !usePassword.checked || (passwordBytes(password.text) > 0 && passwordBytes(password.text) <= 128 && !/[\x00-\x1f\x7f-\x9f]/.test(password.text))
    readonly property bool optionsValid: volumeBytes >= 0 && passwordValid
    readonly property real minimumFormatWidth: Math.ceil(formatTextMetrics.advanceWidth) * formatGroup.options.length + formatGroup.padding * 2
    readonly property real optionLabelMaximumWidth: Math.max(0, optionsGrid.width - minimumFormatWidth - optionsGrid.columnSpacing)
    TextMetrics {
        id: formatTextMetrics
        text: "TAR.GZ"
        font.pixelSize: Theme.operationFooterFontSize
    }
    onClosing: password.text = ""
    readonly property string inputSummary: inputPanel.inputSummary
    function configure() {
        if (visible) {
            present();
            return;
        }
        controller.clearCreationError();
        inputs = [];
        formatGroup.currentIndex = 0;
        fileName.text = qsTrId("ui.58ca935c9fbf");
        target.text = controller.defaultTargetDirectory();
        level.currentIndex = 2;
        volume.currentIndex = 0;
        volumeUnit.currentIndex = 2;
        usePassword.checked = false;
        solid.checked = false;
        encryptHeader.checked = false;
        password.text = "";
        present();
        controller.checkCreationTarget(target.text);
    }
    function addInputPaths(paths: var, directory: bool) {
        inputPanel.addInputPaths(paths, directory);
    }
    Connections {
        target: root
        function onActiveFocusItemChanged() {
            root.revealFocus(createScroll);
        }
    }
    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        ScrollView {
            id: createScroll
            Layout.fillWidth: true
            Layout.fillHeight: true
            contentWidth: availableWidth
            clip: true
            ColumnLayout {
                width: createScroll.availableWidth
                spacing: Theme.operationHeaderBodyGap
                OperationHeader {
                    Layout.fillWidth: true
                    source: Theme.dark ? "qrc:/xdzip/assets/dark/archive-add-large.png" : "qrc:/xdzip/assets/archive-add-large.png"
                    title: qsTrId("ui.fe189b616c1d")
                    subtitle: qsTrId("ui.24d107333a04")
                }
                ColumnLayout {
                    objectName: "operationHeaderBody"
                    Layout.fillWidth: true
                    Layout.leftMargin: Theme.operationCreateContentInset
                    Layout.rightMargin: Theme.operationCreateContentInset
                    Layout.bottomMargin: 12
                    spacing: 10
                    ArchiveInputPanel {
                        id: inputPanel
                        objectName: "createInputPanel"
                        Layout.fillWidth: true
                        controller: root.controller
                        dialogOwner: root
                    }
                    Rectangle {
                        Layout.fillWidth: true
                        implicitHeight: destinationContent.implicitHeight + Theme.operationCreateCardInset * 2
                        color: Theme.operationCard
                        border.color: Theme.operationCardBorder
                        radius: 6
                        ColumnLayout {
                            id: destinationContent
                            anchors.fill: parent
                            anchors.margins: Theme.operationCreateCardInset
                            spacing: 8
                            Text {
                                text: qsTrId("ui.0b7e2876922e")
                                font.pixelSize: Theme.operationCreateBodySize
                                font.bold: true
                                color: Theme.text
                            }
                            Rectangle {
                                Layout.fillWidth: true
                                implicitHeight: destinationForm.implicitHeight + 16
                                color: Theme.surface
                                border.color: Theme.operationCardBorder
                                radius: 6
                                ColumnLayout {
                                    id: destinationForm
                                    anchors.fill: parent
                                    anchors.margins: 8
                                    spacing: 8
                                    GridLayout {
                                        Layout.fillWidth: true
                                        columns: 2
                                        columnSpacing: 12
                                        rowSpacing: 8
                                        Text {
                                            text: qsTrId("ui.a6e48adc26d6")
                                            font.pixelSize: Theme.operationCreateBodySize
                                            color: Theme.text
                                        }
                                        OperationField {
                                            id: fileName
                                            objectName: "createName"
                                            font.family: Theme.userDataFontFamily
                                            Layout.fillWidth: true
                                            density: root.density
                                            implicitHeight: Math.max(Theme.operationCreateFieldHeight, contentHeight + 8)
                                            topPadding: 4
                                            bottomPadding: 4
                                            Accessible.name: qsTrId("ui.0d568cd7b07d")
                                        }
                                        Text {
                                            text: qsTrId("ui.e31cccd7bba0")
                                            font.pixelSize: Theme.operationCreateBodySize
                                            color: Theme.text
                                        }
                                        RowLayout {
                                            Layout.fillWidth: true
                                            spacing: 8
                                            OperationField {
                                                id: target
                                                objectName: "createTarget"
                                                font.family: Theme.userDataFontFamily
                                                Layout.fillWidth: true
                                                density: root.density
                                                implicitHeight: Math.max(Theme.operationCreateFieldHeight, contentHeight + 8)
                                                topPadding: 4
                                                bottomPadding: 4
                                                Accessible.name: qsTrId("ui.18e7649764bb")
                                                onEditingFinished: root.controller.checkCreationTarget(text)
                                            }
                                            OperationButton {
                                                objectName: "createChangeTarget"
                                                text: qsTrId("ui.6b01fce4dbea")
                                                minimumWidth: 58
                                                onClicked: {
                                                    const path = root.controller.chooseTarget(target.text, root);
                                                    if (path) {
                                                        target.text = path;
                                                        root.controller.checkCreationTarget(path);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    InfoRow {
                                        Layout.fillWidth: true
                                        iconSource: root.targetReady ? "qrc:/xdzip/assets/success-large.png" : (Theme.dark ? "qrc:/xdzip/assets/info-dark.png" : "qrc:/xdzip/assets/info-light.png")
                                        text: root.targetReady ? qsTrId("ui.059a6ea7ca12") : root.controller.targetCheck.checking ? qsTrId("ui.2ae4afff62f1") : root.controller.targetCheck.error || qsTrId("ui.cef0425c5413")
                                        labelSize: Theme.operationCreateNoteSize
                                        iconSize: 19.2
                                    }
                                }
                            }
                        }
                    }
                    Rectangle {
                        Layout.fillWidth: true
                        implicitHeight: optionsContent.implicitHeight + Theme.operationCreateCardInset * 2
                        color: Theme.operationCard
                        border.color: Theme.operationCardBorder
                        radius: 6
                        ColumnLayout {
                            id: optionsContent
                            anchors.fill: parent
                            anchors.margins: Theme.operationCreateCardInset
                            spacing: 8
                            Text {
                                text: qsTrId("ui.28c4470de92d")
                                font.pixelSize: Theme.operationCreateBodySize
                                font.bold: true
                                color: Theme.text
                            }
                            Rectangle {
                                Layout.fillWidth: true
                                implicitHeight: optionsForm.implicitHeight + 16
                                color: Theme.surface
                                border.color: Theme.operationCardBorder
                                radius: 6
                                ColumnLayout {
                                    id: optionsForm
                                    anchors.fill: parent
                                    anchors.margins: 8
                                    spacing: 8
                                    GridLayout {
                                        id: optionsGrid
                                        Layout.fillWidth: true
                                        columns: 2
                                        columnSpacing: 12
                                        rowSpacing: 8
                                        Text {
                                            Layout.maximumWidth: root.optionLabelMaximumWidth
                                            text: qsTrId("ui.5e30948d6a8b")
                                            wrapMode: Text.Wrap
                                            font.pixelSize: Theme.operationCreateBodySize
                                            color: Theme.text
                                        }
                                        OperationSegmentedGroup {
                                            id: formatGroup
                                            objectName: "createFormat"
                                            Layout.fillWidth: true
                                            Layout.minimumWidth: root.minimumFormatWidth
                                            accessibleLabel: qsTrId("ui.5e30948d6a8b")
                                            onCurrentIndexChanged: {
                                                if (!fileName) return;
                                                const stem = fileName.text.replace(/\.(tar\.gz|zip|tar|7z)$/i, "");
                                                fileName.text = stem + [".zip", ".tar", ".tar.gz", ".7z"][currentIndex];
                                                volume.currentIndex = 0;
                                                usePassword.checked = false;
                                                solid.checked = false;
                                                encryptHeader.checked = false;
                                                password.text = "";
                                            }
                                            options: [
                                                {
                                                    text: "ZIP",
                                                    enabled: true
                                                },
                                                {
                                                    text: "TAR",
                                                    enabled: true
                                                },
                                                {
                                                    text: "TAR.GZ",
                                                    enabled: true
                                                },
                                                {
                                                    text: "7z",
                                                    enabled: true
                                                }
                                            ]
                                        }
                                        Text {
                                            Layout.maximumWidth: root.optionLabelMaximumWidth
                                            text: qsTrId("ui.e7e4273dd4c8")
                                            wrapMode: Text.Wrap
                                            font.pixelSize: Theme.operationCreateBodySize
                                            color: Theme.text
                                        }
                                        OperationSelect {
                                            id: level
                                            enabled: formatGroup.currentIndex !== 1
                                            objectName: "createLevel"
                                            Layout.fillWidth: true
                                            density: root.density
                                            implicitHeight: Math.max(Theme.operationCreateFieldHeight, contentItem.implicitHeight + 8)
                                            currentIndex: 2
                                            model: [qsTrId("ui.114918a8e262"), qsTrId("ui.0ab258fe71f4"), qsTrId("ui.64209bb8c4df"), qsTrId("ui.bd60ad0e564e")]
                                        }
                                        Text {
                                            Layout.maximumWidth: root.optionLabelMaximumWidth
                                            text: qsTrId("ui.5473b9b24a75")
                                            wrapMode: Text.Wrap
                                            font.pixelSize: Theme.operationCreateBodySize
                                            color: Theme.text
                                        }
                                        RowLayout {
                                            Layout.fillWidth: true
                                            spacing: 8
                                            OperationSelect {
                                                id: volume
                                                enabled: root.supportsPasswordAndVolumes
                                                objectName: "createVolume"
                                                Layout.fillWidth: true
                                                density: root.density
                                                implicitHeight: Math.max(Theme.operationCreateFieldHeight, contentItem.implicitHeight + 8)
                                                model: [qsTrId("ui.13fa566b385e"), "3MB", "4MB", "5MB", "10MB", "20MB", "25MB (Gmail)", "100MB", "300MB", "650MB (CD)", "700MB (CD)", "4GB (FAT32)", "4.7GB (DVE)", qsTrId("ui.4eafa9e925b3")]
                                                editable: currentIndex === 13
                                                numericOnly: true
                                                onCurrentIndexChanged: if (currentIndex === 13) Qt.callLater(function () { volume.editText = ""; });
                                            }
                                            OperationSelect {
                                                id: volumeUnit
                                                objectName: "createVolumeUnit"
                                                visible: volume.currentIndex === 13
                                                Layout.preferredWidth: Math.max(74, contentItem.implicitWidth + leftPadding + rightPadding)
                                                density: root.density
                                                implicitHeight: Math.max(Theme.operationCreateFieldHeight, contentItem.implicitHeight + 8)
                                                currentIndex: 2
                                                model: [qsTrId("ui.b1c5e73e1471"), "KB", "MB", "GB"]
                                            }
                                        }
                                    }
                                    Text {
                                        Layout.fillWidth: true
                                        text: formatGroup.currentIndex === 1 ? qsTrId("ui.a823dd053540") : formatGroup.currentIndex === 2 ? qsTrId("ui.8de83aec57d2") : volume.currentIndex === 13 && root.volumeBytes < 0 ? qsTrId("ui.58e4aeca8ddc") : volume.currentIndex > 0 ? (root.sevenZip ? qsTrId("create.sevenZipVolumes") : qsTrId("ui.17f715707e84")) : root.sevenZip ? qsTrId("create.sevenZipMethod") : qsTrId("ui.13e4af7e4901")
                                        font.pixelSize: Theme.operationCreateNoteSize
                                        color: Theme.textSecondary
                                    }
                                }
                            }
                            OperationCheck {
                                id: usePassword
                                enabled: root.supportsPasswordAndVolumes
                                objectName: "createUsePassword"
                                text: qsTrId("ui.7673702e6ff6")
                                density: root.density
                                onCheckedChanged: if (!checked) { password.text = ""; encryptHeader.checked = false; }
                            }
                            OperationField {
                                id: password
                                objectName: "createPassword"
                                Layout.fillWidth: true
                                enabled: usePassword.checked
                                density: root.density
                                implicitHeight: Math.max(Theme.operationCreateFieldHeight, contentHeight + 8)
                                echoMode: TextInput.Password
                                placeholderText: qsTrId("ui.b1e4901deada")
                                Accessible.name: qsTrId("ui.96d162db488e")
                            }
                            Text {
                                Layout.fillWidth: true
                                text: !root.supportsPasswordAndVolumes ? qsTrId("ui.f0af11af0bbb") : !root.passwordValid ? qsTrId("ui.cbf703a10c7b") : root.sevenZip ? qsTrId("create.sevenZipEncryption") : qsTrId("ui.511f8696a4f3")
                                wrapMode: Text.Wrap
                                font.pixelSize: Theme.operationCreateNoteSize
                                color: root.passwordValid ? Theme.textSecondary : Theme.menuDanger
                            }
                            OperationCheck {
                                id: encryptHeader
                                objectName: "createEncryptHeader"
                                visible: root.sevenZip
                                enabled: root.sevenZip && usePassword.checked
                                text: qsTrId("create.encryptHeader")
                                density: root.density
                            }
                            OperationCheck {
                                id: solid
                                objectName: "createSolid"
                                visible: root.sevenZip
                                text: qsTrId("create.solid")
                                density: root.density
                            }
                        }
                    }
                    Text {
                        Layout.fillWidth: true
                        visible: root.controller.creationError.length > 0
                        text: root.controller.creationError
                        textFormat: Text.PlainText
                        wrapMode: Text.Wrap
                        color: Theme.menuDanger
                        font.pixelSize: Theme.operationCreateNoteSize
                    }
                }
            }
        }
        OperationFooter {
            Layout.fillWidth: true
            contentInset: Theme.operationCreateContentInset
            verticalInset: 10
            minimumFooterHeight: 52
            note: root.inputs.length ? qsTrId("ui.1bd1893c0900") : qsTrId("ui.79cd20ee8749")
            noteIcon: Theme.dark ? "qrc:/xdzip/assets/dark/shield.png" : "qrc:/xdzip/assets/light/shield.png"
            OperationFooterButton {
                objectName: "createCancel"
                text: qsTrId("common.cancel")
                onClicked: root.close()
            }
            OperationFooterButton {
                objectName: "startCreate"
                text: qsTrId("ui.e185d04bcc5f")
                primary: true
                enabled: root.inputs.length > 0 && root.validName && root.targetReady && root.optionsValid && !root.controller.busy
                onClicked: {
                    const method = formatGroup.currentIndex === 1 || (root.supportsPasswordAndVolumes && level.currentIndex === 0) ? 0 : root.sevenZip ? 32 : 8;
                    root.controller.startCreation(root.inputs.map(item => item.path), target.text, fileName.text.trim(), method, formatGroup.currentIndex === 2 && level.currentIndex === 0 ? 0 : [6, 1, 6, 9][level.currentIndex], false, root.volumeBytes, root.supportsPasswordAndVolumes && usePassword.checked ? password.text : "", root.sevenZip && solid.checked, root.sevenZip && usePassword.checked && encryptHeader.checked);
                    password.text = "";
                }
            }
        }
    }
}
