#pragma once
#include <QCoreApplication>
#include <QEvent>
#include <QKeyEvent>
#include <QMouseEvent>
#include <QPointer>
#include <QQuickItem>
#include <QQuickWindow>
#include <QTimer>

// UI-only focus policy. Native file choosers and non-Quick windows keep their
// platform behavior. Never steal an event from a popup or discard an activation.
class FocusDismissPolicy final : public QObject {
    QPointer<QQuickItem> enterButton;
    QPointer<QQuickWindow> enterWindow;
    static bool inPopup(QQuickItem *item) {
        for (; item; item = item->parentItem())
            if (item->inherits("QQuickPopupItem"))
                return true;
        return false;
    }
    static QQuickItem *control(QQuickWindow *window) {
        for (auto *item = window->activeFocusItem(); item && item != window->contentItem();
             item = item->parentItem()) {
            if (item->isVisible() && (item->metaObject()->indexOfProperty("focusPolicy") >= 0 ||
                                      item->inherits("QQuickTextInput") ||
                                      item->inherits("QQuickTextEdit") || item->activeFocusOnTab()))
                return item;
        }
        return nullptr;
    }

    static void dismiss(QQuickWindow *window) {
        if (!window || inPopup(window->activeFocusItem()) || !control(window))
            return;
        auto *content = window->contentItem();
        auto *sink = content->findChild<QQuickItem *>(QStringLiteral("xdzipFocusSink"),
                                                      Qt::FindDirectChildrenOnly);
        if (!sink) {
            sink = new QQuickItem(content);
            sink->setObjectName(QStringLiteral("xdzipFocusSink"));
        }
        sink->forceActiveFocus(Qt::OtherFocusReason);
    }

  public:
    explicit FocusDismissPolicy(QObject *parent) : QObject(parent) {}
    static void install(QCoreApplication *application) {
        if (application->findChild<QObject *>(QStringLiteral("xdzipFocusDismissPolicy"),
                                              Qt::FindDirectChildrenOnly))
            return;
        auto *policy = new FocusDismissPolicy(application);
        policy->setObjectName(QStringLiteral("xdzipFocusDismissPolicy"));
        application->installEventFilter(policy);
    }
    bool eventFilter(QObject *watched, QEvent *event) override {
        // Modal teardown can send WindowUnblocked after Quick's private state is gone.
        // Only input events need focus inspection; never inspect destruction/visibility events.
        if (event->type() != QEvent::MouseButtonPress && event->type() != QEvent::KeyPress &&
            event->type() != QEvent::KeyRelease && event->type() != QEvent::ShortcutOverride)
            return false;
        auto *window = qobject_cast<QQuickWindow *>(watched);
        if (!window)
            return false;
        auto *focused = control(window);
        const bool popup = inPopup(window->activeFocusItem());
        if (event->type() == QEvent::MouseButtonPress && focused && !popup) {
            const auto *mouse = static_cast<QMouseEvent *>(event);
            if (!focused->contains(focused->mapFromScene(mouse->position())))
                dismiss(window);
        }
        if (event->type() != QEvent::KeyPress && event->type() != QEvent::KeyRelease &&
            event->type() != QEvent::ShortcutOverride)
            return false;
        const auto *key = static_cast<QKeyEvent *>(event);
        if (key->isAutoRepeat() || key->modifiers() != Qt::NoModifier)
            return false;
        const bool enter = key->key() == Qt::Key_Return || key->key() == Qt::Key_Enter;
        if (enter && focused && !popup && focused->isEnabled() &&
            focused->inherits("QQuickAbstractButton")) {
            event->accept();
            if (event->type() == QEvent::KeyPress) {
                enterButton = focused;
                enterWindow = window;
                focused->setProperty("down", true);
            } else if (event->type() == QEvent::KeyRelease) {
                const bool activate = enterButton == focused && enterWindow == window;
                enterButton.clear();
                enterWindow.clear();
                focused->setProperty("down", false);
                const QPointer<QQuickWindow> guard(window);
                if (activate)
                    QMetaObject::invokeMethod(focused, "click", Qt::DirectConnection);
                if (guard)
                    dismiss(guard);
            }
            return true;
        }
        if (key->key() == Qt::Key_Escape && focused && !popup &&
            event->type() != QEvent::KeyRelease) {
            event->accept();
            if (event->type() == QEvent::KeyPress)
                dismiss(window);
            return true; // First Escape leaves the control, not its window.
        }
        if (event->type() == QEvent::KeyRelease &&
            (key->key() == Qt::Key_Escape || key->key() == Qt::Key_Return ||
             key->key() == Qt::Key_Enter)) {
            // Let button activation, field commit and popup selection complete first.
            const QPointer<QQuickWindow> guard(window);
            QTimer::singleShot(0, window, [guard] { dismiss(guard); });
        }
        return false;
    }
};
