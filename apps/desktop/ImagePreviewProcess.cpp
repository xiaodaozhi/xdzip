#include "ImagePreviewProcess.h"
#include <QCoreApplication>
#include <QDir>
#include <QElapsedTimer>
#include <QProcess>
#include <QProcessEnvironment>

QImage ImagePreview::render(const QString &worker, const QByteArray &bytes, int kind,
                            const std::function<bool()> &current, int deadlineMs) {
    if (bytes.isEmpty() || bytes.size() > EncodedLimit || format(kind).isEmpty() ||
        deadlineMs < 1 || deadlineMs > DeadlineMs)
        return {};
    QProcess process;
    QProcessEnvironment environment;
    const auto inherited = QProcessEnvironment::systemEnvironment();
    for (const auto *name : {"SystemRoot", "WINDIR", "PATH", "LD_LIBRARY_PATH", "DYLD_LIBRARY_PATH",
                             "DYLD_FRAMEWORK_PATH"}) {
        if (inherited.contains(name))
            environment.insert(name, inherited.value(name));
    }
    environment.insert("QT_QPA_PLATFORM", "offscreen");
    environment.insert("QT_PLUGIN_PATH",
                       QCoreApplication::libraryPaths().join(QDir::listSeparator()));
    process.setProcessEnvironment(environment);
    process.setStandardErrorFile(QProcess::nullDevice());
    process.setProgram(worker);
#ifdef Q_OS_WIN
    process.setCreateProcessArgumentsModifier([](QProcess::CreateProcessArguments *args) {
        args->flags |= 0x08000000; // CREATE_NO_WINDOW for the private console worker.
    });
#endif
    QElapsedTimer timer;
    timer.start();
    struct Reap {
        QProcess &process;
        ~Reap() {
            if (process.state() != QProcess::NotRunning) {
                process.kill();
                process.waitForFinished(1000);
            }
        }
    } reap{process};
    process.start();
    while (process.state() == QProcess::Starting) {
        if (timer.elapsed() >= deadlineMs || !current())
            return {};
        process.waitForStarted(50);
    }
    if (process.state() != QProcess::Running)
        return {};
    QByteArray header("XIP1", 4);
    ImagePreview::append32(header, quint32(kind));
    ImagePreview::append32(header, quint32(bytes.size()));
    if (process.write(header) != header.size() || process.write(bytes) != bytes.size())
        return {};
    process.closeWriteChannel();
    QByteArray output;
    while (process.state() != QProcess::NotRunning) {
        if (timer.elapsed() >= deadlineMs || !current()) {
            output.fill('\0');
            return {};
        }
        process.waitForReadyRead(50);
        output += process.readAllStandardOutput();
        if (output.size() > ImagePreview::OutputLimit + 16) {
            output.fill('\0');
            return {};
        }
    }
    output += process.readAllStandardOutput();
    if (!current() || timer.elapsed() >= deadlineMs ||
        process.exitStatus() != QProcess::NormalExit || process.exitCode() != 0 ||
        output.size() < 16 || output.size() > ImagePreview::OutputLimit + 16 ||
        output.first(4) != "XIR1") {
        output.fill('\0');
        return {};
    }
    const auto width = ImagePreview::read32(output, 4);
    const auto height = ImagePreview::read32(output, 8);
    const auto length = ImagePreview::read32(output, 12);
    if (width == 0 || height == 0 || width > 512 || height > 512 || length != width * height * 4 ||
        output.size() != qsizetype(length) + 16) {
        output.fill('\0');
        return {};
    }
    auto image = QImage(reinterpret_cast<const uchar *>(output.constData() + 16), int(width),
                        int(height), int(width * 4), QImage::Format_RGBA8888)
                     .copy();
    output.fill('\0');
    return image;
}
