// apps/desktop/tests/native_dialog_mac.mm

#import <AppKit/AppKit.h>
#include <QDebug>
#include <QGuiApplication>
#include <QPlatformSurfaceEvent>
#include <QPointer>
#include <QStringList>
#include <QVariant>
#include <QWindow>
#include <functional>

QStringList nativeDesktopMenuEntries(const QString &title) {
    NSMenu *bar = NSApp.mainMenu;
    NSMenu *menu = nil;
    if (title.isEmpty() && bar.numberOfItems > 0) {
        menu = [bar itemAtIndex:0].submenu;
    } else {
        for (NSMenuItem *item in bar.itemArray)
            if (!item.hidden && QString::fromNSString(item.submenu.title) == title)
                menu = item.submenu;
    }
    QStringList entries;
    for (NSMenuItem *item in menu.itemArray)
        if (!item.hidden && !item.separatorItem)
            entries.append(QString::fromNSString(item.title));
    return entries;
}

void updateNativeApplicationMenu() {
    NSMenu *bar = NSApp.mainMenu;
    if (bar.numberOfItems)
        [[bar itemAtIndex:0].submenu update];
}

static NSMenuItem *nativeQuitItem() {
    NSMenu *bar = NSApp.mainMenu;
    NSMenu *menu = bar.numberOfItems ? [bar itemAtIndex:0].submenu : nil;
    [menu update];
    for (NSMenuItem *item in menu.itemArray)
        if (!item.hidden && [item.keyEquivalent isEqualToString:@"q"] &&
            (item.keyEquivalentModifierMask & NSEventModifierFlagCommand))
            return item;
    return nil;
}

bool nativeQuitMenuEnabled() {
    NSMenuItem *item = nativeQuitItem();
    return item && item.enabled;
}

bool activateNativeQuitMenu() {
    NSMenuItem *item = nativeQuitItem();
    return item && item.enabled && [NSApp sendAction:item.action to:item.target from:item];
}

int probeNativeFileDialogQuit(const std::function<void()> &openDialog,
                              const std::function<bool()> &quitStayedBlocked) {
    __block int result = -1;
    NSTimer *timer = [[NSTimer
        timerWithTimeInterval:0.1
                      repeats:YES
                        block:^(NSTimer *current) {
                          for (NSWindow *window in NSApp.windows) {
                              if ([window isKindOfClass:[NSSavePanel class]] && window.visible) {
                                  const bool enabled = nativeQuitMenuEnabled();
                                  NSMenuItem *item = nativeQuitItem();
                                  // Also exercise an already queued action if validation has since
                                  // disabled the item while the selector was opening.
                                  const bool dispatched = item && [NSApp sendAction:item.action
                                                                                 to:item.target
                                                                               from:item];
                                  qInfo() << "Native selector Quit" << "menu enabled" << enabled
                                          << "forced action dispatched" << dispatched;
                                  result = quitStayedBlocked() ? 0 : 1;
                                  [(NSSavePanel *)window cancel:nil];
                                  [current invalidate];
                                  return;
                              }
                          }
                        }] retain];
    [[NSRunLoop mainRunLoop] addTimer:timer forMode:NSModalPanelRunLoopMode];
    [[NSRunLoop mainRunLoop] addTimer:timer forMode:NSRunLoopCommonModes];
    openDialog();
    [timer invalidate];
    [timer release];
    return result;
}

bool runOnNativeFullscreenEntry(QWindow *window, const std::function<void()> &action) {
    if (!window || !window->handle())
        return false;
    NSView *view = reinterpret_cast<NSView *>(window->winId());
    NSWindow *native = view.window;
    if (!native)
        return false;
    const QPointer<QWindow> guarded(window);
    const auto callback = action;
    __block bool fired = false;
    id token = [[NSNotificationCenter defaultCenter]
        addObserverForName:NSWindowDidEnterFullScreenNotification
                    object:native
                     queue:nil
                usingBlock:^(NSNotification *) {
                  if (!fired && guarded) {
                      fired = true;
                      callback();
                  }
                }];
    QObject::connect(window, &QObject::destroyed,
                     [token] { [[NSNotificationCenter defaultCenter] removeObserver:token]; });
    return true;
}

