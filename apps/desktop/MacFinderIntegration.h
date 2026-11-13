#pragma once
#include <QString>

// Only the fixed, same-user Finder configuration is changed by the public
// integration entry. The explicit path overload is used by owned-file tests.
bool writeMacFinderConfiguration(const QString &application, const QString &path, bool enabled,
                                 const QString &language, const QString &revision = {});
bool updateMacFinderIntegration(bool enabled, const QString &language);
bool macFinderIntegrationConfigured();
