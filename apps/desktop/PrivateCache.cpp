#include "PrivateCache.h"

#include <QDir>
#include <QFile>
#ifdef Q_OS_LINUX
#include <fcntl.h>
#include <sys/stat.h>
#include <unistd.h>
#endif

bool preparePrivateCache(const QString &path) {
    if (path.isEmpty() || !QDir().mkpath(path))
        return false;
#ifdef Q_OS_LINUX
    // Operate on the opened directory so a replaced final component cannot
    // redirect chmod. Failure leaves the existing authorization checks intact.
    const int fd =
        open(QFile::encodeName(path).constData(), O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC);
    if (fd < 0)
        return false;
    struct stat metadata {};
    struct stat current {};
    const bool safe = fstat(fd, &metadata) == 0 && S_ISDIR(metadata.st_mode) &&
                      metadata.st_uid == geteuid() && fchmod(fd, S_IRWXU) == 0 &&
                      lstat(QFile::encodeName(path).constData(), &current) == 0 &&
                      S_ISDIR(current.st_mode) && current.st_dev == metadata.st_dev &&
                      current.st_ino == metadata.st_ino;
    close(fd);
    return safe;
#else
    return true;
#endif
}
