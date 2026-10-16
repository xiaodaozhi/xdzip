#include "AppQuitGuard.h"
#include "BrowserController.h"
#include "FocusDismissPolicy.h"
#include "NativeDialogPolicy.h"
#include "UiFont.h"
#include "UiPreferences.h"
#include "UiText.h"
#include "visual_audit.h"
#include <QAbstractItemModelTester>
#include <QAccessible>
#include <QClipboard>
#include <QDesktopServices>
#include <QElapsedTimer>
#include <QFile>
#include <QInputMethodEvent>
#include <QProcess>
#include <QProcessEnvironment>
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QRegularExpression>
#include <QSGRendererInterface>
#include <QScopeGuard>
#include <QScreen>
#include <QSignalSpy>
#include <QTemporaryDir>
#include <QTimer>
#include <QtTest>
#include <thread>
#ifdef Q_OS_WIN
#include <qt_windows.h>
#endif

// TableView delegates belong to its visual tree, not necessarily its QObject tree.
static QQuickItem *findVisual(QQuickItem *item, const QString &name) {
    if (item->property("inReusePool").toBool())
        return nullptr;
    if (item->objectName() == name)
        return item;
    for (auto *child : item->childItems()) {
        if (auto *match = findVisual(child, name))
            return match;
    }
    return nullptr;
}

// Missing images can leave layout dimensions intact. Verify the actual resource state.
static bool visibleImagesReady(QQuickItem *item) {
    if (!item->isVisible())
        return true;
    if (QString::fromLatin1(item->metaObject()->className()).startsWith("QQuickImage") &&
        !item->property("source").toUrl().isEmpty() && item->property("status").toInt() != 1)
        return false;
    for (auto *child : item->childItems())
        if (!visibleImagesReady(child))
            return false;
    return true;
}

// Diagnose only owned fixture directories; never emit their absolute paths.
static bool checkedTarget(BrowserController *controller, const QString &fixturePath) {
    const bool valid = controller->targetCheck().value("valid").toBool();
    if (valid)
        return true;
    qWarning().noquote() << "Owned target check:"
                         << controller->targetCheck().value("errorCode").toString();
#ifdef Q_OS_WIN
    QProcess acl;
    const auto quoted = QString(fixturePath).replace("'", "''");
    const auto shell = qEnvironmentVariable("XDZIP_CI_POWERSHELL", "powershell.exe");
    acl.start(shell, {"-NoProfile", "-Command", "(Get-Acl -LiteralPath '" + quoted + "').Sddl"});
    if (acl.waitForFinished(5000) && acl.exitCode() == 0)
        qWarning().noquote() << "Owned fixture ACL:" << acl.readAllStandardOutput();
    else
        qWarning() << "Owned fixture ACL probe:" << acl.error() << acl.exitCode();
#else
    Q_UNUSED(fixturePath)
#endif
    return false;
}

class DesktopUrlRecorder : public QObject {
    Q_OBJECT
  public:
    int count = 0;
  public slots:
    void record(const QUrl &) {
        ++count;
    }
};

