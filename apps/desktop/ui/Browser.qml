// apps/desktop/ui/Browser.qml

pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import Xdzip.Settings 1.0

ApplicationWindow {
    id: root
    width: Theme.mainWindowWidth
    height: Theme.mainWindowHeight
    minimumWidth: Theme.mainMinimumWidth
    minimumHeight: Theme.mainMinimumHeight
    visible: true
    flags: Qt.Window
    color: Theme.surface
    title: activePage && activePage.controller.title.length ? activePage.controller.title : "xdzip"
    readonly property bool windowsTitlebar: Qt.platform.os === "windows"
    property bool integratedTitlebar: false
    property var desktopShell: null
    property var sessionManager: null
    property var sessionTabs: []
    property var activePage: initialPage
    property bool managedWindow: false
    property bool retiredWindow: false
    property bool allowManagedClose: false
    property alias externalFileOpening: initialPage.externalFileOpening
    property alias desktopExitRequested: initialPage.desktopExitRequested
    property alias restartAfterClose: initialPage.restartAfterClose
    property string viewMode: activePage ? activePage.viewMode : "list"
    onViewModeChanged: if (activePage && activePage.viewMode !== viewMode) activePage.viewMode = viewMode
    property alias darkMode: initialPage.darkMode
    readonly property string activeTaskId: activePage.activeTaskId
    readonly property var taskSnapshot: activePage.taskSnapshot
    readonly property color accentTextColor: activePage.accentTextColor
    readonly property bool welcomeVisible: activePage ? activePage.welcomeVisible : true
    readonly property bool desktopInspectorVisible: activePage ? activePage.desktopInspectorVisible : false
    readonly property bool nativeMenuAvailable: desktopShell ? desktopShell.nativeMenuAvailable : false
    function toggleMaximized() { if (visibility === Window.Maximized) showNormal(); else showMaximized(); }
    function runDesktopCommand(command) { activePage.runDesktopCommand(command); }
    function requestDesktopExit() { activePage.requestDesktopExit(); }
    function presentQuit() { activePage.presentQuit(); }
    function taskStateLabel(status: string): string { return activePage.taskStateLabel(status); }
    function showTask(id: string) { activePage.showTask(id); }
    onClosing: event => {
        if (sessionManager) {
            event.accepted = allowManagedClose;
            if (!allowManagedClose) sessionManager.closeWindow(root);
        } else initialPage.handleClosing(event);
    }
    WindowsFrame { objectName: "windowsNativeFrame"; window: root; dark: Theme.dark }
    SessionPage { id: initialPage; windowHost: root; retainShellPage: true }
}
