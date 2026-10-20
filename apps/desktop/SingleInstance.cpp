// apps/desktop/SingleInstance.cpp

#include "SingleInstance.h"
#include <QCoreApplication>
#include <QCryptographicHash>
#include <QDir>
#include <QElapsedTimer>
#include <QFileInfo>
#include <QJsonArray>
#include <QJsonDocument>
#include <QLocalSocket>
#include <QStandardPaths>
#include <QThread>
#include <QTimer>
#include <QUuid>
#include <utility>
#ifdef Q_OS_WIN
#include <qt_windows.h>
#else
#include <unistd.h>
#endif

namespace {
constexpr int MaxBytes = 256 * 1024;
QString identity() {
#ifdef Q_OS_WIN
    DWORD session = 0;
    if (!ProcessIdToSessionId(GetCurrentProcessId(), &session))
        return {};
    // AppLocalDataLocation is per user, SessionId separates interactive logons.
    return QStandardPaths::writableLocation(QStandardPaths::GenericDataLocation) +
           QString::number(session);
#else
    return QString::number(geteuid()) + ':' + qEnvironmentVariable("XDG_SESSION_ID") + ':' +
           qEnvironmentVariable("DISPLAY") + ':' + qEnvironmentVariable("WAYLAND_DISPLAY") + ':' +
           qEnvironmentVariable("SECURITYSESSIONID");
#endif
}
QByteArray packet(const QJsonObject &value) {
    return QJsonDocument(value).toJson(QJsonDocument::Compact) + '\n';
}
QByteArray readLine(QLocalSocket &socket, QElapsedTimer &timer, int timeoutMs) {
    QByteArray data;
    while (timer.elapsed() < timeoutMs) {
        data += socket.readAll();
        if (data.size() > MaxBytes)
            return {};
        if (data.endsWith('\n'))
            return data;
        socket.waitForReadyRead(qMin(200, timeoutMs - int(timer.elapsed())));
        if (socket.state() == QLocalSocket::UnconnectedState)
            break;
    }
    return {};
}
} // namespace
SingleInstance::SingleInstance(QObject *parent) : QObject(parent) {}
SingleInstance::~SingleInstance() {
    release();
}
void SingleInstance::release() {
    m_accepting = false;
    m_handler = {};
    m_pending.clear();
    if (m_server.isListening())
        m_server.close();
    for (auto *socket : m_server.findChildren<QLocalSocket *>())
        socket->abort();
    // Close the endpoint before releasing election ownership. A replacement
    // process must never have its socket removed by this instance's teardown.
    m_lock.reset();
}
SingleInstance::Result SingleInstance::start(const QStringList &paths, const QString &nameSpace,
                                             int testTimeoutMs) {
    if (m_lock)
        return Failed;
    m_accepting = true;
    const int timeoutMs = nameSpace.isEmpty() ? 15000 : qBound(100, testTimeoutMs, 15000);
    if (paths.size() > 128 || identity().isEmpty())
        return Failed;
    const auto key = QString::fromLatin1(
        QCryptographicHash::hash((identity() + nameSpace).toUtf8(), QCryptographicHash::Sha256)
            .toHex()
            .left(32));
    const auto directory = QStandardPaths::writableLocation(QStandardPaths::RuntimeLocation);
    if (directory.isEmpty() || !QDir().mkpath(directory))
        return Failed;
    const auto name = QStringLiteral("xdzip-") + key;
    m_lock = std::make_unique<QLockFile>(QDir(directory).filePath(name + ".lock"));
    m_lock->setStaleLockTime(0); // Live processes are never evicted by age.
    if (m_lock->tryLock()) {
        QLocalServer::removeServer(name);
        m_server.setSocketOptions(QLocalServer::UserAccessOption);
        if (!m_server.listen(name))
            return Failed;
        connect(&m_server, &QLocalServer::newConnection, this, &SingleInstance::accept,
                Qt::UniqueConnection);
        return Primary;
    }
    QElapsedTimer timer;
    timer.start();
    QLocalSocket socket;
    while (timer.elapsed() < timeoutMs) {
        socket.connectToServer(name);
        if (socket.waitForConnected(200))
            break;
        socket.abort();
        // The elected process can be between its lock and listen operations.
        QThread::msleep(25);
    }
    if (socket.state() != QLocalSocket::ConnectedState)
        return Failed;
    const auto hello = QJsonDocument::fromJson(readLine(socket, timer, timeoutMs)).object();
    if (hello.value("protocol").toInt() != 1)
        return Failed;
#ifdef Q_OS_WIN
    const auto pid = hello.value("pid").toString().toULong();
    if (pid)
        AllowSetForegroundWindow(pid); // User launch permission, never persistent topmost.
#endif
    QJsonArray values;
    for (const auto &path : paths) {
        if (path.isEmpty() || path.contains(QChar::Null))
            return Failed;
        values.append(QFileInfo(path).absoluteFilePath());
    }
    const auto id = QUuid::createUuid().toString(QUuid::WithoutBraces);
    const auto request = packet({{"protocol", 1}, {"id", id}, {"paths", values}});
    if (request.size() > MaxBytes || socket.write(request) != request.size())
        return Failed;
    socket.flush();
    const auto reply = QJsonDocument::fromJson(readLine(socket, timer, timeoutMs)).object();
    return reply.value("id").toString() == id && reply.value("accepted").toBool() ? Forwarded
                                                                                  : Failed;
}
void SingleInstance::attach(std::function<void(const QStringList &)> handler) {
    m_handler = std::move(handler);
    const auto pending = std::exchange(m_pending, {});
    for (const auto &paths : pending)
        m_handler(paths);
}
void SingleInstance::accept() {
    while (auto *socket = m_server.nextPendingConnection()) {
        if (m_server.findChildren<QLocalSocket *>().size() > 32) {
            socket->deleteLater();
            continue;
        }
        socket->write(packet(
            {{"protocol", 1}, {"pid", QString::number(QCoreApplication::applicationPid())}}));
        auto *timeout = new QTimer(socket);
        timeout->setSingleShot(true);
        connect(timeout, &QTimer::timeout, socket, &QLocalSocket::abort);
        connect(socket, &QLocalSocket::disconnected, socket, &QObject::deleteLater);
        timeout->start(15000);
        connect(socket, &QLocalSocket::readyRead, this, [this, socket] {
            if (socket->bytesAvailable() > MaxBytes) {
                socket->abort();
                return;
            }
            if (!socket->canReadLine())
                return;
            const auto request = QJsonDocument::fromJson(socket->readLine(MaxBytes + 1)).object();
            const auto id = request.value("id").toString();
            const auto values = request.value("paths");
            bool valid = request.value("protocol").toInt() == 1 && id.size() == 36 &&
                         values.isArray() && values.toArray().size() <= 128;
            QStringList paths;
            for (const auto &value : values.toArray()) {
                const auto path = value.toString();
                valid &= value.isString() && !path.isEmpty() && !path.contains(QChar::Null) &&
                         QFileInfo(path).isAbsolute();
                paths.append(path);
            }
            valid &= m_accepting && (m_handler || m_pending.size() < 128);
            if (valid && !m_seen.contains(id)) {
                m_seen.append(id);
                if (m_seen.size() > 128)
                    m_seen.removeFirst();
                if (m_handler)
                    m_handler(paths);
                else
                    m_pending.append(paths);
            }
            socket->write(packet({{"id", id}, {"accepted", valid}}));
            socket->disconnectFromServer();
        });
    }
}
