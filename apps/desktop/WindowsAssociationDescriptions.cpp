#include "WindowsAssociationDescriptions.h"

#include <QCoreApplication>
#include <QDir>
#include <QFileInfo>
#include <QSettings>
#include <QStringList>

#ifdef Q_OS_WIN
#include <qt_windows.h>
#include <shlobj.h>
#endif

AssociationDescriptionSync syncAssociationDescriptions(QSettings &install, QSettings &classes,
                                                       const QString &executable,
                                                       const QString &descriptionPattern) {
    const QStringList formats{"ZIP", "TAR", "TGZ", "GZ", "7Z", "RAR", "CAB", "ISO", "UDF"};
    const auto location = install.value(QStringLiteral("Location")).toString();
    const QFileInfo rootInfo(location);
    const auto root = rootInfo.canonicalFilePath();
    const auto installedExecutable =
        QFileInfo(QDir(root).filePath("xdzip.exe")).canonicalFilePath();
    const auto runningExecutable = QFileInfo(executable).canonicalFilePath();
    if (descriptionPattern.count(QStringLiteral("%1")) != 1 ||
        descriptionPattern.contains(QChar::Null) || install.status() != QSettings::NoError ||
        install.value(QStringLiteral("InstallId")).toString() !=
            QStringLiteral("56DF0D26-2B26-40C4-9E85-FF0F8C8937CA") ||
        root.isEmpty() || installedExecutable.isEmpty() || runningExecutable.isEmpty() ||
        QString::compare(installedExecutable, runningExecutable, Qt::CaseInsensitive) != 0 ||
        !QFileInfo(QDir(root).filePath("install-manifest.txt")).isFile() ||
        !QFileInfo(QDir(root).filePath("xdzip-association.exe")).isFile())
        return AssociationDescriptionSync::Skipped;

    const auto launcher =
        QDir::toNativeSeparators(QDir::cleanPath(QDir(location).filePath("xdzip-association.exe")));
    const auto expectedCommand =
        QStringLiteral("\"%1\" -- \"%2\"").arg(launcher, QStringLiteral("%1"));
    // Validate every existing owned handler before changing any description.
    // Never create missing associations or redirect a foreign command.
    for (const auto &format : formats) {
        const auto base = QStringLiteral("xdzip.Archive.") + format;
        const auto value = base + QStringLiteral("/Default");
        const auto command = base + QStringLiteral("/shell/open/command/Default");
        if (!classes.contains(value) || !classes.contains(command) ||
            QString::compare(QDir::fromNativeSeparators(classes.value(command).toString()),
                             QDir::fromNativeSeparators(expectedCommand), Qt::CaseInsensitive) != 0)
            return AssociationDescriptionSync::Skipped;
    }
    if (classes.status() != QSettings::NoError)
        return AssociationDescriptionSync::Failed;

    bool changed = false;
    for (const auto &format : formats) {
        const auto key = QStringLiteral("xdzip.Archive.") + format + QStringLiteral("/Default");
        const auto description = descriptionPattern.arg(format);
        if (classes.value(key).toString() != description) {
            classes.setValue(key, description);
            changed = true;
        }
    }
    if (!changed)
        return AssociationDescriptionSync::Unchanged;
    classes.sync();
    return classes.status() == QSettings::NoError ? AssociationDescriptionSync::Updated
                                                  : AssociationDescriptionSync::Failed;
}

#ifdef Q_OS_WIN
void syncInstalledAssociationDescriptions(const QString &descriptionPattern) {
    QSettings install(QStringLiteral("HKEY_CURRENT_USER\\Software\\xdzip\\Install"),
                      QSettings::NativeFormat);
    QSettings classes(QStringLiteral("HKEY_CURRENT_USER\\Software\\Classes"),
                      QSettings::NativeFormat);
    const auto result = syncAssociationDescriptions(
        install, classes, QCoreApplication::applicationFilePath(), descriptionPattern);
    if (result == AssociationDescriptionSync::Updated ||
        result == AssociationDescriptionSync::Failed) {
        // A failed multi-value write may be partial; invalidate the shell cache
        // and retry the selected language at the next normal application start.
        SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, nullptr, nullptr);
    }
}
#endif
