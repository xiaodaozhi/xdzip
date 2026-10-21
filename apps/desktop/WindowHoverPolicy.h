// apps/desktop/WindowHoverPolicy.h

#pragma once
#include <QCoreApplication>
#include <QCursor>
#include <QEvent>
#include <QGuiApplication>
#include <QMouseEvent>
#include <QPointingDevice>
#include <QQuickWindow>

// A hidden/minimized window has no pointer hover. End delivery before its next
// frame, including Qt's cached pointer position, without changing control bindings.
class WindowHoverPolicy final : public QObject {
  public:
    static void install(QCoreApplication *application) {
        if (application->findChild<QObject *>(QStringLiteral("xdzipWindowHoverPolicy"),
                                              Qt::FindDirectChildrenOnly))
            return;
        auto *policy = new WindowHoverPolicy(application);
        policy->setObjectName(QStringLiteral("xdzipWindowHoverPolicy"));
        application->installEventFilter(policy);
    }

  protected:
    bool eventFilter(QObject *object, QEvent *event) override {
        const auto type = event->type();
        const bool pointer =
            type == QEvent::Enter || type == QEvent::MouseMove || type == QEvent::MouseButtonPress;
        if (type != QEvent::Show && type != QEvent::Hide && !pointer)
            return false;
        auto *window = qobject_cast<QQuickWindow *>(object);
        if (!window)
            return false; // Native choosers and non-Quick windows keep platform behavior.
        if (pointer) {
            if (!hasSystemPointer() || !event->spontaneous() ||
                !window->property("xdzipHoverRestoring").toBool())
                return false;
            const auto *point = static_cast<QSinglePointEvent *>(event);
            if (point->pointingDevice()->type() != QInputDevice::DeviceType::Mouse)
                return false;
            const auto global = QCursor::pos();
            const QPointF local = window->mapFromGlobal(global);
            if (type == QEvent::MouseButtonPress ||
                (type == QEvent::MouseMove && QGuiApplication::topLevelAt(global) == window &&
                 qAbs(point->globalPosition().x() - global.x()) <= 1 &&
                 qAbs(point->globalPosition().y() - global.y()) <= 1 &&
                 qAbs(point->position().x() - local.x()) <= 1 &&
                 qAbs(point->position().y() - local.y()) <= 1)) {
                window->setProperty("xdzipHoverRestoring", false);
                return false;
            }
            // Enter can replay the application's cached pre-hide position even
            // after the first swap. Normalize it until real mouse input is fresh.
            refresh(window);
            return true;
        }
        if (event->type() == QEvent::Show && !window->property("xdzipHoverTracked").toBool()) {
            window->setProperty("xdzipHoverTracked", true);
            connect(window, &QWindow::windowStateChanged, window, [window](Qt::WindowState state) {
                if (state == Qt::WindowMinimized)
                    leave(window);
                else
                    window->setProperty("xdzipHoverRefreshPending", true);
            });
            connect(window, &QQuickWindow::afterAnimating, window, [window] {
                if (!window->isVisible() || window->windowState() == Qt::WindowMinimized ||
                    !window->property("xdzipHoverRefreshPending").toBool())
                    return;
                window->setProperty("xdzipHoverRefreshPending", false);
                // A queued Enter may still carry the pre-hide cursor coordinates.
                // Reconcile after frame-synchronous delivery, before scene sync.
                if (hasSystemPointer())
                    refresh(window);
            });
        }
        if (event->type() == QEvent::Show)
            window->setProperty("xdzipHoverRefreshPending", true);
        window->setProperty("xdzipHoverRestoring", true);
        // A late release/hover can arrive after Hide. End it again before Show
        // hands the window to the render loop; fresh native input may re-enter.
        leave(window);
        return false;
    }

  private:
    static bool hasSystemPointer() {
        // Headless surfaces have only injected input, not a system cursor to
        // reconcile. Wayland also does not expose global pointer coordinates.
        const auto platform = QGuiApplication::platformName();
        return platform == QStringLiteral("windows") || platform == QStringLiteral("cocoa") ||
               platform == QStringLiteral("xcb");
    }
    static void refresh(QQuickWindow *window) {
        leave(window);
        const auto global = QCursor::pos();
        if (window->isVisible() && QGuiApplication::mouseButtons() == Qt::NoButton &&
            QGuiApplication::topLevelAt(global) == window) {
            const QPointF local = window->mapFromGlobal(global);
            QMouseEvent move(QEvent::MouseMove, local, local, global, Qt::NoButton, Qt::NoButton,
                             QGuiApplication::keyboardModifiers());
            QCoreApplication::sendEvent(window, &move);
        }
    }
    static void leave(QQuickWindow *window) {
        // Qt Quick's normal Leave path clears Controls, MouseAreas, HoverHandlers
        // and frame-synchronous hover history. Do not synthesize a click or focus loss.
        QEvent event(QEvent::Leave);
        QCoreApplication::sendEvent(window, &event);
    }
    explicit WindowHoverPolicy(QObject *parent) : QObject(parent) {}
};
