#import <AppKit/AppKit.h>
#include <QDebug>
#include <QEventLoop>
#include <QGuiApplication>
#include <QLocale>
#include <QPointer>
#include <QQuickItem>
#include <QQuickWindow>
#include <QTimer>
#include <memory>
#include <vector>

namespace {
// Native buttons retain their system behavior and accessibility. Convert the
// QML title center into their actual parent coordinates instead of assuming DPR.
void alignTrafficLights(QQuickWindow *window) {
    NSView *view = reinterpret_cast<NSView *>(window->winId());
    NSWindow *native = view.window;
    if (!native || (native.styleMask & NSWindowStyleMaskFullScreen))
        return;
    auto *title = window->findChild<QQuickItem *>(QStringLiteral("archiveTitlebar"));
    const qreal height = title ? title->height() : 48;
    // Expand the short titlebar containers from the outside in. Moving only the
    // buttons (or their immediate parent) can clip them in an AppKit ancestor.
    std::vector<NSView *> containers;
    for (NSView *host = [native standardWindowButton:NSWindowCloseButton].superview;
         host && host.superview && NSHeight(host.frame) <= qMax<qreal>(96, height);
         host = host.superview)
        containers.push_back(host);
    for (auto it = containers.rbegin(); it != containers.rend(); ++it) {
        NSView *host = *it;
        NSRect frame = host.frame;
        frame.origin.y = host.superview.isFlipped ? NSMinY(host.superview.bounds)
                                                  : NSMaxY(host.superview.bounds) - height;
        frame.size.height = height;
        if (!NSEqualRects(host.frame, frame))
            host.frame = frame;
    }
    const qreal topCenter = title ? title->mapToScene(QPointF(0, height / 2)).y() : height / 2;
    const NSPoint center =
        NSMakePoint(NSMinX(view.bounds), view.isFlipped ? NSMinY(view.bounds) + topCenter
                                                        : NSMaxY(view.bounds) - topCenter);
    for (NSWindowButton kind :
         {NSWindowCloseButton, NSWindowMiniaturizeButton, NSWindowZoomButton}) {
        NSButton *button = [native standardWindowButton:kind];
        if (!button || !button.superview)
            continue;
        const NSPoint target = [button.superview convertPoint:center fromView:view];
        NSRect frame = button.frame;
        frame.origin.y = target.y - NSHeight(frame) / 2;
        if (!NSEqualRects(button.frame, frame))
            [button setFrame:frame];
    }
}
} // namespace

void showMacWindowAfterLaunch(QQuickWindow *window) {
    if (QGuiApplication::platformName() != QStringLiteral("cocoa")) {
        QTimer::singleShot(0, window, &QWindow::show);
        return;
    }
    const QPointer<QQuickWindow> guard(window);
    const auto scheduled = std::make_shared<bool>(false);
    const auto schedule = [guard, scheduled] {
        if (!guard || *scheduled)
            return;
        *scheduled = true;
        QTimer::singleShot(0, guard, [guard] {
            // AppKit's launch openFiles callback queues QFileOpenEvent in Qt.
            // Drain it while hidden, after launch completion, without waiting
            // for archive I/O or borrowing a fixed startup delay.
            QCoreApplication::processEvents(QEventLoop::ExcludeUserInputEvents);
            if (guard)
                guard->show();
        });
    };
    if (NSRunningApplication.currentApplication.finishedLaunching) {
        schedule();
        return;
    }
    id token = [[NSNotificationCenter defaultCenter]
        addObserverForName:NSApplicationDidFinishLaunchingNotification
                    object:NSApp
                     queue:nil
                usingBlock:^(NSNotification *note) {
                  Q_UNUSED(note);
                  schedule();
                }];
    // Qt retains its own application delegate. Scope our notification token to
    // the window, including early startup failure or shutdown.
    QObject::connect(window, &QObject::destroyed,
                     [token] { [[NSNotificationCenter defaultCenter] removeObserver:token]; });
}