class BrowserTest : public QObject {
    QTemporaryDir m_settingsProfile;
    Q_OBJECT
  private slots:
    void windowsCaptionControlsAndWindowState() {
#ifdef Q_OS_WIN
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(window);
        QVERIFY(!window->flags().testFlag(Qt::FramelessWindowHint));
        QTRY_VERIFY(window->isVisible());
        QCOMPARE(window->size(), QSize(1024, 622));
        QVariantMap nativeFrame;
        if (QGuiApplication::platformName() == QStringLiteral("windows")) {
            auto *frame = window->findChild<QObject *>("windowsNativeFrame");
            QVERIFY(frame);
            QVERIFY(QMetaObject::invokeMethod(frame, "diagnostics",
                                              Q_RETURN_ARG(QVariantMap, nativeFrame)));
            QVERIFY(nativeFrame.value("attached").toBool());
            QVERIFY(nativeFrame.value("captionStyle").toBool());
            QVERIFY(nativeFrame.value("resizeStyle").toBool());
            if (nativeFrame.value("composition").toBool())
                QVERIFY(nativeFrame.value("nonClientRendering").toBool());
            QCOMPARE(nativeFrame.value("topInsetPixels").toInt(), 1);
            if (nativeFrame.value("cornerPreferenceSupported").toBool())
                QCOMPARE(nativeFrame.value("cornerPreference").toInt(), 2);
        }
        auto *bar = findVisual(window->contentItem(), "archiveTitlebar");
        auto *settings = findVisual(window->contentItem(), "archiveSettings");
        auto *minimize = findVisual(window->contentItem(), "windowMinimize");
        auto *maximize = findVisual(window->contentItem(), "windowMaximize");
        auto *close = findVisual(window->contentItem(), "windowClose");
        QVERIFY(bar && settings && minimize && maximize && close);
        QCOMPARE(bar->height(), 48.0);
        QCOMPARE(minimize->width(), 46.0);
        QVERIFY(settings->mapToScene(QPointF(settings->width(), 0)).x() <
                minimize->mapToScene(QPointF()).x());
        QCOMPARE(close->mapToScene(QPointF(close->width(), 0)).x(), qreal(window->width()));
        const auto audit = qEnvironmentVariable("XDZIP_WINDOWS_AUDIT");
        if (!audit.isEmpty()) {
            QVERIFY(QDir().mkpath(audit));
            QVERIFY(visual_audit::save(window, audit, "windows-caption"));
            QFile frameRecord(audit + "/windows-native-frame.json");
            QVERIFY(frameRecord.open(QIODevice::WriteOnly));
            QVERIFY(frameRecord.write(
                        QJsonDocument(QJsonObject::fromVariantMap(nativeFrame)).toJson()) > 0);
            if (QGuiApplication::platformName() == QStringLiteral("windows")) {
                // A solid, owned backdrop makes the external DWM border/shadow
                // visible without recording unrelated desktop content.
                window->resize(900, 560);
                const auto available = window->screen()->availableGeometry();
                window->setPosition(available.center() - QPoint(450, 280));
                QQuickWindow backdrop;
                backdrop.setFlags(Qt::Tool | Qt::FramelessWindowHint | Qt::WindowStaysOnTopHint);
                backdrop.setColor(QColor("#CFD8E5"));
                backdrop.setGeometry(window->frameGeometry().adjusted(-32, -32, 32, 32));
                backdrop.show();
                window->setFlag(Qt::WindowStaysOnTopHint, true);
                window->show();
                window->raise();
                window->requestActivate();
                QTest::qWait(300);
                QVariantMap frameState;
                QVERIFY(QMetaObject::invokeMethod(
                    window->findChild<QObject *>("windowsNativeFrame"), "diagnostics",
                    Q_RETURN_ARG(QVariantMap, frameState)));
                QVERIFY(frameState.value("visibleAtCenter").toBool());
                const auto bounds = frameState.value("frameBoundsNative").toList();
                const auto origin = frameState.value("monitorOriginNative").toList();
                QCOMPARE(bounds.size(), 4);
                QCOMPARE(origin.size(), 2);
                const QRect crop(bounds[0].toInt() - origin[0].toInt() - 24,
                                 bounds[1].toInt() - origin[1].toInt() - 24, bounds[2].toInt() + 48,
                                 bounds[3].toInt() + 48);
                const auto desktop = window->screen()->grabWindow(0).toImage();
                QVERIFY(desktop.rect().contains(crop));
                const auto image = desktop.copy(crop);
                for (const auto point :
                     {crop.topLeft(), crop.topRight(), crop.bottomLeft(), crop.bottomRight()}) {
                    const POINT nativePoint{point.x() + origin[0].toInt(),
                                            point.y() + origin[1].toInt()};
                    const auto owner = WindowFromPoint(nativePoint);
                    QVERIFY(owner == reinterpret_cast<HWND>(backdrop.winId()));
                    const auto color = image.pixelColor(point - crop.topLeft());
                    // DWM's shadow darkens the owned blue backdrop. Verify its
                    // hue and ownership instead of requiring an unshaded pixel.
                    const auto base = backdrop.color();
                    QVERIFY(color.blue() <= base.blue() + 5 && color.blue() >= base.blue() / 3);
                    QVERIFY(qAbs(color.red() * base.blue() - color.blue() * base.red()) <=
                            8 * base.blue());
                    QVERIFY(qAbs(color.green() * base.blue() - color.blue() * base.green()) <=
                            8 * base.blue());
                }
                QVERIFY(image.save(audit + "/windows-dwm-frame.png"));
                window->setFlag(Qt::WindowStaysOnTopHint, false);
                window->show();
            }
        }
        const auto click = [window](QQuickItem *item) {
            QTest::mouseClick(
                window, Qt::LeftButton, Qt::NoModifier,
                item->mapToScene(QPointF(item->width() / 2, item->height() / 2)).toPoint());
        };
        click(maximize);
        QTRY_COMPARE(window->visibility(), QWindow::Maximized);
        if (QGuiApplication::platformName() == QStringLiteral("windows"))
            QTRY_VERIFY(window->screen()
                            ->availableGeometry()
                            .adjusted(-1, -1, 1, 1)
                            .contains(window->geometry()));
        QCOMPARE(maximize->property("captionAction").toString(), QStringLiteral("restore"));
        click(maximize);
        QTRY_COMPARE(window->visibility(), QWindow::Windowed);
        QTRY_COMPARE(close->mapToScene(QPointF(close->width(), 0)).x(), qreal(window->width()));
        QTest::mouseDClick(window, Qt::LeftButton, Qt::NoModifier, QPoint(window->width() / 2, 24));
        QTRY_COMPARE(window->visibility(), QWindow::Maximized);
        QVERIFY(QMetaObject::invokeMethod(window, "toggleMaximized"));
        QTRY_COMPARE(window->visibility(), QWindow::Windowed);
        QTRY_COMPARE(close->mapToScene(QPointF(close->width(), 0)).x(), qreal(window->width()));
        click(minimize);
        QTRY_COMPARE(window->visibility(), QWindow::Minimized);
        window->showNormal();
        QTRY_COMPARE(window->visibility(), QWindow::Windowed);
        QTRY_COMPARE(close->mapToScene(QPointF(close->width(), 0)).x(), qreal(window->width()));
        // Closing the new button still passes through the runtime shutdown barrier.
        click(close);
        QTRY_VERIFY(!window->isVisible());
#else
        QSKIP("Windows custom caption regression");
#endif
    }
    void initTestCase() {
        FocusDismissPolicy::install(QCoreApplication::instance());
        QQuickStyle::setStyle("Basic");
        Q_INIT_RESOURCE(ui);
        QVERIFY(m_settingsProfile.isValid());
        if (qEnvironmentVariableIsEmpty("XDZIP_SETTINGS_PATH"))
            qputenv("XDZIP_SETTINGS_PATH", m_settingsProfile.filePath("settings.ini").toUtf8());
        QVERIFY(initializeUiText(qEnvironmentVariable("XDZIP_UI_TEST_LANGUAGE", "zh-CN")));
        QVERIFY(installUiFont());
        qmlRegisterType<BrowserController>("Xdzip.Native", 1, 0, "BrowserController");
    }
    void init() {
        // Platform offscreen notices are expected; broken bindings or image
        // resources must never pass a visual or interaction regression silently.
        QTest::failOnWarning(QRegularExpression(
            "Binding loop detected|Cannot open:|ReferenceError|TypeError|Unable to assign"));
    }
    void cleanup() {
        QTRY_COMPARE_WITH_TIMEOUT(RuntimeReceiver::activeReceivers(), 0, 5000);
    }
    void privateTestDesktopInputAndAccessibility() {
        const bool accessibilityWasActive = QAccessible::isActive();
        QAccessible::setActive(true);
        const auto restoreAccessibility =
            qScopeGuard([&] { QAccessible::setActive(accessibilityWasActive); });
        QElapsedTimer firstFrame;
        firstFrame.start();
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(window);
        auto *create = window->findChild<QQuickWindow *>("createWindow");
        QVERIFY(create);
        QSignalSpy frames(create, &QQuickWindow::frameSwapped);
        QVERIFY(QMetaObject::invokeMethod(create, "configure"));
        QTRY_VERIFY(create->isVisible());
        create->update();
        QTRY_VERIFY_WITH_TIMEOUT(!frames.isEmpty(), 5000);
        const auto firstPresentedMs = firstFrame.elapsed();
        QVERIFY(!create->grabWindow().isNull());
        const auto api = create->rendererInterface()->graphicsApi();
        const bool hardwareApi =
            api == QSGRendererInterface::Metal || api == QSGRendererInterface::OpenGL ||
            api == QSGRendererInterface::Direct3D11 || api == QSGRendererInterface::Direct3D12 ||
            api == QSGRendererInterface::Vulkan;
        if (qEnvironmentVariable("XDZIP_REQUIRE_GPU") == "1")
            QVERIFY2(hardwareApi, "GPU validation must not silently fall back to software or Null");
        auto *field = findVisual(create->contentItem(), "createName");
        auto *action = findVisual(create->contentItem(), "startCreate");
        QVERIFY(field && action);
        field->setProperty("text", QString{});
        field->forceActiveFocus();
        QTRY_VERIFY(field->hasActiveFocus());
        QInputMethodEvent composing(QStringLiteral("中文"), {});
        QCoreApplication::sendEvent(create, &composing);
        QTRY_VERIFY(field->property("inputMethodComposing").toBool());
        QCOMPARE(field->property("text").toString(), QString{});
        QInputMethodEvent commit;
        const auto name = QString::fromUtf8("中文文件📦.zip");
        commit.setCommitString(name);
        QCoreApplication::sendEvent(create, &commit);
        QTRY_COMPARE(field->property("text").toString(), name);
        QTRY_VERIFY(!field->property("inputMethodComposing").toBool());
        auto *editable = QAccessible::queryAccessibleInterface(field);
        auto *button = QAccessible::queryAccessibleInterface(action);
        QVERIFY(editable && button);
        QCOMPARE(editable->role(), QAccessible::EditableText);
        QCOMPARE(editable->text(QAccessible::Name), qtTrId("ui.0d568cd7b07d"));
        QCOMPARE(editable->text(QAccessible::Value), name);
        QVERIFY(editable->textInterface());
        QCOMPARE(button->role(), QAccessible::Button);
        QCOMPARE(button->text(QAccessible::Name), qtTrId("ui.e185d04bcc5f"));
        QVERIFY(button->state().disabled);
        QTest::keyClick(create, Qt::Key_Escape);
        QTRY_VERIFY(!field->hasActiveFocus());
        QCOMPARE(field->property("text").toString(), name);
        const auto directory = qEnvironmentVariable("XDZIP_PRIVATE_TEST_AUDIT");
        if (!directory.isEmpty()) {
            QVERIFY(QDir().mkpath(directory));
            const QJsonObject evidence{
                {"schema", 1},
                {"qt", qVersion()},
                {"platform", QGuiApplication::platformName()},
                {"architecture", QSysInfo::currentCpuArchitecture()},
                {"graphicsApi", static_cast<int>(api)},
                {"hardwareApi", hardwareApi},
                {"framesPresented", frames.count()},
                {"firstPresentedMs", firstPresentedMs},
                {"measurementScope",
                 "Warm in-process test startup, not release cold-start benchmark"},
                {"syntheticIme", true},
                {"accessibleInterfaces", true},
                {"nativeImeManual", "pending"},
                {"screenReaderManual", "pending"}};
            QFile output(QDir(directory).filePath("desktop-input-accessibility.json"));
            QVERIFY(output.open(QIODevice::WriteOnly));
            const auto bytes = QJsonDocument(evidence).toJson();
            QCOMPARE(output.write(bytes), bytes.size());
        }
        create->close();
    }
    void settingsWindowPersistsAndRollsBack() {
        QTemporaryDir profile;
        QVERIFY(profile.isValid());
        const auto previous = qgetenv("XDZIP_SETTINGS_PATH");
        const auto restore = qScopeGuard([previous] {
            if (previous.isNull())
                qunsetenv("XDZIP_SETTINGS_PATH");
            else
                qputenv("XDZIP_SETTINGS_PATH", previous);
        });
        const auto path = profile.filePath("settings.ini");
        qputenv("XDZIP_SETTINGS_PATH", path.toUtf8());
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(QTest::qWaitForWindowExposed(window));
        auto *settings = window->findChild<QQuickWindow *>("settingsWindow");
        auto *entry = findVisual(window->contentItem(), "archiveSettings");
        QVERIFY(settings && entry);
        const auto click = [](QQuickWindow *owner, QQuickItem *item) {
            QTest::mouseClick(
                owner, Qt::LeftButton, Qt::NoModifier,
                item->mapToScene(QPointF(item->width() / 2, item->height() / 2)).toPoint());
        };
        const bool originalDark = window->property("darkMode").toBool();
        click(window, entry);
        QTRY_VERIFY(settings->isVisible());
        QCOMPARE(settings->width(), 612);
        QVERIFY(visibleImagesReady(settings->contentItem()));
        const auto captureSettings = [&](const QString &name) {
            const auto directory = qEnvironmentVariable("XDZIP_SETTINGS_VISUAL_AUDIT");
            return directory.isEmpty() ||
                   (QDir().mkpath(directory) && visual_audit::save(settings, directory, name));
        };
        QVERIFY(captureSettings("settings-light"));
        auto *themeChoice = findVisual(settings->contentItem(),
                                       originalDark ? "settingsTheme-1" : "settingsTheme-2");
        QVERIFY(themeChoice);
        click(settings, themeChoice);
        QTRY_COMPARE(window->property("darkMode").toBool(), !originalDark);
        auto *scaleChoice = findVisual(settings->contentItem(), "settingsTextScale");
        QVERIFY(scaleChoice);
        QCOMPARE(scaleChoice->property("font").value<QFont>().pixelSize(), 13);
        scaleChoice->forceActiveFocus();
        QTest::keyClick(settings, Qt::Key_Down);
        QTest::keyClick(settings, Qt::Key_End);
        QTest::keyClick(settings, Qt::Key_Return);
        QTRY_COMPARE(settings->property("draftScale").toDouble(), 1.5);
        QCOMPARE(scaleChoice->property("font").value<QFont>().pixelSize(), 20);
        QCOMPARE(scaleChoice->property("count").toInt(), 3);
        auto *cancel = findVisual(settings->contentItem(), "settingsCancel");
        auto *save = findVisual(settings->contentItem(), "settingsSave");
        QVERIFY(cancel && save);
        auto *defaults = findVisual(settings->contentItem(), "settingsDefaults");
        QVERIFY(defaults);
        QTRY_VERIFY(qAbs(cancel->mapToScene(QPointF()).y() - save->mapToScene(QPointF()).y()) <= 1);
        QTRY_VERIFY(qAbs(defaults->mapToScene(QPointF()).y() - save->mapToScene(QPointF()).y()) <=
                    1);
        QTRY_VERIFY(save->mapToScene(QPointF(0, save->height())).y() <= settings->height() + 1);
        QVERIFY(captureSettings("settings-large"));
        auto *languageChoice = findVisual(settings->contentItem(), "settingsLanguage");
        auto *settingsFooter = findVisual(settings->contentItem(), "settingsFooter");
        QVERIFY(languageChoice && settingsFooter);
        languageChoice->forceActiveFocus();
        auto *languagePopup = languageChoice->property("popup").value<QObject *>();
        QVERIFY(languagePopup);
        QVERIFY(QMetaObject::invokeMethod(languagePopup, "open"));
        QTRY_VERIFY(languagePopup->property("visible").toBool());
        QVERIFY(captureSettings("settings-language-menu-large"));
        QTest::keyClick(settings, Qt::Key_Escape);
        languageChoice->forceActiveFocus();
        QTRY_VERIFY(languageChoice->mapToScene(QPointF(0, languageChoice->height())).y() <=
                    settingsFooter->y() + 1);
        auto *densityChoice = findVisual(settings->contentItem(), "settingsDensity");
        QVERIFY(densityChoice);
        settings->setProperty("draftDensity", "compact");
        settings->setProperty("clearRecent", true);
        click(settings, cancel);
        QTRY_VERIFY(!settings->isVisible());
        QTRY_COMPARE(window->property("darkMode").toBool(), originalDark);
        QVERIFY(!QFile::exists(path));
        click(window, entry);
        QTRY_VERIFY(settings->isVisible());
        QCOMPARE(settings->property("draftScale").toDouble(), 1.0);
        settings->setProperty("draftTheme", "dark");
        settings->setProperty("draftLanguage", "en-US");
        QCOMPARE(settings->property("draftDensity").toString(), "standard");
        QCOMPARE(settings->property("clearRecent").toBool(), false);
        settings->setProperty("draftDensity", "compact");
        click(settings, save);
        QTRY_VERIFY(!settings->isVisible());
        QSettings persisted(path, QSettings::IniFormat);
        QCOMPARE(persisted.value("locale/language").toString(), "en-US");
        QCOMPARE(persisted.value("appearance/theme").toString(), "dark");
        QCOMPARE(persisted.value("schemaVersion").toInt(), 1);
        QCOMPARE(qtTrId("settings.title"), QStringLiteral("设置"));
        UiPreferences restored;
        QCOMPARE(restored.language(), "en-US");
        QCOMPARE(restored.savedTheme(), "dark");
        QCOMPARE(restored.savedDensity(), "compact");
        const auto recentPath = profile.filePath("recent.json");
        restored.rememberArchive(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QVERIFY(QFile::exists(recentPath));
        QFile recent(recentPath);
        QVERIFY(recent.open(QIODevice::ReadOnly));
        const auto before = recent.readAll();
        recent.close();
        QVERIFY(restored.saveDetails("dark", 1, "en-US", "compact", false, false));
        restored.rememberArchive(QStringLiteral(XDZIP_FIXTURES "/invalid.zip"));
        QVERIFY(recent.open(QIODevice::ReadOnly));
        QCOMPARE(recent.readAll(), before);
        recent.close();
        QVERIFY(restored.saveDetails("dark", 1, "en-US", "compact", false, true));
        QVERIFY(!QFile::exists(recentPath));
        QVERIFY(QFile::exists(QStringLiteral(XDZIP_FIXTURES "/normal.zip")));
        persisted.setValue("appearance/textScale", 2);
        persisted.sync();
        UiPreferences migrated;
        QCOMPARE(migrated.savedTextScale(), 1.5);
        QVERIFY(!migrated.save("dark", 2, "en-US"));
        QVERIFY(!restored.save("invalid", 1, "en-US"));
        QCOMPARE(restored.savedTheme(), "dark");
        // A separate process proves language restoration across an actual restart.
        QProcess child;
        child.setProcessChannelMode(QProcess::MergedChannels);
        auto environment = QProcessEnvironment::systemEnvironment();
        environment.insert("XDZIP_UI_TEST_LANGUAGE", "");
        environment.remove("XDZIP_LANGUAGE");
        environment.insert("XDZIP_SETTINGS_CHILD", "1");
        child.setProcessEnvironment(environment);
        child.start(QCoreApplication::applicationFilePath(), {"settingsEnglishStartup"});
        QVERIFY(child.waitForFinished(20000));
        QVERIFY2(child.exitStatus() == QProcess::NormalExit && child.exitCode() == 0,
                 child.readAllStandardOutput().constData());
        window->close();
    }
    void settingsEnglishStartup() {
        if (qEnvironmentVariable("XDZIP_SETTINGS_CHILD") != "1")
            QSKIP("Covered by the isolated settings restart child process");
        QCOMPARE(QCoreApplication::instance()->property("xdzipUiLanguage").toString(), "en-US");
        QCOMPARE(qtTrId("settings.title"), QStringLiteral("Settings"));
        const auto systemLanguages = QLocale::system().uiLanguages();
        const bool systemChinese =
            !systemLanguages.isEmpty() && systemLanguages.first().startsWith("zh");
        QCOMPARE(systemDialogText("ui.1b59353f1874"),
                 systemChinese ? QStringLiteral("打开压缩包") : QStringLiteral("Open archive"));
        if (systemChinese)
            QVERIFY(QCoreApplication::translate("QFileDialog", "Open")
                        .contains(QStringLiteral("打开")));
        QCOMPARE(qtTrId("ui.2e84a56c5f19", 1), QStringLiteral("1 file"));
        QCOMPARE(qtTrId("ui.2e84a56c5f19", 2), QStringLiteral("2 files"));
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(QTest::qWaitForWindowExposed(window));
        auto *open = findVisual(window->contentItem(), "welcomeOpen");
        QVERIFY(open);
        QCOMPARE(open->property("title").toString(), "Open archive");
        auto *settings = window->findChild<QQuickWindow *>("settingsWindow");
        QVERIFY(settings);
        QVERIFY(QMetaObject::invokeMethod(settings, "configure"));
        QTRY_VERIFY(settings->isVisible());
        QCOMPARE(settings->title(), "Settings");
        const auto directory = qEnvironmentVariable("XDZIP_SETTINGS_VISUAL_AUDIT");
        if (!directory.isEmpty())
            QVERIFY(visual_audit::save(settings, directory, "settings-en-US"));
        for (auto *owned : window->findChildren<QQuickWindow *>())
            QVERIFY(!owned->title().contains(QRegularExpression("[\\x{3400}-\\x{9fff}]")));
        settings->close();
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        QVERIFY(!controller->message().contains(QRegularExpression("[\\x{3400}-\\x{9fff}]")));
        window->close();
    }
    void controlsDismissFocusWithoutLosingActions() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(QTest::qWaitForWindowExposed(window));
        QTest::qWait(50);
        auto *open = findVisual(window->contentItem(), "welcomeOpen");
        QVERIFY(open);
        open->forceActiveFocus();
        QVERIFY(open->hasActiveFocus());
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!open->hasActiveFocus());
        QVERIFY(window->isVisible());
        open->forceActiveFocus();
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier,
                          QPoint(window->width() - 8, window->height() - 8));
        QTRY_VERIFY(!open->hasActiveFocus());
        auto *create = findVisual(window->contentItem(), "welcomeCreate");
        auto *dialog = window->findChild<QQuickWindow *>("createWindow");
        QVERIFY(create && dialog);
        create->forceActiveFocus();
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_VERIFY(dialog->isVisible());
        QTRY_VERIFY(!create->hasActiveFocus());
        for (auto *ownedWindow : window->findChildren<QQuickWindow *>())
            QVERIFY(!ownedWindow->title().contains(QStringLiteral(" — xdzip")));
        auto *name = findVisual(dialog->contentItem(), "createName");
        QVERIFY(name);
        name->forceActiveFocus();
        QTest::keyClick(dialog, Qt::Key_Escape);
        QTRY_VERIFY(!name->hasActiveFocus());
        QVERIFY(dialog->isVisible());
        name->forceActiveFocus();
        QTest::keyClick(dialog, Qt::Key_Enter);
        QTRY_VERIFY(!name->hasActiveFocus());
        QVERIFY(dialog->isVisible());
        dialog->close();
        window->close();
    }
    void welcomeOpensRealArchivesAndReturns() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        auto *welcome = findVisual(window->contentItem(), "welcomePage");
        auto *workspace = findVisual(window->contentItem(), "archiveWorkspace");
        auto *open = findVisual(window->contentItem(), "welcomeOpen");
        auto *createAction = findVisual(window->contentItem(), "welcomeCreate");
        auto *create = window->findChild<QQuickWindow *>("createWindow");
        QVERIFY(controller && welcome && workspace && open && createAction && create);
        QTRY_VERIFY(controller->runtimeAvailable() && welcome->isVisible());
        QVERIFY(!workspace->isVisible());
        window->resize(1024, 622);
        QTRY_COMPARE(open->width(), 296.0);
        QTRY_COMPARE(open->height(), 124.0);
        QTRY_VERIFY(visibleImagesReady(window->contentItem()));
        auto *titlebar = findVisual(window->contentItem(), "archiveTitlebar");
        auto *tab = findVisual(window->contentItem(), "activeArchiveTab");
        auto *settings = findVisual(window->contentItem(), "archiveSettings");
        QVERIFY(titlebar && tab && settings);
        const QRectF titlebarBefore(titlebar->position(), titlebar->size());
        const QRectF tabBefore(tab->position(), tab->size());
        const QRectF settingsBefore(settings->position(), settings->size());
        const auto captures = qEnvironmentVariable("XDZIP_WELCOME_VISUAL_AUDIT");
        if (!captures.isEmpty()) {
            QVERIFY(QDir().mkpath(captures));
            QVERIFY(visual_audit::save(window, captures, "welcome-light"));
            window->setProperty("darkMode", true);
            QVERIFY(visual_audit::save(window, captures, "welcome-dark"));
            window->resize(832, 544);
            QVERIFY(visual_audit::save(window, captures, "welcome-dark-minimum"));
            window->setProperty("darkMode", false);
            QVERIFY(visual_audit::save(window, captures, "welcome-minimum"));
            window->resize(1024, 622);
        }
        const auto click = [&](QQuickItem *item) {
            const auto point = item->mapToScene(QPointF(item->width() / 2, item->height() / 2));
            QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, point.toPoint());
        };
        click(findVisual(window->contentItem(), "browserTasks"));
        auto *taskMenu = window->findChild<QObject *>("browserTaskMenu");
        QVERIFY(taskMenu);
        QTRY_VERIFY(taskMenu->property("visible").toBool());
        QCOMPARE(taskMenu->property("width").toReal(), 240.0);
        QCOMPARE(taskMenu->property("font").value<QFont>().pixelSize(), 13);
        QVERIFY(QMetaObject::invokeMethod(taskMenu, "close"));
        click(createAction);
        QTRY_VERIFY(create->isVisible());
        create->close();
        QTRY_VERIFY(!create->isVisible() && welcome->isVisible() && !controller->busy());
        // The welcome entry must not fabricate a task when a draft is dismissed.
        QCOMPARE(controller->tasks().size(), 0);
        const auto archive = QUrl::fromLocalFile(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QVERIFY(!controller->openDroppedUrls({}));
        QVERIFY(!controller->openDroppedUrls({archive, archive}));
        QVERIFY(!controller->openDroppedUrls({QUrl("https://example.invalid/a.zip")}));
        QVERIFY(
            !controller->openDroppedUrls({QUrl::fromLocalFile(QStringLiteral(XDZIP_FIXTURES))}));
        QTemporaryDir inputs;
        QVERIFY(inputs.isValid());
        QFile text(inputs.filePath("notes.txt"));
        QVERIFY(text.open(QIODevice::WriteOnly));
        text.write("not an archive");
        text.close();
        QVERIFY(!controller->openDroppedUrls({QUrl::fromLocalFile(text.fileName())}));
        QVERIFY(welcome->isVisible());
        QVERIFY(controller->canOpenDroppedUrls({archive}));
        QQuickWindow dragSource;
        dragSource.show();
        QVERIFY(QTest::qWaitForWindowExposed(&dragSource));
        dragSource.requestActivate();
        if (QGuiApplication::platformName() == "cocoa")
            QTRY_COMPARE(QGuiApplication::focusWindow(), &dragSource);
        QVERIFY(QMetaObject::invokeMethod(
            welcome, "archiveDropped", Q_ARG(QVariant, QVariant::fromValue(QList<QUrl>{archive}))));
        QTRY_VERIFY(!controller->busy() && controller->ready());
        QVERIFY(!welcome->isVisible() && workspace->isVisible());
        if (QGuiApplication::platformName() == "cocoa")
            QTRY_COMPARE(QGuiApplication::focusWindow(), window);
        QCOMPARE(QRectF(titlebar->position(), titlebar->size()), titlebarBefore);
        QCOMPARE(QRectF(tab->position(), tab->size()), tabBefore);
        QCOMPARE(QRectF(settings->position(), settings->size()), settingsBefore);
        controller->closeArchive();
        QTRY_VERIFY(!controller->busy() && !controller->ready());
        QTRY_VERIFY(welcome->isVisible() && !workspace->isVisible());
        QTRY_VERIFY(open->hasActiveFocus());
        // An empty, valid archive is a browser state, never the welcome page.
        QVERIFY(controller->openDroppedUrls(
            {QUrl::fromLocalFile(QStringLiteral(XDZIP_FIXTURES "/empty.zip"))}));
        QTRY_VERIFY(!controller->busy() && controller->ready());
        QCOMPARE(controller->archiveTotal(), 0);
        QVERIFY(!welcome->isVisible());
        controller->closeArchive();
        QTRY_VERIFY(!controller->busy() && welcome->isVisible());
        QFile invalid(inputs.filePath("broken.zip"));
        QVERIFY(invalid.open(QIODevice::WriteOnly));
        invalid.write("not an archive");
        invalid.close();
        QVERIFY(controller->openDroppedUrls({QUrl::fromLocalFile(invalid.fileName())}));
        QTRY_VERIFY(!controller->busy());
        QVERIFY(!controller->ready() && welcome->isVisible());
        QVERIFY(!controller->message().isEmpty());
        QCOMPARE(controller->tasks().size(), 0);
        controller->requestClose();
    }
    void welcomeOpeningCanCancelRealRead() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        auto *welcome = findVisual(window->contentItem(), "welcomePage");
        auto *overlay = findVisual(window->contentItem(), "welcomeOpeningOverlay");
        auto *cancel = findVisual(window->contentItem(), "welcomeOpeningCancel");
        QVERIFY(controller && welcome && overlay && cancel);
        controller->openPath(QStringLiteral(XDZIP_M2_FIXTURE));
        QVERIFY(controller->browsingBusy() && !controller->ready());
        QVERIFY(overlay->isVisible());
        QVERIFY(QMetaObject::invokeMethod(cancel, "clicked"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 20000);
        QVERIFY(!overlay->isVisible());
        // Completion may beat cancellation; either terminal state must remove the mask.
        if (!controller->ready())
            QVERIFY(welcome->isVisible());
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy(), 10000);
        QVERIFY(!welcome->isVisible() && !overlay->isVisible());
    }
    void creationControlsMatchReviewedSource() {
        QTemporaryDir input, output;
        QVERIFY(input.isValid() && output.isValid());
        const QStringList names{QStringLiteral("冰川湖.jpg"), QStringLiteral("山间日出.jpg"),
                                QStringLiteral("行程手册.pdf")};
        const QList<qint64> sizes{8400000, 6200000, 12800000};
        QStringList paths;
        for (int i = 0; i < names.size(); ++i) {
            paths << input.filePath(names[i]);
            QFile file(paths.last());
            QVERIFY(file.open(QIODevice::WriteOnly));
            QVERIFY(file.resize(sizes[i]));
        }
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        auto *create = window->findChild<QQuickWindow *>("createWindow");
        QVERIFY(controller && create);
        QVERIFY(QMetaObject::invokeMethod(create, "configure"));
        QTRY_VERIFY(create->isVisible() && !controller->busy());
        QVERIFY(QMetaObject::invokeMethod(create, "addInputPaths", Q_ARG(QVariant, QVariant(paths)),
                                          Q_ARG(bool, false)));
        // Metadata is advisory and asynchronous; wait for it rather than putting
        // the illustrative 27.4 MB string directly into the product.
        QTRY_COMPARE(controller->creationInputs().size(), 3);
        auto *summary = findVisual(create->contentItem(), "createInputSummary");
        QVERIFY(summary);
        QTRY_COMPARE(summary->property("text").toString(), QStringLiteral("3 个文件，27.4 MB"));
        auto *name = findVisual(create->contentItem(), "createName");
        auto *target = findVisual(create->contentItem(), "createTarget");
        auto *method = findVisual(create->contentItem(), "createLevel");
        auto *remove = findVisual(create->contentItem(), "createRemoveInput-0");
        auto *group = findVisual(create->contentItem(), "createFormat");
        QVERIFY(name && target && method && remove && group);
        name->setProperty("text", QStringLiteral("北境旅行.zip"));
        target->setProperty("text", output.path());
        controller->checkCreationTarget(output.path());
        QTRY_VERIFY(!controller->busy() && checkedTarget(controller, output.path()));
        auto *volume = findVisual(create->contentItem(), "createVolume");
        auto *unit = findVisual(create->contentItem(), "createVolumeUnit");
        auto *usePassword = findVisual(create->contentItem(), "createUsePassword");
        auto *password = findVisual(create->contentItem(), "createPassword");
        auto *start = findVisual(create->contentItem(), "startCreate");
        QVERIFY(volume && unit && usePassword && password && start);
        QCOMPARE(method->property("currentIndex").toInt(), 2);
        volume->setProperty("currentIndex", 13);
        QTRY_COMPARE(volume->property("editText").toString(), QString{});
        QTRY_VERIFY(!start->isEnabled());
        volume->setProperty("editText", QStringLiteral("25"));
        unit->setProperty("currentIndex", 2);
        QTRY_COMPARE(create->property("volumeBytes").toDouble(), 25000000.0);
        QTRY_VERIFY(start->isEnabled());
        volume->setProperty("editText", QStringLiteral("0"));
        QTRY_VERIFY(!start->isEnabled());
        volume->setProperty("currentIndex", 0);
        usePassword->setProperty("checked", true);
        QTRY_VERIFY(password->isEnabled() && !start->isEnabled());
        password->setProperty("text", QStringLiteral("public fixture"));
        QTRY_VERIFY(start->isEnabled());
        password->setProperty("text", QStringLiteral("invalid\n"));
        QTRY_VERIFY(!start->isEnabled());
        usePassword->setProperty("checked", false);
        QTRY_COMPARE(password->property("text").toString(), QString{});
        QTRY_VERIFY(start->isEnabled());
        auto *tar = findVisual(create->contentItem(), "createFormat-1");
        auto *zip = findVisual(create->contentItem(), "createFormat-0");
        QVERIFY(zip && tar && zip->isEnabled() && tar->isEnabled());
        zip->forceActiveFocus(Qt::TabFocusReason);
        QTest::keyClick(create, Qt::Key_Right);
        QCOMPARE(group->property("currentIndex").toInt(), 1);
        QTRY_VERIFY(name->property("text").toString().endsWith(QStringLiteral(".tar")));
        QVERIFY(!method->isEnabled() && !volume->isEnabled() && !usePassword->isEnabled());
        group->setProperty("currentIndex", 2);
        QTRY_VERIFY(name->property("text").toString().endsWith(QStringLiteral(".tar.gz")));
        QVERIFY(method->isEnabled() && !volume->isEnabled());
        group->setProperty("currentIndex", 0);
        QTRY_VERIFY(method->isEnabled() && volume->isEnabled());
        name->forceActiveFocus(Qt::TabFocusReason);
        QTest::keyClick(create, Qt::Key_Tab);
        const auto captures = qEnvironmentVariable("XDZIP_CREATE_CONTROLS_AUDIT");
        const auto capture = [&](const QString &scene) {
            return captures.isEmpty() ||
                   (QDir().mkpath(captures) && visual_audit::save(create, captures, scene));
        };
        // Clear keyboard focus for normal-state border sampling.
        target->setProperty("focus", false);
        create->contentItem()->forceActiveFocus();
        QTRY_VERIFY(visibleImagesReady(create->contentItem()));
        QVERIFY(capture("create-light"));
        window->setProperty("darkMode", true);
        QTRY_VERIFY(visibleImagesReady(create->contentItem()));
        QVERIFY(capture("create-dark"));
        window->setProperty("darkMode", false);
        remove->forceActiveFocus(Qt::TabFocusReason);
        QTest::qWait(20); // reveal controls in the body on a short high-DPR screen
        create->raise();
        create->requestActivate();
        QVERIFY(QTest::qWaitForWindowActive(create));
        create->contentItem()->forceActiveFocus();
        QTest::mouseMove(
            create,
            remove->mapToScene(QPointF(remove->width() / 2, remove->height() / 2)).toPoint());
        QTRY_VERIFY(remove->property("hovered").toBool());
        QVERIFY(capture("create-remove-hover"));
        name->forceActiveFocus(Qt::TabFocusReason);
        QVERIFY(capture("create-field-focus"));
        method->forceActiveFocus(Qt::TabFocusReason);
        QTest::qWait(20);
        QTest::mouseClick(
            create, Qt::LeftButton, Qt::NoModifier,
            method->mapToScene(QPointF(method->width() / 2, method->height() / 2)).toPoint());
        auto *popup = method->property("popup").value<QObject *>();
        QVERIFY(popup);
        QTRY_VERIFY(popup->property("visible").toBool());
        QVERIFY(capture("create-dropdown"));
        QTest::keyClick(create, Qt::Key_End);
        QTest::keyClick(create, Qt::Key_Return);
        QTRY_COMPARE(method->property("currentIndex").toInt(), 3);
        QTRY_VERIFY(!popup->property("visible").toBool());
        auto *level = findVisual(create->contentItem(), "createLevel");
        QVERIFY(level && level->isEnabled());
        remove->forceActiveFocus(Qt::TabFocusReason);
        QTest::qWait(20);
        QTest::mouseClick(
            create, Qt::LeftButton, Qt::NoModifier,
            remove->mapToScene(QPointF(remove->width() / 2, remove->height() / 2)).toPoint());
        QTRY_COMPARE(controller->creationInputs().size(), 2);
        QTRY_COMPARE(summary->property("text").toString(), QStringLiteral("2 个文件，19.0 MB"));
        // Removing an input is selection editing, never a filesystem deletion.
        for (const auto &path : paths)
            QVERIFY(QFileInfo::exists(path));
        QVERIFY(capture("create-removed"));
        create->close();
        controller->requestClose();
        QTRY_VERIFY(controller->requestClose());
    }
    void tarCreationBrowseAndSafeExtraction() {
        for (int formatIndex : {1, 2}) {
            QTemporaryDir input, output, extracted;
            QVERIFY(input.isValid() && output.isValid() && extracted.isValid());
            const auto source = input.path() + QStringLiteral("/资料");
            QVERIFY(QDir().mkpath(source + QStringLiteral("/空目录")));
            const auto data = QStringLiteral("TAR 与 GZIP 真实读写\n").repeated(200).toUtf8();
            QFile file(source + QStringLiteral("/内容.txt"));
            QVERIFY(file.open(QIODevice::WriteOnly));
            QCOMPARE(file.write(data), data.size());
            file.close();
            QQmlApplicationEngine engine;
            engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
            QVERIFY(!engine.rootObjects().isEmpty());
            auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
            auto *controller = window->findChild<BrowserController *>();
            auto *create = window->findChild<QQuickWindow *>("createWindow");
            QVERIFY(controller && create);
            QVERIFY(QMetaObject::invokeMethod(create, "configure"));
            QTRY_VERIFY(create->isVisible() && !controller->busy());
            QCOMPARE(create->width(), 491);
            const auto captures = qEnvironmentVariable("XDZIP_TAR_VISUAL_AUDIT");
            if (!captures.isEmpty())
                QVERIFY(QDir().mkpath(captures));
            QVERIFY(QMetaObject::invokeMethod(create, "addInputPaths",
                                              Q_ARG(QVariant, QVariant(QStringList{source})),
                                              Q_ARG(bool, true)));
            auto *group = findVisual(create->contentItem(), "createFormat");
            auto *name = findVisual(create->contentItem(), "createName");
            auto *target = findVisual(create->contentItem(), "createTarget");
            QVERIFY(group && name && target);
            group->setProperty("currentIndex", formatIndex);
            const QString suffix =
                formatIndex == 1 ? QStringLiteral(".tar") : QStringLiteral(".tar.gz");
            QTRY_VERIFY(name->property("text").toString().endsWith(suffix));
            name->setProperty("text", QStringLiteral("归档") + suffix);
            target->setProperty("text", output.path());
            QTRY_VERIFY(!controller->busy());
            controller->checkCreationTarget(output.path());
            QTRY_VERIFY(!controller->busy());
            QVERIFY(checkedTarget(controller, output.path()));
            auto *start = findVisual(create->contentItem(), "startCreate");
            QVERIFY(start);
            QTRY_VERIFY(start->isEnabled());
            if (!captures.isEmpty())
                QVERIFY(visual_audit::save(create, captures,
                                           formatIndex == 1 ? "create-tar" : "create-tar-gzip"));
            QSignalSpy started(controller, &BrowserController::creationStarted);
            QTest::mouseClick(
                create, Qt::LeftButton, Qt::NoModifier,
                start->mapToScene(QPointF(start->width() / 2, start->height() / 2)).toPoint());
            QTRY_COMPARE(started.count(), 1);
            QTRY_COMPARE_WITH_TIMEOUT(controller->activeTaskCount(), 0, 10000);
            QCOMPARE(controller->tasks().first().toMap().value("status").toString(),
                     QString("succeeded"));
            QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy() &&
                                         controller->title() == QStringLiteral("归档") + suffix,
                                     10000);
            QCOMPARE(controller->formatLabel(),
                     formatIndex == 1 ? QString("TAR") : QString("TAR.GZ"));
            QCOMPARE(controller->archiveTotal(), 3);
            auto *badge = findVisual(window->contentItem(), "archiveFormatBadge");
            auto *label = findVisual(window->contentItem(), "archiveFormatLabel");
            QVERIFY(badge && label);
            QTRY_VERIFY(badge->width() >= label->width() + 12);
            controller->enterDirectory(0);
            QTRY_VERIFY(!controller->busy());
            controller->selectAll(true);
            QVERIFY(!controller->detail().value("modified").toString().contains(QChar(0x2014)));
            QCOMPARE(controller->detail().value("packed").toString().isEmpty(), formatIndex == 2);
            if (!captures.isEmpty())
                QVERIFY(visual_audit::save(window, captures,
                                           formatIndex == 1 ? "opened-tar" : "opened-tar-gzip"));
            controller->prepareExtraction(true);
            auto *extract = window->findChild<QQuickWindow *>("extractWindow");
            QVERIFY(extract);
            QTRY_VERIFY(extract->isVisible());
            auto *extractTarget = findVisual(extract->contentItem(), "extractTarget");
            QVERIFY(extractTarget);
            extractTarget->setProperty("text", extracted.path());
            QTRY_VERIFY(!controller->busy());
            controller->checkTarget(extracted.path());
            QTRY_VERIFY(!controller->busy());
            QVERIFY(checkedTarget(controller, extracted.path()));
            auto *folder = findVisual(extract->contentItem(), "extractSameFolder");
            QVERIFY(folder);
            QCOMPARE(controller->extractionIntent().value("folder").toString(),
                     QStringLiteral("归档"));
            QTRY_COMPARE(extract->property("archiveFolder").toString(), QStringLiteral("归档"));
            if (formatIndex == 1)
                folder->setProperty("checked", false);
            const auto resultRoot =
                extracted.path() + (formatIndex == 2 ? QStringLiteral("/归档") : QString{});
            auto *extractStart = findVisual(extract->contentItem(), "startExtract");
            QVERIFY(extractStart);
            QTRY_VERIFY(extractStart->isEnabled());
            QTest::mouseClick(
                extract, Qt::LeftButton, Qt::NoModifier,
                extractStart
                    ->mapToScene(QPointF(extractStart->width() / 2, extractStart->height() / 2))
                    .toPoint());
            QTRY_COMPARE_WITH_TIMEOUT(controller->tasks().size(), 2, 10000);
            QTRY_COMPARE_WITH_TIMEOUT(controller->activeTaskCount(), 0, 10000);
            for (const auto &task : controller->tasks())
                QCOMPARE(task.toMap().value("status").toString(), QString("succeeded"));
            QFile actual(resultRoot + QStringLiteral("/资料/内容.txt"));
            QVERIFY(actual.open(QIODevice::ReadOnly));
            QCOMPARE(actual.readAll(), data);
            QVERIFY(QFileInfo(resultRoot + QStringLiteral("/资料/空目录")).isDir());
            if (!captures.isEmpty())
                QVERIFY(visual_audit::save(window->findChild<QQuickWindow *>("taskWindow"),
                                           captures,
                                           formatIndex == 1 ? "result-tar" : "result-tar-gzip"));
            controller->requestClose();
            QTRY_VERIFY(controller->requestClose());
        }
    }
    void m4CreationWindowAndRoundTrip() {
        QTemporaryDir input, output;
        QVERIFY(input.isValid() && output.isValid());
        const auto source = input.path() + QStringLiteral("/输入");
        QVERIFY(QDir().mkpath(source + QStringLiteral("/空目录")));
        QFile file(source + QStringLiteral("/文件.txt"));
        QVERIFY(file.open(QIODevice::WriteOnly));
        const auto data = QStringLiteral("真实 ZIP 创建与读回\n").repeated(1024).toUtf8();
        QCOMPARE(file.write(data), data.size());
        file.close();
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        auto *create = window->findChild<QQuickWindow *>("createWindow");
        QVERIFY(controller && create);
        auto *newButton = findVisual(window->contentItem(), "welcomeCreate");
        QVERIFY(newButton && newButton->isEnabled());
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            newButton->mapToScene(QPointF(newButton->width() / 2, newButton->height() / 2))
                .toPoint());
        QTRY_VERIFY(create->isVisible());
        QTRY_VERIFY(!controller->busy());
        QCOMPARE(create->width(), 491);
        QVERIFY(QMetaObject::invokeMethod(create, "addInputPaths",
                                          Q_ARG(QVariant, QVariant(QStringList{source})),
                                          Q_ARG(bool, true)));
        QVERIFY(QMetaObject::invokeMethod(
            create, "addInputPaths",
            Q_ARG(QVariant, QVariant(QStringList{source + QStringLiteral("/文件.txt")})),
            Q_ARG(bool, false)));
        QVERIFY(QMetaObject::invokeMethod(
            create, "addInputPaths",
            Q_ARG(QVariant, QVariant(QStringList{source + QStringLiteral("/空目录")})),
            Q_ARG(bool, true)));
        auto *name = findVisual(create->contentItem(), "createName");
        auto *target = findVisual(create->contentItem(), "createTarget");
        auto *method = findVisual(create->contentItem(), "createLevel");
        QVERIFY(name && target && method);
        name->setProperty("text", QStringLiteral("创建.zip"));
        target->setProperty("text", output.path());
        method->setProperty("currentIndex", 0);
        auto *level = findVisual(create->contentItem(), "createLevel");
        QVERIFY(level);
        QTRY_VERIFY(level->isEnabled());
        controller->checkCreationTarget(output.path());
        QTRY_VERIFY(!controller->busy());
        QVERIFY(checkedTarget(controller, output.path()));
        QTRY_VERIFY(visibleImagesReady(create->contentItem()));
        const auto captures = qEnvironmentVariable("XDZIP_M4_VISUAL_AUDIT");
        if (!captures.isEmpty()) {
            QVERIFY(QDir().mkpath(captures));
            QVERIFY(visual_audit::save(create, captures, "create-selected"));
            window->setProperty("darkMode", true);
            QTest::qWait(20);
            QVERIFY(visual_audit::save(create, captures, "create-dark"));
            window->setProperty("darkMode", false);
        }
        auto *start = findVisual(create->contentItem(), "startCreate");
        QVERIFY(start);
        QTRY_VERIFY(start->isEnabled());
        QSignalSpy started(controller, &BrowserController::creationStarted);
        QTest::mouseClick(
            create, Qt::LeftButton, Qt::NoModifier,
            start->mapToScene(QPointF(start->width() / 2, start->height() / 2)).toPoint());
        QTRY_COMPARE(started.count(), 1);
        QTRY_VERIFY(!create->isVisible());
        QTRY_COMPARE_WITH_TIMEOUT(controller->activeTaskCount(), 0, 10000);
        const auto tasks = controller->tasks();
        QVERIFY(!tasks.isEmpty());
        const auto task = tasks.first().toMap();
        QCOMPARE(task.value("status").toString(), QString("succeeded"));
        QVERIFY(task.value("creating").toBool() && task.value("totalKnown").toBool());
        QCOMPARE(task.value("committedFiles").toString(), QString("1"));
        const auto archive = output.path() + QStringLiteral("/创建.zip");
        QVERIFY(QFileInfo::exists(archive));
        // Successful creation opens its committed output without a second user action.
        QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy() &&
                                     controller->title() == QStringLiteral("创建.zip"),
                                 10000);
        QCOMPARE(controller->archiveTotal(), 3);
        QCOMPARE(controller->entries()->rowCount(), 1);
        controller->enterDirectory(0);
        QTRY_VERIFY(!controller->busy());
        QCOMPARE(controller->entries()->rowCount(), 2);
        if (!captures.isEmpty()) {
            auto *progress = window->findChild<QQuickWindow *>("taskWindow");
            QVERIFY(progress);
            QVERIFY(visual_audit::save(progress, captures, "create-completed"));
            QVERIFY(visual_audit::save(window, captures, "created-archive-open"));
        }
        QFile archiveFile(archive);
        QVERIFY(archiveFile.open(QIODevice::ReadOnly));
        const auto originalArchive = archiveFile.readAll();
        archiveFile.close();
        // Drive the actual conflict action, then prove whole-archive cancellation
        // preserves the existing file and removes only our staging directory.
        controller->startCreation({source}, output.path(), QStringLiteral("创建.zip"), 8, 6, false);
        auto *conflict = window->findChild<QQuickWindow *>("conflictWindow");
        QVERIFY(conflict);
        QTRY_VERIFY(conflict->isVisible());
        auto *applyAll = findVisual(conflict->contentItem(), "conflictApplyAll");
        QVERIFY(applyAll && !applyAll->isVisible());
        auto *skip = findVisual(conflict->contentItem(), "conflictSkip");
        QVERIFY(skip && skip->isEnabled());
        QTest::mouseClick(
            conflict, Qt::LeftButton, Qt::NoModifier,
            skip->mapToScene(QPointF(skip->width() / 2, skip->height() / 2)).toPoint());
        QTRY_COMPARE(controller->activeTaskCount(), 0);
        QVERIFY(archiveFile.open(QIODevice::ReadOnly));
        QCOMPARE(archiveFile.readAll(), originalArchive);
        archiveFile.close();
        QCOMPARE(QDir(output.path()).entryList(QDir::NoDotAndDotDot | QDir::AllEntries).size(), 1);
        controller->requestClose();
        QTRY_VERIFY(controller->requestClose());
    }
    void explicitOpenConsumesQueuedCreationOutput() {
        DesktopUrlRecorder recorder;
        QDesktopServices::setUrlHandler("file", &recorder, "record");
        const auto restoreHandler = qScopeGuard([] { QDesktopServices::unsetUrlHandler("file"); });
        QTemporaryDir input, output;
        QVERIFY(input.isValid() && output.isValid());
        const auto source = input.path() + "/data.txt";
        QFile file(source);
        QVERIFY(file.open(QIODevice::WriteOnly));
        QCOMPARE(file.write("owned payload"), qint64(13));
        file.close();
        BrowserController controller;
        // Explicit spelling differs from the runtime path, including native separators on Windows.
        const auto archive = output.path() + "/./created.zip";
        bool openedExplicitly = false;
        connect(&controller, &BrowserController::tasksChanged, &controller, [&] {
            if (openedExplicitly || controller.busy() || controller.tasks().isEmpty())
                return;
            if (controller.tasks().first().toMap().value("status").toString() == "succeeded") {
                openedExplicitly = true;
                controller.openPath(archive);
            }
        });
        QSignalSpy opened(&controller, &BrowserController::archiveOpened);
        controller.startCreation({source}, output.path(), "created.zip", 8, 6, true);
        QTRY_VERIFY_WITH_TIMEOUT(openedExplicitly && controller.ready() && !controller.busy(),
                                 10000);
        QTest::qWait(100);
        QCOMPARE(opened.count(), 1);
        QCOMPARE(controller.archiveTotal(), 1);
        QCOMPARE(recorder.count, 0);
        controller.requestClose();
        QTRY_VERIFY(controller.requestClose());
    }
    void operationPreparationKeepsBrowserPresentationStable_data() {
        QTest::addColumn<int>("operation");
        QTest::newRow("extract-all") << 0;
        QTest::newRow("extract-selection") << 1;
        QTest::newRow("target-recheck") << 2;
        QTest::newRow("creation-target") << 3;
        QTest::newRow("creation-input-preview") << 4;
    }
    void operationPreparationKeepsBrowserPresentationStable() {
        QFETCH(int, operation);
        QTemporaryDir target;
        QVERIFY(target.isValid());
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(QTest::qWaitForWindowExposed(window));
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/../extract/tree.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        controller->selectAll(operation != 0);
        QTest::qWait(50); // Allow the real toolbar hit area to finish its layout.
        const auto rows = controller->entries()->rowCount();
        const auto selected = controller->selectedCount();
        auto *overlay = findVisual(window->contentItem(), "archiveLoadingOverlay");
        QVERIFY(overlay && !overlay->isVisible());
        bool controlsStable = true, overlayStable = true, sawPending = false;
        bool presentationStable = true;
        QObject monitor;
        for (const auto &name :
             {"archiveOpen", "archiveNew", "archiveMore", "archiveSearch", "archiveExtract"}) {
            auto *control = findVisual(window->contentItem(), QString::fromLatin1(name));
            QVERIFY(control && control->isEnabled());
            connect(control, &QQuickItem::enabledChanged, &monitor,
                    [&, control] { controlsStable &= control->isEnabled(); });
        }
        connect(overlay, &QQuickItem::visibleChanged, &monitor,
                [&] { overlayStable &= !overlay->isVisible(); });
        connect(controller, &BrowserController::changed, &monitor, [&] {
            if (controller->busy()) {
                sawPending = true;
                presentationStable &=
                    !controller->browsingBusy() && controller->browserQueryAvailable();
            }
        });
        QSignalSpy resets(controller->entries(), &QAbstractItemModel::modelReset);
        if (operation == 0) {
            auto *button = findVisual(window->contentItem(), "archiveExtract");
            QTest::mouseClick(
                window, Qt::LeftButton, Qt::NoModifier,
                button->mapToScene(QPointF(button->width() / 4, button->height() / 2)).toPoint());
        } else if (operation == 1) {
            controller->prepareExtraction(false);
        } else if (operation == 2) {
            controller->prepareExtraction(true);
            QTRY_VERIFY(!controller->busy());
            controller->checkTarget(target.path());
        } else if (operation == 3) {
            controller->checkCreationTarget(target.path());
        } else {
            QFile input(target.filePath("input.txt"));
            QVERIFY(input.open(QIODevice::WriteOnly));
            QCOMPARE(input.write("preview"), 7);
            input.close();
            auto *create = window->findChild<QQuickWindow *>("createWindow");
            QVERIFY(create);
            QVERIFY(QMetaObject::invokeMethod(
                create, "addInputPaths", Q_ARG(QVariant, QVariant(QStringList{input.fileName()})),
                Q_ARG(bool, false)));
            QTRY_COMPARE(controller->creationInputs().size(), 1);
        }
        QTRY_VERIFY(!controller->busy());
        QVERIFY(sawPending);
        QVERIFY(presentationStable);
        QVERIFY(controlsStable);
        QVERIFY(overlayStable);
        QCOMPARE(resets.count(), 0);
        QCOMPARE(controller->entries()->rowCount(), rows);
        QCOMPARE(controller->selectedCount(), selected);
        window->close();
    }
    void extractionSubmissionKeepsBrowserPresentationStable() {
        QTemporaryDir target;
        QVERIFY(target.isValid());
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(QTest::qWaitForWindowExposed(window));
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/../extract/tree.zip"));
        QVERIFY(controller->busy());
        QVERIFY(controller->browsingBusy());
        QVERIFY(!controller->taskStarting());
        QTRY_VERIFY(controller->ready() && !controller->busy());
        controller->selectAll(true);
        const auto selected = controller->selectedCount();
        const auto rows = controller->entries()->rowCount();
        controller->prepareExtraction(false);
        auto *extract = window->findChild<QQuickWindow *>("extractWindow");
        QVERIFY(extract);
        QTRY_VERIFY(extract->isVisible());
        auto *field = findVisual(extract->contentItem(), "extractTarget");
        auto *openAfter = findVisual(extract->contentItem(), "extractOpenAfter");
        auto *start = findVisual(extract->contentItem(), "startExtract");
        auto *change = findVisual(extract->contentItem(), "extractChangeTarget");
        auto *overlay = findVisual(window->contentItem(), "archiveLoadingOverlay");
        QVERIFY(field && openAfter && start && change && overlay);
        for (const auto &name :
             {"archiveOpen", "archiveNew", "archiveTest", "archiveMore", "archiveExtract"}) {
            auto *action = findVisual(window->contentItem(), QString::fromLatin1(name));
            QVERIFY(action);
            QVERIFY(!action->property("tooltipEnabled").toBool());
        }
        field->setProperty("text", target.path());
        QTRY_VERIFY(!controller->busy());
        controller->checkTarget(target.path());
        QTRY_VERIFY(!controller->busy());
        QVERIFY(checkedTarget(controller, target.path()));
        openAfter->setProperty("checked", false);
        QTRY_VERIFY(start->isEnabled());
        QVERIFY(!overlay->isVisible());
        bool controlsStable = true;
        bool overlayStable = true;
        bool sawSubmitting = false;
        bool sawStartDisabled = false;
        bool retried = false;
        QObject monitor;
        for (const auto &name :
             {"archiveOpen", "archiveNew", "archiveMore", "archiveSearch", "archiveExtract"}) {
            auto *control = findVisual(window->contentItem(), QString::fromLatin1(name));
            QVERIFY(control && control->isEnabled());
            connect(control, &QQuickItem::enabledChanged, &monitor,
                    [&, control] { controlsStable &= control->isEnabled(); });
        }
        QVERIFY(change->isEnabled());
        connect(change, &QQuickItem::enabledChanged, &monitor,
                [&] { controlsStable &= change->isEnabled(); });
        connect(overlay, &QQuickItem::visibleChanged, &monitor,
                [&] { overlayStable &= !overlay->isVisible(); });
        connect(controller, &BrowserController::changed, &monitor, [&] {
            if (controller->taskStarting()) {
                sawSubmitting = true;
                if (!retried) {
                    retried = true;
                    controller->startExtraction(target.path(), false, true, 0, false, false);
                }
                controlsStable &=
                    !controller->browsingBusy() && controller->browserQueryAvailable();
            }
        });
        connect(start, &QQuickItem::enabledChanged, &monitor,
                [&] { sawStartDisabled |= controller->taskStarting() && !start->isEnabled(); });
        QSignalSpy resets(controller->entries(), &QAbstractItemModel::modelReset);
        QSignalSpy started(controller, &BrowserController::extractionStarted);
        // Use the real pointer path; the changed handler also attempts a duplicate request.
        QTest::mouseClick(
            extract, Qt::LeftButton, Qt::NoModifier,
            start->mapToScene(QPointF(start->width() / 2, start->height() / 2)).toPoint());
        QTRY_COMPARE(started.count(), 1);
        QTRY_VERIFY(!extract->isVisible());
        QTRY_VERIFY_WITH_TIMEOUT(!controller->tasks().isEmpty() &&
                                     controller->tasks().first().toMap().value("terminal").toBool(),
                                 10000);
        QVERIFY(sawSubmitting);
        QVERIFY(!sawStartDisabled);
        QVERIFY(controlsStable);
        QVERIFY(overlayStable);
        QCOMPARE(resets.count(), 0);
        QCOMPARE(controller->entries()->rowCount(), rows);
        QCOMPARE(controller->selectedCount(), selected);
        QCOMPARE(controller->tasks().size(), 1);
        QCOMPARE(controller->tasks().first().toMap().value("status").toString(), "succeeded");
        window->close();
    }
    void rejectedExtractionKeepsStartButtonEnabledWithoutDuplicateSubmission() {
        QTemporaryDir target;
        QVERIFY(target.isValid());
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        QVERIFY(QTest::qWaitForWindowExposed(window));
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/../extract/unsafe.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        controller->prepareExtraction(true);
        auto *extract = window->findChild<QQuickWindow *>("extractWindow");
        QTRY_VERIFY(extract && extract->isVisible());
        auto *field = findVisual(extract->contentItem(), "extractTarget");
        auto *start = findVisual(extract->contentItem(), "startExtract");
        QVERIFY(field && start);
        QTRY_VERIFY(!controller->busy());
        field->setProperty("text", target.path());
        controller->checkTarget(target.path());
        QTRY_VERIFY(!controller->busy() && checkedTarget(controller, target.path()));
        QTRY_VERIFY(start->isEnabled());
        bool stayedEnabled = true;
        bool retried = false;
        QObject monitor;
        connect(start, &QQuickItem::enabledChanged, &monitor,
                [&] { stayedEnabled &= start->isEnabled(); });
        connect(controller, &BrowserController::changed, &monitor, [&] {
            if (controller->taskStarting() && !retried) {
                retried = true;
                controller->startExtraction(target.path(), true, false, 0, false, false);
            }
        });
        QSignalSpy accepted(controller, &BrowserController::extractionStarted);
        QTest::mouseClick(
            extract, Qt::LeftButton, Qt::NoModifier,
            start->mapToScene(QPointF(start->width() / 2, start->height() / 2)).toPoint());
        QTRY_VERIFY(retried && !controller->busy());
        QVERIFY(stayedEnabled);
        QVERIFY(start->isEnabled());
        QVERIFY(extract->isVisible());
        QVERIFY(!controller->extractionIntent().value("error").toString().isEmpty());
        QCOMPARE(accepted.count(), 0);
        QCOMPARE(controller->tasks().size(), 0);
        QCOMPARE(QDir(target.path()).entryList(QDir::NoDotAndDotDot | QDir::AllEntries).size(), 0);
        window->close();
    }
    void unixBundleExtractionThroughNativeController() {
#ifndef Q_OS_UNIX
        QSKIP("ZIP symbolic link restoration is supported on POSIX only");
#else
        QTemporaryDir target;
        QVERIFY(target.isValid());
        BrowserController controller;
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/../extract/unix_bundle.zip"));
        QTRY_VERIFY(controller.ready() && !controller.busy());
        controller.prepareExtraction(true);
        controller.checkTarget(target.path());
        QTRY_VERIFY(!controller.busy() && checkedTarget(&controller, target.path()));
        QSignalSpy started(&controller, &BrowserController::extractionStarted);
        controller.startExtraction(target.path(), true, false, 0, false, false);
        QTRY_COMPARE(started.count(), 1);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.tasks().isEmpty() &&
                                     controller.tasks().first().toMap().value("terminal").toBool(),
                                 10000);
        const auto task = controller.tasks().first().toMap();
        QCOMPARE(task.value("status").toString(), QStringLiteral("succeeded"));
        QCOMPARE(task.value("committedFiles").toString(), QStringLiteral("7"));
        const auto base = target.path() + QStringLiteral("/Demo.app/Contents/");
        QVERIFY(QFileInfo(base + "MacOS/demo").permissions().testFlag(QFile::ExeOwner));
        const auto link = base + "Frameworks/F.framework/F";
        QVERIFY(QFileInfo(link).isSymLink());
        QFile file(link);
        QVERIFY(file.open(QIODevice::ReadOnly));
        QCOMPARE(file.readAll(), QByteArray("owned library\n"));
        file.close();
        controller.requestClose();
        QTRY_VERIFY(controller.requestClose());
