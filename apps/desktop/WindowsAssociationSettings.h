#pragma once
#include <QUrl>

namespace file_associations {
inline QUrl windowsSettingsUrl(bool packaged, const QString &applicationId) {
    if (!packaged)
        return QUrl(QStringLiteral("ms-settings:defaultapps?registeredAppUser=xdzip"));
    // A failed identity query falls back to the system's app picker, never to
    // the unrelated registration owned by the ordinary installed edition.
    if (applicationId.isEmpty())
        return QUrl(QStringLiteral("ms-settings:defaultapps"));
    return QUrl(QStringLiteral("ms-settings:defaultapps?registeredAUMID=") +
                QString::fromLatin1(QUrl::toPercentEncoding(applicationId)));
}
} // namespace file_associations
