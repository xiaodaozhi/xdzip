#include "BuildInfo.h"
#include "BuildVersion.h"
#include <QClipboard>
#include <QCursor>
#include <QGuiApplication>
#include <QJsonDocument>
#include <QJsonObject>
#include <QScreen>
#include <QSysInfo>
#include <QWindow>
#include <algorithm>
#include <cstdio>
#ifdef Q_OS_WIN
#include <windows.h>

#include <appmodel.h>
#endif

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
bool BuildInfo::packaged() {
#ifdef Q_OS_WIN
    UINT32 length = 0;
    // Only the documented no-package result permits the standalone updater.
    // Treat unexpected API failures as packaged to avoid crossing channels.
    return GetCurrentPackageFullName(&length, nullptr) != APPMODEL_ERROR_NO_PACKAGE;
#else
    return false;
#endif
}
bool BuildInfo::linuxX11() {
#ifdef Q_OS_LINUX
    return QGuiApplication::platformName() == QStringLiteral("xcb");
#else
    return false;
#endif
}
void BuildInfo::centerOnScreen(QWindow *window, QWindow *owner) const {
    if (!window)
        return;
    auto *screen =
        owner && owner->screen() ? owner->screen() : QGuiApplication::screenAt(QCursor::pos());
    if (!screen)
        screen = window->screen() ? window->screen() : QGuiApplication::primaryScreen();
    if (!screen)
        return;
    if (window->screen() != screen)
        window->setScreen(screen);
    const auto area = screen->availableGeometry();
    const auto margins = window->frameMargins();
    const QSize frame(window->width() + margins.left() + margins.right(),
                      window->height() + margins.top() + margins.bottom());
    const auto x = area.x() + qMax(0, (area.width() - frame.width()) / 2);
    const auto y = area.y() + qMax(0, (area.height() - frame.height()) / 2);
    window->setFramePosition(QPoint(x, y));
}
int BuildInfo::availableScreenHeight(QWindow *window) const {
    auto *screen = window && window->screen() ? window->screen() : QGuiApplication::primaryScreen();
    return screen ? screen->availableGeometry().height() : 0;
}
bool BuildInfo::printVersionIfRequested(int argc, char **argv) {
    if (argc != 2)
        return false;
    const auto argument = QString::fromLocal8Bit(argv[1]);
    if (argument == QStringLiteral("--version-json")) {
        const QJsonObject identity{{"product", "xdzip"},     {"version", version()},
                                   {"build", buildNumber()}, {"revision", revision()},
                                   {"dirty", dirty()},       {"packaged_update_policy", 1}};
        const auto bytes = QJsonDocument(identity).toJson(QJsonDocument::Compact);
        std::fwrite(bytes.constData(), 1, size_t(bytes.size()), stdout);
        std::fputc('\n', stdout);
        return true;
    }
    if (argument != QStringLiteral("--version"))
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
