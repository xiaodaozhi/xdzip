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
// Returns true when normal application exit must wait for Sparkle to finish
// cancelling its active cycle. A committed installation owns its normal quit.
bool deferMacUpdateQuit(std::function<void()> quit);
QJsonObject macUpdateExitStatus();
#ifdef XDZIP_UPDATE_TESTS
bool macUpdateLifecycleProbe(UpdateController *owner);
#endif
