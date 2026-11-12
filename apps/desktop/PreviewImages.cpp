#include "PreviewImages.h"
#include "ImagePreviewProcess.h"
#include <QCoreApplication>
#include <QDir>
#include <QHash>
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
bool current(const QString &id) {
    QMutexLocker lock(&mutex);
    return images.contains(id);
}
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
        while (!decoders.tryAcquire(1, 50)) {
            if (!current(id)) {
                input.bytes.fill('\0');
                return {};
            }
        }
        struct Release {
            ~Release() {
                decoders.release();
            }
        } release;
        const auto worker = QDir(QCoreApplication::applicationDirPath())
                                .filePath(
#ifdef Q_OS_WIN
                                    "xdzip-image-preview.exe"
#else
                                    "xdzip-image-preview"
#endif
                                );
        QImage image = current(id) ? ImagePreview::render(worker, input.bytes, input.kind,
                                                          [id] { return current(id); })
                                   : QImage{};
        input.bytes.fill('\0');
        if (image.isNull())
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
    if (ImagePreview::format(kind).isEmpty() || bytes.isEmpty() ||
        bytes.size() > ImagePreview::EncodedLimit)
        return {};
    QMutexLocker lock(&mutex);
    if (retainedBytes + bytes.size() > ImagePreview::PendingLimit)
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