#endif
    }
    void m3ExtractionWindowsAndRealOutput() {
        QTemporaryDir target;
        QVERIFY(target.isValid());
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/../extract/tree.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        QCOMPARE(controller->entries()->rowCount(), 3);
        controller->selectAll(true);
        controller->prepareExtraction(false);
        auto *extract = window->findChild<QQuickWindow *>("extractWindow");
        QVERIFY(extract);
        QTRY_VERIFY(extract->isVisible());
        QTRY_VERIFY(visibleImagesReady(extract->contentItem()));
        QCOMPARE(extract->width(), 491);
        auto *targetField = findVisual(extract->contentItem(), "extractTarget");
        QVERIFY(targetField);
        for (const auto key : {Qt::Key_Escape, Qt::Key_Return, Qt::Key_Enter}) {
            targetField->forceActiveFocus(Qt::TabFocusReason);
            QTRY_VERIFY(targetField->hasActiveFocus());
            QTest::keyClick(extract, key);
            QTRY_VERIFY(!targetField->hasActiveFocus());
            QVERIFY(extract->isVisible());
        }
        targetField->forceActiveFocus(Qt::TabFocusReason);
        QTest::mouseClick(extract, Qt::LeftButton, Qt::NoModifier, QPoint(8, 8));
        QTRY_VERIFY(!targetField->hasActiveFocus());
        targetField->setProperty("text", target.path());
        QTRY_VERIFY(!controller->busy());
        controller->checkTarget(target.path());
        QTRY_VERIFY(!controller->busy());
        QVERIFY(checkedTarget(controller, target.path()));
        auto *start = findVisual(extract->contentItem(), "startExtract");
        QVERIFY(start);
        QTRY_VERIFY(start->isEnabled());
        if (!qEnvironmentVariableIsEmpty("XDZIP_M3_VISUAL_AUDIT")) {
            const auto directory = qEnvironmentVariable("XDZIP_M3_VISUAL_AUDIT");
            QVERIFY(QDir().mkpath(directory));
            QVERIFY(visual_audit::save(extract, directory, "extract-selected"));
            window->setProperty("darkMode", true);
            QTRY_VERIFY(visibleImagesReady(extract->contentItem()));
            QVERIFY(visual_audit::save(extract, directory, "extract-dark"));
            window->setProperty("darkMode", false);
            QTRY_VERIFY(visibleImagesReady(extract->contentItem()));
            auto *policy = findVisual(extract->contentItem(), "extractConflictPolicy");
            QVERIFY(policy);
            policy->forceActiveFocus(Qt::TabFocusReason);
            QTest::mouseClick(
                extract, Qt::LeftButton, Qt::NoModifier,
                policy->mapToScene(QPointF(policy->width() / 2, policy->height() / 2)).toPoint());
            auto *popup = policy->property("popup").value<QObject *>();
            QVERIFY(popup);
            QTRY_VERIFY(popup->property("visible").toBool());
            QTest::qWait(50);
            auto *popupContent = popup->property("contentItem").value<QQuickItem *>();
            QVERIFY(popupContent);
            const auto popupTop =
                popupContent->mapToScene(QPointF()).y() - popup->property("topPadding").toDouble();
            const auto fieldTop = policy->mapToScene(QPointF()).y();
            const auto menuGap = popupTop >= fieldTop
                                     ? popupTop - fieldTop - policy->height()
                                     : fieldTop - popupTop - popup->property("height").toDouble();
            QVERIFY(qAbs(menuGap - 4) < 1);
            QVERIFY(visual_audit::save(extract, directory, "extract-dropdown"));
            QTest::keyClick(extract, Qt::Key_Escape);
            QTRY_VERIFY(!popup->property("visible").toBool());
            QVERIFY(extract->isVisible());
        }
        // Test the explicit action without opening a desktop file manager on CI.
        controller->startExtraction(target.path(), false, false, 0, false, false);
        QTRY_VERIFY_WITH_TIMEOUT(!controller->tasks().isEmpty() &&
                                     controller->tasks().first().toMap().value("terminal").toBool(),
                                 10000);
        auto result = controller->tasks().first().toMap();
        QCOMPARE(result.value("status").toString(), "succeeded");
        QCOMPARE(result.value("committedFiles").toString(), "3");
        QCOMPARE(result.value("skippedFiles").toString(), "0");
        QVERIFY(QFile::exists(target.filePath(QStringLiteral("照片/说明.txt"))));
        QFile binary(target.filePath("data.bin"));
        QVERIFY(binary.open(QIODevice::ReadOnly));
        const auto data = binary.readAll();
        QCOMPARE(data.size(), 262144);
        for (qsizetype i = 0; i < data.size(); ++i)
            QCOMPARE(static_cast<unsigned char>(data[i]), static_cast<unsigned char>(i % 256));
        binary.close();
        QCOMPARE(controller->entries()->rowCount(), 3);
        QCOMPARE(controller->selectedCount(), 3);
        auto *task = window->findChild<QQuickWindow *>("taskWindow");
        QVERIFY(task);
        QTRY_VERIFY(task->isVisible());
        QTRY_VERIFY(visibleImagesReady(task->contentItem()));
        QCOMPARE(task->width(), 491);
        QVERIFY(!(task->flags() & Qt::FramelessWindowHint));
        QCOMPARE(task->transientParent(), window);
        const auto button = findVisual(task->contentItem(), "cancelButton");
        QVERIFY(button);
        QVERIFY(!button->isVisible());
        const auto capture = [&](QQuickWindow *w, const QString &name) {
            if (qEnvironmentVariableIsEmpty("XDZIP_M3_VISUAL_AUDIT"))
                return true;
            const auto directory = qEnvironmentVariable("XDZIP_M3_VISUAL_AUDIT");
            return QDir().mkpath(directory) && visual_audit::save(w, directory, name);
        };
        QVERIFY(capture(task, "task-success"));
        auto *tasksButton = findVisual(window->contentItem(), "browserTasks");
        QVERIFY(tasksButton);
        QCOMPARE(tasksButton->property("text").toString(),
                 QStringLiteral("任务 %1").arg(controller->tasks().size()));
        QCOMPARE(controller->activeTaskCount(), 0);
        window->requestActivate();
        QTRY_VERIFY(window->isActive());
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            tasksButton->mapToScene(QPointF(tasksButton->width() / 2, tasksButton->height() / 2))
                .toPoint());
        auto *tasksMenu = window->findChild<QObject *>("browserTaskMenu");
        QVERIFY(tasksMenu);
        QTRY_VERIFY(tasksMenu->property("visible").toBool());
        QCOMPARE(tasksMenu->property("width").toDouble(), 240.0);
        QTRY_VERIFY(tasksMenu->property("height").toDouble() <= 50.0);
        auto *taskEntry = window->findChild<QObject *>(QStringLiteral("taskMenuItem-") +
                                                       result.value("id").toString());
        QVERIFY(taskEntry);
        QCOMPARE(taskEntry->property("statusLabel").toString(), QStringLiteral("已完成"));
        QCOMPARE(taskEntry->property("labelSize").toDouble(), 13.0);
        QVERIFY(capture(window, "tasks-menu"));
        auto *taskItem = qobject_cast<QQuickItem *>(taskEntry);
        QVERIFY(taskItem);
        taskItem->forceActiveFocus(Qt::TabFocusReason);
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_VERIFY(!tasksMenu->property("visible").toBool());
        QCOMPARE(window->property("activeTaskId").toString(), result.value("id").toString());
        task->requestActivate();
        QTRY_VERIFY(task->isActive());
        QCOMPARE(result.value("details").toList().size(), 3);
        auto *detailsToggle = findVisual(task->contentItem(), "resultDetailsToggle");
        QVERIFY(detailsToggle);
        QTRY_COMPARE(task->height(), task->property("requestedHeight").toInt());
        const auto collapsedHeight = task->height();
        QVERIFY(collapsedHeight < 600);
        detailsToggle->forceActiveFocus(Qt::TabFocusReason);
        QTRY_VERIFY(detailsToggle->hasActiveFocus());
        QTest::keyClick(task, Qt::Key_Space);
        QTRY_VERIFY(detailsToggle->property("checked").toBool());
        const auto availableHeight =
            std::min(600, task->screen()->availableGeometry().height() - 80);
        QTRY_VERIFY(task->height() > collapsedHeight || collapsedHeight >= availableHeight);
        QVERIFY(task->height() <= 600);
        QTRY_VERIFY(visibleImagesReady(task->contentItem()));
        QVERIFY(capture(task, "task-result-details"));
        QTest::keyClick(task, Qt::Key_Space);
        QTRY_VERIFY(!detailsToggle->property("checked").toBool());
        QTRY_COMPARE(task->height(), collapsedHeight);
        controller->prepareExtraction(true);
        QTRY_VERIFY(extract->isVisible());
        QTRY_VERIFY(!controller->busy());
        targetField->setProperty("text", target.path());
        controller->checkTarget(target.path());
        QTRY_VERIFY(!controller->busy());
        QVERIFY(checkedTarget(controller, target.path()));
        QVERIFY(capture(extract, "extract-options"));
        controller->startExtraction(target.path(), true, false, 0, false, false);
        QTRY_VERIFY_WITH_TIMEOUT(controller->activeTaskCount() == 1 &&
                                     controller->tasks().first().toMap().value("status") ==
                                         "waiting",
                                 10000);
        result = controller->tasks().first().toMap();
        const auto taskId = result.value("id").toString();
        const auto prompt = result.value("promptId").toString();
        QVERIFY(
            !controller->decideTask(taskId, QString::number(prompt.toULongLong() + 1), 3, false));
        auto *conflict = window->findChild<QQuickWindow *>("conflictWindow");
        QVERIFY(conflict);
        QTRY_VERIFY(conflict->isVisible());
        QTRY_VERIFY(visibleImagesReady(conflict->contentItem()));
        QCOMPARE(conflict->width(), 491);
        QVERIFY(capture(conflict, "conflict"));
        QVERIFY(capture(task, "task-waiting"));
        conflict->close();
        task->close();
        QCOMPARE(controller->activeTaskCount(), 1);
        AppQuitGuard guard;
        guard.controller = controller;
        guard.window = window;
        QEvent quitEvent(QEvent::Quit);
        QVERIFY(guard.eventFilter(nullptr, &quitEvent));
        QVERIFY(window->isVisible());
        QCOMPARE(controller->activeTaskCount(), 1);
        auto *quit = window->findChild<QQuickWindow *>("quitWindow");
        QVERIFY(quit);
        QTRY_VERIFY(quit->isVisible());
        QVERIFY(capture(quit, "quit"));
        quit->close();
        controller->closeArchive();
        QTRY_VERIFY(!controller->ready() && !controller->busy());
        // A task retains its input authority after its browser session is closed.
        QVERIFY(controller->decideTask(taskId, prompt, 1, true));
        QVERIFY(!controller->decideTask(taskId, prompt, 3, false));
        QTRY_COMPARE_WITH_TIMEOUT(controller->activeTaskCount(), 0, 10000);
        QCOMPARE(controller->tasks().first().toMap().value("skippedFiles").toString(), "3");
        QCOMPARE(controller->tasks().first().toMap().value("committedFiles").toString(), "0");
        controller->requestClose();
    }
    void partialResultUsesActualOutcomes() {
        QTemporaryDir target;
        QVERIFY(target.isValid());
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/../extract/partial_crc.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        controller->prepareExtraction(true);
        auto *extract = window->findChild<QQuickWindow *>("extractWindow");
        QVERIFY(extract);
        auto *field = findVisual(extract->contentItem(), "extractTarget");
        QVERIFY(field);
        field->setProperty("text", target.path());
        QTRY_VERIFY(!controller->busy());
        controller->checkTarget(target.path());
        QTRY_VERIFY(!controller->busy() && checkedTarget(controller, target.path()));
        controller->startExtraction(target.path(), true, false, 0, false, false);
        QTRY_VERIFY(!controller->tasks().isEmpty() &&
                    controller->tasks().first().toMap().value("terminal").toBool());
        const auto taskData = controller->tasks().first().toMap();
        QCOMPARE(taskData.value("status").toString(), "failed");
        QCOMPARE(taskData.value("committedFiles").toString(), "2");
        QCOMPARE(taskData.value("errorCode").toString(), "integrity_error");
        QCOMPARE(taskData.value("details").toList().size(), 3);
        QVERIFY(QFileInfo::exists(target.filePath(QStringLiteral("冰川湖.jpg"))));
        QVERIFY(QFileInfo::exists(target.filePath(QStringLiteral("山间日出.jpg"))));
        QVERIFY(!QFileInfo::exists(target.filePath(QStringLiteral("行程手册.pdf"))));
        auto *task = window->findChild<QQuickWindow *>("taskWindow");
        QVERIFY(task);
        QTRY_VERIFY(task->isVisible() && visibleImagesReady(task->contentItem()));
        const auto output = qEnvironmentVariable("XDZIP_RESULT_AUDIT");
        const auto capture = [&](const QString &name) {
            return output.isEmpty() ||
                   (QDir().mkpath(output) && visual_audit::save(task, output, name));
        };
        QVERIFY(capture("result-partial"));
        window->setProperty("darkMode", true);
        QTRY_VERIFY(visibleImagesReady(task->contentItem()));
        QVERIFY(capture("result-dark"));
        window->setProperty("darkMode", false);
        auto *toggle = findVisual(task->contentItem(), "resultDetailsToggle");
        QVERIFY(toggle);
        toggle->forceActiveFocus(Qt::TabFocusReason);
        QTest::qWait(20);
        QTest::keyClick(task, Qt::Key_Space);
        QTRY_VERIFY(toggle->property("checked").toBool());
        QTRY_VERIFY(visibleImagesReady(task->contentItem()));
        QVERIFY(capture("result-details"));
        controller->requestClose();
        QTRY_VERIFY(controller->requestClose());
    }
    void m3IntegrityFailurePreservesDestinationAndCancelEndsWaiting() {
        QTemporaryDir target;
        QVERIFY(target.isValid());
        const auto leaf = target.filePath(QStringLiteral("说明.txt"));
        QFile old(leaf);
        QVERIFY(old.open(QIODevice::WriteOnly));
        old.write("old important file");
        old.close();
        BrowserController controller;
        const auto start = [&](const QString &fixture, int policy) {
            controller.openPath(QStringLiteral(XDZIP_FIXTURES "/../extract/") + fixture);
            QTRY_VERIFY(controller.ready() && !controller.busy());
            controller.prepareExtraction(true);
            controller.checkTarget(target.path());
            QTRY_VERIFY(!controller.busy());
            QVERIFY(checkedTarget(&controller, target.path()));
            controller.startExtraction(target.path(), true, false, policy, false, false);
        };
        start("crc_bad.zip", 3);
        QTRY_VERIFY(!controller.tasks().isEmpty() &&
                    controller.tasks().first().toMap().value("terminal").toBool());
        QCOMPARE(controller.tasks().first().toMap().value("errorCode").toString(),
                 "integrity_error");
        QVERIFY(old.open(QIODevice::ReadOnly));
        QCOMPARE(old.readAll(), QByteArray("old important file"));
        old.close();
        start("deflate.zip", 0);
        QTRY_VERIFY(controller.activeTaskCount() == 1 &&
                    controller.tasks().first().toMap().value("status") == "waiting");
        controller.cancelTask(controller.tasks().first().toMap().value("id").toString());
        QTRY_COMPARE(controller.activeTaskCount(), 0);
        QCOMPARE(controller.tasks().first().toMap().value("status").toString(), "cancelled");
        QVERIFY(old.open(QIODevice::ReadOnly));
        QCOMPARE(old.readAll(), QByteArray("old important file"));
        old.close();
        QCOMPARE(QDir(target.path())
                     .entryList(QDir::Files | QDir::Dirs | QDir::NoDotAndDotDot | QDir::Hidden)
                     .size(),
                 1);
        controller.requestClose();
    }
    void mailboxBackpressureShutdownAndIdle() {
        RuntimeConnection connection;
        QSignalSpy disconnected(&connection, &RuntimeConnection::disconnected);
        QSet<quint64> accepted;
        QSet<quint64> finished;
        bool duplicate = false;
        bool probeScheduled = false;
        bool guiYieldedBeforeDrainCompleted = false;
        connection.setConsumer([&](const xdzip::reader::Event &event) {
            if (!probeScheduled) {
                probeScheduled = true;
                // Posted during the first batch, before drain posts its continuation. Unlike a
                // millisecond timer, this tests yielding independently of machine speed.
                QMetaObject::invokeMethod(
                    &connection,
                    [&] { guiYieldedBeforeDrainCompleted = finished.size() < accepted.size(); },
                    Qt::QueuedConnection);
            }
            if (event.kind == xdzip::reader::EventKind::Finished &&
                accepted.contains(event.request_id)) {
                duplicate |= finished.contains(event.request_id);
                finished.insert(event.request_id);
            }
        });
        QElapsedTimer limit;
        limit.start();
        // Deliberately do not drain GUI notifications while filling both bounded transports.
        while (connection.statistics().peakQueued < 256 && limit.elapsed() < 5000) {
            xdzip::reader::Command command{};
            command.kind = xdzip::reader::CommandKind::Snapshot;
            const auto result = connection.submit(std::move(command));
            if (result.status == xdzip::reader::Admission::Accepted)
                accepted.insert(result.request_id);
            else
                QVERIFY(result.status == xdzip::reader::Admission::Busy);
            std::this_thread::yield();
        }
        QCOMPARE(connection.statistics().peakQueued, std::size_t(256));
        QVERIFY(connection.statistics().peakBytes <= 8 * 1024 * 1024);
        connection.shutdown();
        connection.shutdown();
        QTRY_COMPARE_WITH_TIMEOUT(disconnected.count(), 1, 5000);
        QVERIFY(disconnected.first().first().toBool());
        QCOMPARE(finished, accepted);
        QVERIFY(!duplicate);
        QVERIFY(guiYieldedBeforeDrainCompleted);
        QVERIFY(connection.stopped());
        const auto notifications = connection.statistics().notifications;
        QTest::qWait(30);
        QCOMPARE(connection.statistics().notifications, notifications);
    }
    void idleAndDestructionDuringWork() {
        {
            RuntimeConnection connection;
            QTest::qWait(30);
            QCOMPARE(connection.statistics().notifications, std::size_t(0));
            QSignalSpy stopped(&connection, &RuntimeConnection::disconnected);
            connection.shutdown();
            QTRY_COMPARE_WITH_TIMEOUT(stopped.count(), 1, 5000);
            QVERIFY(stopped.first().first().toBool());
        }
        for (int i = 0; i < 20; ++i) {
            auto controller = std::make_unique<BrowserController>();
            controller->openPath(QStringLiteral(XDZIP_FIXTURES "/pages.zip"));
            // Destructor requests shutdown but never waits on a worker or queued GUI notification.
        }
        QTRY_COMPARE_WITH_TIMEOUT(RuntimeReceiver::activeReceivers(), 0, 5000);
    }
    void consumerCanDestroyConnectionAndInvalidInputIsExplicit() {
        auto connection = std::make_unique<RuntimeConnection>();
        connection->setConsumer([&](const xdzip::reader::Event &) { connection.reset(); });
        bool submitted = false;
        QTRY_VERIFY_WITH_TIMEOUT(([&] {
                                     if (submitted)
                                         return true;
                                     xdzip::reader::Command command{};
                                     command.kind = xdzip::reader::CommandKind::Snapshot;
                                     submitted = connection->submit(std::move(command)).status ==
                                                 xdzip::reader::Admission::Accepted;
                                     return submitted;
                                 })(),
                                 5000);
        QTRY_VERIFY_WITH_TIMEOUT(!connection, 5000);
        BrowserController controller;
        controller.openPath(QString(4097, 'x'));
        QVERIFY(!controller.busy());
        QVERIFY(!controller.ready());
        QVERIFY(controller.message().contains(QString::fromUtf8("支持范围")));
        controller.requestClose();
        QTRY_VERIFY_WITH_TIMEOUT(!controller.runtimeAvailable(), 5000);
    }
    void realPagesAndFailure() {
        BrowserController controller;
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/pages.zip"));
        QVERIFY(controller.busy());
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QVERIFY(controller.ready());
        QCOMPARE(controller.total(), 1);
        QCOMPARE(controller.archiveTotal(), 270);
        QCOMPARE(controller.entries()->data(controller.entries()->index(0, 0)).toString(),
                 "folder");
        controller.selectRow(0);
        QVERIFY(!controller.canPreflight());
        controller.enterDirectory(0);
        QTRY_VERIFY(!controller.busy());
        QCOMPARE(controller.total(), 270);
        QCOMPARE(controller.entries()->rowCount(), 270);
        const auto session = controller.sessionId();
        controller.selectRow(0);
        QCOMPARE(controller.selectedCount(), 1);
        QCOMPARE(controller.entries()->rowCount(), 270);
        QCOMPARE(controller.entries()
                     ->data(controller.entries()->index(256, 0), Qt::UserRole)
                     .toString(),
                 "257");
        QCOMPARE(controller.sessionId(), session);
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/truncated.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QVERIFY(!controller.ready());
        QCOMPARE(controller.entries()->rowCount(), 0);
        QVERIFY(!controller.message().isEmpty());
    }
    void selectionSearchAndDirectoryNavigation() {
        BrowserController controller;
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        controller.selectRow(2);
        controller.selectRow(3);
        QCOMPARE(controller.selectedCount(), 2);
        QCOMPARE(controller.detail().value("id").toString(), "5");
        controller.selectRow(2);
        QCOMPARE(controller.selectedCount(), 1);
        controller.enterDirectory(0);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.total(), 1);
        QCOMPARE(controller.selectedCount(), 0);
        QVERIFY(controller.detail().isEmpty());
        QCOMPARE(controller.location(), QString::fromUtf8("旅行/"));
        controller.showRoot();
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        controller.search("NOTES");
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.total(), 2);
        QCOMPARE(
            controller.entries()->data(controller.entries()->index(0, 0), Qt::UserRole).toString(),
            "3");
        controller.search("missing");
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QVERIFY(controller.ready());
        QCOMPARE(controller.total(), 0);
        controller.search("");
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.total(), 4);
    }
    void typeHeaderSortGroupsDirectoriesBeforeExtensionlessFiles() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/type-order.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        QVERIFY(QTest::qWaitForWindowExposed(window));
        auto *header = findVisual(window->contentItem(), "sort-column-3");
        QVERIFY(header);
        const auto click = [window, header] {
            QTest::mouseClick(
                window, Qt::LeftButton, Qt::NoModifier,
                header->mapToScene(QPointF(header->width() / 2, header->height() / 2)).toPoint());
        };
        const auto names = [controller] {
            QStringList result;
            for (int row = 0; row < controller->entries()->rowCount(); ++row)
                result.append(
                    controller->entries()->data(controller->entries()->index(row, 0)).toString());
            return result;
        };
        click();
        QTRY_VERIFY(controller->sortOrder() == 9 && !controller->busy());
        QCOMPARE(names(),
                 QStringList({"item-alpha", "item-folder.ZIP", "item-implicit", "item-README",
                              "item-LICENSE", "item-duplicate", "item-duplicate", "item-image.JPG",
                              "item-image.jpeg", "item-last.ZIP"}));
        QCOMPARE(controller->entries()->data(controller->entries()->index(2, 3)).toString(),
                 QString::fromUtf8("文件夹"));
        QCOMPARE(controller->entries()->data(controller->entries()->index(3, 3)).toString(),
                 QString::fromUtf8("文件"));
        click();
        QTRY_VERIFY(controller->sortOrder() == 10 && !controller->busy());
        QCOMPARE(names(),
                 QStringList({"item-alpha", "item-folder.ZIP", "item-implicit", "item-last.ZIP",
                              "item-image.JPG", "item-image.jpeg", "item-README", "item-LICENSE",
                              "item-duplicate", "item-duplicate"}));
        controller->search("item");
        QTRY_VERIFY(!controller->busy());
        QCOMPARE(names(),
                 QStringList({"item-alpha/", "item-folder.ZIP/", "item-last.ZIP", "item-image.JPG",
                              "item-image.jpeg", "item-README", "item-LICENSE", "item-duplicate",
                              "item-duplicate", "item-implicit/item-child"}));
        click();
        QTRY_VERIFY(controller->sortOrder() == 9 && !controller->busy());
        QCOMPARE(names(),
                 QStringList({"item-alpha/", "item-folder.ZIP/", "item-README", "item-LICENSE",
                              "item-duplicate", "item-duplicate", "item-implicit/item-child",
                              "item-image.JPG", "item-image.jpeg", "item-last.ZIP"}));
        controller->requestClose();
        QTRY_VERIFY(!controller->runtimeAvailable());
    }
    void sourceChangeDuringPreflightInvalidatesSnapshot() {
        QTemporaryDir owned;
        const auto path = owned.filePath("preflight.zip");
        QVERIFY(QFile::copy(QStringLiteral(XDZIP_FIXTURES "/normal.zip"), path));
        BrowserController controller;
        QSignalSpy report(&controller, &BrowserController::preflightReady);
        controller.openPath(path);
        QTRY_VERIFY(controller.ready() && !controller.busy());
        controller.selectOnlyRow(1);
        QVERIFY(controller.canPreflight());
        QFile changed(path);
        QVERIFY(changed.open(QIODevice::Append));
        QCOMPARE(changed.write("changed"), 7);
        changed.close();
        controller.inspectSelection();
        QTRY_COMPARE(report.count(), 1);
        QVERIFY(!controller.busy());
        QVERIFY(!controller.ready());
        QCOMPARE(controller.total(), 0);
        QCOMPARE(controller.archiveTotal(), 0);
        QCOMPARE(controller.entries()->rowCount(), 0);
        QCOMPARE(controller.directories()->rowCount(), 0);
        QCOMPARE(controller.selectedCount(), 0);
        QVERIFY(controller.detail().isEmpty());
        QVERIFY(controller.preflightReport().contains("error"));
        controller.requestClose();
        QTRY_VERIFY(!controller.runtimeAvailable());
    }
    void m2LargeArchiveLatestQueryAndSourceChange() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        QElapsedTimer elapsed;
        elapsed.start();
        controller->openPath(QStringLiteral(XDZIP_M2_FIXTURE));
        QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy(), 20000);
        const auto openMilliseconds = elapsed.elapsed();
        const auto capturePath = qEnvironmentVariable("XDZIP_VISUAL_AUDIT");
        if (!capturePath.isEmpty()) {
            QVERIFY(QDir().mkpath(capturePath));
            QVERIFY(visual_audit::save(window, capturePath, "m2-large-flat"));
        }
        QCOMPARE(controller->total(), 100000);
        QCOMPARE(controller->archiveTotal(), 100000);
        QCOMPARE(controller->entries()->rowCount(), 100000);
        QCOMPARE(controller->entries()->data(controller->entries()->index(99999, 0)).toString(),
                 "item-099999.txt");
        QVERIFY(QTest::qWaitForWindowExposed(window));
        // Model publication and TableView incubation are distinct events. Wait
        // for a real visible cell, not a fixed 50ms delay on a shared runner.
        QTRY_VERIFY_WITH_TIMEOUT(findVisual(window->contentItem(), "entry-0-0"), 5000);
        int delegates = 0;
        const auto countDelegates = [&delegates](auto &&self, QQuickItem *item) -> void {
            if (item->objectName().startsWith("entry-") && item->objectName().count('-') == 2)
                ++delegates;
            for (auto *child : item->childItems())
                self(self, child);
        };
        countDelegates(countDelegates, window->contentItem());
        QVERIFY(delegates > 0 && delegates < 512);
        controller->selectAll(true);
        QCOMPARE(controller->selectedCount(), 100000);
        QVERIFY(!controller->canPreflight());
        QCOMPARE(controller->selectedBytes(), "0");
        elapsed.restart();
        controller->sortByColumn(0);
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 20000);
        const auto sortMilliseconds = elapsed.elapsed();
        QCOMPARE(controller->total(), 100000);
        QCOMPARE(controller->selectedCount(), 0);
        controller->selectAll(true);
        controller->search("099998");
        controller->search("099997");
        controller->search("099999");
        controller->sortByColumn(0);
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 10000);
        QCOMPARE(controller->total(), 1);
        QCOMPARE(controller->selectedCount(), 0);
        QCOMPARE(controller->sortOrder(), 2);
        QCOMPARE(controller->entries()->data(controller->entries()->index(0, 0)).toString(),
                 "item-099999.txt");
        if (!capturePath.isEmpty())
            QVERIFY(visual_audit::save(window, capturePath, "m2-latest-query"));
        controller->openPath(QStringLiteral(XDZIP_M2_HIERARCHICAL));
        QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy(), 20000);
        QCOMPARE(controller->total(), 100);
        QCOMPARE(controller->archiveTotal(), 100000);
        controller->navigateDirectory("d1");
        controller->navigateDirectory("d2");
        controller->showRoot();
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 10000);
        QCOMPARE(controller->total(), 100);
        QCOMPARE(controller->location(), "");
        controller->navigateDirectory("d100");
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 10000);
        QCOMPARE(controller->total(), 1000);
        if (!capturePath.isEmpty())
            QVERIFY(visual_audit::save(window, capturePath, "m2-directory"));
        QCOMPARE(controller->entries()->data(controller->entries()->index(0, 0)).toString(),
                 "item-099000.txt");
        controller->search("099999");
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 10000);
        QCOMPARE(controller->entries()->data(controller->entries()->index(0, 0)).toString(),
                 "folder-099/item-099999.txt");
        qInfo().noquote()
            << QString("M2_QT_METRICS open_ms=%1 rows=100000 delegates=%2 full_sort_ms=%3")
                   .arg(openMilliseconds)
                   .arg(delegates)
                   .arg(sortMilliseconds);
        QTemporaryDir owned;
        QVERIFY(owned.isValid());
        const auto path = owned.filePath("source.zip");
        QVERIFY(QFile::copy(QStringLiteral(XDZIP_FIXTURES "/normal.zip"), path));
        controller->openPath(path);
        QTRY_VERIFY(controller->ready() && !controller->busy());
        QFile changed(path);
        QVERIFY(changed.open(QIODevice::Append));
        QCOMPARE(changed.write("x"), 1);
        changed.close();
        controller->search("notes");
        QTRY_VERIFY(!controller->busy());
        QVERIFY(!controller->ready());
        QCOMPARE(controller->entries()->rowCount(), 0);
        QVERIFY(controller->message().contains(QString::fromUtf8("源文件")));
        controller->requestClose();
        QTRY_VERIFY(!controller->runtimeAvailable());
    }
    void completeNavigationAndGlobalSort() {
        BrowserController controller;
        QAbstractItemModelTester modelCheck(controller.directories(),
                                            QAbstractItemModelTester::FailureReportingMode::QtTest);
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/pages.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        auto *tree = controller.directories();
        QCOMPARE(tree->rowCount(), 1);
        const auto node = tree->index(0, 0);
        QCOMPARE(tree->data(node).toString(), "folder");
        QCOMPARE(tree->data(node, Qt::UserRole + 1).toInt(), 270);
        controller.navigateDirectory(tree->data(node, Qt::UserRole).toString());
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.total(), 270);
        QCOMPARE(controller.location(), "folder/");
        controller.sortByColumn(0);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        controller.selectRow(0);
        controller.sortByColumn(0);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.sortOrder(), 2);
        QCOMPARE(controller.selectedCount(), 0);
        QCOMPARE(
            controller.entries()->data(controller.entries()->index(0, 0), Qt::UserRole).toString(),
            "270");
        QCOMPARE(controller.entries()->rowCount(), 270);
        QCOMPARE(controller.entries()
                     ->data(controller.entries()->index(128, 0), Qt::UserRole)
                     .toString(),
                 "142");
        QCOMPARE(tree->rowCount(), 1);
        QCOMPARE(tree->data(tree->index(0, 0), Qt::UserRole + 1).toInt(), 270);
        controller.sortByColumn(-1);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.offset(), 0);
        QCOMPARE(controller.sortOrder(), 0);
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/navigation-depth.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QVERIFY(controller.ready());
        QVERIFY(controller.navigationLimited());
        QCOMPARE(tree->rowCount(), 0);
        QCOMPARE(controller.total(), 0);
        controller.search("file");
        QTRY_VERIFY(!controller.busy());
        QCOMPARE(controller.total(), 1);
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/empty.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QVERIFY(controller.ready());
        QVERIFY(!controller.navigationLimited());
        QCOMPARE(tree->rowCount(), 0);
    }
    void contextualSelectionRangeAndClipboard() {
        BrowserController controller;
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.archiveTotal(), 5);
        controller.selectOnlyRow(2);
        controller.selectRow(3);
        controller.selectForContext(2);
        QCOMPARE(controller.selectedCount(), 2);
        QCOMPARE(controller.detail().value("id").toString(), "4");
        controller.selectForContext(0);
        QCOMPARE(controller.selectedCount(), 1);
        controller.copySelection(false);
        QCOMPARE(QGuiApplication::clipboard()->text(), QString::fromUtf8("旅行"));
        controller.copySelection(true);
        QCOMPARE(QGuiApplication::clipboard()->text(), QString::fromUtf8("旅行/"));
        controller.selectOnlyRow(1);
        controller.selectRangeTo(3);
        QCOMPARE(controller.selectedCount(), 3);
        controller.copySelection(false);
        QCOMPARE(QGuiApplication::clipboard()->text().split('\n').size(), 3);
        controller.selectAll(true);
        QCOMPARE(controller.selectedCount(), 4);
        QVERIFY(!controller.detail().isEmpty());
        QCOMPARE(controller.detail().value("id").toString(), "5");
        QVERIFY(!controller.detail().value("size").toString().isEmpty());
        QVERIFY(!controller.detail().value("method").toString().isEmpty());
        controller.selectAll(false);
        QCOMPARE(controller.selectedCount(), 0);
        QVERIFY(controller.detail().isEmpty());
        controller.search("NOTES");
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QCOMPARE(controller.archiveTotal(), 5);
        controller.selectRangeTo(1);
        QCOMPARE(controller.selectedCount(), 1);
        controller.selectRangeTo(0);
        QCOMPARE(controller.selectedCount(), 2);
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/pages.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        controller.enterDirectory(0);
        QTRY_VERIFY(!controller.busy());
        controller.selectAll(true);
        QCOMPARE(controller.selectedCount(), 270);
        controller.selectAll(false);
        QCOMPARE(controller.selectedCount(), 0);
        controller.selectRangeTo(269);
        QVERIFY(controller.selectedCount() > 0);
    }
    void treeExpansionAndColumnInteraction() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/navigation.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QVERIFY(QTest::qWaitForWindowExposed(window));
        auto *treeView = findVisual(window->contentItem(), "directoryTree");
        QVERIFY(treeView);
        QVERIFY(treeView->property("selectionModel").value<QObject *>());
        QTRY_VERIFY(treeView->property("contentWidth").toDouble() <= treeView->width() + 1);
        auto *model = controller->directories();
        QCOMPARE(model->rowCount(), 3);
        const auto parent = model->index(0, 0);
        QCOMPARE(model->rowCount(parent), 1);
        QCOMPARE(model->data(parent, Qt::UserRole + 1).toInt(), 3);
        QQuickItem *node = nullptr;
        QTRY_VERIFY((node = findVisual(window->contentItem(), "directory-d1")));
        auto *expand = findVisual(window->contentItem(), "directoryDisclosure-d1");
        QVERIFY(expand);
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            expand->mapToScene(QPointF(expand->width() / 2, expand->height() / 2)).toPoint());
        QQuickItem *child = nullptr;
        QTRY_VERIFY((child = findVisual(window->contentItem(), "directory-d2")));
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier,
                          child->mapToScene(QPointF(80, child->height() / 2)).toPoint());
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QCOMPARE(controller->directoryId(), "d2");
        QCOMPARE(controller->total(), 2);
        QCOMPARE(controller->location(), "photos/2026/");
        // Arrow keys move the tree current item; Enter activates that item's scope.
        treeView->forceActiveFocus();
        QTest::keyClick(window, Qt::Key_Down);
        QCOMPARE(treeView->property("currentRow").toInt(), 2);
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QCOMPARE(controller->location(), "documents/");
        QCOMPARE(controller->total(), 1);
        QTRY_VERIFY(!treeView->hasActiveFocus());
        treeView->forceActiveFocus();
        QTest::keyClick(window, Qt::Key_Up);
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QCOMPARE(controller->location(), "photos/2026/");
        auto *header = findVisual(window->contentItem(), "sort-column-1");
        QVERIFY(header);
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            header->mapToScene(QPointF(header->width() / 2, header->height() / 2)).toPoint());
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QCOMPARE(controller->sortOrder(), 3);
        header->forceActiveFocus();
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QCOMPARE(controller->sortOrder(), 4);
        QCOMPARE(controller->entries()
                     ->data(controller->entries()->index(0, 0), Qt::UserRole)
                     .toString(),
                 "2");
        // Any relayout can reuse even the parent delegate for another model identity.
        auto clickParent = [&] {
            auto *button = findVisual(window->contentItem(), "directoryDisclosure-d1");
            if (!button)
                return false;
            QTest::mouseClick(
                window, Qt::LeftButton, Qt::NoModifier,
                button->mapToScene(QPointF(button->width() / 2, button->height() / 2)).toPoint());
            return true;
        };
        QVERIFY(clickParent());
        QTRY_COMPARE(treeView->property("rows").toInt(), 3);
        QCOMPARE(controller->directoryId(), "d2");
        QCOMPARE(controller->location(), "photos/2026/");
        auto branchMarkerVisible = [&] {
            auto *marker = findVisual(window->contentItem(), "directoryBranchMarker-d1");
            return marker && marker->isVisible();
        };
        QTRY_VERIFY(branchMarkerVisible());
        QVERIFY(clickParent());
        QTRY_COMPARE(treeView->property("rows").toInt(), 4);
        QTRY_VERIFY(!branchMarkerVisible());
        // Collapse/reopen reuses delegates. Assert the rendered hierarchy, not merely
        // the expansion flag, before capturing or allowing a later frame to hide drift.
        for (int cycle = 0; cycle < 8; ++cycle) {
            QTRY_VERIFY([&] {
                auto *parentIcon = findVisual(window->contentItem(), "directoryFolder-d1");
                auto *childIcon = findVisual(window->contentItem(), "directoryFolder-d2");
                return parentIcon && childIcon && childIcon->isVisible() &&
                       qAbs(childIcon->mapToScene(QPointF()).x() -
                            parentIcon->mapToScene(QPointF()).x() - 12.8) < 0.5;
            }());
            if (cycle < 7) {
                QVERIFY(clickParent());
                QTRY_COMPARE(treeView->property("rows").toInt(), 3);
                QTRY_VERIFY([&] {
                    auto *collapsed = findVisual(window->contentItem(), "directory-d2");
                    return !collapsed || !collapsed->isVisible();
                }());
                QVERIFY(clickParent());
                QTRY_COMPARE(treeView->property("rows").toInt(), 4);
            }
        }
        if (!qEnvironmentVariableIsEmpty("XDZIP_VISUAL_AUDIT")) {
            const auto directory = qEnvironmentVariable("XDZIP_VISUAL_AUDIT");
            QVERIFY(QDir().mkpath(directory));
            QVERIFY(visual_audit::save(window, directory, "navigation-tree"));
        }
    }
    void treeWidthTracksOnlyExpandedNodes() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/tree-layout.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        auto *tree = findVisual(window->contentItem(), "directoryTree");
        QVERIFY(tree);
        QTRY_COMPARE(tree->property("rows").toInt(), 2);
        QTRY_VERIFY(tree->property("contentWidth").toDouble() <= tree->width() + 1);
        QVERIFY(
            QMetaObject::invokeMethod(tree, "expandRecursively", Q_ARG(int, 0), Q_ARG(int, -1)));
        QTRY_VERIFY(tree->property("rows").toInt() > 12);
        QTRY_VERIFY(tree->property("contentWidth").toDouble() > tree->width());
        QVERIFY(QMetaObject::invokeMethod(tree, "collapseRecursively", Q_ARG(int, 0)));
        QTRY_COMPARE(tree->property("rows").toInt(), 2);
        QTRY_VERIFY(tree->property("contentWidth").toDouble() <= tree->width() + 1);
    }
    void preflightPreservesSelectionAndReportsDuplicates() {
        BrowserController controller;
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        controller.selectRow(1);
        controller.selectRow(2);
        QSignalSpy completed(&controller, &BrowserController::preflightReady);
        controller.inspectSelection();
        QTRY_COMPARE_WITH_TIMEOUT(completed.count(), 1, 5000);
        QCOMPARE(controller.selectedCount(), 2);
        QCOMPARE(controller.entries()->rowCount(), 4);
        QCOMPARE(controller.preflightReport().value("entries").toString(), "2");
        QCOMPARE(controller.preflightReport().value("findingCount").toString(), "1");
        controller.selectRow(1);
        QVERIFY(controller.preflightReport().isEmpty());
        controller.selectRow(2);
        controller.selectRow(0);
        controller.inspectSelection();
        QTRY_COMPARE_WITH_TIMEOUT(completed.count(), 2, 5000);
        QCOMPARE(controller.preflightReport().value("entries").toString(), "2");
        QCOMPARE(controller.preflightReport().value("bytes").toString(), "12");
        QVERIFY(controller.preflightReport().value("unicodeReview").toBool());
    }
    void realContextMenusAndGrid() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QVERIFY(QTest::qWaitForWindowExposed(window));
        controller->selectOnlyRow(2);
        controller->selectRow(3);
        QQuickItem *cell = nullptr;
        QTRY_VERIFY((cell = findVisual(window->contentItem(), "entry-2-0")));
        auto *menu = window->findChild<QObject *>("browserEntryMenu");
        QVERIFY(menu);
        QTest::mouseClick(window, Qt::RightButton, Qt::NoModifier,
                          cell->mapToScene(QPointF(80, cell->height() / 2)).toPoint());
        QTRY_VERIFY(menu->property("visible").toBool());
        QCOMPARE(controller->selectedCount(), 2);
        auto *enterDirectory = window->findChild<QQuickItem *>("contextEnterDirectory");
        auto *properties = window->findChild<QQuickItem *>("contextProperties");
        QVERIFY(enterDirectory);
        QVERIFY(properties);
        QVERIFY(!enterDirectory->isVisible());
        QTRY_COMPARE(enterDirectory->height(), 0);
        QTRY_COMPARE(properties->y(), 0);
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!menu->property("visible").toBool());
        cell->forceActiveFocus();
        QTest::keyClick(window, Qt::Key_F10, Qt::ShiftModifier);
        QTRY_VERIFY(menu->property("visible").toBool());
        controller->search("notes");
        QTRY_VERIFY(!menu->property("visible").toBool());
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QCOMPARE(controller->selectedCount(), 0);
        auto *viewButton = findVisual(window->contentItem(), "browserViewMenuButton");
        QVERIFY(viewButton);
        QTest::mouseClick(window, Qt::RightButton, Qt::NoModifier,
                          viewButton->mapToScene(QPointF(12, 12)).toPoint());
        auto *viewMenu = window->findChild<QObject *>("browserViewMenu");
        QVERIFY(viewMenu);
        QTRY_VERIFY(viewMenu->property("visible").toBool());
        QTest::keyClick(window, Qt::Key_Down);
        QTest::keyClick(window, Qt::Key_Down);
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_COMPARE(window->property("viewMode").toString(), "grid");
        QQuickItem *card = nullptr;
        QTRY_VERIFY((card = findVisual(window->contentItem(), "grid-entry-0")));
        QTest::mouseClick(window, Qt::RightButton, Qt::NoModifier,
                          card->mapToScene(QPointF(30, 30)).toPoint());
        QTRY_VERIFY(menu->property("visible").toBool());
        QCOMPARE(controller->selectedCount(), 1);
        if (!qEnvironmentVariableIsEmpty("XDZIP_VISUAL_AUDIT")) {
            QVERIFY(QDir().mkpath(qEnvironmentVariable("XDZIP_VISUAL_AUDIT")));
            QVERIFY(visual_audit::save(window, qEnvironmentVariable("XDZIP_VISUAL_AUDIT"),
                                       "command-context-grid"));
        }
        QTest::keyClick(window, Qt::Key_Escape);
    }
    void unicodeSourcePath() {
        QTemporaryDir directory;
        QVERIFY(directory.isValid());
        const auto path = directory.filePath(QString::fromUtf8("旅行 archive.zip"));
        QVERIFY(QFile::copy(QStringLiteral(XDZIP_FIXTURES "/normal.zip"), path));
        BrowserController controller;
        controller.openPath(path);
        QTRY_VERIFY_WITH_TIMEOUT(!controller.busy(), 5000);
        QVERIFY(controller.ready());
        QCOMPARE(controller.total(), 4);
    }
    void closeDuringReadRemainsAsynchronous() {
        BrowserController controller;
        QSignalSpy closed(&controller, &BrowserController::readyToClose);
        controller.openPath(QStringLiteral(XDZIP_FIXTURES "/pages.zip"));
        QVERIFY(!controller.requestClose());
        QTRY_COMPARE_WITH_TIMEOUT(closed.count(), 1, 5000);
        QVERIFY(!controller.busy());
        QVERIFY(controller.requestClose());
    }
    void nativeChooserOwnershipAndCancelRestoreCaller() {
        QQuickWindow mainWindow;
        QQuickWindow caller;
        caller.setTransientParent(&mainWindow);
        mainWindow.show();
        caller.show();
        QVERIFY(QTest::qWaitForWindowExposed(&caller));
        for (const auto mode :
             {QFileDialog::ExistingFile, QFileDialog::ExistingFiles, QFileDialog::Directory}) {
            QFileDialog dialog;
            // Inspect the same ownership policy without depending on OS automation permissions.
            dialog.setOption(QFileDialog::DontUseNativeDialog);
            dialog.setFileMode(mode);
            bool owned = false;
            QTimer::singleShot(0, &dialog, [&] {
                owned = dialog.windowHandle()->transientParent() == &caller;
                dialog.reject();
            });
            QVERIFY(!execOwnedFileDialog(dialog, &caller));
            QVERIFY(owned);
            if (QGuiApplication::platformName() == "cocoa")
                QTRY_COMPARE(QGuiApplication::focusWindow(), &caller);
        }
    }
    void rootListingAndReviewedInteractions() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/tree-layout.zip"));
        QTRY_VERIFY(!controller->busy());
        QCOMPARE(controller->total(), 2);
        QCOMPARE(controller->archiveTotal(), 2);
        QCOMPARE(controller->entries()->data(controller->entries()->index(0, 0)).toString(),
                 "GitHub Desktop.app");
        controller->selectOnlyRow(0);
        QVERIFY(!controller->canPreflight());
        controller->copySelection(true);
        QCOMPARE(QGuiApplication::clipboard()->text(), "GitHub Desktop.app/");
        controller->selectAll(false);
        QVERIFY(QTest::qWaitForWindowExposed(window));
        const auto item = [window](const char *name) {
            return findVisual(window->contentItem(), QString::fromLatin1(name));
        };
        const auto click = [window](QQuickItem *target) {
            QTest::mouseClick(
                window, Qt::LeftButton, Qt::NoModifier,
                target->mapToScene(QPointF(target->width() / 2, target->height() / 2)).toPoint());
        };
        auto *caption = item("archiveCaption");
        auto *title = item("filePaneTitle");
        QCOMPARE(caption->property("text").toString(), QString::fromUtf8("压缩包"));
        QTRY_VERIFY(qAbs(caption->mapToScene(QPointF(0, caption->height() / 2)).y() -
                         title->mapToScene(QPointF(0, title->height() / 2)).y()) < .5);
        auto *cell = item("entry-0-0");
        QTest::mouseMove(window, cell->mapToScene(QPointF(100, cell->height() / 2)).toPoint());
        for (int column = 0; column < 5; ++column) {
            auto *band = findVisual(window->contentItem(), QString("entryBand-0-%1").arg(column));
            QVERIFY(band);
            auto *separator =
                findVisual(window->contentItem(), QString("entrySeparator-0-%1").arg(column));
            QVERIFY(separator);
            // The upper band's gap to its line equals the next band's gap after that line.
            QCOMPARE(separator->y() - (band->y() + band->height()), band->y());
            QTRY_COMPARE(band->property("color").value<QColor>(), QColor("#EDF2FF"));
        }
        const auto captures = qEnvironmentVariable("XDZIP_VISUAL_AUDIT");
        if (!captures.isEmpty())
            QVERIFY(QDir().mkpath(captures));
        if (!captures.isEmpty())
            QVERIFY(visual_audit::save(window, captures, "root-row-hover"));
        auto *tree = item("directoryTree");
        QVERIFY(tree);
        QVERIFY(QMetaObject::invokeMethod(tree, "expandRecursively", Q_ARG(int, 0), Q_ARG(int, 2)));
        QQuickItem *guide = nullptr, *disclosure = nullptr;
        QTRY_VERIFY((guide = item("directoryParentGuide-d2")));
        QTRY_VERIFY((disclosure = item("directoryDisclosure-d1")));
        QVERIFY(qAbs(guide->mapToScene(QPointF(guide->width() / 2, 0)).x() -
                     disclosure->mapToScene(QPointF(disclosure->width() / 2, 0)).x()) < .01);
        QQmlComponent scaleSettings(&engine);
        scaleSettings.setData(
            "import QtQuick\nimport \"qrc:/xdzip/ui\" as Ui\n"
            "QtObject { property real scale: 1; onScaleChanged: Ui.Theme.fontScale = scale }",
            QUrl());
        std::unique_ptr<QObject> scaling(scaleSettings.create());
        QVERIFY(scaling);
        scaling->setProperty("scale", 2);
        QTRY_VERIFY(qAbs(caption->mapToScene(QPointF(0, caption->height() / 2)).y() -
                         title->mapToScene(QPointF(0, title->height() / 2)).y()) < .5);
        QTRY_VERIFY(qAbs(guide->mapToScene(QPointF(guide->width() / 2, 0)).x() -
                         disclosure->mapToScene(QPointF(disclosure->width() / 2, 0)).x()) < .01);
        scaling->setProperty("scale", 1);
        QTest::qWait(30);
        const auto closePosition = item("inspectorClose")->mapToScene(QPointF());
        click(item("inspectorClose"));
        QQuickItem *toggle = nullptr;
        QTRY_VERIFY((toggle = item("inspectorToggle")) && toggle->isVisible());
        // Visibility updates before the RowLayout's next polish pass.
        // Keep the exact size contract, but inspect the settled layout.
        QTRY_COMPARE(toggle->width(), 28.0);
        QTRY_COMPARE(toggle->height(), 28.0);
        QTRY_VERIFY2(qAbs(toggle->mapToScene(QPointF()).x() - closePosition.x()) < .01,
                     qPrintable(QString("show=%1 close=%2 window=%3")
                                    .arg(toggle->mapToScene(QPointF()).x())
                                    .arg(closePosition.x())
                                    .arg(window->width())));
        QCOMPARE(toggle->mapToScene(QPointF()).y(), closePosition.y());
        click(item("browserGridButton"));
        QQuickItem *card = nullptr, *check = nullptr;
        QTRY_VERIFY((card = item("grid-entry-0")));
        QTRY_VERIFY((check = item("gridSelection-0")));
        auto *grid = item("archiveGrid");
        QVERIFY(qAbs(grid->x() - 16) < .01);
        QTest::mouseMove(window, item("archiveOpen")->mapToScene(QPointF(10, 10)).toPoint());
        QTRY_VERIFY(!check->isVisible());
        QTest::mouseMove(window, card->mapToScene(QPointF(40, 40)).toPoint());
        QTRY_VERIFY(check->isVisible());
        click(check);
        QTRY_COMPARE(controller->selectedCount(), 1);
        QTest::mouseMove(window, item("archiveOpen")->mapToScene(QPointF(10, 10)).toPoint());
        QTRY_VERIFY(check->isVisible());
        if (!captures.isEmpty())
            QVERIFY(visual_audit::save(window, captures, "root-grid-selected"));
        window->setProperty("darkMode", true);
        QTRY_COMPARE(item("actionSplitDivider")->property("color").value<QColor>(),
                     QColor("#363D4B"));
        window->setProperty("darkMode", false);
        click(item("browserViewMenuButton"));
        auto *more = item("archiveMore");
        QTest::mouseMove(window, more->mapToScene(QPointF(10, 10)).toPoint());
        const QImage before = window->grabWindow();
        click(more);
        auto *menu = window->findChild<QObject *>("archiveMoreMenu");
        QVERIFY(menu);
        QTRY_VERIFY(menu->property("visible").toBool());
        auto *content = menu->property("contentItem").value<QQuickItem *>();
        auto *background = menu->property("background").value<QQuickItem *>();
        QVERIFY(background);
        auto *shadow = findVisual(background, "menuShadowLayer-0");
        QVERIFY(shadow && shadow->x() < 0 && shadow->width() > background->width());
        const auto top =
            content->mapToScene(QPointF()).y() - menu->property("topPadding").toDouble();
        QVERIFY(qAbs(top - more->mapToScene(QPointF(0, more->height())).y() - 4) < .1);
        const auto center = background->mapToScene(QPointF(-4, background->height() / 2));
        // Canvas paints its blurred silhouette on the next frame.
        QTest::qWait(50);
        const QImage after = window->grabWindow();
        const auto pixel = QPoint(qRound(center.x() * after.width() / window->width()),
                                  qRound(center.y() * after.height() / window->height()));
        QVERIFY(after.valid(pixel));
        QVERIFY(after.pixelColor(pixel) != before.pixelColor(pixel));
        if (!captures.isEmpty())
            QVERIFY(visual_audit::save(window, captures, "root-menu-shadow"));
        QTest::keyClick(window, Qt::Key_Escape);
        controller->enterDirectory(0);
        QTRY_VERIFY(!controller->busy());
        QCOMPARE(controller->entries()->data(controller->entries()->index(0, 0)).toString(),
                 "Contents");
        controller->showRoot();
        QTRY_VERIFY(!controller->busy());
        controller->search("file.txt");
        QTRY_VERIFY(!controller->busy());
        QCOMPARE(controller->total(), 1);
        QVERIFY(controller->entries()
                    ->data(controller->entries()->index(0, 0))
                    .toString()
                    .contains("GitHub Desktop.app/Contents/Resources/"));
        controller->requestClose();
        QTRY_VERIFY(!controller->runtimeAvailable());
    }
    void reviewedMainControlsAndMenus() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/visual-main.zip"));
        QTRY_VERIFY(controller->ready() && !controller->busy());
        QVERIFY(QTest::qWaitForWindowExposed(window));
        QSignalSpy initialFrame(window, &QQuickWindow::afterAnimating);
        window->requestUpdate();
        QTRY_VERIFY(initialFrame.count() > 0);
        const auto item = [window](const char *name) {
            return findVisual(window->contentItem(), QString::fromLatin1(name));
        };
        const auto click = [window](QQuickItem *target) {
            const auto point =
                target->mapToScene(QPointF(target->width() / 2, target->height() / 2)).toPoint();
            // Move before pressing, as a real pointer does. This lets hovered
            // controls dismiss their tooltips before the next hit test.
            QTest::mouseMove(window, point);
            QCoreApplication::processEvents();
            QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, point);
        };
        for (const auto *name : {"archiveNewTab", "archiveSettings", "browserViewMenuButton",
                                 "browserGridButton", "browserSortMenuButton", "inspectorClose"}) {
            auto *button = item(name);
            QVERIFY(button);
            auto *icon = findVisual(button, "actionIcon");
            QVERIFY(icon);
            QTRY_VERIFY(
                qAbs(icon->mapToItem(button, QPointF(icon->width() / 2, icon->height() / 2)).x() -
                     button->width() / 2) < .5);
            QVERIFY(
                qAbs(icon->mapToItem(button, QPointF(icon->width() / 2, icon->height() / 2)).y() -
                     button->height() / 2) < .5);
        }
        for (const auto *name : {"archiveOpen", "archiveNew", "archiveTest"}) {
            auto *button = item(name);
            auto *icon = findVisual(button, "actionIcon");
            auto *label = findVisual(button, "actionLabel");
            QVERIFY(qAbs(label->mapToItem(button, QPointF()).x() -
                         icon->mapToItem(button, QPointF(icon->width(), 0)).x() - 6) < .01);
        }
        auto *close = item("inspectorClose");
        // Native pointer delivery may coalesce a repeated position left by
        // the previous window. Enter from outside this control after layout.
        QTest::mouseMove(window, QPoint(0, 0));
        QTest::mouseMove(
            window, close->mapToScene(QPointF(close->width() / 2, close->height() / 2)).toPoint());
        QTRY_VERIFY(close->property("hovered").toBool());
        QVERIFY(findVisual(close, "actionBackground")->property("radius").toDouble() > 0);
        auto *extract = item("archiveExtract");
        QCOMPARE(findVisual(extract, "actionBackground")->property("color").value<QColor>(),
                 QColor("#1956ff"));
        for (const auto *name :
             {"browserViewMenuButton", "browserGridButton", "browserSortMenuButton"}) {
            QCOMPARE(item(name)->width(), 28.0);
            QCOMPARE(item(name)->height(), 28.0);
        }
        auto *grid = item("browserGridButton");
        const auto fixedGrid = grid->mapToScene(QPointF());
        controller->enterDirectory(0);
        QTRY_VERIFY(!controller->busy());
        QTRY_COMPARE(grid->mapToScene(QPointF()), fixedGrid);
        controller->showRoot();
        QTRY_VERIFY(!controller->busy());
        click(grid);
        QTRY_VERIFY(!item("fileTableHeader")->isVisible());
        const auto captures = qEnvironmentVariable("XDZIP_VISUAL_AUDIT");
        if (!captures.isEmpty())
            QVERIFY(QDir().mkpath(captures));
        if (!captures.isEmpty())
            QVERIFY(visual_audit::save(window, captures, "review-grid"));
        QSignalSpy frames(window, &QQuickWindow::afterAnimating);
        click(item("browserViewMenuButton"));
        // Visibility changes synchronously; the layout and hit-test tree update on the next frame.
        QTRY_VERIFY(item("fileTableHeader")->isVisible());
        frames.clear();
        window->requestUpdate();
        QTRY_VERIFY(frames.count() > 0);
        for (int column : {3, 4}) {
            auto *header = item(column == 3 ? "sort-column-3" : "sort-column-4");
            QVERIFY(header && header->isVisible() && header->width() > 0);
            const auto point =
                header->mapToScene(QPointF(header->width() / 2, header->height() / 2));
            auto *bar = item("fileTableHeader");
            QVERIFY(bar->contains(bar->mapFromScene(point)));
            click(header);
            QTRY_COMPARE(controller->sortOrder(), column == 3 ? 9 : 11);
            QTRY_VERIFY(!controller->busy());
            click(header);
            QTRY_COMPARE(controller->sortOrder(), column == 3 ? 10 : 12);
            QTRY_VERIFY(!controller->busy());
        }
        for (const auto &[buttonName, menuName] :
             {std::pair{"browserSortMenuButton", "browserSortMenu"},
              std::pair{"browserTasks", "browserTaskMenu"}}) {
            auto *button = item(buttonName);
            click(button);
            auto *menu = window->findChild<QObject *>(QString::fromLatin1(menuName));
            QVERIFY(menu);
            QTRY_VERIFY(menu->property("visible").toBool());
            // Popup coordinates are relative to its QML parent, which differs for the two menus.
            auto *popup = menu->property("contentItem").value<QQuickItem *>();
            QVERIFY(popup);
            const auto top =
                popup->mapToScene(QPointF()).y() - menu->property("topPadding").toDouble();
            const auto bottom = top + menu->property("height").toDouble();
            const auto buttonTop = button->mapToScene(QPointF()).y();
            if (QString::fromLatin1(buttonName) == "browserTasks")
                QVERIFY(bottom <= buttonTop - 3);
            else
                QVERIFY(top >= buttonTop + button->height() + 3);
            if (!captures.isEmpty())
                QVERIFY(visual_audit::save(window, captures, QString::fromLatin1(buttonName)));
            QTest::keyClick(window, Qt::Key_Escape);
        }
        auto *rootNode = item("directoryRoot");
        QTest::mouseClick(window, Qt::RightButton, Qt::NoModifier,
                          rootNode->mapToScene(QPointF(40, rootNode->height() / 2)).toPoint());
        auto *directoryMenu = window->findChild<QObject *>("browserDirectoryMenu");
        QTRY_VERIFY(directoryMenu->property("visible").toBool());
        auto *menuContent = directoryMenu->property("contentItem").value<QQuickItem *>();
        auto *separator = findVisual(menuContent, "commandSeparator");
        QVERIFY(separator);
        QVERIFY(separator->width() > 200);
        if (!captures.isEmpty())
            QVERIFY(visual_audit::save(window, captures, "review-directory-menu"));
        QTest::keyClick(window, Qt::Key_Escape);
    }
    void mainWindowFidelityAndSessionClose() {
        QTemporaryDir directory;
        QVERIFY(directory.isValid());
        const auto archivePath = directory.filePath(QString::fromUtf8("北境旅行.zip"));
        QVERIFY(QFile::copy(QStringLiteral(XDZIP_FIXTURES "/visual-main.zip"), archivePath));
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        window->raise();
        window->requestActivate();
        QTRY_VERIFY(window->isActive());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        controller->openPath(archivePath);
        QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy(), 5000);
        QCOMPARE(controller->total(), 9);
        controller->selectOnlyRow(3);
        QCOMPARE(controller->detail().value("type").toString(), "PNG");
        QCOMPARE(controller->detail().value("modified").toString(), QStringLiteral("2026/10/01"));
        QVERIFY(controller->selectedBytes().toULongLong() > 0);
        QVERIFY(QTest::qWaitForWindowExposed(window));
        const auto captures = qEnvironmentVariable("XDZIP_VISUAL_AUDIT");
        auto *more = findVisual(window->contentItem(), "archiveMore");
        QVERIFY(more);
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            more->mapToScene(QPointF(more->width() / 2, more->height() / 2)).toPoint());
        auto *dropdown = window->findChild<QObject *>("archiveMoreMenu");
        QVERIFY(dropdown);
        QTRY_VERIFY(dropdown->property("visible").toBool());
        QCOMPARE(dropdown->property("width").toDouble(), 240.0);
        QCOMPARE(dropdown->property("font").value<QFont>().pixelSize(), 13);
        if (!captures.isEmpty()) {
            QVERIFY(QDir().mkpath(captures));
            QVERIFY(visual_audit::save(window, captures, "main-dropdown"));
        }
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!dropdown->property("visible").toBool());
        QQuickItem *entry = nullptr;
        QTRY_VERIFY((entry = findVisual(window->contentItem(), "entry-3-0")));
        QTest::mouseClick(window, Qt::RightButton, Qt::NoModifier,
                          entry->mapToScene(QPointF(100, entry->height() / 2)).toPoint());
        auto *context = window->findChild<QObject *>("browserEntryMenu");
        QVERIFY(context);
        QTRY_VERIFY(context->property("visible").toBool());
        QCOMPARE(context->property("width").toDouble(), 240.0);
        auto *properties = window->findChild<QQuickItem *>("contextProperties");
        QVERIFY(properties);
        QCOMPARE(properties->property("font").value<QFont>().pixelSize(), 13);
        if (!captures.isEmpty())
            QVERIFY(visual_audit::save(window, captures, "main-context"));
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!context->property("visible").toBool());
        if (!captures.isEmpty()) {
            QVERIFY(QDir().mkpath(captures));
            QVERIFY(visual_audit::save(window, captures, "main-light"));
            window->setProperty("darkMode", true);
            QVERIFY(visual_audit::save(window, captures, "main-dark"));
            window->setProperty("darkMode", false);
            // Layout evidence only: native Cocoa window buttons are not drawn by Qt.
            window->setProperty("integratedTitlebar", true);
            QVERIFY(visual_audit::save(window, captures, "main-integrated-layout"));
        }
        auto *all = findVisual(window->contentItem(), "selectPage");
        QVERIFY(all);
        const auto selectPoint =
            all->mapToScene(QPointF(all->width() / 2, all->height() / 2)).toPoint();
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, selectPoint);
        QTRY_COMPARE(controller->selectedCount(), 9);
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, selectPoint);
        QTRY_COMPARE(controller->selectedCount(), 0);
        const auto session = controller->sessionId();
        auto *close = findVisual(window->contentItem(), "archiveTabClose");
        QVERIFY(close);
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            close->mapToScene(QPointF(close->width() / 2, close->height() / 2)).toPoint());
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy() && !controller->ready(), 5000);
        QVERIFY(controller->sessionId().isEmpty());
        QVERIFY(controller->runtimeAvailable());
        QCOMPARE(controller->total(), 0);
        QCOMPARE(controller->selectedBytes(), "0");
        QCOMPARE(controller->entries()->rowCount(), 0);
        QVERIFY(window->isVisible());
        controller->openPath(archivePath);
        QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy(), 5000);
        QVERIFY(controller->sessionId() != session);
        QCOMPARE(controller->total(), 9);
        // Restoring the design's columns must not hide the existing review state.
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/unsafe-names.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(controller->ready() && !controller->busy(), 5000);
        controller->selectOnlyRow(0);
        QVERIFY(controller->detail().value("attention").toBool());
        QVERIFY(
            controller->detail().value("status").toString().contains(QString::fromUtf8("需检查")));
        QVERIFY(controller->entries()
                    ->data(controller->entries()->index(0, 0), Qt::UserRole + 5)
                    .toBool());
    }
    void renderRealArchive() {
        QQmlApplicationEngine engine;
        engine.load(QUrl("qrc:/xdzip/ui/Browser.qml"));
        QVERIFY(!engine.rootObjects().isEmpty());
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        auto *controller = window->findChild<BrowserController *>();
        QVERIFY(controller);
        QCOMPARE(window->property("accentTextColor").value<QColor>(), QColor(Qt::white));
        controller->openPath(QStringLiteral(XDZIP_FIXTURES "/normal.zip"));
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        QCOMPARE(controller->total(), 4);
        QCOMPARE(controller->entries()
                     ->data(controller->entries()->index(1, 0), Qt::DisplayRole)
                     .toString(),
                 QString::fromUtf8("notes.txt"));
        QVERIFY(QTest::qWaitForWindowExposed(window));
        QQuickItem *cell = nullptr;
        QTRY_VERIFY((cell = findVisual(window->contentItem(), "entry-1-0")));
        const auto point = cell->mapToScene(QPointF(120, 24)).toPoint();
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, point);
        QTRY_COMPARE(controller->selectedCount(), 1);
        QCOMPARE(controller->detail().value("id").toString(), "3");
        auto *selection = findVisual(cell, "entrySelection");
        QVERIFY(selection);
        const auto checkPoint =
            selection->mapToScene(QPointF(selection->width() / 2, selection->height() / 2))
                .toPoint();
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, checkPoint);
        QTRY_COMPARE(controller->selectedCount(), 0);
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, checkPoint);
        QTRY_COMPARE(controller->selectedCount(), 1);
        QTest::qWait(30);
        QVERIFY(window->grabWindow().save("zip-browser.png"));
        window->setProperty("darkMode", true);
        QTest::qWait(30);
        QVERIFY(window->grabWindow().save("zip-browser-dark.png"));
        window->setProperty("darkMode", false);
        controller->inspectSelection();
        QTRY_VERIFY_WITH_TIMEOUT(!controller->busy(), 5000);
        auto *reportWindow = window->findChild<QQuickWindow *>("preflightWindow");
        QVERIFY(reportWindow);
        QTRY_VERIFY(reportWindow->isVisible());
        QCOMPARE(reportWindow->transientParent(), window);
        QCOMPARE(reportWindow->modality(), Qt::NonModal);
        QVERIFY(reportWindow->flags().testFlag(Qt::WindowTitleHint));
        QVERIFY(QTest::qWaitForWindowExposed(reportWindow));
        QVERIFY(reportWindow->grabWindow().save("zip-preflight.png"));
        if (!qEnvironmentVariableIsEmpty("XDZIP_VISUAL_AUDIT")) {
            const auto directory = qEnvironmentVariable("XDZIP_VISUAL_AUDIT");
            QVERIFY(QDir().mkpath(directory));
            QVERIFY(visual_audit::save(reportWindow, directory, "preflight-light"));
        }
        reportWindow->close();
        QVERIFY(window->isVisible());
        QCOMPARE(controller->selectedCount(), 1);
        QVERIFY(visual_audit::capture(engine, window));
        // Hiding/reopening details must preserve the actual archive selection.
        window->resize(1024, 622);
        auto *inspector = findVisual(window->contentItem(), "archiveInspector");
        auto *toggle = findVisual(window->contentItem(), "inspectorToggle");
        QVERIFY(inspector && toggle);
        QTRY_VERIFY(inspector->isVisible());
        auto *closeDetails = findVisual(inspector, "inspectorClose");
        QVERIFY(closeDetails);
        QTest::mouseClick(
            window, Qt::LeftButton, Qt::NoModifier,
            closeDetails->mapToScene(QPointF(closeDetails->width() / 2, closeDetails->height() / 2))
                .toPoint());
        QTRY_VERIFY(!inspector->isVisible());
        QVERIFY(QMetaObject::invokeMethod(toggle, "clicked"));
        QTRY_VERIFY(inspector->isVisible());
        window->resize(832, 544);
        QTRY_VERIFY(!inspector->isVisible());
        QVERIFY(QMetaObject::invokeMethod(toggle, "clicked"));
        auto *detailsWindow = window->findChild<QQuickWindow *>("inspectorWindow");
        QVERIFY(detailsWindow);
        QTRY_VERIFY(detailsWindow->isVisible());
        QCOMPARE(detailsWindow->transientParent(), window);
        QCOMPARE(detailsWindow->modality(), Qt::NonModal);
        QVERIFY(!detailsWindow->flags().testFlag(Qt::FramelessWindowHint));
        QCOMPARE(window->width(), 832);
        QCOMPARE(controller->selectedCount(), 1);
        if (!qEnvironmentVariableIsEmpty("XDZIP_VISUAL_AUDIT"))
            QVERIFY(visual_audit::save(detailsWindow, qEnvironmentVariable("XDZIP_VISUAL_AUDIT"),
                                       "inspector-independent"));
        detailsWindow->close();
    }
};
QTEST_MAIN(BrowserTest)
#include "browser_test.moc"
