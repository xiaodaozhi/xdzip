// Bounded developer fixture. Never linked into the product or release packages.
#if defined(__aarch64__)
#include <arm_acle.h>
#endif
#include <QCoreApplication>
#include <QCryptographicHash>
#include <QElapsedTimer>
#include <QFile>
#include <QJsonDocument>
#include <QJsonObject>
#include <QNetworkAccessManager>
#include <QNetworkProxy>
#include <QNetworkReply>
#include <QSaveFile>
#include <QSslCertificate>
#include <QSslConfiguration>
#include <QSslSocket>
#include <QTimer>
#ifdef Q_OS_WIN
#include <windows.h>
#else
#include <dlfcn.h>
#endif

static QString networkDigest() {
    QString path;
#ifdef Q_OS_WIN
    wchar_t buffer[32768];
    const auto size = GetModuleFileNameW(GetModuleHandleW(L"Qt6Network.dll"), buffer, 32768);
    path = QString::fromWCharArray(buffer, int(size));
#else
    Dl_info info{};
    if (dladdr(&QNetworkAccessManager::staticMetaObject, &info))
        path = QString::fromLocal8Bit(info.dli_fname);
#endif
    QFile file(path);
    QCryptographicHash hash(QCryptographicHash::Sha256);
    if (!file.open(QIODevice::ReadOnly) || !hash.addData(&file))
        return {};
    return QString::fromLatin1(hash.result().toHex());
}

int main(int argc, char **argv) {
    QCoreApplication app(argc, argv);
    if (argc != 2)
        return 2;
    QFile input(QString::fromLocal8Bit(argv[1]));
    if (!input.open(QIODevice::ReadOnly) || input.size() > 16384)
        return 2;
    const auto params = QJsonDocument::fromJson(input.readAll()).object();
    const QUrl url(params.value("url").toString());
    if ((url.host() != "127.0.0.1" && url.host() != "xdzip-owned-direct.invalid") ||
        (url.scheme() != "http" && url.scheme() != "https") || url.port() < 1)
        return 2;
    QNetworkAccessManager manager;
    manager.setProxy(QNetworkProxy::NoProxy);
    if (params.contains("proxy_port"))
        manager.setProxy(QNetworkProxy(QNetworkProxy::HttpProxy, "127.0.0.1",
                                       quint16(params.value("proxy_port").toInt())));
    QNetworkRequest request(url);
    request.setAttribute(QNetworkRequest::Http2AllowedAttribute, false);
    if (params.contains("ca")) {
        auto ssl = request.sslConfiguration();
        ssl.addCaCertificates(QSslCertificate::fromPath(params.value("ca").toString()));
        request.setSslConfiguration(ssl);
    }
    auto *reply = manager.get(request);
    bool cancelled = false, overflow = false;
    int tlsErrors = 0;
    QByteArray body;
    QElapsedTimer elapsed;
    elapsed.start();
    QObject::connect(reply, &QNetworkReply::sslErrors, [&](const QList<QSslError> &errors) {
        tlsErrors += errors.size(); // No ignoreSslErrors or global trust changes.
    });
    QObject::connect(reply, &QIODevice::readyRead, [&] {
        const auto data = reply->readAll();
        if (body.size() + data.size() > 1024) {
            overflow = true;
            reply->abort();
        } else {
            body += data;
        }
    });
    QObject::connect(reply, &QNetworkReply::finished, [&] {
        QJsonObject record{
            {"qt", qVersion()},
            {"network_sha256", networkDigest()},
            {"cancelled_by_fixture", cancelled},
            {"body_overflow", overflow},
            {"body", QString::fromLatin1(body.toHex())},
            {"error", int(reply->error())},
            {"http_status", reply->attribute(QNetworkRequest::HttpStatusCodeAttribute).toInt()},
            {"tls_errors", tlsErrors},
            {"tls_backend", QSslSocket::activeBackend()},
            {"elapsed_ms", elapsed.elapsed()}};
        QSaveFile file(params.value("result").toString());
        const auto data = QJsonDocument(record).toJson();
        if (!file.open(QIODevice::WriteOnly) || file.write(data) != data.size() || !file.commit())
            app.exit(3);
        else
            app.quit();
    });
    QTimer::singleShot(3000, reply, [&] {
        cancelled = true;
        reply->abort();
    });
    return app.exec();
}
