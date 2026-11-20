#include "PrivateCache.h"

#include <QCoreApplication>
#include <QDir>
#include <QFile>
#include <QTemporaryDir>

#include <sys/stat.h>
#include <unistd.h>

int main(int argc, char **argv) {
    QCoreApplication application(argc, argv);
    QTemporaryDir temporary;
    if (!temporary.isValid())
        return 1;

    const auto existing = temporary.filePath("opened-files");
    if (!QDir().mkdir(existing) || chmod(QFile::encodeName(existing).constData(), 0775) != 0 ||
        !preparePrivateCache(existing))
        return 2;
    struct stat metadata {};
    if (stat(QFile::encodeName(existing).constData(), &metadata) != 0 ||
        (metadata.st_mode & 0777) != 0700)
        return 3;

    const auto created = temporary.filePath("exported-files");
    const mode_t oldMask = umask(0002);
    const bool createdSafely = preparePrivateCache(created);
    umask(oldMask);
    if (!createdSafely || stat(QFile::encodeName(created).constData(), &metadata) != 0 ||
        (metadata.st_mode & 0777) != 0700)
        return 4;

    const auto outside = temporary.filePath("outside");
    const auto link = temporary.filePath("linked-cache");
    if (!QDir().mkdir(outside) || chmod(QFile::encodeName(outside).constData(), 0775) != 0 ||
        symlink(QFile::encodeName(outside).constData(), QFile::encodeName(link).constData()) != 0)
        return 5;
    if (preparePrivateCache(link) || stat(QFile::encodeName(outside).constData(), &metadata) != 0 ||
        (metadata.st_mode & 0777) != 0775)
        return 6;
    return 0;
}
