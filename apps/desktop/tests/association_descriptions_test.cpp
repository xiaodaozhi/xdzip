#include "WindowsAssociationDescriptions.h"

#include <QDir>
#include <QFile>
#include <QSettings>
#include <QTemporaryDir>
#include <QUuid>
#include <QtTest>

namespace {
const QStringList formats{"ZIP", "TAR", "TGZ", "GZ", "7Z", "RAR", "CAB", "ISO", "UDF"};

struct Fixture {
    QTemporaryDir directory;
    QString root;
    QSettings install;
    QSettings classes;

    Fixture()
        : root(directory.filePath(QStringLiteral("install \u5b89\u88c5"))),
          install(directory.filePath("install.ini"), QSettings::IniFormat),
          classes(directory.filePath("classes.ini"), QSettings::IniFormat) {
        QDir().mkpath(root);
        for (const auto &name : {"xdzip.exe", "xdzip-association.exe", "install-manifest.txt"}) {
            QFile file(QDir(root).filePath(QLatin1String(name)));
            if (file.open(QIODevice::WriteOnly))
                file.write("fixture");
        }
        install.setValue("InstallId", "56DF0D26-2B26-40C4-9E85-FF0F8C8937CA");
        install.setValue("Location", root);
        const auto launcher =
            QDir::toNativeSeparators(QDir(root).filePath("xdzip-association.exe"));
        const auto command = QStringLiteral("\"%1\" -- \"%2\"").arg(launcher, QStringLiteral("%1"));
        for (const auto &format : formats) {
            const auto key = QStringLiteral("xdzip.Archive.") + format;
            classes.setValue(key + "/Default", "xdzip " + format + " archive");
            classes.setValue(key + "/shell/open/command/Default", command);
        }
        classes.setValue("foreign/Default", "other application");
        install.sync();
        classes.sync();
    }

    QString executable() const {
        return QDir(root).filePath("xdzip.exe");
    }
};
} // namespace

class AssociationDescriptionsTest final : public QObject {
    Q_OBJECT

  private slots:
    void switchesAllFormatsAndPreservesOtherValues() {
        Fixture f;
        QVERIFY(f.directory.isValid());
        const auto chinese = QStringLiteral("%1 \u538b\u7f29\u5305");
        QCOMPARE(syncAssociationDescriptions(f.install, f.classes, f.executable(), chinese),
                 AssociationDescriptionSync::Updated);
        for (const auto &format : formats)
            QCOMPARE(f.classes.value("xdzip.Archive." + format + "/Default").toString(),
                     chinese.arg(format));
        QCOMPARE(f.classes.value("foreign/Default").toString(),
                 QStringLiteral("other application"));
        QCOMPARE(syncAssociationDescriptions(f.install, f.classes, f.executable(), chinese),
                 AssociationDescriptionSync::Unchanged);

        const auto english = QStringLiteral("%1 archive");
        QCOMPARE(syncAssociationDescriptions(f.install, f.classes, f.executable(), english),
                 AssociationDescriptionSync::Updated);
        for (const auto &format : formats)
            QCOMPARE(f.classes.value("xdzip.Archive." + format + "/Default").toString(),
                     english.arg(format));
    }

    void rejectsMismatchedOrMissingHandlerWithoutPartialWrites() {
        Fixture f;
        f.classes.setValue("xdzip.Archive.UDF/shell/open/command/Default", "foreign command");
        QCOMPARE(syncAssociationDescriptions(f.install, f.classes, f.executable(), "%1 archive"),
                 AssociationDescriptionSync::Skipped);
        QCOMPARE(f.classes.value("xdzip.Archive.ZIP/Default").toString(),
                 QStringLiteral("xdzip ZIP archive"));
        f.classes.remove("xdzip.Archive.UDF");
        QCOMPARE(syncAssociationDescriptions(f.install, f.classes, f.executable(), "%1 archive"),
                 AssociationDescriptionSync::Skipped);
        QVERIFY(!f.classes.contains("xdzip.Archive.UDF/Default"));
    }

    void rejectsOtherInstallationsAndInvalidPatterns() {
        Fixture f;
        const auto run = [&](const QString &executable, const QString &pattern) {
            return syncAssociationDescriptions(f.install, f.classes, executable, pattern);
        };
        QCOMPARE(run(f.executable(), "archive"), AssociationDescriptionSync::Skipped);
        QCOMPARE(run(QDir(f.root).filePath("xdzip-association.exe"), "%1 archive"),
                 AssociationDescriptionSync::Skipped);
        f.install.setValue("InstallId", "other");
        QCOMPARE(run(f.executable(), "%1 archive"), AssociationDescriptionSync::Skipped);
        f.install.setValue("InstallId", "56DF0D26-2B26-40C4-9E85-FF0F8C8937CA");
        QVERIFY(QFile::remove(QDir(f.root).filePath("install-manifest.txt")));
        QCOMPARE(run(f.executable(), "%1 archive"), AssociationDescriptionSync::Skipped);
        QCOMPARE(f.classes.value("xdzip.Archive.ZIP/Default").toString(),
                 QStringLiteral("xdzip ZIP archive"));
    }

#ifdef Q_OS_WIN
    void windowsRegistryDefaultValuesInDisposableCi() {
        if (qEnvironmentVariable("GITHUB_ACTIONS") != "true" ||
            qEnvironmentVariable("RUNNER_OS") != "Windows")
            QSKIP("Native registry fixture is restricted to disposable Windows CI");
        Fixture f;
        const auto id = QUuid::createUuid().toString(QUuid::WithoutBraces);
        QSettings parent(QStringLiteral("HKEY_CURRENT_USER\\Software\\xdzip-association-tests"),
                         QSettings::NativeFormat);
        QSettings classes(QStringLiteral("HKEY_CURRENT_USER\\Software\\xdzip-association-tests\\") +
                              id,
                          QSettings::NativeFormat);
        struct Cleanup {
            QSettings &settings;
            QString id;
            ~Cleanup() {
                settings.remove(id);
                settings.sync();
            }
        } cleanup{parent, id};
        for (const auto &format : formats) {
            const auto base = QStringLiteral("xdzip.Archive.") + format;
            for (const auto &suffix : {"/Default", "/shell/open/command/Default"}) {
                const auto key = base + QLatin1String(suffix);
                classes.setValue(key, f.classes.value(key));
            }
        }
        classes.sync();
        QCOMPARE(classes.status(), QSettings::NoError);
        const auto pattern = QStringLiteral("%1 \u538b\u7f29\u5305");
        QCOMPARE(syncAssociationDescriptions(f.install, classes, f.executable(), pattern),
                 AssociationDescriptionSync::Updated);
        for (const auto &format : formats)
            QCOMPARE(classes.value("xdzip.Archive." + format + "/Default").toString(),
                     pattern.arg(format));
        QCOMPARE(syncAssociationDescriptions(f.install, classes, f.executable(), pattern),
                 AssociationDescriptionSync::Unchanged);
    }
#endif
};

QTEST_GUILESS_MAIN(AssociationDescriptionsTest)
#include "association_descriptions_test.moc"
