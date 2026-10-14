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
    function baseName(path: string): string {
        return path.replace(/\\/g, "/").split("/").pop();
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
        method.currentIndex = 0;
        level.currentIndex = 0;
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
                                                AppButton {
                                                    objectName: "createRemoveInput-" + inputRow.index
                                                    text: qsTr("移除")
                                                    labelSize: Theme.operationCreateNoteSize
                                                    baseHeight: 26
                                                    minimumWidth: 44
                                                    horizontalInset: 5
                                                    verticalInset: 3
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
                                        text: qsTr("%1 个选择项 · 目录将在开始前扫描").arg(root.inputs.length)
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
                                            AppButton {
                                                objectName: "createChangeTarget"
                                                baseHeight: 28
                                                verticalInset: 4
                                                text: qsTr("更改")
                                                labelSize: Theme.operationCreateBodySize
                                                horizontalInset: 12
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
                                        RowLayout {
                                            Layout.fillWidth: true
                                            spacing: 0
                                            Repeater {
                                                model: ["ZIP", "TAR", "TAR.GZ"]
                                                AppButton {
                                                    required property string modelData
                                                    required property int index
                                                    Layout.fillWidth: true
                                                    Layout.preferredWidth: 1
                                                    text: modelData
                                                    labelSize: Theme.operationCreateBodySize
                                                    baseHeight: 28
                                                    verticalInset: 4
                                                    horizontalInset: 3
                                                    primary: index === 0
                                                    enabled: index === 0
                                                    explanation: index === 0 ? "" : qsTr("此格式将在后续版本提供")
                                                }
                                            }
                                        }
                                        Text {
                                            text: qsTr("压缩算法")
                                            font.pixelSize: Theme.operationCreateBodySize
                                            color: Theme.text
                                        }
                                        OperationSelect {
                                            id: method
                                            objectName: "createMethod"
                                            Layout.fillWidth: true
                                            density: root.density
                                            implicitHeight: Math.max(Theme.operationCreateFieldHeight, contentItem.implicitHeight + 8)
                                            model: ["Deflate", qsTr("Store · 不压缩")]
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
                                            enabled: method.currentIndex === 0
                                            model: [qsTr("均衡 (6)"), qsTr("快速 (1)"), qsTr("更小 (9)")]
                                        }
                                    }
                                    Text {
                                        Layout.fillWidth: true
                                        text: method.currentIndex === 0 ? qsTr("均衡兼顾速度与压缩率。") : qsTr("Store 保留原始字节，不进行压缩。")
                                        font.pixelSize: Theme.operationCreateNoteSize
                                        color: Theme.textSecondary
                                    }
                                }
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
            AppButton {
                text: qsTr("取消")
                labelSize: Theme.operationFooterFontSize
                baseHeight: 32
                minimumWidth: 77
                horizontalInset: 12
                onClicked: root.close()
            }
            AppButton {
                objectName: "startCreate"
                text: qsTr("创建压缩包")
                labelSize: Theme.operationFooterFontSize
                baseHeight: 32
                minimumWidth: 101
                horizontalInset: 12
                primary: true
                enabled: root.inputs.length > 0 && root.validName && root.targetReady && !root.controller.busy
                onClicked: root.controller.startCreation(root.inputs.map(item => item.path), target.text, fileName.text.trim(), method.currentIndex === 0 ? 8 : 0, [6, 1, 9][level.currentIndex], openAfter.checked)
            }
        }
    }
}