// Qt owns the NSView/NSWindow. All callbacks are GUI-thread and scoped to QWindow.
void integrateMacTitlebar(QQuickWindow *window) {
    if (!window || QGuiApplication::platformName() != QStringLiteral("cocoa"))
        return;
    NSView *view = reinterpret_cast<NSView *>(window->winId());
    NSWindow *native = view.window;
    if (!native)
        return;
    native.titleVisibility = NSWindowTitleHidden;
    native.titlebarAppearsTransparent = YES;
    native.styleMask |= NSWindowStyleMaskFullSizeContentView;
    native.movableByWindowBackground = NO;
    window->setProperty("integratedTitlebar", true);
    const QPointer<QQuickWindow> guard(window);
    const auto pending = std::make_shared<bool>(false);
    const auto schedule = [guard, pending] {
        if (!guard || *pending)
            return;
        *pending = true;
        QTimer::singleShot(0, guard, [guard, pending] {
            *pending = false;
            if (guard)
                alignTrafficLights(guard);
        });
    };
    // AppKit can lay out standard buttons again after Qt's resize/activation
    // callback. Observe that layout instead of fixing only the startup frame.
    // Notification tokens and deferred callbacks are scoped to the QWindow.
    const auto observe = [window, schedule](NSString *name, id object) {
        id token =
            [[NSNotificationCenter defaultCenter] addObserverForName:name
                                                              object:object
                                                               queue:nil
                                                          usingBlock:^(NSNotification *note) {
                                                            Q_UNUSED(note);
                                                            schedule();
                                                          }];
        QObject::connect(window, &QObject::destroyed,
                         [token] { [[NSNotificationCenter defaultCenter] removeObserver:token]; });
    };
    for (NSString *name in @[
             NSWindowDidResizeNotification, NSWindowDidBecomeKeyNotification,
             NSWindowDidUpdateNotification, NSWindowDidExitFullScreenNotification
         ])
        observe(name, native);
    view.postsFrameChangedNotifications = YES;
    observe(NSViewFrameDidChangeNotification, view);
    for (NSWindowButton kind :
         {NSWindowCloseButton, NSWindowMiniaturizeButton, NSWindowZoomButton}) {
        NSButton *button = [native standardWindowButton:kind];
        button.postsFrameChangedNotifications = YES;
        observe(NSViewFrameDidChangeNotification, button);
    }
    QObject::connect(window, &QWindow::visibleChanged, window, schedule);
    QObject::connect(window, &QWindow::activeChanged, window, schedule);
    QObject::connect(window, &QWindow::widthChanged, window, schedule);
    QObject::connect(window, &QWindow::heightChanged, window, schedule);
    QObject::connect(window, &QWindow::windowStateChanged, window, schedule);
    if (auto *title = window->findChild<QQuickItem *>(QStringLiteral("archiveTitlebar")))
        QObject::connect(title, &QQuickItem::heightChanged, window, schedule);
    schedule();
}

// Native smoke evidence complements offscreen Qt captures: inspect AppKit frames
// after layout, including clipping by the actual titlebar button container.
bool verifyMacTitlebar(QQuickWindow *window) {
    if (QGuiApplication::platformName() != QStringLiteral("cocoa"))
        return true;
    NSView *view = reinterpret_cast<NSView *>(window->winId());
    auto *title = window->findChild<QQuickItem *>(QStringLiteral("archiveTitlebar"));
    if (!view.window || !title)
        return false;
    for (NSWindowButton kind :
         {NSWindowCloseButton, NSWindowMiniaturizeButton, NSWindowZoomButton}) {
        NSButton *button = [view.window standardWindowButton:kind];
        if (!button || !NSContainsRect(button.superview.bounds, button.frame)) {
            qWarning("Native titlebar button lies outside its host");
            return false;
        }
        for (NSView *ancestor = button.superview; ancestor; ancestor = ancestor.superview) {
            const NSRect inAncestor = [button convertRect:button.bounds toView:ancestor];
            if (!NSContainsRect(NSInsetRect(ancestor.bounds, -.5, -.5), inAncestor)) {
                qWarning("Native titlebar button is clipped by an ancestor");
                return false;
            }
        }
        NSPoint center = [view convertPoint:NSMakePoint(NSMidX(button.frame), NSMidY(button.frame))
                                   fromView:button.superview];
        const double topCenter =
            view.isFlipped ? center.y - NSMinY(view.bounds) : NSMaxY(view.bounds) - center.y;
        qInfo("NATIVE_TITLEBAR button=%d centerY=%.2f expected=%.2f hostHeight=%.2f", int(kind),
              topCenter, title->mapToScene(QPointF(0, title->height() / 2)).y(),
              NSHeight(button.superview.bounds));
        if (qAbs(topCenter - title->mapToScene(QPointF(0, title->height() / 2)).y()) > .5)
            return false;
    }
    return true;
}

