#pragma once
#include <QAbstractNativeEventFilter>
#include <QObject>
#include <QPointer>
#include <QVariantMap>
#include <QWindow>
#include <QtQml/qqmlregistration.h>

// Only main-window chrome lives here; archive and task logic remain outside Qt.
class WindowsFrame : public QObject, public QAbstractNativeEventFilter {
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QWindow *window READ window WRITE setWindow NOTIFY windowChanged)
    Q_PROPERTY(bool dark READ dark WRITE setDark NOTIFY darkChanged)
    QPointer<QWindow> m_window;
    WId m_id = 0;
#ifdef Q_OS_WIN
    WId m_frameAttachAttemptId = 0;
    bool m_applying = false;
#endif
    bool m_dark = false;
    void apply();

  public:
    explicit WindowsFrame(QObject *parent = nullptr);
    ~WindowsFrame() override;
    QWindow *window() const {
        return m_window;
    }
    void setWindow(QWindow *window);
    bool dark() const {
        return m_dark;
    }
    void setDark(bool dark);
    Q_INVOKABLE QVariantMap diagnostics() const;
    bool eventFilter(QObject *watched, QEvent *event) override;
    bool nativeEventFilter(const QByteArray &type, void *message, qintptr *result) override;

  signals:
    void windowChanged();
    void darkChanged();
};
