#include "PreviewImages.h"
#include <QBuffer>
#include <QCoreApplication>
#include <QHash>
#include <QImageReader>
#include <QMutex>
#include <QMutexLocker>
#include <QQmlEngine>
#include <QQuickImageProvider>
#include <QSemaphore>
#include <QThread>
#include <QUuid>

namespace {
struct EncodedImage {
    QByteArray bytes;
    int kind;
};
QMutex mutex;
QHash<QString, EncodedImage> images;
qsizetype retainedBytes = 0;
QSemaphore decoders(2);
constexpr qsizetype MaximumEncodedBytes = 4 * 1024 * 1024;
constexpr qsizetype MaximumPendingBytes = 8 * 1024 * 1024;
class Provider final : public QQuickImageProvider {
  public:
    Provider()
        : QQuickImageProvider(QQuickImageProvider::Image,
                              QQmlImageProviderBase::ForceAsynchronousImageLoading) {}
    QImage requestImage(const QString &id, QSize *size, const QSize &) override {
        if (QThread::currentThread() == QCoreApplication::instance()->thread())
            return {};
        EncodedImage input;
        {
            QMutexLocker lock(&mutex);
            auto found = images.find(id);
            if (found == images.end())
                return {};
            input = found.value();
        }
        decoders.acquire();
        struct Release {
            ~Release() {
                decoders.release();
            }
        } release;
        QBuffer buffer(&input.bytes);
        if (!buffer.open(QIODevice::ReadOnly))
            return {};
        QImageReader reader(&buffer, input.kind == 2 ? QByteArray("png") : QByteArray("jpeg"));
        reader.setAutoDetectImageFormat(false);
        reader.setDecideFormatFromContent(false);
        const QSize original = reader.size();
        if (!original.isValid() || original.width() > 4096 || original.height() > 4096 ||
            qint64(original.width()) * original.height() > 4194304)
            return {};
        reader.setScaledSize(original.scaled(QSize(512, 512), Qt::KeepAspectRatio));
        QImage image = reader.read();
        input.bytes.fill('\0');
        if (image.isNull() || reader.error() != QImageReader::UnknownError || image.width() > 512 ||
            image.height() > 512)
            return {};
        if (size)
            *size = image.size();
        return image;
    }
};
} // namespace
PreviewImages *PreviewImages::create(QQmlEngine *engine, QJSEngine *) {
    engine->addImageProvider(QStringLiteral("contentpreview"), new Provider);
    return new PreviewImages;
}
QString PreviewImages::store(QByteArray bytes, int kind) {
    if ((kind != 2 && kind != 3) || bytes.isEmpty() || bytes.size() > MaximumEncodedBytes)
        return {};
    QMutexLocker lock(&mutex);
    if (retainedBytes + bytes.size() > MaximumPendingBytes)
        return {};
    const QString token = QUuid::createUuid().toString(QUuid::WithoutBraces);
    retainedBytes += bytes.size();
    images.insert(token, EncodedImage{std::move(bytes), kind});
    return QStringLiteral("image://contentpreview/") + token;
}
void PreviewImages::remove(const QString &url) {
    const auto token = url.section('/', -1);
    QMutexLocker lock(&mutex);
    auto found = images.find(token);
    if (found == images.end())
        return;
    retainedBytes -= found->bytes.size();
    found->bytes.fill('\0');
    images.erase(found);
}
