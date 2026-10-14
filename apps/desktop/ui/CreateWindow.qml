pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

OperationWindow {
    id: root
    objectName: "createWindow"
    required property var controller
    property var inputs: []
    readonly property real density: Theme.operationCreateDensity
    function unit(value: real): real {
        return value * density;
    }
    width: Theme.operationCreateWidth
    requestedHeight: Math.min(Theme.operationCreateClientHeight, Screen.desktopAvailableHeight - 60)
    title: qsTr("新建压缩文件 — xdzip")
    readonly property bool targetReady: Boolean(controller.targetCheck.valid) && controller.targetMatches(target.text)
    readonly property bool validName: /^.+\.zip$/i.test(fileName.text.trim()) && !/[\\/:*?"<>|\x00-\x1f]/.test(fileName.text)
    readonly property real volumeBytes: {
        if (volume.currentIndex !== 13)
            return [0, 3000000, 4000000, 5000000, 10000000, 20000000, 25000000, 100000000, 300000000, 650000000, 700000000, 4000000000, 4700000000][volume.currentIndex];
        if (!/^[0-9]+$/.test(volume.editText)) return -1;
        const bytes = Number(volume.editText) * [1, 1000, 1000000, 1000000000][volumeUnit.currentIndex];
        return Number.isSafeInteger(bytes) && bytes > 0 && bytes <= 25769803776 ? bytes : -1;
    }
    function passwordBytes(text: string): int {
        try { return unescape(encodeURIComponent(text)).length; } catch (_) { return -1; }
    }
    readonly property bool passwordValid: !usePassword.checked || (passwordBytes(password.text) > 0 && passwordBytes(password.text) <= 128 && !/[\x00-\x1f\x7f-\x9f]/.test(password.text))
    readonly property bool optionsValid: volumeBytes >= 0 && passwordValid
    onClosing: password.text = ""
    function baseName(path: string): string {
        return path.replace(/\\/g, "/").split("/").pop();
    }
    function formatInputSize(value: var): string {
        const bytes = Number(value || 0);
        return bytes >= 1000000 ? (bytes / 1000000).toFixed(1) + " MB" : bytes >= 1000 ? Math.round(bytes / 1000) + " KB" : bytes + " B";
    }
    readonly property string inputSummary: {
        const files = inputs.filter(item => !item.directory);
        const directories = inputs.length - files.length;
        const parts = [];
        if (files.length)
            parts.push(qsTr("%1 个文件").arg(files.length));
        if (directories)
            parts.push(qsTr("%1 个文件夹").arg(directories));
        if (files.length && files.every(item => item.known))
            parts.push(formatInputSize(files.reduce((total, item) => total + Number(item.sizeBytes || 0), 0)));
        else if (files.length)
            parts.push(qsTr("大小将在开始前检查"));
        if (directories)
            parts.push(qsTr("文件夹大小将在开始前扫描"));
        return parts.length ? parts.join("，") : qsTr("尚未添加文件");
    }
    onInputsChanged: if (controller)
        controller.inspectCreationInputs(inputs.map(item => item.path))
    Connections {
        target: root.controller
        function onCreationInputsChanged() {
            const metadata = new Map();
            for (const item of root.controller.creationInputs)
                metadata.set(item.path, item);
            root.inputs = root.inputs.map(item => metadata.has(item.path) ? Object.assign({}, item, metadata.get(item.path)) : item);
        }
    }
    function configure() {
        if (visible) {
            present();
            return;
        }
        controller.clearCreationError();
        inputs = [];
        fileName.text = qsTr("新建压缩文件.zip");
        target.text = controller.defaultTargetDirectory();
        level.currentIndex = 2;
        volume.currentIndex = 0;
        volumeUnit.currentIndex = 2;
        usePassword.checked = false;
        password.text = "";
        present();
        controller.checkCreationTarget(target.text);
    }
    function addInputPaths(paths: var, directory: bool) {
        const next = inputs.slice();
        // Native multi-selection can be large; deduplicate in one pass.
        const selected = new Set(next.map(item => item.path));
        for (let i = 0; i < paths.length; ++i) {
            const path = String(paths[i]);
            if (!selected.has(path)) {
                next.push({
                    path,
                    name: baseName(path),
                    directory
                });
                selected.add(path);
            }
        }
        inputs = next;
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
                spacing: 12
                RowLayout {
                    Layout.fillWidth: true
                    Layout.leftMargin: Theme.operationCreateContentInset
                    Layout.rightMargin: Theme.operationCreateContentInset
                    Layout.topMargin: 20
                    spacing: 14
                    OperationHero {
                        source: Theme.dark ? "qrc:/xdzip/assets/dark/archive-add-large.png" : "qrc:/xdzip/assets/archive-add-large.png"
                        density: root.density
                        Layout.preferredWidth: canvasWidth
                        Layout.preferredHeight: canvasHeight
                    }
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 6
                        Text {
                            text: qsTr("新建压缩文件")
                            font.pixelSize: Theme.operationCreateTitleSize
                            font.bold: true
                            color: Theme.text
                        }
                        Text {
                            Layout.fillWidth: true
                            text: qsTr("将文件整理为一个压缩包。")
                            wrapMode: Text.Wrap
                            font.pixelSize: Theme.operationCreateBodySize
                            color: Theme.textSecondary
                        }
                    }
                }
                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.leftMargin: Theme.operationCreateContentInset
                    Layout.rightMargin: Theme.operationCreateContentInset
                    Layout.bottomMargin: 12
                    spacing: 10
                    Rectangle {
                        Layout.fillWidth: true
                        implicitHeight: sourceContent.implicitHeight + Theme.operationCreateCardInset * 2
                        color: Theme.operationCard
                        border.color: Theme.operationCardBorder
                        radius: 6
                        ColumnLayout {
                            id: sourceContent
                            anchors.fill: parent
                            anchors.margins: Theme.operationCreateCardInset
                            spacing: 8
                            Text {
                                text: qsTr("添加文件")
                                font.pixelSize: Theme.operationCreateBodySize
                                font.bold: true
                                color: Theme.text
                            }
                            Rectangle {
                                Layout.fillWidth: true
                                implicitHeight: sourceList.implicitHeight + 20
                                color: Theme.surface
                                border.color: Theme.operationCardBorder
                                radius: 6
                                ColumnLayout {
                                    id: sourceList
                                    anchors.fill: parent
                                    anchors.margins: 10
                                    spacing: 6
                                    RowLayout {
                                        Layout.fillWidth: true
                                        spacing: 8
                                        OperationAction {
                                            objectName: "createAddFiles"
                                            text: qsTr("添加文件")
                                            primary: true
                                            actionIcon: AssetLibrary.fileIcon("", false, false)
                                            enabled: !root.controller.busy
                                            onClicked: root.addInputPaths(root.controller.chooseCreationInputs(false), false)
                                        }
                                        OperationAction {
                                            objectName: "createAddDirectories"
                                            text: qsTr("添加文件夹")
                                            actionIcon: AssetLibrary.fileIcon("", true, false)
                                            enabled: !root.controller.busy
                                            onClicked: root.addInputPaths(root.controller.chooseCreationInputs(true), true)
                                        }
                                        Item {
                                            Layout.fillWidth: true
                                        }
                                    }
                                    ListView {
                                        id: inputList
                                        objectName: "createInputList"
                                        Layout.fillWidth: true
                                        Layout.preferredHeight: root.inputs.length ? Math.min(148, root.inputs.length * Math.max(Theme.operationCreateInputRowHeight, Theme.operationCreateBodySize * 1.4 + 12)) : Theme.operationCreateInputRowHeight
                                        implicitHeight: Layout.preferredHeight
                                        clip: true
                                        model: root.inputs
                                        ScrollBar.vertical: ScrollBar {
                                            policy: ScrollBar.AsNeeded
                                        }
                                        delegate: Rectangle {
                                            id: inputRow
                                            required property var modelData
                                            required property int index
                                            width: inputList.width
                                            height: Math.max(Theme.operationCreateInputRowHeight, Theme.operationCreateBodySize * 1.4 + 12)
                                            color: rowHover.hovered ? Theme.controlHover : Theme.surface
                                            Rectangle {
                                                anchors.left: parent.left
                                                anchors.right: parent.right
                                                anchors.bottom: parent.bottom
                                                height: 1
                                                color: Theme.operationCardBorder
                                            }
                                            HoverHandler {
                                                id: rowHover
                                            }
                                            RowLayout {
                                                anchors.fill: parent
                                                spacing: 8
                                                Image {
                                                    source: AssetLibrary.fileIcon(inputRow.modelData.name, inputRow.modelData.directory, false)
                                                    Layout.preferredWidth: 24
                                                    Layout.preferredHeight: 24
                                                }
                                                Text {
                                                    Layout.fillWidth: true
                                                    text: inputRow.modelData.name
                                                    textFormat: Text.PlainText
                                                    elide: Text.ElideMiddle
                                                    font.pixelSize: Theme.operationCreateBodySize
                                                    color: Theme.text
                                                    Accessible.description: inputRow.modelData.path
                                                }
                                                Text {
                                                    objectName: "createInputSize-" + inputRow.index
                                                    visible: inputRow.modelData.known === true
                                                    text: root.formatInputSize(inputRow.modelData.sizeBytes)
                                                    font.pixelSize: Theme.operationCreateNoteSize
                                                    color: Theme.textSecondary
                                                    horizontalAlignment: Text.AlignRight
                                                }
                                                OperationRemoveButton {
                                                    objectName: "createRemoveInput-" + inputRow.index
                                                    entryName: inputRow.modelData.name
                                                    enabled: !root.controller.busy
                                                    onClicked: root.inputs = root.inputs.filter((_, i) => i !== inputRow.index)
                                                }
                                            }
                                        }
                                        Text {
                                            anchors.centerIn: parent
                                            visible: root.inputs.length === 0
                                            text: qsTr("添加文件或文件夹开始")
                                            font.pixelSize: Theme.operationCreateBodySize
                                            color: Theme.textSecondary
                                        }
                                    }
                                    Text {
                                        objectName: "createInputSummary"
                                        Layout.fillWidth: true
                                        wrapMode: Text.Wrap
                                        text: root.inputSummary
                                        font.pixelSize: Theme.operationCreateNoteSize
                                        color: Theme.textSecondary
                                    }
                                }
                            }
                        }
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
                                text: qsTr("保存位置")
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
                                            text: qsTr("文件名")
                                            font.pixelSize: Theme.operationCreateBodySize
                                            color: Theme.text
                                        }
                                        OperationField {
                                            id: fileName
                                            objectName: "createName"
                                            Layout.fillWidth: true
                                            density: root.density
                                            implicitHeight: Math.max(Theme.operationCreateFieldHeight, contentHeight + 8)
                                            topPadding: 4
                                            bottomPadding: 4
                                            Accessible.name: qsTr("压缩文件名称")
                                        }
                                        Text {
                                            text: qsTr("保存到")
                                            font.pixelSize: Theme.operationCreateBodySize
                                            color: Theme.text
                                        }
                                        RowLayout {
                                            Layout.fillWidth: true
                                            spacing: 8
                                            OperationField {
                                                id: target
                                                objectName: "createTarget"
                                                Layout.fillWidth: true
                                                density: root.density
                                                implicitHeight: Math.max(Theme.operationCreateFieldHeight, contentHeight + 8)
                                                topPadding: 4
                                                bottomPadding: 4
                                                Accessible.name: qsTr("保存目录")
                                                onEditingFinished: root.controller.checkCreationTarget(text)
                                            }
                                            OperationButton {
                                                objectName: "createChangeTarget"
                                                text: qsTr("更改")
                                                minimumWidth: 58
                                                onClicked: {
                                                    const path = root.controller.chooseTarget(target.text);
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
                                        text: root.targetReady ? qsTr("目标位置可用") : root.controller.targetCheck.checking ? qsTr("正在检查目标目录…") : root.controller.targetCheck.error || qsTr("更改目录后请重新检查")
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
                                text: qsTr("压缩选项")
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
                                        Layout.fillWidth: true
                                        columns: 2
                                        columnSpacing: 12
                                        rowSpacing: 8
                                        Text {
                                            text: qsTr("压缩格式")
                                            font.pixelSize: Theme.operationCreateBodySize
                                            color: Theme.text
                                        }
                                        OperationSegmentedGroup {
                                            id: formatGroup
                                            objectName: "createFormat"
                                            Layout.fillWidth: true
                                            accessibleLabel: qsTr("压缩格式")
                                            options: [
                                                {
                                                    text: "ZIP",
                                                    enabled: true
                                                },
                                                {
                                                    text: "TAR",
                                                    enabled: false,
                                                    reason: qsTr("此格式将在后续版本提供")
                                                },
                                                {
                                                    text: "TAR.GZ",
                                                    enabled: false,
                                                    reason: qsTr("此格式将在后续版本提供")
                                                }
                                            ]
                                        }
                                        Text {
                                            text: qsTr("压缩级别")
                                            font.pixelSize: Theme.operationCreateBodySize
                                            color: Theme.text
                                        }
                                        OperationSelect {
                                            id: level
                                            objectName: "createLevel"
                                            Layout.fillWidth: true
                                            density: root.density
                                            implicitHeight: Math.max(Theme.operationCreateFieldHeight, contentItem.implicitHeight + 8)
                                            currentIndex: 2
                                            model: [qsTr("0-不压缩"), qsTr("1-快速压缩"), qsTr("2-正常压缩"), qsTr("3-最大压缩")]
                                        }
                                        Text {
                                            text: qsTr("分卷")
                                            font.pixelSize: Theme.operationCreateBodySize
                                            color: Theme.text
                                        }
                                        RowLayout {
                                            Layout.fillWidth: true
                                            spacing: 8
                                            OperationSelect {
                                                id: volume
                                                objectName: "createVolume"
                                                Layout.fillWidth: true
                                                density: root.density
                                                implicitHeight: Math.max(Theme.operationCreateFieldHeight, contentItem.implicitHeight + 8)
                                                model: [qsTr("不分卷"), "3MB", "4MB", "5MB", "10MB", "20MB", "25MB (Gmail)", "100MB", "300MB", "650MB (CD)", "700MB (CD)", "4GB (FAT32)", "4.7GB (DVE)", qsTr("自定义")]
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
                                                model: [qsTr("字节"), "KB", "MB", "GB"]
                                            }
                                        }
                                    }
                                    Text {
                                        Layout.fillWidth: true
                                        text: volume.currentIndex === 13 && root.volumeBytes < 0 ? qsTr("请输入正整数，每卷上限为 24 GiB，最多 1024 卷。") : volume.currentIndex > 0 ? qsTr("输出 .zip.001 等分卷，现有分卷只能整组改名。") : qsTr("级别 0 不压缩，其余级别使用 Deflate。")
                                        font.pixelSize: Theme.operationCreateNoteSize
                                        color: Theme.textSecondary
                                    }
                                }
                            }
                            OperationCheck {
                                id: usePassword
                                objectName: "createUsePassword"
                                text: qsTr("输入密码")
                                density: root.density
                                onCheckedChanged: if (!checked) password.text = ""
                            }
                            OperationField {
                                id: password
                                objectName: "createPassword"
                                Layout.fillWidth: true
                                enabled: usePassword.checked
                                density: root.density
                                implicitHeight: Math.max(Theme.operationCreateFieldHeight, contentHeight + 8)
                                echoMode: TextInput.Password
                                placeholderText: qsTr("1–128 个 UTF-8 字节，不含控制字符")
                                Accessible.name: qsTr("压缩文件密码")
                            }
                            Text {
                                Layout.fillWidth: true
                                text: root.passwordValid ? qsTr("AES-256 加密文件内容，文件名仍可见。") : qsTr("请输入有效密码；密码不得为空或包含控制字符。")
                                wrapMode: Text.Wrap
                                font.pixelSize: Theme.operationCreateNoteSize
                                color: root.passwordValid ? Theme.textSecondary : Theme.menuDanger
                            }
                        }
                    }
                    OperationCheck {
                        id: openAfter
                        objectName: "createOpenAfter"
                        density: root.density
                        text: qsTr("完成后打开目标文件夹")
                        checked: true
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
            note: root.inputs.length ? qsTr("准备就绪") : qsTr("等待添加内容")
            noteIcon: Theme.dark ? "qrc:/xdzip/assets/dark/shield.png" : "qrc:/xdzip/assets/light/shield.png"
            OperationFooterButton {
                objectName: "createCancel"
                text: qsTr("取消")
                onClicked: root.close()
            }
            OperationFooterButton {
                objectName: "startCreate"
                text: qsTr("创建压缩包")
                primary: true
                enabled: root.inputs.length > 0 && root.validName && root.targetReady && root.optionsValid && !root.controller.busy
                onClicked: {
                    root.controller.startCreation(root.inputs.map(item => item.path), target.text, fileName.text.trim(), level.currentIndex === 0 ? 0 : 8, [6, 1, 6, 9][level.currentIndex], openAfter.checked, root.volumeBytes, usePassword.checked ? password.text : "");
                    password.text = "";
                }
            }
        }
    }
}
