#import <AppKit/AppKit.h>
#include <QDebug>
#include <QGuiApplication>
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
