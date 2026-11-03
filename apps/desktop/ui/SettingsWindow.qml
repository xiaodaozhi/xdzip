pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Xdzip.Settings 1.0

OperationWindow {
    id: root
    objectName: "settingsWindow"
    title: qsTrId("settings.title")
    property int currentTab: 0
    property string pendingProxyMode: "system"
    property bool copiedVersion: false
    property var updater: typeof UpdateService === "undefined" ? null : UpdateService
    readonly property var associations: typeof FileAssociationService === "undefined" ? null : FileAssociationService
    readonly property string updateState: updater ? updater.state : "unavailable"

    function configure() { configureTab(0); }
    function configureTab(tabIndex) {
        currentTab = tabIndex;
        pendingProxyMode = AppSettings.proxyMode;
        if (visible) { raise(); requestActivate(); return; }
        present();
    }
    function commit(theme, scale, language, density, recent, tray, openMode) {
        return AppSettings.applyDetails(theme, scale, language, density, recent, tray, openMode);
    }
    function chooseProxy(mode) {
        if (mode === "custom" && !AppSettings.proxyAddress.length) {
            pendingProxyMode = mode;
            return;
        }
        if (AppSettings.applyNetwork(mode, AppSettings.proxyAddress, AppSettings.mirrorAddress))
            pendingProxyMode = mode;
    }
    function updateMessage() {
        switch (updateState) {
        case "checking": return qsTrId("update.checking");
        case "up_to_date": return qsTrId("update.current");
        case "updated": return qsTrId("update.updated");
        case "available": return qsTrId("update.available").arg(updater.candidateVersion);
        case "downloading": return qsTrId("update.downloading");
        case "verifying": return qsTrId("update.verifying");
        case "downloaded": return qsTrId("update.downloaded");
        case "waiting_tasks": return qsTrId("update.waitingTasks");
        case "preparing": return qsTrId("update.preparing");
        case "installing": return qsTrId("update.installing");
        case "recovery_required": return qsTrId("update.recovery");
        case "error": {
            const code = updater.errorCode;
            if (code === "rate_limited") return qsTrId("update.rateLimited");
            if (code === "tls") return qsTrId("update.tls");
            if (code === "network" || code === "download") return qsTrId("update.network");
            if (code === "timeout" || code === "authorization_timeout") return qsTrId("update.timeout");
            if (code === "authorization_cancelled") return qsTrId("update.authorizationCancelled");
            if (code.startsWith("authorization_") || code === "helper_exit") return qsTrId("update.authorizationFailed");
            if (code === "disk" || code === "cache") return qsTrId("update.disk");
            if (code === "unsafe_installation") return qsTrId("update.unsafeInstallation");
            if (code === "minimum_system") return qsTrId("update.minimumSystem");
            if (code.includes("signature") || code.includes("hash") || code === "download_size") return qsTrId("update.verificationFailed");
            return qsTrId("update.failed");
        }
        case "unavailable": return qsTrId("update.preview");
        default: return qsTrId("update.idle");
        }
    }
    // Disabling and reenabling the update button transfers focus through the page Loader.
    // Mouse focus and automatic focus restoration must not reposition the page.
    onActiveFocusItemChanged: if (visible && activeFocusItem && activeFocusItem !== pageLoader) {
        const focusedItem = activeFocusItem;
        const updateButtonWithoutKeyboardNavigation = focusedItem.objectName === "aboutCheckUpdate" &&
            (focusedItem.focusReason === Qt.MouseFocusReason || focusedItem.focusReason === Qt.OtherFocusReason);
        if (!updateButtonWithoutKeyboardNavigation) Qt.callLater(() => {
            if (root.visible && root.activeFocusItem === focusedItem) root.revealFocus(pageScroll);
        });
    }
    onCurrentTabChanged: if (currentTab === 3 && associations) associations.refresh()
    width: Theme.settingsLogicalWidth
    requestedHeight: Math.min(Theme.settingsClientHeight, Screen.desktopAvailableHeight - 80)

    component PageTitle: Text {
        Layout.fillWidth: true
        color: Theme.text
        font.pixelSize: Math.round(Theme.settingsTitleSize * Theme.fontScale)
        font.bold: true
        wrapMode: Text.Wrap
    }
    component SectionTitle: Text {
        Layout.fillWidth: true
        color: Theme.text
        font.pixelSize: Math.round(Theme.settingsBodySize * Theme.fontScale)
        font.bold: true
        wrapMode: Text.Wrap
    }
    component PageNote: Text {
        Layout.fillWidth: true
        color: Theme.textSecondary
        font.pixelSize: Math.round(Theme.settingsNoteSize * Theme.fontScale)
        textFormat: Text.PlainText
        wrapMode: Text.Wrap
    }
    component PageSeparator: Rectangle {
        Layout.fillWidth: true
        implicitHeight: 1
        color: Theme.border
    }
    component ProxyChoice: IntentRadioButton {
        id: proxyChoice
        required property string mode
        required property string label
        Layout.fillWidth: true
        implicitHeight: Math.max(28, proxyLabel.implicitHeight + 8)
        checked: root.pendingProxyMode === mode
        Accessible.name: label
        onSelectionRequested: root.chooseProxy(mode)
        contentItem: RowLayout {
            spacing: 8
            Rectangle {
                implicitWidth: 16; implicitHeight: 16; radius: 8
                color: "transparent"; border.width: 1
                border.color: proxyChoice.checked ? Theme.accent : Theme.textSecondary
                Rectangle { anchors.centerIn: parent; width: 10; height: 10; radius: 5; color: Theme.accent; visible: proxyChoice.checked }
            }
            Text {
                id: proxyLabel
                Layout.fillWidth: true
                text: proxyChoice.label; color: Theme.text
                font.pixelSize: Math.round(Theme.settingsBodySize * Theme.fontScale)
                wrapMode: Text.Wrap
            }
        }
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: Theme.operationHeaderBodyGap
        OperationHeader {
            Layout.fillWidth: true
            source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark" : "light") + "/settings-tile.png"
            title: qsTrId("settings.title")
            subtitle: qsTrId("settings.subtitle")
            Rectangle {
                objectName: "settingsHeaderDivider"
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                height: Theme.settingsHeaderDividerHeight
                color: Theme.border
            }
        }
        RowLayout {
            objectName: "operationHeaderBody"
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0
            Rectangle {
                objectName: "settingsSidebar"
                Layout.preferredWidth: Theme.settingsSidebarWidth
                Layout.fillHeight: true
                color: Theme.sidebar
                Rectangle { anchors.right: parent.right; width: 1; height: parent.height; color: Theme.border }
                ScrollView {
                    anchors.fill: parent
                    clip: true
                    contentWidth: availableWidth
                    ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
                    Column {
                        width: parent.width
                        Repeater {
                            model: [qsTrId("settings.tabAppearance"), qsTrId("settings.tabGeneral"),
                                    qsTrId("settings.tabPrivacy"), qsTrId("settings.tabAssociations"),
                                    qsTrId("settings.tabNetwork"), qsTrId("settings.tabAbout")]
                            delegate: Button {
                                id: tab
                                required property int index
                                required property string modelData
                                objectName: "settingsTab-" + index
                                width: parent ? parent.width : 0
                                implicitHeight: Math.max(Theme.settingsSidebarRowHeight, tabLabel.implicitHeight + 12)
                                leftPadding: Theme.settingsSidebarIconInset
                                hoverEnabled: true
                                checkable: true
                                checked: root.currentTab === index
                                Accessible.role: Accessible.PageTab
                                Accessible.name: modelData
                                onClicked: root.currentTab = index
                                background: Rectangle {
                                    color: tab.checked ? Theme.selection : tab.hovered ? Theme.controlHover : "transparent"
                                    Rectangle { visible: tab.checked; width: Theme.settingsSidebarSelectionWidth; height: parent.height; color: Theme.accent }
                                }
                                contentItem: RowLayout {
                                    spacing: Theme.settingsSidebarIconGap
                                    Image {
                                        objectName: "settingsTabIcon-" + tab.index
                                        Layout.preferredWidth: Theme.settingsSidebarIconSize
                                        Layout.preferredHeight: Theme.settingsSidebarIconSize
                                        fillMode: Image.PreserveAspectFit
                                        source: tab.index === 5 ? "qrc:/xdzip/assets/info-" + (Theme.dark ? "dark" : "light") + ".png" :
                                            "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") +
                                            ["view-list", "settings", "shield", "file-text", "external-open"][tab.index] + ".png"
                                        Accessible.ignored: true
                                    }
                                    Text {
                                        id: tabLabel
                                        Layout.fillWidth: true
                                        Layout.rightMargin: 8
                                        text: tab.modelData
                                        textFormat: Text.PlainText
                                        font.pixelSize: Math.round(Theme.settingsBodySize * Theme.fontScale)
                                        font.bold: tab.checked
                                        color: tab.checked ? Theme.accentInk : Theme.text
                                        wrapMode: Text.Wrap
                                    }
                                }
                            }
                        }
                    }
                }
            }
            ScrollView {
                id: pageScroll
                objectName: "settingsScroll"
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                contentWidth: availableWidth
                // Measure the About page independently of the changing update section.
                contentHeight: root.currentTab === 5 && pageLoader.item ?
                    pageLoader.item.stablePageHeight + Theme.settingsPageInset + Theme.settingsPageSectionGap +
                        (settingsError.visible ? settingsError.implicitHeight + Theme.settingsPageSectionGap : 0) :
                    pageContent.implicitHeight
                ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
                ScrollBar.vertical: ScrollBar { objectName: "settingsVerticalScrollBar" }
                ColumnLayout {
                    id: pageContent
                    objectName: "settingsPageContent"
                    width: pageScroll.availableWidth
                    spacing: Theme.settingsPageSectionGap
                    Item { Layout.preferredHeight: 0 }
                    Text {
                        id: settingsError
                        Layout.leftMargin: Theme.settingsPageInset
                        Layout.rightMargin: Theme.settingsPageInset
                        Layout.fillWidth: true
                        visible: AppSettings.error.length > 0
                        text: AppSettings.error
                        textFormat: Text.PlainText
                        color: Theme.menuDanger
                        font.pixelSize: Math.round(Theme.settingsNoteSize * Theme.fontScale)
                        wrapMode: Text.Wrap
                        Accessible.role: Accessible.AlertMessage
                    }
                    Loader {
                        id: pageLoader
                        Layout.leftMargin: Theme.settingsPageInset
                        Layout.rightMargin: Theme.settingsPageInset
                        Layout.bottomMargin: Theme.settingsPageInset
                        Layout.fillWidth: true
                        sourceComponent: [appearancePage, generalPage, privacyPage, associationsPage, networkPage, aboutPage][root.currentTab]
                    }
                }
            }
        }
    }

    Component {
        id: appearancePage
        ColumnLayout {
            objectName: "settingsPage-appearance"
            spacing: Theme.settingsPageSectionGap
            PageTitle { text: qsTrId("settings.tabAppearance") }
            PageSeparator {}
            SectionTitle { text: qsTrId("settings.theme") }
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
                        checked: AppSettings.savedTheme === modelData
                        Accessible.name: [qsTrId("settings.system"), qsTrId("settings.light"), qsTrId("settings.dark")][index]
                        onSelectionRequested: root.commit(modelData, AppSettings.savedTextScale, AppSettings.language, AppSettings.savedDensity, AppSettings.recentEnabled, AppSettings.trayEnabled, AppSettings.openMode)
                        background: OperationFrame {
                            control: card; hovered: card.hovered; pressed: card.down
                            color: card.down ? Theme.controlPressed : card.checked ? Theme.selection : card.hovered ? Theme.operationButtonSurface : Theme.surface
                            border.color: card.checked ? Theme.accent : Theme.operationFieldBorder
                        }
                        contentItem: Item {
                            Image {
                                anchors.horizontalCenter: parent.horizontalCenter; y: 8
                                width: Theme.settingsThemeArtworkWidth; height: Theme.settingsThemeArtworkHeight
                                source: "qrc:/xdzip/assets/" + (Theme.dark ? "dark" : "light") + "/theme-" + card.modelData + ".png"
                                fillMode: Image.PreserveAspectFit; Accessible.ignored: true
                            }
                            Row {
                                id: choice
                                anchors.horizontalCenter: parent.horizontalCenter
                                anchors.bottom: parent.bottom; anchors.bottomMargin: 10; spacing: 8
                                Rectangle {
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: 16; height: 16; radius: 8; color: "transparent"
                                    border.width: 1; border.color: card.checked ? Theme.accent : Theme.textSecondary
                                    Rectangle { anchors.centerIn: parent; width: 10; height: 10; radius: 5; color: Theme.accent; visible: card.checked }
                                }
                                Text {
                                    width: Math.min(implicitWidth, card.width - 48)
                                    text: [qsTrId("settings.system"), qsTrId("settings.light"), qsTrId("settings.dark")][card.index]
                                    font.pixelSize: Math.round(Theme.settingsBodySize * Theme.fontScale)
                                    color: Theme.text; wrapMode: Text.Wrap
                                }
                            }
                        }
                    }
                }
            }
            PageSeparator {}
            SettingsSelectRow {
                Layout.fillWidth: true; controlName: "settingsDensity"
                label: qsTrId("settings.density"); description: qsTrId("settings.densityDescription")
                options: [qsTrId("settings.standard"), qsTrId("settings.compact")]
                selectedIndex: AppSettings.savedDensity === "compact" ? 1 : 0
                onSelectionRequested: index => root.commit(AppSettings.savedTheme, AppSettings.savedTextScale, AppSettings.language, index ? "compact" : "standard", AppSettings.recentEnabled, AppSettings.trayEnabled, AppSettings.openMode)
            }
            PageSeparator {}
            SettingsSelectRow {
                Layout.fillWidth: true; controlName: "settingsTextScale"
                label: qsTrId("settings.textSize"); description: qsTrId("settings.scaleDescription")
                options: ["100%", "125%", "150%"]
                selectedIndex: [1, 1.25, 1.5].indexOf(AppSettings.savedTextScale)
                onSelectionRequested: index => root.commit(AppSettings.savedTheme, [1, 1.25, 1.5][index], AppSettings.language, AppSettings.savedDensity, AppSettings.recentEnabled, AppSettings.trayEnabled, AppSettings.openMode)
            }
        }
    }
    Component {
        id: generalPage
        ColumnLayout {
            objectName: "settingsPage-general"
            spacing: Theme.settingsPageSectionGap
            PageTitle { text: qsTrId("settings.tabGeneral") }
            PageSeparator {}
            SettingsSelectRow {
                Layout.fillWidth: true; controlName: "settingsOpenMode"
                label: qsTrId("sessions.openMode"); description: qsTrId("settings.openDescription")
                options: [qsTrId("sessions.tabs"), qsTrId("sessions.windows")]
                selectedIndex: AppSettings.openMode === "windows" ? 1 : 0
                onSelectionRequested: index => root.commit(AppSettings.savedTheme, AppSettings.savedTextScale, AppSettings.language, AppSettings.savedDensity, AppSettings.recentEnabled, AppSettings.trayEnabled, index ? "windows" : "tabs")
            }
            PageSeparator {}
            SettingsSelectRow {
                Layout.fillWidth: true; controlName: "settingsLanguage"
                label: qsTrId("settings.language"); description: qsTrId("settings.languageDescription")
                options: [qsTrId("language.zhCN"), qsTrId("language.enUS")]
                selectedIndex: AppSettings.language === "en-US" ? 1 : 0
                onSelectionRequested: index => root.commit(AppSettings.savedTheme, AppSettings.savedTextScale, index ? "en-US" : "zh-CN", AppSettings.savedDensity, AppSettings.recentEnabled, AppSettings.trayEnabled, AppSettings.openMode)
            }
            PageSeparator {}
            SettingsToggle {
                Layout.fillWidth: true; objectName: "settingsTray"
                text: qsTrId("settings.trayLabel"); description: qsTrId("settings.trayDescription")
                value: AppSettings.trayEnabled
                onSelectionRequested: value => root.commit(AppSettings.savedTheme, AppSettings.savedTextScale, AppSettings.language, AppSettings.savedDensity, AppSettings.recentEnabled, value, AppSettings.openMode)
            }
        }
    }
    Component {
        id: privacyPage
        ColumnLayout {
            objectName: "settingsPage-privacy"
            spacing: Theme.settingsPageSectionGap
            PageTitle { text: qsTrId("settings.tabPrivacy") }
            PageSeparator {}
            SettingsToggle {
                Layout.fillWidth: true; objectName: "settingsRecent"
                text: qsTrId("settings.recent"); description: qsTrId("settings.recentDescription")
                value: AppSettings.recentEnabled
                onSelectionRequested: value => root.commit(AppSettings.savedTheme, AppSettings.savedTextScale, AppSettings.language, AppSettings.savedDensity, value, AppSettings.trayEnabled, AppSettings.openMode)
            }
            PageSeparator {}
            RowLayout {
                Layout.fillWidth: true
                ColumnLayout {
                    Layout.fillWidth: true
                    SectionTitle { text: qsTrId("settings.clearRecent") }
                    PageNote { text: qsTrId("settings.clearDescription") }
                }
                OperationButton { objectName: "settingsClearRecent"; text: qsTrId("settings.clearRecent"); onClicked: AppSettings.clearRecentArchives() }
            }
            Rectangle {
                Layout.fillWidth: true
                implicitHeight: privacyText.implicitHeight + 28
                radius: Theme.operationControlRadius; color: Theme.operationCard; border.color: Theme.taskBorder
                ColumnLayout {
                    id: privacyText
                    anchors.left: parent.left; anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
                    anchors.leftMargin: 14; anchors.rightMargin: 14
                    SectionTitle { text: qsTrId("settings.localTitle") }
                    PageNote { text: qsTrId("settings.localDescription") }
                }
            }
        }
    }
    Component {
        id: associationsPage
        ColumnLayout {
            objectName: "settingsPage-associations"
            spacing: Theme.settingsPageSectionGap
            PageTitle { text: qsTrId("settings.tabAssociations") }
            PageNote { text: qsTrId("settings.associationDescription") }
            GridLayout {
                Layout.fillWidth: true; columns: 2; columnSpacing: 12; rowSpacing: 8
                Repeater {
                    model: [".zip", ".tgz", ".7z", ".cab", ".rar", ".iso", ".tar", ".udf", ".tar.gz"]
                    delegate: SettingsToggle {
                        required property string modelData
                        objectName: "settingsAssociation-" + modelData
                        Layout.fillWidth: true
                        text: modelData
                        enabled: root.associations && root.associations.available && root.associations.ready && !root.associations.busy
                        value: {
                            const revision = root.associations ? root.associations.revision : 0;
                            return root.associations && revision >= 0 && root.associations.isDefault(modelData);
                        }
                        onSelectionRequested: value => root.associations.requestChange(modelData, value)
                    }
                }
            }
            PageNote { text: root.associations && root.associations.available ? qsTrId("settings.associationSystem") : qsTrId("settings.associationUnavailable") }
            PageNote { visible: root.associations && root.associations.error.length > 0; text: qsTrId("settings.associationFailed"); color: Theme.menuDanger }
        }
    }
    Component {
        id: networkPage
        ColumnLayout {
            objectName: "settingsPage-network"
            spacing: Theme.settingsPageSectionGap
            PageTitle { text: qsTrId("settings.tabNetwork") }
            PageSeparator {}
            SectionTitle { text: qsTrId("settings.proxy") }
            ColumnLayout {
                Layout.fillWidth: true; spacing: 2
                ProxyChoice { mode: "none"; label: qsTrId("settings.proxyNone"); objectName: "settingsProxy-none" }
                ProxyChoice { mode: "system"; label: qsTrId("settings.proxySystem"); objectName: "settingsProxy-system" }
                ProxyChoice { mode: "custom"; label: qsTrId("settings.proxyCustom"); objectName: "settingsProxy-custom" }
            }
            SectionTitle { visible: root.pendingProxyMode === "custom"; text: qsTrId("settings.proxyAddress") }
            OperationField {
                visible: root.pendingProxyMode === "custom"
                Layout.fillWidth: true
                objectName: "settingsProxyAddress"
                Accessible.name: qsTrId("settings.proxyAddress")
                text: AppSettings.proxyAddress
                placeholderText: "http://127.0.0.1:7890"
                density: Theme.settingsBodySize * Theme.fontScale / Theme.operationBodySize
                onEditingFinished: AppSettings.applyNetwork(root.pendingProxyMode, text, AppSettings.mirrorAddress)
            }
            PageNote { visible: root.pendingProxyMode === "custom"; text: qsTrId("settings.proxyHint") }
            PageSeparator {}
            SectionTitle { text: qsTrId("settings.mirror") }
            PageNote { text: qsTrId("settings.mirrorDescription") }
            SectionTitle { text: qsTrId("settings.mirrorAddress") }
            OperationField {
                Layout.fillWidth: true
                objectName: "settingsMirrorAddress"
                Accessible.name: qsTrId("settings.mirrorAddress")
                text: AppSettings.mirrorAddress
                placeholderText: "https://mirror.example.com"
                density: Theme.settingsBodySize * Theme.fontScale / Theme.operationBodySize
                onEditingFinished: AppSettings.applyNetwork(AppSettings.proxyMode, AppSettings.proxyAddress, text)
            }
            PageNote { text: qsTrId("settings.mirrorHint") }
        }
    }
    Component {
        id: aboutPage
        ColumnLayout {
            objectName: "settingsPage-about"
            spacing: Theme.settingsPageSectionGap
            readonly property real stablePageHeight: aboutBeforeUpdate.implicitHeight + updateReserveHeight +
                                                     aboutAfterUpdate.implicitHeight + Theme.settingsPageSectionGap * 2
            readonly property real updateActionHeight: Math.max(cancelUpdate.implicitHeight, installUpdate.implicitHeight,
                                                                recoveryUpdate.implicitHeight, openUpdate.implicitHeight)
            readonly property real updateReserveHeight: Math.max(
                Math.max(updateMeasurements.statusHeight, checkUpdate.implicitHeight)
                    + Theme.aboutUpdateProgressHeight + updateActionHeight + Theme.settingsPageSectionGap * 2,
                Math.max(updateMeasurements.installStatusHeight, checkUpdate.implicitHeight)
                    + updateMeasurements.noteHeight + updateActionHeight + Theme.settingsPageSectionGap * 2)
            Item {
                id: updateMeasurements
                visible: false
                width: updateSection.width
                property int revision: 0
                readonly property real statusHeight: {
                    let height = revision * 0;
                    for (let i = 0; i < statusMeasures.count; ++i) {
                        const item = statusMeasures.itemAt(i);
                        if (item) height = Math.max(height, item.implicitHeight);
                    }
                    return height;
                }
                readonly property real noteHeight: {
                    let height = revision * 0;
                    for (let i = 0; i < noteMeasures.count; ++i) {
                        const item = noteMeasures.itemAt(i);
                        if (item) height = Math.max(height, item.implicitHeight);
                    }
                    return height;
                }
                readonly property real installStatusHeight: {
                    let height = revision * 0;
                    for (let i = 0; i < installMeasures.count; ++i) {
                        const item = installMeasures.itemAt(i);
                        if (item) height = Math.max(height, item.implicitHeight);
                    }
                    return height;
                }
                Repeater {
                    id: statusMeasures
                    onItemAdded: updateMeasurements.revision++
                    onItemRemoved: updateMeasurements.revision++
                    // Update manifests bound each version component to 65535.
                    model: [qsTrId("update.idle"), qsTrId("update.checking"), qsTrId("update.current"),
                            qsTrId("update.updated"), qsTrId("update.available").arg("65535.65535.65535-alpha.65535"),
                            qsTrId("update.downloading"), qsTrId("update.verifying"), qsTrId("update.downloaded"),
                            qsTrId("update.waitingTasks"), qsTrId("update.preparing"), qsTrId("update.installing"),
                            qsTrId("update.recovery"), qsTrId("update.rateLimited"), qsTrId("update.tls"),
                            qsTrId("update.network"), qsTrId("update.timeout"), qsTrId("update.authorizationCancelled"),
                            qsTrId("update.authorizationFailed"), qsTrId("update.disk"), qsTrId("update.unsafeInstallation"),
                            qsTrId("update.minimumSystem"), qsTrId("update.verificationFailed"), qsTrId("update.failed"),
                            qsTrId("update.preview")]
                    delegate: Text {
                        required property string modelData
                        width: Math.max(1, updateSection.width - checkUpdate.implicitWidth - updateStatusRow.spacing)
                        text: modelData
                        font.pixelSize: Math.round(Theme.settingsNoteSize * Theme.fontScale)
                        textFormat: Text.PlainText
                        wrapMode: Text.Wrap
                    }
                }
                Repeater {
                    id: installMeasures
                    onItemAdded: updateMeasurements.revision++
                    onItemRemoved: updateMeasurements.revision++
                    model: [qsTrId("update.available").arg("65535.65535.65535-alpha.65535"), qsTrId("update.downloaded")]
                    delegate: Text {
                        required property string modelData
                        width: Math.max(1, updateSection.width - checkUpdate.implicitWidth - updateStatusRow.spacing)
                        text: modelData
                        font.pixelSize: Math.round(Theme.settingsNoteSize * Theme.fontScale)
                        textFormat: Text.PlainText
                        wrapMode: Text.Wrap
                    }
                }
                Repeater {
                    id: noteMeasures
                    onItemAdded: updateMeasurements.revision++
                    onItemRemoved: updateMeasurements.revision++
                    model: [qsTrId("update.manual"), qsTrId("update.unsigned")]
                    delegate: Text {
                        required property string modelData
                        width: Math.max(1, updateSection.width)
                        text: modelData
                        font.pixelSize: Math.round(Theme.settingsNoteSize * Theme.fontScale)
                        textFormat: Text.PlainText
                        wrapMode: Text.Wrap
                    }
                }
            }
            ColumnLayout {
                id: aboutBeforeUpdate
                Layout.fillWidth: true
                spacing: Theme.settingsPageSectionGap
                PageTitle { text: qsTrId("settings.tabAbout") }
                RowLayout {
                    Layout.fillWidth: true; spacing: 16
                    Image {
                        objectName: "aboutArtwork"
                        Layout.preferredWidth: 56; Layout.preferredHeight: 56
                        source: Qt.platform.os === "osx" ? "qrc:/xdzip/assets/app-icon-macos.png" : "qrc:/xdzip/assets/app-icon.png"
                        fillMode: Image.PreserveAspectFit; Accessible.ignored: true
                    }
                    ColumnLayout {
                        Layout.fillWidth: true
                        Text { text: "xdzip"; color: Theme.text; font.bold: true; font.pixelSize: Math.round(Theme.settingsTitleSize * Theme.fontScale) }
                        PageNote { text: BuildInfo.version }
                        PageNote { text: qsTrId("about.description") }
                        PageNote { visible: BuildInfo.channel !== "stable" || BuildInfo.dirty; text: (BuildInfo.channel !== "stable" ? qsTrId("about.development") : "") + (BuildInfo.dirty ? "\n" + qsTrId("about.modified") : "") }
                    }
                }
                PageSeparator {}
                SectionTitle { text: qsTrId("settings.updateTitle") }
            }
            ColumnLayout {
                id: updateSection
                Layout.fillWidth: true
                spacing: Theme.settingsPageSectionGap
                RowLayout {
                    id: updateStatusRow
                    Layout.fillWidth: true
                    PageNote { id: updateStatus; objectName: "aboutUpdateStatus"; text: root.updateMessage(); color: root.updateState === "error" || root.updateState === "recovery_required" ? Theme.menuDanger : Theme.textSecondary }
                    OperationButton { id: checkUpdate; objectName: "aboutCheckUpdate"; text: qsTrId("update.check"); enabled: root.updater && !root.updater.busy; onClicked: root.updater.check() }
                }
                ProgressBar {
                    objectName: "aboutUpdateProgress"; Layout.fillWidth: true
                    Layout.preferredHeight: Theme.aboutUpdateProgressHeight
                    visible: root.updater && ["downloading", "verifying", "preparing"].includes(root.updateState)
                    indeterminate: !root.updater || root.updater.progress < 0 || root.updateState !== "downloading"
                    value: root.updater ? Math.max(0, root.updater.progress) : 0
                    Accessible.name: qsTrId("update.progress")
                }
                PageNote {
                    visible: root.updater && ["available", "downloaded"].includes(root.updateState) && !root.updater.canInstall
                    text: root.updater && !root.updater.trusted ? qsTrId("update.unsigned") : qsTrId("update.manual")
                }
                RowLayout {
                    Layout.fillWidth: true
                    OperationButton {
                        id: cancelUpdate
                        objectName: "aboutCancelUpdate"
                        visible: root.updater && root.updater.busy && root.updateState !== "installing"
                        text: qsTrId("update.cancel"); onClicked: root.updater.cancel()
                    }
                    OperationButton {
                        id: installUpdate
                        objectName: "aboutInstallUpdate"
                        visible: root.updater && ["available", "downloaded"].includes(root.updateState)
                        text: root.updater && root.updater.canInstall ? qsTrId("update.install") : root.updater && root.updater.trusted ? qsTrId("update.download") : qsTrId("update.release")
                        onClicked: {
                            if (root.updater.canInstall) root.updater.downloadAndInstall();
                            else if (root.updater.trusted) root.updater.download();
                            else root.updater.openReleasePage();
                        }
                    }
                    OperationButton { id: recoveryUpdate; visible: root.updater && root.updateState === "recovery_required"; text: qsTrId("update.release"); onClicked: root.updater.openReleasePage() }
                    OperationButton { id: openUpdate; visible: root.updater && root.updater.downloaded; text: qsTrId("update.openDownload"); onClicked: root.updater.openDownloadFolder() }
                }
            }
            ColumnLayout {
                id: aboutAfterUpdate
                Layout.fillWidth: true
                spacing: Theme.settingsPageSectionGap
                PageSeparator {}
                SectionTitle { text: qsTrId("settings.versionTitle") }
                GridLayout {
                    Layout.fillWidth: true; columns: 2; columnSpacing: 16; rowSpacing: 6
                    PageNote { text: qsTrId("about.version") }
                    PageNote { objectName: "aboutValue-1"; text: BuildInfo.version; color: Theme.text }
                    PageNote { text: qsTrId("about.build") }
                    PageNote { text: String(BuildInfo.buildNumber); color: Theme.text }
                    PageNote { text: qsTrId("about.revision") }
                    PageNote { text: BuildInfo.revision === "unknown" ? qsTrId("about.unknown") : BuildInfo.revision; color: Theme.text }
                    PageNote { text: qsTrId("about.runtime") }
                    PageNote { text: "Qt " + BuildInfo.qtVersion + ", " + BuildInfo.platform + ", " + BuildInfo.architecture; color: Theme.text }
                }
                OperationButton {
                    objectName: "aboutCopy"
                    text: root.copiedVersion ? qsTrId("about.copied") : qsTrId("about.copy")
                    onClicked: { BuildInfo.copyVersionInfo(); root.copiedVersion = true; copyTimer.restart(); }
                }
                PageSeparator {}
                OperationButton { objectName: "aboutHomepage"; text: qsTrId("about.homepage"); onClicked: Qt.openUrlExternally("https://github.com/xiaodaozhi/xdzip") }
                PageNote { objectName: "aboutLicense"; text: qsTrId("about.license") }
            }
        }
    }
    Timer { id: copyTimer; interval: 2000; onTriggered: root.copiedVersion = false }
}
