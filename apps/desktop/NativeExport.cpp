// apps/desktop/NativeExport.cpp

#include "NativeExport.h"
#include <QCoreApplication>
#include <QFile>
#include <QFont>
#include <QGuiApplication>
#include <QJsonDocument>
#include <QJsonObject>
#include <QPainter>
#include <QThread>
#include <cmath>

QImage nativeExportPreview(const QList<ExportItem> &entries, qreal devicePixelRatio) {
    int count = 0;
    bool directory = false;
    for (const auto &entry : entries) {
        if (entry.name.contains('/'))
            continue;
        if (!count)
            directory = entry.directory;
        ++count;
    }
    if (!count || !std::isfinite(devicePixelRatio) || devicePixelRatio <= 0)
        return {};
    QFile specification(QStringLiteral(":/xdzip/drag-preview.json"));
    if (!specification.open(QIODevice::ReadOnly))
        return {};
    const auto spec = QJsonDocument::fromJson(specification.readAll()).object();
    QFile paletteFile(QStringLiteral(":/xdzip/drag-preview-palette.json"));
    if (!paletteFile.open(QIODevice::ReadOnly))
        return {};
    const auto palette =
        QJsonDocument::fromJson(paletteFile.readAll()).object().value("light").toObject();
    const int size = spec.value("native_size").toInt();
    const int inset = spec.value("native_inset").toInt();
    const int iconSize = spec.value("native_icon_size").toInt();
    const int badgeSize = spec.value("native_badge_size").toInt();
    if (size <= 0 || size * devicePixelRatio > 2048)
        return {};
    QImage image(
        QSize(int(std::ceil(size * devicePixelRatio)), int(std::ceil(size * devicePixelRatio))),
        QImage::Format_ARGB32_Premultiplied);
    image.setDevicePixelRatio(devicePixelRatio);
    image.fill(Qt::transparent);
    QPainter painter(&image);
    painter.setRenderHint(QPainter::Antialiasing);
    const QImage icon(directory ? ":/xdzip/assets/light/folder.png"
                                : ":/xdzip/assets/light/file-text.png");
    painter.drawImage(QRect(inset, inset, iconSize, iconSize), icon);
    if (count > 1) {
        const QRect badge(size - badgeSize, size - badgeSize, badgeSize, badgeSize);
        painter.setPen(Qt::NoPen);
        painter.setBrush(
            QColor(palette.value(spec.value("native_badge_color").toString()).toString()));
        painter.drawEllipse(badge);
        painter.setPen(
            QColor(palette.value(spec.value("native_badge_text").toString()).toString()));
        QFont font = QGuiApplication::font();
        font.setPixelSize(spec.value("native_badge_font_size").toInt());
        font.setWeight(QFont::DemiBold);
        painter.setFont(font);
        painter.drawText(badge, Qt::AlignCenter,
                         count > 99 ? QStringLiteral("99+") : QString::number(count));
    }
    return image;
}

void NativeExportState::request() {
    std::function<void()> callback;
    {
        std::lock_guard<std::mutex> lock(mutex);
        if (requested || done)
            return;
        requested = true;
        callback = prepare;
    }
    if (callback)
        callback();
    else
        finish({}, QStringLiteral("source_changed"));
}
bool NativeExportState::wait() {
    request();
    // A synchronous native consumer cannot make the GUI wait for its own worker.
    if (QThread::currentThread() == QCoreApplication::instance()->thread())
        return false;
    std::unique_lock<std::mutex> lock(mutex);
    changed.wait(lock, [this] { return done || cancelled.load(); });
    return done && error.isEmpty() && !cancelled.load();
}
void NativeExportState::finish(const QString &path, const QString &failure) {
    {
        std::lock_guard<std::mutex> lock(mutex);
        if (done)
            return;
        directory = path;
        error = failure;
        done = true;
    }
    changed.notify_all();
}
#if !defined(Q_OS_WIN) && !defined(Q_OS_MACOS)
bool startNativeExport(QWindow *, const std::shared_ptr<NativeExportState> &) {
    return false;
}
#endif

#ifndef Q_OS_MACOS
void captureNativeExportGesture(QWindow *, const std::shared_ptr<NativeExportState> &) {}
#endif