// Inspect the actual AppKit menu after Qt has merged native action roles.
// No screen automation or pixel assumptions are involved in this smoke assertion.
bool verifyMacDesktopMenus() {
    if (QGuiApplication::platformName() != QStringLiteral("cocoa"))
        return true;
    NSMenu *bar = NSApp.mainMenu;
    if (!bar || bar.numberOfItems < 7)
        return false;
    // AppKit uses a localized NSMenuItem title for Edit-menu heuristics. The
    // submenu's title is the application's actual visible title.
    auto menuForTitle = [bar](NSString *title) -> NSMenu * {
        for (NSMenuItem *item in bar.itemArray)
            if (!item.hidden && [item.submenu.title isEqualToString:title])
                return item.submenu;
        return nil;
    };
    for (const char *id : {"desktop.file", "desktop.edit", "desktop.view", "desktop.operation",
                           "desktop.window", "desktop.help"}) {
        const auto title = qtTrId(id).toUtf8();
        NSString *nativeTitle = [NSString stringWithUTF8String:title.constData()];
        if (!menuForTitle(nativeTitle)) {
            qWarning("Native menu group missing: %s", id);
            return false;
        }
    }
    const auto helpTitle = qtTrId("desktop.help").toUtf8();
    const auto aboutTitle = qtTrId("desktop.about").toUtf8();
    NSMenu *help = menuForTitle([NSString stringWithUTF8String:helpTitle.constData()]);
    if (![help itemWithTitle:[NSString stringWithUTF8String:aboutTitle.constData()]]) {
        qWarning("Native Help menu is missing About");
        return false;
    }
    const auto editTitle = qtTrId("desktop.edit").toUtf8();
    NSMenu *edit = menuForTitle([NSString stringWithUTF8String:editTitle.constData()]);
    for (const char *command : {"cut", "copy", "paste", "selectAll"}) {
        const auto label =
            qtTrId(qPrintable(QStringLiteral("desktop.") + QString::fromLatin1(command))).toUtf8();
        NSMenuItem *item = [edit itemWithTitle:[NSString stringWithUTF8String:label.constData()]];
        const auto selector = QByteArray(command) + ":";
        if (!item || item.action != NSSelectorFromString(
                                        [NSString stringWithUTF8String:selector.constData()])) {
            qWarning("Native edit role missing: %s", command);
            return false;
        }
    }
    NSMenu *application = [bar itemAtIndex:0].submenu;
    // PreferencesRole is localized by Qt's macOS plugin (for example 首选项...),
    // so verify native key equivalents instead of imposing an application label.
    NSMenuItem *aboutItem = [application itemAtIndex:0];
    bool preferences = false, quit = false;
    for (NSMenuItem *item in application.itemArray) {
        if (item.hidden)
            continue;
        if ([item.keyEquivalent isEqualToString:@","])
            preferences = true;
        if ([item.keyEquivalent isEqualToString:@"q"])
            quit = true;
    }
    if (!aboutItem || aboutItem.hidden || !preferences || !quit) {
        qWarning("Native application roles missing: about=%d preferences=%d quit=%d",
                 aboutItem && !aboutItem.hidden, preferences, quit);
        return false;
    }
    int visibleGroups = 0;
    for (NSMenuItem *item in bar.itemArray)
        if (!item.hidden)
            ++visibleGroups;
    if (visibleGroups != 7) {
        qWarning("Native menu group count: %d", visibleGroups);
        for (NSMenuItem *item in bar.itemArray)
            qInfo("Native group: %s hidden=%d", item.title.UTF8String, item.hidden);
        return false;
    }
    qInfo("NATIVE_DESKTOP_MENU groups=7 roles=about,preferences,quit editResponders=4 services=%d "
          "ui=%s system=%s",
          NSApp.servicesMenu != nil,
          qPrintable(QCoreApplication::instance()->property("xdzipUiLanguage").toString()),
          qPrintable(QLocale::system().uiLanguages().value(0)));
    return NSApp.servicesMenu != nil;
}

// Native panels own their field editor. Do not forward their text commands to
// the archive selection or try to operate their text through a Quick item.
bool macNativeEditorActive() {
    return [NSApp.keyWindow.firstResponder isKindOfClass:[NSTextView class]];
}
bool macNativeEditorEnabled(const QString &command) {
    if (!macNativeEditorActive())
        return false;
    NSTextView *editor = static_cast<NSTextView *>(NSApp.keyWindow.firstResponder);
    if (command == "copy")
        return editor.selectedRange.length > 0;
    if (command == "selectAll")
        return editor.string.length > 0;
    if (!editor.editable)
        return false;
    if (command == "cut")
        return editor.selectedRange.length > 0;
    if (command == "paste")
        return [NSPasteboard.generalPasteboard stringForType:NSPasteboardTypeString] != nil;
    if (command == "undo")
        return editor.undoManager.canUndo;
    if (command == "redo")
        return editor.undoManager.canRedo;
    return false;
}
void macNativeEditorExecute(const QString &command) {
    if (!macNativeEditorEnabled(command))
        return;
    const auto selector = command.toUtf8() + ":";
    [NSApp sendAction:NSSelectorFromString([NSString stringWithUTF8String:selector.constData()])
                   to:NSApp.keyWindow.firstResponder
                 from:nil];
}
