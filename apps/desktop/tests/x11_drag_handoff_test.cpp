#include "NativeDragGesture.h"

#include <QDir>
#include <QDrag>
#include <QDropEvent>
#include <QElapsedTimer>
#include <QEventLoop>
#include <QFile>
#include <QGuiApplication>
#include <QMimeData>
#include <QProcess>
#include <QTemporaryDir>
#include <QThread>
#include <QTimer>
#include <QWindow>
#include <X11/Xlib.h>
#include <dlfcn.h>

class Receiver final : public QWindow {
public:
    explicit Receiver(QString destination) : m_destination(std::move(destination)) {}

protected:
    bool event(QEvent *event) override {
        if (event->type() == QEvent::DragEnter || event->type() == QEvent::DragMove) {
            auto *drag = static_cast<QDropEvent *>(event);
            drag->setDropAction(Qt::CopyAction);
            drag->accept();
            return true;
        }
        if (event->type() == QEvent::Drop) {
            auto *drop = static_cast<QDropEvent *>(event);
            // Request the selection after the source's drag loop has returned.
            QEventLoop delay;
            QTimer::singleShot(250, &delay, &QEventLoop::quit);
            delay.exec();
            const auto urls = drop->mimeData()->urls();
            const bool copied = urls.size() == 1 && urls.first().isLocalFile() &&
                                QFile::copy(urls.first().toLocalFile(), m_destination);
            drop->setDropAction(Qt::CopyAction);
            drop->setAccepted(copied);
            QTimer::singleShot(0, qApp, [copied] { QCoreApplication::exit(copied ? 0 : 3); });
            return true;
        }
        return QWindow::event(event);
    }

private:
    QString m_destination;
};

int main(int argc, char **argv) {
    QGuiApplication application(argc, argv);
    if (application.arguments().size() == 3 && application.arguments().at(1) == "--receiver") {
        Receiver receiver(application.arguments().at(2));
        receiver.setGeometry(500, 30, 300, 300);
        receiver.show();
        QTimer::singleShot(100, &receiver, [] { fprintf(stdout, "ready\n"); fflush(stdout); });
        QTimer::singleShot(10000, qApp, [] { QCoreApplication::exit(4); });
        return application.exec();
    }
    auto *xcb = qGuiApp->nativeInterface<QNativeInterface::QX11Application>();
    if (!xcb || !xcb->display())
        return 5;
    auto *display = xcb->display();
    void *xtest = dlopen("libXtst.so.6", RTLD_NOW | RTLD_LOCAL);
    using FakeButton = int (*)(Display *, unsigned int, int, unsigned long);
    auto fakeButton = xtest ? reinterpret_cast<FakeButton>(dlsym(xtest, "XTestFakeButtonEvent"))
                            : nullptr;
    if (!fakeButton)
        return 6;
    QTemporaryDir fixture;
    QFile input(fixture.filePath("source.txt"));
    if (!fixture.isValid() || !input.open(QIODevice::WriteOnly) || input.write("drag-handoff") != 12)
        return 7;
    input.close();
    const auto destination = fixture.filePath("received.txt");
    QProcess receiver;
    receiver.start(application.applicationFilePath(), {"--receiver", destination});
    if (!receiver.waitForStarted(3000) || !receiver.waitForReadyRead(3000) ||
        !receiver.readAllStandardOutput().contains("ready"))
        return 8;
    QWindow source;
    source.setGeometry(30, 30, 300, 300);
    source.show();
    QGuiApplication::processEvents();
    XWarpPointer(display, None, source.winId(), 0, 0, 0, 0, 50, 50);
    XSync(display, False);
    QGuiApplication::processEvents();
    if (!fakeButton(display, 1, True, CurrentTime))
        return 9;
    XSync(display, False);
    QElapsedTimer press;
    press.start();
    while (!(QGuiApplication::mouseButtons() & Qt::LeftButton) && press.elapsed() < 2000) {
        QGuiApplication::processEvents();
        QThread::msleep(1);
    }
    if (!(QGuiApplication::mouseButtons() & Qt::LeftButton))
        return 10;
    auto *drag = new QDrag(&source);
    auto *mime = new QMimeData;
    mime->setUrls({QUrl::fromLocalFile(input.fileName())});
    drag->setMimeData(mime);
    QTimer::singleShot(200, &source, [display] {
        XWarpPointer(display, None, DefaultRootWindow(display), 0, 0, 0, 0, 600, 100);
        XSync(display, False);
    });
    QTimer::singleShot(600, &source, [display, fakeButton] {
        fakeButton(display, 1, False, CurrentTime);
        XSync(display, False);
    });
    QTimer::singleShot(5000, &source, [] { QDrag::cancel(); });
    executeNativeFileDrag(drag);
    QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
    QElapsedTimer finished;
    finished.start();
    while (receiver.state() != QProcess::NotRunning && finished.elapsed() < 5000) {
        QGuiApplication::processEvents();
        QThread::msleep(1);
    }
    if (receiver.state() != QProcess::NotRunning) {
        receiver.kill();
        receiver.waitForFinished();
        return 11;
    }
    QFile output(destination);
    if (receiver.exitStatus() != QProcess::NormalExit || receiver.exitCode() != 0 ||
        !output.open(QIODevice::ReadOnly) || output.readAll() != "drag-handoff")
        return 12;
    return 0;
}
