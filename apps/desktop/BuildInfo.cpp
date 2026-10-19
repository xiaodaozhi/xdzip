#include "BuildInfo.h"
#include "BuildVersion.h"
#include <QClipboard>
#include <QGuiApplication>
#include <QJsonDocument>
#include <QJsonObject>
#include <QSysInfo>
#include <cstdio>

QString BuildInfo::version() {
    return QStringLiteral(XDZIP_DISPLAY_VERSION);
}
int BuildInfo::buildNumber() {
    return XDZIP_BUILD_NUMBER;
}
QString BuildInfo::revision() {
    return QStringLiteral(XDZIP_REVISION);
}
QString BuildInfo::channel() {
    return QStringLiteral(XDZIP_CHANNEL);
}
bool BuildInfo::dirty() {
    return XDZIP_DIRTY;
}
QString BuildInfo::qtVersion() {
    return QString::fromLatin1(qVersion());
}
QString BuildInfo::platform() {
    return QSysInfo::prettyProductName();
}
QString BuildInfo::configuration() {
    return QStringLiteral(XDZIP_CONFIGURATION);
}
QString BuildInfo::architecture() {
    return QSysInfo::currentCpuArchitecture();
}
bool BuildInfo::printVersionIfRequested(int argc, char **argv) {
    if (argc != 2 || QString::fromLocal8Bit(argv[1]) != QStringLiteral("--version"))
        return false;
    std::printf("xdzip %s (build %d, %s%s)\n", qPrintable(version()), buildNumber(),
                qPrintable(revision()), dirty() ? ", dirty" : "");
    return true;
}
void BuildInfo::copyVersionInfo() const {
    const QJsonObject info{{"product", "xdzip"},     {"version", version()},
                           {"build", buildNumber()}, {"revision", revision()},
                           {"dirty", dirty()},       {"configuration", configuration()},
                           {"channel", channel()},   {"qt", qtVersion()},
                           {"platform", platform()}, {"architecture", architecture()}};
    QGuiApplication::clipboard()->setText(QString::fromUtf8(QJsonDocument(info).toJson()));
}
