#pragma once

#include <QObject>
#include <QPointer>
#include <QWindow>
#include <QtQml/qqmlregistration.h>

class LinuxFrame : public QObject {
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(bool shadowAvailable READ shadowAvailable CONSTANT)
    Q_PROPERTY(QWindow *window READ window WRITE setWindow NOTIFY windowChanged)
    Q_PROPERTY(int inset READ inset WRITE setInset NOTIFY insetChanged)

  public:
    explicit LinuxFrame(QObject *parent = nullptr);
    bool shadowAvailable() const {
        return m_shadowAvailable;
    }
    QWindow *window() const {
        return m_window;
    }
    void setWindow(QWindow *window);
    int inset() const {
        return m_inset;
    }
    void setInset(int inset);
    bool eventFilter(QObject *watched, QEvent *event) override;

  signals:
    void windowChanged();
    void insetChanged();

  private:
    void updateNativeFrame(int inset);

    bool m_shadowAvailable = false;
    QPointer<QWindow> m_window;
    int m_inset = 0;
    int m_nativeInset = 0;
    WId m_id = 0;
};
