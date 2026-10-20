// apps/desktop/SingleInstance.h

#pragma once
#include <QJsonObject>
#include <QLocalServer>
#include <QLockFile>
#include <QObject>
#include <functional>
#include <memory>

// The election lock and the private transport have different lifetimes: only
// the elected owner may remove a stale socket. Failure never creates a peer UI.
class SingleInstance final : public QObject {
    Q_OBJECT
  public:
    enum Result { Primary, Forwarded, Failed };
    explicit SingleInstance(QObject *parent = nullptr);
    Result start(const QStringList &paths, const QString &nameSpace = {},
                 int testTimeoutMs = 15000);
    void attach(std::function<void(const QStringList &)> handler);

  private:
    void accept();
    QLocalServer m_server;
    std::unique_ptr<QLockFile> m_lock;
    QList<QStringList> m_pending;
    QStringList m_seen;
    std::function<void(const QStringList &)> m_handler;
};
