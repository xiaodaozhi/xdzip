#pragma once

#include <QHash>
#include <QObject>
#include <QStringList>

class FileAssociations final : public QObject {
    Q_OBJECT
    Q_PROPERTY(bool available READ available NOTIFY changed)
    Q_PROPERTY(bool busy READ busy NOTIFY changed)
    Q_PROPERTY(bool ready READ ready NOTIFY changed)
    Q_PROPERTY(int revision READ revision NOTIFY changed)
    Q_PROPERTY(QString error READ error NOTIFY changed)
  public:
    explicit FileAssociations(QObject *parent = nullptr);
    bool available() const {
        return m_available;
    }
    bool busy() const {
        return m_busy;
    }
    bool ready() const {
        return m_ready;
    }
    int revision() const {
        return m_revision;
    }
    QString error() const {
        return m_error;
    }
    Q_INVOKABLE bool isDefault(const QString &extension) const;
    Q_INVOKABLE void refresh();
    Q_INVOKABLE void requestChange(const QString &extension, bool enable);
    void nativeFinished(bool success);
  signals:
    void changed();

  private:
    void setBusy(bool busy);
    void notify();
    bool m_available = false;
    bool m_busy = false;
    bool m_ready = false;
    bool m_queryFailed = false;
    int m_revision = 0;
    int m_pending = 0;
    QString m_error;
    QHash<QString, bool> m_defaults;
    QHash<QString, bool> m_pendingDefaults;
};
