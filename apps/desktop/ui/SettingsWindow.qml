pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Xdzip.Settings 1.0

OperationWindow {
    id: root
    objectName: "settingsWindow"
    title: qsTrId("settings.title")
    signal aboutRequested()
    signal restartConfirmationRequested()
    property string draftTheme: "system"
    property real draftScale: 1
    property string draftLanguage: "zh-CN"
    property string initialLanguage: "zh-CN"
    property bool saved: false
    property string draftDensity: "standard"
    property string draftOpenMode: "tabs"
    property bool draftTray: true
    property bool draftRecent: true
    property bool clearRecent: false
    function configure() {
        if (visible) { raise(); requestActivate(); return; }
        draftTheme = AppSettings.savedTheme;
        draftScale = AppSettings.savedTextScale;
        initialLanguage = AppSettings.language;
        draftLanguage = initialLanguage;
        draftDensity = AppSettings.savedDensity;
        draftRecent = AppSettings.recentEnabled;
        draftTray = AppSettings.trayEnabled;
        draftOpenMode = AppSettings.openMode;
        clearRecent = false;
        saved = false;
        syncControls();
        present();
    }
    function syncControls() {
        if (!scaleSelect || !languageSelect || !densitySelect) return;
        openModeSelect.currentIndex = draftOpenMode === "windows" ? 1 : 0;
        densitySelect.currentIndex = draftDensity === "compact" ? 1 : 0;
        scaleSelect.currentIndex = [1, 1.25, 1.5].indexOf(draftScale);
        languageSelect.currentIndex = draftLanguage === "en-US" ? 1 : 0;
    }
    function preview() { if (visible) AppSettings.preview(draftTheme, draftScale); }
    onDraftThemeChanged: { syncControls(); preview(); }
    onDraftScaleChanged: { syncControls(); preview(); }
    onDraftLanguageChanged: syncControls()
    onDraftDensityChanged: { syncControls(); if (visible) AppSettings.previewDensity(draftDensity); }
    onClosing: { if (!saved) AppSettings.cancelPreview(); }
    onActiveFocusItemChanged: if (visible) Qt.callLater(() => root.revealFocus(scroll))
    width: Theme.settingsLogicalWidth
    requestedHeight: Math.min(Theme.settingsMaximumHeight, Screen.desktopAvailableHeight - 80,
        body.implicitHeight + footer.implicitHeight)
    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        ScrollView {
            id: scroll
            objectName: "settingsScroll"
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            contentWidth: availableWidth
            ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
            ColumnLayout {
                id: body
                width: scroll.availableWidth
                spacing: Theme.operationHeaderBodyGap
                OperationHeader {
                    Layout.fillWidth: true
                    source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark" : "light") + "/settings-tile.png"
                    title: qsTrId("settings.title")
                    subtitle: qsTrId("settings.subtitle")
                }
                ColumnLayout {
                    objectName: "operationHeaderBody"
                    Layout.fillWidth: true
                    Layout.bottomMargin: Theme.settingsContentInset
                    spacing: Theme.settingsSectionGap
                    ColumnLayout {
                        Layout.leftMargin: Theme.settingsContentInset; Layout.rightMargin: Theme.settingsContentInset; Layout.fillWidth: true
                        spacing: Theme.settingsRowGap
                        Text { text: qsTrId("settings.theme"); font.pixelSize: Theme.settingsBodySize * Theme.fontScale; font.bold: true; color: Theme.text }
                        RowLayout {
                            Layout.fillWidth: true
                            spacing: Theme.settingsThemeCardGap
                            Repeater {
                                model: ["system", "light", "dark"]
                                delegate: IntentRadioButton {
                                    id: card
                                    required property string modelData
                                    required property int index
                                    objectName: "settingsTheme-" + index
                                    Layout.fillWidth: true
                                    implicitHeight: Theme.settingsThemeArtworkHeight + choice.implicitHeight + Theme.settingsThemeCardVerticalInset
                                    checked: root.draftTheme === modelData
                                    Accessible.name: [qsTrId("settings.system"), qsTrId("settings.light"), qsTrId("settings.dark")][index]
                                    onSelectionRequested: root.draftTheme = modelData
                                    background: OperationFrame {
                                        control: card
                                        hovered: card.hovered
                                        pressed: card.down
                                        color: card.down ? Theme.controlPressed : card.checked ? Theme.selection : card.hovered ? Theme.operationButtonSurface : Theme.surface
                                        border.color: card.checked ? Theme.accent : Theme.operationFieldBorder
                                    }
                                    contentItem: Item {
                                        Image {
                                            anchors.horizontalCenter: parent.horizontalCenter
                                            y: 8
                                            width: Theme.settingsThemeArtworkWidth; height: Theme.settingsThemeArtworkHeight
                                            source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark" : "light") + "/theme-" + card.modelData + ".png"
                                            fillMode: Image.PreserveAspectFit
                                            Accessible.ignored: true
                                        }
                                        Row {
                                            id: choice
                                            anchors.horizontalCenter: parent.horizontalCenter
                                            anchors.bottom: parent.bottom
                                            anchors.bottomMargin: 10
                                            spacing: 8
                                            Rectangle {
                                                anchors.verticalCenter: parent.verticalCenter
                                                width: 16; height: 16; radius: 8; color: "transparent"
                                                border.width: 1; border.color: card.checked ? Theme.accent : Theme.textSecondary
                                                Rectangle { anchors.centerIn: parent; width: 10; height: 10; radius: 5; color: Theme.accent; visible: card.checked }
                                            }
                                            Text { width: Math.min(implicitWidth, card.width - 48); text: [qsTrId("settings.system"), qsTrId("settings.light"), qsTrId("settings.dark")][card.index]; font.pixelSize: Theme.settingsBodySize * Theme.fontScale; color: Theme.text; wrapMode: Text.Wrap }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    RowLayout {
                        Layout.leftMargin: Theme.settingsContentInset; Layout.rightMargin: Theme.settingsContentInset; Layout.fillWidth: true
                        Text { Layout.fillWidth: true; text: qsTrId("settings.density"); font.pixelSize: Theme.settingsBodySize * Theme.fontScale; color: Theme.text; wrapMode: Text.Wrap }
                        OperationSelect {
                            id: densitySelect
                            objectName: "settingsDensity"
                            Accessible.name: qsTrId("settings.density")
                            density: Theme.settingsBodySize * Theme.fontScale / Theme.operationBodySize
                            implicitHeight: Math.max(Theme.operationControlBodyButtonHeight, contentItem.implicitHeight + 8)
                            model: [qsTrId("settings.standard"), qsTrId("settings.compact")]
                            currentIndex: root.draftDensity === "compact" ? 1 : 0
                            onCurrentIndexChanged: if (currentIndex >= 0) root.draftDensity = ["standard", "compact"][currentIndex]
                        }
                    }
                    RowLayout {
                        Layout.leftMargin: Theme.settingsContentInset; Layout.rightMargin: Theme.settingsContentInset; Layout.fillWidth: true
                        Text { Layout.fillWidth: true; text: qsTrId("settings.textSize"); font.pixelSize: Theme.settingsBodySize * Theme.fontScale; color: Theme.text; wrapMode: Text.Wrap }
                        OperationSelect {
                            id: scaleSelect
                            objectName: "settingsTextScale"
                            Accessible.name: qsTrId("settings.textSize")
                            density: Theme.settingsBodySize * Theme.fontScale / Theme.operationBodySize
                            implicitHeight: Math.max(Theme.operationControlBodyButtonHeight, contentItem.implicitHeight + 8)
                            model: ["100%", "125%", "150%"]
                            currentIndex: [1, 1.25, 1.5].indexOf(root.draftScale)
                            onCurrentIndexChanged: if (currentIndex >= 0) root.draftScale = [1, 1.25, 1.5][currentIndex]
                        }
                    }
                    ColumnLayout {
                        Layout.leftMargin: Theme.settingsContentInset; Layout.rightMargin: Theme.settingsContentInset; Layout.fillWidth: true
                        spacing: 6
                        RowLayout {
                            Layout.fillWidth: true
                            Text { Layout.fillWidth: true; text: qsTrId("settings.language"); font.pixelSize: Theme.settingsBodySize * Theme.fontScale; color: Theme.text; wrapMode: Text.Wrap }
                            OperationSelect {
                                id: languageSelect
                                objectName: "settingsLanguage"
                                Accessible.name: qsTrId("settings.language")
                                density: Theme.settingsBodySize * Theme.fontScale / Theme.operationBodySize
                                implicitHeight: Math.max(Theme.operationControlBodyButtonHeight, contentItem.implicitHeight + 8)
                                model: [qsTrId("language.zhCN"), qsTrId("language.enUS")]
                                currentIndex: root.draftLanguage === "en-US" ? 1 : 0
                                onCurrentIndexChanged: if (currentIndex >= 0) root.draftLanguage = ["zh-CN", "en-US"][currentIndex]
                            }
                        }
                        Text { Layout.fillWidth: true; text: qsTrId("ui.3a593f5653b1"); font.pixelSize: Theme.settingsNoteSize * Theme.fontScale; color: Theme.textSecondary; wrapMode: Text.Wrap }
                    }
                    RowLayout {
                        Layout.leftMargin: Theme.settingsContentInset; Layout.rightMargin: Theme.settingsContentInset; Layout.fillWidth: true
                        Text { Layout.fillWidth: true; text: qsTrId("sessions.openMode"); font.pixelSize: Theme.settingsBodySize * Theme.fontScale; color: Theme.text; wrapMode: Text.Wrap }
                        OperationSelect {
                            id: openModeSelect
                            objectName: "settingsOpenMode"
                            Accessible.name: qsTrId("sessions.openMode")
                            density: Theme.settingsBodySize * Theme.fontScale / Theme.operationBodySize
                            implicitHeight: Math.max(Theme.operationControlBodyButtonHeight, contentItem.implicitHeight + 8)
                            model: [qsTrId("sessions.tabs"), qsTrId("sessions.windows")]
                            currentIndex: root.draftOpenMode === "windows" ? 1 : 0
                            onActivated: root.draftOpenMode = currentIndex === 1 ? "windows" : "tabs"
                        }
                    }
                    OperationCheck {
                        id: trayToggle
                        objectName: "settingsTray"
                        text: qsTrId("desktop.trayEnabled")
                        Layout.leftMargin: Theme.settingsContentInset; Layout.rightMargin: Theme.settingsContentInset; Layout.fillWidth: true
                        density: Theme.settingsBodySize * Theme.fontScale / Theme.operationBodySize
                        checked: root.draftTray
                        onToggled: root.draftTray = checked
                    }
                    OperationCheck {
                        id: recentToggle
                        objectName: "settingsRecent"
                        text: qsTrId("settings.recent")
                        Layout.leftMargin: Theme.settingsContentInset; Layout.rightMargin: Theme.settingsContentInset; Layout.fillWidth: true
                        density: Theme.settingsBodySize * Theme.fontScale / Theme.operationBodySize
                        checked: root.draftRecent
                        onToggled: root.draftRecent = checked
                    }
                    OperationButton {
                        objectName: "settingsClearRecent"
                        Layout.leftMargin: Theme.settingsContentInset
                        text: root.clearRecent ? qsTrId("settings.clearPending") : qsTrId("settings.clearRecent")
                        enabled: !root.clearRecent
                        onClicked: root.clearRecent = true
                    }
                    Rectangle {
                        Layout.leftMargin: Theme.settingsContentInset; Layout.rightMargin: Theme.settingsContentInset; Layout.fillWidth: true
                        implicitHeight: privacy.implicitHeight + 28
                        radius: Theme.operationControlRadius; color: Theme.operationCard; border.color: Theme.taskBorder
                        Text {
                            id: privacy
                            anchors.fill: parent; anchors.margins: 14
                            text: qsTrId("settings.privacy"); font.pixelSize: Theme.settingsNoteSize * Theme.fontScale; color: Theme.text; wrapMode: Text.Wrap
                        }
                    }
                    OperationButton {
                        objectName: "settingsAbout"
                        Layout.leftMargin: Theme.settingsContentInset
                        text: qsTrId("about.title")
                        onClicked: root.aboutRequested()
                    }
                    Text {
                        Layout.leftMargin: Theme.settingsContentInset; Layout.rightMargin: Theme.settingsContentInset; Layout.fillWidth: true
                        visible: AppSettings.error.length > 0
                        text: AppSettings.error
                        color: Theme.menuDanger; font.pixelSize: Theme.settingsNoteSize * Theme.fontScale; wrapMode: Text.Wrap
                    }
                }
            }
        }
        OperationFooter {
            id: footer
            objectName: "settingsFooter"
            Layout.fillWidth: true
            note: qsTrId("ui.5fb5c0083f18")
            noteIcon: "qrc:/xdzip/assets/" + (Theme.dark ? "dark" : "light") + "/shield.png"
            OperationFooterButton {
                objectName: "settingsDefaults"
                text: qsTrId("settings.defaults")
                onClicked: { root.draftTheme = "system"; root.draftScale = 1; root.draftLanguage = "zh-CN"; root.draftDensity = "standard"; root.draftRecent = true; root.draftTray = true; root.draftOpenMode = "tabs"; root.clearRecent = false; root.syncControls(); }
            }
            OperationFooterButton { objectName: "settingsCancel"; text: qsTrId("common.cancel"); onClicked: root.close() }
            OperationFooterButton {
                objectName: "settingsSave"
                text: qsTrId("settings.done")
                onClicked: {
                    const restartNeeded = root.draftLanguage !== root.initialLanguage && root.draftLanguage !== AppSettings.activeLanguage;
                    if (AppSettings.saveDetails(root.draftTheme, root.draftScale, root.draftLanguage, root.draftDensity, root.draftRecent, root.clearRecent, root.draftTray, root.draftOpenMode)) {
                        root.saved = true;
                        root.close();
                        if (restartNeeded) root.restartConfirmationRequested();
                    }
                }
            }
        }
    }
}
