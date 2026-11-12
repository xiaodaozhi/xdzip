#include <QDir>
#include <QGuiApplication>
#include <QImage>
#include <QImageWriter>
#include <QTextStream>

// Owned test inputs. Generation needs Apple's HEIC encoder; decoding tests do not.
int main(int argc, char **argv) {
    QGuiApplication app(argc, argv);
    if (app.arguments().size() != 2)
        return 2;
    const QDir output(app.arguments().at(1));
    if (!QDir().mkpath(output.absolutePath()))
        return 2;
    for (const auto &size : {QSize(320, 180), QSize(4032, 3024)}) {
        QImage image(size, QImage::Format_RGB32);
        image.fill(QColor("#3492bd"));
        QImageWriter writer(
            output.filePath(size.width() == 320 ? "owned-wide.heic" : "owned-phone.heic"), "heic");
        writer.setQuality(90);
        if (!writer.write(image)) {
            QTextStream(stderr) << writer.errorString() << '\n';
            return 1;
        }
    }
    return 0;
}
