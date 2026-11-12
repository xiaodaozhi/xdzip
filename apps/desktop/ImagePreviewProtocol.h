#pragma once
#include <QByteArray>
#include <QtEndian>

namespace ImagePreview {
constexpr qsizetype EncodedLimit = 16 * 1024 * 1024;
constexpr qsizetype PendingLimit = 32 * 1024 * 1024;
constexpr qsizetype EventQueueLimit = 32 * 1024 * 1024;
constexpr qint64 SourcePixels = 32 * 1024 * 1024;
constexpr int SourceDimension = 16384;
constexpr int ThumbnailSide = 512;
constexpr qsizetype OutputLimit = ThumbnailSide * ThumbnailSide * 4;
constexpr int DeadlineMs = 5000;
inline void append32(QByteArray &out, quint32 value) {
    char bytes[4];
    qToLittleEndian(value, bytes);
    out.append(bytes, 4);
}
inline quint32 read32(const QByteArray &input, qsizetype at) {
    // Both endpoints validate the complete fixed header before reading fields.
    return qFromLittleEndian<quint32>(input.constData() + at);
}
inline QByteArray format(int kind) {
    static const char *const names[] = {"",     "",     "png", "jpeg", "gif", "bmp", "ico",
                                        "webp", "tiff", "tga", "icns", "svg", "heic"};
    if (kind < 2 || kind > 12)
        return {};
#ifndef Q_OS_MACOS
    if (kind == 12)
        return {};
#endif
    return names[kind];
}
} // namespace ImagePreview
