pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

OperationWindow {
    id: root
    objectName: "extractWindow"
    required property var controller
    property var intent: controller.extractionIntent
    property var targetState: controller.targetCheck
    property bool allFiles: true
    readonly property real density: Theme.operationExtractDensity
    function unit(value: real): real { return value * density; }
    width: Theme.operationExtractWidth
    requestedHeight: Math.min(Math.round(unit(Theme.operationExtractClientHeight)), Screen.desktopAvailableHeight - 80)
    title: qsTr("解压文件 — xdzip")
    readonly property string archiveFolder: String(intent.archive || "").replace(/\.[^.]+$/, "")
    readonly property string destinationPreview: target.text + (sameFolder.checked && target.text
        ? (target.text.includes("\\") ? "\\" : "/") + archiveFolder : "")
    function configure() {
        allFiles = Boolean(intent.all);
        target.text = intent.target || "";
        sameFolder.checked = true;
        conflict.currentIndex = 0;
        highRatio.checked = false;
        openAfter.checked = true;
        present();
        controller.checkTarget(target.text);
    }
    readonly property bool targetReady: Boolean(targetState.valid) && controller.targetMatches(target.text)
    Connections {
        target: root
        function onActiveFocusItemChanged() { root.revealFocus(body); }
    }
    ColumnLayout {
        anchors.fill: parent; spacing: 0
        ScrollView {
            id: body
            objectName: "extractScrollView"
            Layout.fillWidth: true; Layout.fillHeight: true
            contentWidth: availableWidth; clip: true
            ColumnLayout {
                width: body.availableWidth; spacing: 0
                RowLayout {
                    Layout.fillWidth: true
                    Layout.leftMargin: root.unit(Theme.operationGutter); Layout.rightMargin: root.unit(Theme.operationGutter)
                    Layout.topMargin: root.unit(21); Layout.bottomMargin: root.unit(16); spacing: root.unit(20)
                    OperationHero {
                        source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "archive-large.png"
                        canvasWidth: root.unit(70); canvasHeight: root.unit(81); artworkWidth: root.unit(66); artworkHeight: root.unit(79)
                        Layout.preferredWidth: canvasWidth; Layout.preferredHeight: canvasHeight
                    }
                    ColumnLayout {
                        Layout.fillWidth: true; spacing: root.unit(7)
                        Text { text: qsTr("解压文件"); font.pixelSize: Math.round(root.unit(Theme.operationTitleSize)); lineHeight: Math.round(Math.round(root.unit(Theme.operationTitleSize)) * 1.35); lineHeightMode: Text.FixedHeight; font.bold: true; color: Theme.text }
                        Text {
                            Layout.fillWidth: true
                            text: root.intent.archive || ""; textFormat: Text.PlainText
                            font.pixelSize: Math.round(root.unit(Theme.operationBodySize)); color: Theme.textSecondary; elide: Text.ElideMiddle
                        }
                    }
                }
                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.leftMargin: root.unit(Theme.operationExtractContentInset); Layout.rightMargin: root.unit(Theme.operationExtractContentInset)
                    Layout.bottomMargin: root.unit(20); spacing: root.unit(Theme.operationCardGap)
                    Rectangle {
                        objectName: "extractDestinationCard"
                        Layout.fillWidth: true; implicitHeight: targetContent.implicitHeight + root.unit(32)
                        color: Theme.operationCard; radius: root.unit(Theme.operationFieldRadius); border.color: Theme.operationCardBorder
                        ColumnLayout {
                            id: targetContent
                            anchors.fill: parent; anchors.margins: root.unit(Theme.operationCardInset); spacing: root.unit(10)
                            Text { text: qsTr("解压到"); font.pixelSize: Math.round(root.unit(Theme.operationBodySize)); font.bold: true; color: Theme.text }
                            Rectangle {
                                objectName: "extractDestinationField"
                                Layout.fillWidth: true; implicitHeight: Math.max(root.unit(44), target.contentHeight + root.unit(10), changeTarget.implicitHeight + root.unit(10))
                                radius: root.unit(6); color: Theme.surface
                                border.color: target.activeFocus ? Theme.accent : Theme.operationFieldBorder
                                border.width: target.activeFocus ? 2 : 1
                                RowLayout {
                                    anchors.fill: parent; anchors.margins: root.unit(5); spacing: root.unit(12)
                                    // Match the right inset to the actual vertical clearance,
                                    // including the height required by enlarged text.
                                    anchors.rightMargin: (parent.height - changeTarget.height) / 2
                                    Image { source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "folder.png"; Layout.preferredWidth: root.unit(27); Layout.preferredHeight: root.unit(27); Layout.leftMargin: root.unit(6) }
                                    OperationField {
                                        id: target
                                        objectName: "extractTarget"
                                        density: root.density; framed: false
                                        previewText: root.destinationPreview
                                        Layout.fillWidth: true; Layout.fillHeight: true
                                        Accessible.name: qsTr("目标目录")
                                    }
                                    AppButton {
                                        id: changeTarget
                                        objectName: "extractChangeTarget"
                                        verticalInset: 2.5; surfaceColor: Theme.operationButtonSurface; outlineColor: Theme.operationFieldBorder
                                        text: qsTr("更改"); labelSize: Math.round(root.unit(Theme.operationBodySize)); baseHeight: root.unit(30); cornerRadius: root.unit(6); horizontalInset: root.unit(17); minimumWidth: root.unit(86)
                                        enabled: !root.controller.busy
                                        onClicked: { const path = root.controller.chooseTarget(target.text); if (path) { target.text = path; root.controller.checkTarget(path); } }
                                    }
                                }
                            }
                            OperationCheck {
                                density: root.density; spacing: Theme.operationDestinationIndicatorGap
                                indicatorSlotSize: Theme.operationDestinationIndicatorSlotSize
                                id: sameFolder; objectName: "extractSameFolder"; Layout.fillWidth: true; text: qsTr("创建同名文件夹"); checked: true }
                            RowLayout {
                                Layout.fillWidth: true; spacing: Theme.operationDestinationIndicatorGap
                                Item {
                                    objectName: "extractTargetStatusIndicator"
                                    Layout.preferredWidth: Theme.operationDestinationIndicatorSlotSize; Layout.preferredHeight: Theme.operationDestinationIndicatorSlotSize
                                    Image {
                                        anchors.centerIn: parent; visible: root.targetReady
                                        source: "qrc:/xdzip/assets/success-large.png"
                                        // The shared 24-unit canvas contains a 20-unit disc.
                                        width: Theme.operationDestinationIndicatorSize * 24 / 20; height: width
                                    }
                                }
                                Text {
                                    objectName: "extractTargetStatusText"
                                    Layout.fillWidth: true; Layout.minimumHeight: Theme.operationDestinationIndicatorSlotSize
                                    verticalAlignment: Text.AlignVCenter
                                    text: root.targetState.checking ? qsTr("正在检查目标目录…") : root.targetReady ? qsTr("目标目录可写") : root.targetState.error || qsTr("修改目录后请重新检查")
                                    textFormat: Text.PlainText; wrapMode: Text.Wrap
                                    color: root.targetState.error && !root.targetState.checking && !root.targetReady ? Theme.menuDanger : Theme.textSecondary; font.pixelSize: Math.round(root.unit(Theme.operationBodySize))
                                }
                                AppButton { objectName: "extractCheckTarget"; visible: !root.targetReady && !root.targetState.checking; text: qsTr("检查目标"); labelSize: Theme.operationExtractNoteSize; enabled: !root.controller.busy; onClicked: root.controller.checkTarget(target.text) }
                            }
                            Text {
                                visible: Number(root.targetState.leftovers || 0) > 0
                                Layout.fillWidth: true
                                text: qsTr("发现 %1 个可能的暂存目录，将保留供检查。").arg(root.targetState.leftovers)
                                wrapMode: Text.Wrap; font.pixelSize: Theme.operationExtractNoteSize; color: Theme.textSecondary
                            }
                        }
                    }
                    Rectangle {
                        objectName: "extractScopeCard"
                        Layout.fillWidth: true; implicitHeight: scopeContent.implicitHeight + root.unit(32)
                        color: Theme.surface; radius: root.unit(Theme.operationFieldRadius); border.color: Theme.operationCardBorder
                        Rectangle { x: 1; y: 1; width: parent.width - 2; height: Math.max(root.unit(42), scopeHeading.implicitHeight + root.unit(18)) - 1; radius: root.unit(Theme.operationFieldRadius); bottomLeftRadius: 0; bottomRightRadius: 0; color: Theme.operationCard }
                        ColumnLayout {
                            id: scopeContent
                            anchors.fill: parent; anchors.margins: root.unit(Theme.operationCardInset); spacing: root.unit(10)
                            Text { id: scopeHeading; Layout.topMargin: -5.5; Layout.bottomMargin: 5.5; text: qsTr("解压范围"); font.pixelSize: Math.round(root.unit(Theme.operationBodySize)); font.bold: true; color: Theme.text }
                            OperationCheck {
                                density: root.density; spacing: 13;
                                objectName: "extractSelectedScope"
                                visible: Number(root.intent.selectedCount || 0) > 0
                                Layout.fillWidth: true; radioStyle: true
                                text: qsTr("所选文件和目录，%1 项").arg(root.intent.selectedCount || 0)
                                checked: !root.allFiles
                                onClicked: root.allFiles = false
                            }
                            Rectangle {
                                visible: !root.allFiles
                                Layout.fillWidth: true; Layout.leftMargin: root.unit(28)
                                implicitHeight: sampleContent.implicitHeight
                                color: Theme.surface; radius: 6; border.color: Theme.operationCardBorder
                                ColumnLayout {
                                    id: sampleContent
                                    anchors.fill: parent; spacing: 0
                                    Repeater {
                                        model: root.intent.samples || []
                                        RowLayout {
                                            id: sampleRow
                                            required property var modelData
                                            Layout.fillWidth: true; Layout.minimumHeight: root.unit(45)
                                            Layout.leftMargin: root.unit(10); Layout.rightMargin: root.unit(10); spacing: root.unit(10)
                                            Image { source: AssetLibrary.fileIcon(String(sampleRow.modelData.name), Boolean(sampleRow.modelData.directory), false); Layout.preferredWidth: root.unit(30); Layout.preferredHeight: root.unit(30) }
                                            Text { Layout.fillWidth: true; text: sampleRow.modelData.name; textFormat: Text.PlainText; elide: Text.ElideMiddle; color: Theme.text; font.pixelSize: Math.round(root.unit(Theme.operationBodySize)) }
                                        }
                                    }
                                }
                            }
                            OperationCheck {
                                density: root.density; spacing: 13;
                                objectName: "extractAllScope"
                                Layout.fillWidth: true; radioStyle: true
                                text: qsTr("全部归档，%1 条记录").arg(root.intent.archiveCount || 0)
                                checked: root.allFiles
                                onClicked: root.allFiles = true
                            }
                        }
                    }
                    Rectangle {
                        objectName: "extractConflictCard"
                        Layout.fillWidth: true; implicitHeight: policyContent.implicitHeight + root.unit(32)
                        color: Theme.operationCard; radius: root.unit(Theme.operationFieldRadius); border.color: Theme.operationCardBorder
                        ColumnLayout {
                            id: policyContent
                            anchors.fill: parent; anchors.margins: root.unit(Theme.operationCardInset); spacing: root.unit(10)
                            Text { text: qsTr("重名文件"); font.pixelSize: Math.round(root.unit(Theme.operationBodySize)); font.bold: true; color: Theme.text }
                            OperationSelect {
                                density: root.density;
                                id: conflict; objectName: "extractConflictPolicy"
                                Layout.fillWidth: true
                                model: [qsTr("询问我如何处理"), qsTr("跳过已有文件"), qsTr("自动改名，保留两个版本"), qsTr("覆盖已有文件")]
                            }
                            Text { Layout.fillWidth: true; text: qsTr("只处理普通文件；不会覆盖源归档或跟随链接。"); wrapMode: Text.Wrap; font.pixelSize: Theme.operationExtractNoteSize; color: Theme.textSecondary }
                        }
                    }
                    OperationCheck {
                                density: root.density; spacing: 13; id: openAfter; objectName: "extractOpenAfter"; Layout.fillWidth: true; text: qsTr("完成后打开目标文件夹") }
                    OperationCheck {
                                density: root.density; spacing: 13; id: highRatio; objectName: "extractHighRatio"; Layout.fillWidth: true; text: qsTr("允许高压缩比文件（仍受字节限额约束）") }
                    Text {
                        Layout.fillWidth: true
                        visible: Boolean(root.intent.error)
                        text: root.intent.error || ""; textFormat: Text.PlainText
                        font.pixelSize: Theme.operationExtractNoteSize; color: Theme.menuDanger; wrapMode: Text.Wrap
                    }
                }
            }
        }
        OperationFooter {
            Layout.fillWidth: true
            contentInset: root.unit(24); verticalInset: root.unit(13); minimumFooterHeight: root.unit(67)
            noteGap: 4; noteIcon: "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + "shield.png"
            note: root.targetReady ? qsTr("准备就绪") : qsTr("请检查目标目录")
            OperationFooterButton { text: qsTr("取消"); onClicked: root.close() }
            OperationFooterButton {
                objectName: "startExtract"
                text: qsTr("开始解压"); primary: true
                enabled: root.targetReady && !root.controller.busy && root.intent.session === root.controller.sessionId
                onClicked: root.controller.startExtraction(target.text, root.allFiles, sameFolder.checked, conflict.currentIndex, highRatio.checked, openAfter.checked)
            }
        }
    }
}
