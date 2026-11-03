#pragma once

#include <QString>

class QSettings;

enum class AssociationDescriptionSync { Skipped, Unchanged, Updated, Failed };

AssociationDescriptionSync syncAssociationDescriptions(QSettings &install, QSettings &classes,
                                                       const QString &executable,
                                                       const QString &descriptionPattern);

#ifdef Q_OS_WIN
void syncInstalledAssociationDescriptions(const QString &descriptionPattern);
#endif
