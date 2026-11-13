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
    flags: Qt.Window | (linuxTitlebar ? Qt.FramelessWindowHint : 0)
    color: linuxTitlebar ? "transparent" : Theme.surface
    background: Item {
        visible: root.linuxTitlebar
        Rectangle {
            x: 0
            y: root.linuxFrameRadius
            width: parent.width
            height: Math.max(0, parent.height - 2 * root.linuxFrameRadius)
            color: Theme.surface
        }
    }
    title: activePage && !activePage.closedPage && activePage.controller.title.length ? activePage.controller.title : "xdzip"
    readonly property bool linuxTitlebar: BuildInfo.linuxX11
    readonly property real linuxFrameRadius: linuxTitlebar && visibility !== Window.Maximized && visibility !== Window.FullScreen ? Theme.mainLinuxCornerRadius : 0
    readonly property bool windowsTitlebar: Qt.platform.os === "windows" || linuxTitlebar
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
    readonly property bool escapeGestureActive: activePage ? activePage.archiveInternalDragging : false
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
    component ResizeHandle: MouseArea {
        required property int edges
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton
        onPressed: root.startSystemResize(edges)
    }
    Item {
        anchors.fill: parent
        z: 1000
        visible: root.linuxTitlebar && root.visibility !== Window.Maximized && root.visibility !== Window.FullScreen
        readonly property real edge: Theme.mainLinuxResizeEdge
        ResizeHandle { x: parent.edge; y: 0; width: parent.width - 2 * parent.edge; height: parent.edge; edges: Qt.TopEdge; cursorShape: Qt.SizeVerCursor }
        ResizeHandle { x: parent.edge; y: parent.height - height; width: parent.width - 2 * parent.edge; height: parent.edge; edges: Qt.BottomEdge; cursorShape: Qt.SizeVerCursor }
        ResizeHandle { x: 0; y: parent.edge; width: parent.edge; height: parent.height - 2 * parent.edge; edges: Qt.LeftEdge; cursorShape: Qt.SizeHorCursor }
        ResizeHandle { x: parent.width - width; y: parent.edge; width: parent.edge; height: parent.height - 2 * parent.edge; edges: Qt.RightEdge; cursorShape: Qt.SizeHorCursor }
        ResizeHandle { x: 0; y: 0; width: parent.edge; height: parent.edge; edges: Qt.TopEdge | Qt.LeftEdge; cursorShape: Qt.SizeFDiagCursor }
        ResizeHandle { x: parent.width - width; y: 0; width: parent.edge; height: parent.edge; edges: Qt.TopEdge | Qt.RightEdge; cursorShape: Qt.SizeBDiagCursor }
        ResizeHandle { x: 0; y: parent.height - height; width: parent.edge; height: parent.edge; edges: Qt.BottomEdge | Qt.LeftEdge; cursorShape: Qt.SizeBDiagCursor }
        ResizeHandle { x: parent.width - width; y: parent.height - height; width: parent.edge; height: parent.edge; edges: Qt.BottomEdge | Qt.RightEdge; cursorShape: Qt.SizeFDiagCursor }
    }
}
