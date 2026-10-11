#include <QCoreApplication>
#include <QGuiApplication>
#include <QQuickWindow>
#include <QTimer>
#include <QWindow>
#include <iostream>

// Exercise Qt object ownership only. Offscreen success cannot validate native
// decorations, compositor focus, real GPU rendering or asynchronous Rust shutdown.
int main(int argc, char* argv[])
{
    QGuiApplication application(argc, argv);
    application.setQuitOnLastWindowClosed(false);

    QQuickWindow owner;
    owner.setTitle(QStringLiteral("xdzip toolchain probe"));
    owner.resize(640, 480);

    QQuickWindow operation;
    operation.setTitle(QStringLiteral("独立操作窗口 · 工具链验证"));
    operation.setTransientParent(&owner);
    operation.setModality(Qt::WindowModal);
    operation.resize(320, 240);

    // Both stack-owned windows outlive the event loop. A transient owner does
    // not transfer QObject ownership or require manual deletion of its peer.
    owner.show();
    operation.show();
    QTimer::singleShot(100, &application, [&] {
        const bool valid = owner.isTopLevel() && operation.isTopLevel()
            && operation.transientParent() == &owner
            && operation.modality() == Qt::WindowModal
            && owner.isVisible() && operation.isVisible();
        const bool closed = operation.close() && owner.close();
        operation.close();
        owner.close();
        std::cout << "qt=" << qVersion() << " platform="
                  << QGuiApplication::platformName().toStdString()
                  << " independent_windows=" << valid << " close=" << closed << '\n';
        application.exit(valid && closed ? 0 : 1);
    });
    return application.exec();
}