namespace {
// A QWindow can acquire a new NSWindow; transition state belongs to the old surface.
class NativeWindowTransitionObserver final : public QObject {
  public:
    explicit NativeWindowTransitionObserver(QWindow *window)
        : QObject(window), m_window(window), m_surfaceAlive(window->handle() != nullptr) {
        m_tokens = [[NSMutableArray alloc] init];
        window->installEventFilter(this);
    }
    ~NativeWindowTransitionObserver() override {
        clear();
        [m_tokens release];
    }
    bool watch() {
        if (!m_surfaceAlive || !m_window || !m_window->handle())
            return false;
        NSView *view = reinterpret_cast<NSView *>(m_window->winId());
        NSWindow *native = view.window;
        if (native == m_native)
            return native != nil;
        clear();
        if (!native)
            return false;
        m_native = native;
        m_window->setProperty("xdzipNativeWatchedIdentity",
                              QVariant::fromValue<qulonglong>(reinterpret_cast<quintptr>(native)));
        const QPointer<QWindow> guarded(m_window);
        for (NSNotificationName name in @[
                 NSWindowWillEnterFullScreenNotification, NSWindowWillExitFullScreenNotification,
                 NSWindowWillMiniaturizeNotification
             ]) {
            [m_tokens
                addObject:
                    [[NSNotificationCenter defaultCenter]
                        addObserverForName:name
                                    object:native
                                     queue:nil
                                usingBlock:^(NSNotification *notification) {
                                  if (guarded && guarded->property("xdzipTraceActivation").toBool())
                                      qInfo() << "activation native notification"
                                              << QString::fromNSString(notification.name)
                                              << "states" << guarded->windowStates()
                                              << "miniaturized" << native.miniaturized << "pending"
                                              << guarded->property("xdzipWakePending");
                                  if (guarded) {
                                      const bool mini = [notification.name
                                          isEqualToString:NSWindowWillMiniaturizeNotification];
                                      guarded->setProperty(mini ? "xdzipTestMiniTransition"
                                                                : "xdzipTestFullTransition",
                                                           true);
                                      guarded->setProperty("xdzipNativeTransition", true);
                                  }
                                }]];
        }
        for (NSNotificationName name in @[
                 NSWindowDidEnterFullScreenNotification, NSWindowDidExitFullScreenNotification,
                 NSWindowDidMiniaturizeNotification, NSWindowDidDeminiaturizeNotification
             ]) {
            [m_tokens
                addObject:
                    [[NSNotificationCenter defaultCenter]
                        addObserverForName:name
                                    object:native
                                     queue:nil
                                usingBlock:^(NSNotification *notification) {
                                  if (guarded && guarded->property("xdzipTraceActivation").toBool())
                                      qInfo() << "activation native notification"
                                              << QString::fromNSString(notification.name)
                                              << "states" << guarded->windowStates()
                                              << "miniaturized" << native.miniaturized << "pending"
                                              << guarded->property("xdzipWakePending");
                                  if (guarded) {
                                      const bool mini =
                                          [notification.name
                                              isEqualToString:NSWindowDidMiniaturizeNotification] ||
                                          [notification.name
                                              isEqualToString:NSWindowDidDeminiaturizeNotification];
                                      guarded->setProperty(mini ? "xdzipTestMiniTransition"
                                                                : "xdzipTestFullTransition",
                                                           false);
                                      guarded->setProperty(
                                          "xdzipNativeTransition",
                                          guarded->property("xdzipTestMiniTransition").toBool() ||
                                              guarded->property("xdzipTestFullTransition")
                                                  .toBool());
                                  }
                                }]];
        }
        return true;
    }

  protected:
    bool eventFilter(QObject *, QEvent *event) override {
        if (event->type() == QEvent::PlatformSurface) {
            const auto type = static_cast<QPlatformSurfaceEvent *>(event)->surfaceEventType();
            if (type == QPlatformSurfaceEvent::SurfaceAboutToBeDestroyed) {
                m_surfaceAlive = false;
                clear();
            } else if (type == QPlatformSurfaceEvent::SurfaceCreated) {
                m_surfaceAlive = true;
            }
        } else if (event->type() == QEvent::Show || event->type() == QEvent::Expose)
            watch();
        return false;
    }

