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
    ~SingleInstance() override;
    Result start(const QStringList &paths, const QString &nameSpace = {}, int testTimeoutMs = 15000,
                 const QString &cliRequest = {}, const QJsonObject &shellRequest = {});
    void attach(std::function<void(const QStringList &)> handler);
    void attachShell(std::function<bool(const QJsonObject &)> handler);
    void attachCli(std::function<void(const QString &)> handler);
    void stopAccepting() {
        m_accepting = false;
    }
    void release();

  private:
    void accept();
    QLocalServer m_server;
    std::unique_ptr<QLockFile> m_lock;
    QList<QStringList> m_pending;
    QList<QJsonObject> m_pendingShell;
    std::function<bool(const QJsonObject &)> m_shellHandler;
    QStringList m_seen;
    QStringList m_pendingCli;
    std::function<void(const QString &)> m_cliHandler;
    std::function<void(const QStringList &)> m_handler;
    bool m_accepting = true;
};
