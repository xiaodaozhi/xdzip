#include "ImagePreviewProcess.h"
#include <QBuffer>
#include <QDataStream>
#include <QDir>
#include <QElapsedTimer>
#include <QFile>
#include <QImageWriter>
#include <QJsonArray>
#include <QJsonDocument>
#include <QProcess>
#include <QtTest>

namespace {
QByteArray encode(const QImage &image, const char *format) {
    QByteArray bytes;
    QBuffer buffer(&bytes);
    buffer.open(QIODevice::WriteOnly);
    QImageWriter writer(&buffer, format);
    if (QByteArray(format) != "png")
        writer.setQuality(90);
    return writer.write(image) ? bytes : QByteArray{};
}
QImage render(const QByteArray &bytes, int kind) {
    auto image = ImagePreview::render(QString::fromUtf8(XDZIP_IMAGE_WORKER), bytes, kind,
                                      [] { return true; });
    const auto audit = qEnvironmentVariable("XDZIP_IMAGE_FIXTURE_AUDIT");
    if (image.isNull() && !audit.isEmpty()) {
        QDir().mkpath(audit);
        QFile file(QDir(audit).filePath(QString::number(kind) + ".bin"));
        if (file.open(QIODevice::WriteOnly))
            file.write(bytes);
    }
    return image;
}
QByteArray twoPageTiff() {
    QByteArray bytes;
    QDataStream out(&bytes, QIODevice::WriteOnly);
    out.setByteOrder(QDataStream::LittleEndian);
    out.writeRawData("II", 2);
    out << quint16(42) << quint32(8);
    for (int page = 0; page < 2; ++page) {
        out << quint16(10);
        auto tag = [&](quint16 id, quint16 type, quint32 count, quint32 value) {
            out << id << type << count << value;
        };
        tag(256, 4, 1, 1);
        tag(257, 4, 1, 1);
        tag(258, 3, 3, 260 + quint32(page) * 6);
        tag(259, 3, 1, 1);
        tag(262, 3, 1, 2);
        tag(273, 4, 1, 272 + quint32(page) * 3);
        tag(277, 3, 1, 3);
        tag(278, 4, 1, 1);
        tag(279, 4, 1, 3);
        tag(284, 3, 1, 1);
        out << quint32(page == 0 ? 134 : 0);
    }
    for (int n = 0; n < 6; ++n)
        out << quint16(8);
    out.writeRawData("\xff\0\0\0\0\xff", 6);
    return bytes;
}
} // namespace
class ImagePreviewTest final : public QObject {
    Q_OBJECT
  private slots:
    void installedPluginsAreAvailable() {
        QProcess worker;
        worker.start(QString::fromUtf8(XDZIP_IMAGE_WORKER), {"--formats"});
        QVERIFY(worker.waitForFinished(7000));
        QCOMPARE(worker.exitCode(), 0);
        const auto formats = QJsonDocument::fromJson(worker.readAllStandardOutput()).array();
        for (const char *format :
             {"png", "jpeg", "gif", "bmp", "ico", "webp", "tiff", "tga", "icns", "svg"})
            QVERIFY2(formats.contains(QString::fromLatin1(format)), format);
#ifdef Q_OS_MACOS
        QVERIFY(formats.contains("heic"));
#else
        QVERIFY(!formats.contains("heic"));
        QVERIFY(render("owned", 12).isNull());
#endif
    }
    void nativeRasterFormats_data() {
        QTest::addColumn<QByteArray>("format");
        QTest::addColumn<int>("kind");
        QTest::newRow("PNG") << QByteArray("png") << 2;
        QTest::newRow("JPEG") << QByteArray("jpeg") << 3;
        QTest::newRow("BMP") << QByteArray("bmp") << 5;
        QTest::newRow("ICO") << QByteArray("ico") << 6;
        QTest::newRow("WebP") << QByteArray("webp") << 7;
        QTest::newRow("TIFF") << QByteArray("tiff") << 8;
        QTest::newRow("ICNS") << QByteArray("icns") << 10;
    }
    void nativeRasterFormats() {
        QFETCH(QByteArray, format);
        QFETCH(int, kind);
        QImage source(64, 64, QImage::Format_ARGB32);
        source.fill(QColor(30, 80, 180));
        const auto bytes = encode(source, format.constData());
        QVERIFY2(!bytes.isEmpty(), format.constData());
        const auto image = render(bytes, kind);
        QVERIFY2(!image.isNull(), format.constData());
        QCOMPARE(image.size(), QSize(512, 512));
        QVERIFY(image.pixelColor(256, 256).blue() > 150);
        QVERIFY(render(bytes.first(12), kind).isNull());
        if (kind == 6) {
            auto cursor = bytes;
            cursor[2] = 2;
            cursor[10] = cursor[11] = cursor[12] = cursor[13] = 0;
            QVERIFY(!render(cursor, 6).isNull());
        }
    }
    void firstFramesAndPagesAndTga() {
        // Two owned one-pixel GIF frames, red then blue. Preview reads only the first.
        const auto gif = QByteArray::fromHex("47494638396101000100800000ff00000000ff"
                                             "21f90400000000002c0000000001000100000202440100"
                                             "21f90400000000002c00000000010001000002024c01003b");
        const auto firstGif = render(gif, 4);
        QVERIFY(!firstGif.isNull());
        QCOMPARE(firstGif.pixelColor(256, 256), QColor(Qt::red));
        const auto firstTiff = render(twoPageTiff(), 8);
        QVERIFY(!firstTiff.isNull());
        QCOMPARE(firstTiff.pixelColor(256, 256), QColor(Qt::red));
        auto tga = QByteArray(18, '\0');
        tga[2] = 2;
        tga[12] = 2;
        tga[14] = 1;
        tga[16] = 24;
        tga[17] = 0x20;
        tga += QByteArray::fromHex("0000ffff0000");
        const auto image = render(tga, 9);
        QVERIFY(!image.isNull());
        QCOMPARE(image.size(), QSize(512, 256));
        QVERIFY(image.pixelColor(20, 128).red() > 200);
    }
    void transparencyAndPhoneSizedImages() {
        QImage source(4032, 3024, QImage::Format_RGB32);
        source.fill(QColor(30, 80, 180));
        for (const auto &format : {QByteArray("jpeg"), QByteArray("png")}) {
            const auto image = render(encode(source, format.constData()), format == "png" ? 2 : 3);
            QVERIFY(!image.isNull());
            QCOMPARE(image.size(), QSize(512, 384));
        }
#ifdef Q_OS_MACOS
        QFile heif(QStringLiteral(XDZIP_IMAGE_FIXTURES "/owned-phone.heic"));
        QVERIFY(heif.open(QIODevice::ReadOnly));
        const auto heifThumbnail = render(heif.readAll(), 12);
        QVERIFY(!heifThumbnail.isNull());
        QCOMPARE(heifThumbnail.size(), QSize(512, 384));
#endif
        QImage alpha(64, 32, QImage::Format_ARGB32);
        alpha.fill(QColor(30, 80, 180, 80));
        const auto thumbnail = render(encode(alpha, "webp"), 7);
        QVERIFY(!thumbnail.isNull());
        QCOMPARE(thumbnail.pixelColor(256, 128).alpha(), 80);
    }
    void jpegOrientationIsAppliedBeforeDisplay() {
        QImage source(64, 32, QImage::Format_RGB32);
        source.fill(QColor(30, 80, 180));
        auto jpeg = encode(source, "jpeg");
        QVERIFY(!jpeg.isEmpty());
        QByteArray exif("Exif\0\0", 6);
        QDataStream data(&exif, QIODevice::Append);
        data.setByteOrder(QDataStream::LittleEndian);
        data.writeRawData("II", 2);
        data << quint16(42) << quint32(8) << quint16(1) << quint16(0x112) << quint16(3)
             << quint32(1) << quint32(6) << quint32(0);
        QByteArray segment("\xff\xe1", 2);
        char length[2];
        qToBigEndian(quint16(exif.size() + 2), length);
        segment.append(length, 2);
        segment.append(exif);
        jpeg.insert(2, segment);
        const auto image = render(jpeg, 3);
        QVERIFY(!image.isNull());
        QCOMPARE(image.size(), QSize(256, 512));
    }
    void svgUsesLocalDefinitionsAndStaticText() {
        const auto image =
            render("<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 32'>"
                   "<metadata><owned xmlns='urn:owned'>ignored</owned></metadata>"
                   "<style>.paint {fill:u<![CDATA[rl(#color)]]>}</style><defs>"
                   "<linearGradient id='color'><stop stop-color='#123456'/></linearGradient>"
                   "<rect id='shape' width='64' height='32'/></defs>"
                   "<use href='#shape' class='paint'/><text x='2' y='12'>SVG</text></svg>",
                   11);
        QVERIFY(!image.isNull());
        QCOMPARE(image.size(), QSize(512, 256));
        QCOMPARE(image.pixelColor(500, 240), QColor("#123456"));
    }
    void svgRejectsResourcesAndExecutableContent_data() {
        QTest::addColumn<QByteArray>("body");
        QTest::newRow("file") << QByteArray(
            "<image href='file:///owned.png' width='64' height='32'/>");
        QTest::newRow("relative") << QByteArray("<image href='owned.png' width='64' height='32'/>");
        QTest::newRow("network") << QByteArray("<use href='https://example.invalid/owned.svg#x'/>");
        QTest::newRow("css") << QByteArray(
            "<rect width='64' height='32' fill='url(file:///owned.png)'/>");
        QTest::newRow("css-import") << QByteArray("<style>@import 'owned.css';</style>");
        QTest::newRow("css-escape") << QByteArray("<style>u\\72l(owned.png)</style>");
        QTest::newRow("css-cdata-split")
            << QByteArray("<style>rect{fill:u<![CDATA[rl(file:///owned.png)]]>}</style>");
        QTest::newRow("css-comment-split")
            << QByteArray("<style>rect{fill:ur<!-- owned -->l(file:///owned.png)}</style>");
        QTest::newRow("css-nested-element")
            << QByteArray("<style>rect{fill:u<g/>rl(file:///owned.png)}</style>");
        QTest::newRow("script") << QByteArray("<script>owned</script>");
        QTest::newRow("event") << QByteArray("<rect onload='owned' width='64' height='32'/>");
        QTest::newRow("foreign") << QByteArray("<foreignObject>owned</foreignObject>");
        QTest::newRow("animation") << QByteArray("<animate attributeName='x' dur='1s'/>");
        QTest::newRow("entity") << QByteArray("&owned;");
        QTest::newRow("deep") << QByteArray("<g>").repeated(65) + QByteArray("</g>").repeated(65);
    }
    void svgRejectsResourcesAndExecutableContent() {
        QFETCH(QByteArray, body);
        QVERIFY(render("<svg width='64' height='32'>" + body + "</svg>", 11).isNull());
    }
    void processDeadlineCancellationAndMalformedReplies() {
        const auto fault = QString::fromUtf8(XDZIP_IMAGE_FAULT_WORKER);
        QElapsedTimer timer;
        timer.start();
        QVERIFY(ImagePreview::render(fault, "owned", 2, [] { return true; }, 120).isNull());
        QVERIFY(timer.elapsed() < 2000);
        timer.restart();
        QVERIFY(
            ImagePreview::render(fault, "owned", 2, [&] { return timer.elapsed() < 50; }).isNull());
        QVERIFY(timer.elapsed() < 2000);
        for (int kind : {4, 5, 6})
            QVERIFY(ImagePreview::render(fault, "owned", kind, [] { return true; }).isNull());
        QVERIFY(!ImagePreview::render(fault, "owned", 3, [] { return true; }).isNull());
    }
    void iconContainersCannotSelectUnadvertisedNestedDecoders() {
        const QByteArray nested(
            "<svg width='64' height='64'><rect width='64' height='64' fill='red'/></svg>");
        QByteArray icon("icns", 4);
        char length[4];
        qToBigEndian(quint32(16 + nested.size()), length);
        icon.append(length, 4);
        icon.append("icp6", 4);
        qToBigEndian(quint32(8 + nested.size()), length);
        icon.append(length, 4);
        icon.append(nested);
        QVERIFY(render(icon, 10).isNull());
    }
};
QTEST_MAIN(ImagePreviewTest)
#include "image_preview_test.moc"
