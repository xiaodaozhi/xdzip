#include "PasswordAccessibility.h"

#include <QAccessible>
#include <QQuickItem>
#include <QtQuick/private/qaccessiblequickitem_p.h>

namespace {
constexpr auto passwordLabel = "xdzipAccessiblePasswordLabel";

// Qt 6.8.2 suppresses Accessible.name for password edits. Preserve the Qt
// implementation for state, value, text and actions, and supply only a fixed
// UI label that is independent of the entered secret.
class NamedPasswordAccessible final : public QAccessibleQuickItem {
  public:
    using QAccessibleQuickItem::QAccessibleQuickItem;

    QString text(QAccessible::Text kind) const override {
        if (kind == QAccessible::Name)
            return object()->property(passwordLabel).toString();
        return QAccessibleQuickItem::text(kind);
    }
};

QAccessibleInterface *passwordFactory(const QString &, QObject *object) {
    auto *item = qobject_cast<QQuickItem *>(object);
    if (!item || object->metaObject()->indexOfProperty(passwordLabel) < 0)
        return nullptr;
    return new NamedPasswordAccessible(item);
}
} // namespace

void installPasswordAccessibility() {
    static bool installed = false;
    if (!installed) {
        QAccessible::installFactory(passwordFactory);
        installed = true;
    }
}