  private:
    void clear() {
        for (id token in m_tokens)
            [[NSNotificationCenter defaultCenter] removeObserver:token];
        [m_tokens removeAllObjects];
        m_native = nil;
        if (m_window) {
            m_window->setProperty("xdzipTestMiniTransition", false);
            m_window->setProperty("xdzipTestFullTransition", false);
            m_window->setProperty("xdzipNativeTransition", false);
            m_window->setProperty("xdzipNativeWatchedIdentity", QVariant());
        }
    }
    QPointer<QWindow> m_window;
    bool m_surfaceAlive;
    NSMutableArray *m_tokens;
    NSWindow *m_native = nil;
};
} // namespace

bool watchNativeWindowTransitions(QWindow *window) {
    if (!window || QGuiApplication::platformName() != QStringLiteral("cocoa"))
        return false;
    auto *base = window->property("xdzipNativeTransitionObserver").value<QObject *>();
    if (base && base->parent() != window)
        return false;
    auto *observer = static_cast<NativeWindowTransitionObserver *>(base);
    if (!observer) {
        observer = new NativeWindowTransitionObserver(window);
        window->setProperty("xdzipNativeTransitionObserver",
                            QVariant::fromValue<QObject *>(observer));
    }
    return observer->watch();
}

void traceNativeWindowState(QWindow *window) {
    if (!window || QGuiApplication::platformName() != QStringLiteral("cocoa"))
        return;
    NSView *view = reinterpret_cast<NSView *>(window->winId());
    NSWindow *native = view.window;
    qInfo() << "activation native snapshot" << "hasWindow" << bool(native) << "sameObservedWindow"
            << (native && reinterpret_cast<quintptr>(native) ==
                              window->property("xdzipNativeWatchedIdentity").toULongLong())
            << "miniaturized" << (native && native.miniaturized) << "visible"
            << (native && native.visible) << "fullscreen"
            << (native && ((native.styleMask & NSWindowStyleMaskFullScreen) != 0)) << "exposed"
            << window->isExposed() << "transition" << window->property("xdzipNativeTransition")
            << "qtStates" << window->windowStates() << "cocoaMiniTransition"
            << window->property("xdzipCocoaMiniTransition") << "cocoaFullTransition"
            << window->property("xdzipCocoaFullTransition") << "nativeMinimized"
            << window->property("xdzipNativeMinimized") << "saved"
            << window->property("xdzipRestoreState") << "pending"
            << window->property("xdzipWakePending");
}

bool nativeWindowMatchesState(QWindow *window, Qt::WindowState state) {
    if (!watchNativeWindowTransitions(window))
        return false;
    NSView *view = reinterpret_cast<NSView *>(window->winId());
    NSWindow *native = view.window;
    if (!native || window->property("xdzipNativeTransition").toBool())
        return false;
    if (state == Qt::WindowMinimized)
        return native.miniaturized;
    if (state == Qt::WindowFullScreen)
        return !native.miniaturized && (native.styleMask & NSWindowStyleMaskFullScreen);
    if (state == Qt::WindowMaximized)
        return !native.miniaturized && native.zoomed;
    return !native.miniaturized && !(native.styleMask & NSWindowStyleMaskFullScreen) &&
           !native.zoomed;
}

// Cocoa's modal file panel runs its own loop. Cancel the actual panel instead
// of relying on a Qt timer firing before that panel has finished opening.
void scheduleNativeFileDialogCancellation() {
    __block int attempts = 0;
    NSTimer *timer = [NSTimer
        timerWithTimeInterval:0.1
                      repeats:YES
                        block:^(NSTimer *current) {
                          for (NSWindow *window in NSApp.windows) {
                              if ([window isKindOfClass:[NSSavePanel class]] && window.visible) {
                                  [(NSSavePanel *)window cancel:nil];
                                  [current invalidate];
                                  return;
                              }
                          }
                          if (++attempts >= 100)
                              [current invalidate];
                        }];
    [[NSRunLoop mainRunLoop] addTimer:timer forMode:NSModalPanelRunLoopMode];
    [[NSRunLoop mainRunLoop] addTimer:timer forMode:NSRunLoopCommonModes];
}
