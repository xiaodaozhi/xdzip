// apps/desktop/MacUpdateProvider.h

#pragma once
#include <QJsonObject>
#include <QString>
#include <functional>
class UpdateController;
QString macUpdateInstallRoot();
void startMacUpdate(UpdateController *owner, const QJsonObject &candidate,
                    std::function<void(QString)> prepared);
bool commitMacUpdate();
void cancelMacUpdate();
#ifdef XDZIP_UPDATE_TESTS
bool macUpdateLifecycleProbe(UpdateController *owner);
#endif
