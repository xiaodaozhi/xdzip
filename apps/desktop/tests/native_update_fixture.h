// apps/desktop/tests/native_update_fixture.h

#pragma once
class QObject;
class QNetworkAccessManager;
class QQuickWindow;
class ApplicationSessions;
class UiPreferences;
class UpdateController;

QNetworkAccessManager *nativeUpdateFixtureTransport(QObject *parent);
void installNativeUpdateFixture(UpdateController *updates, ApplicationSessions *sessions,
                                UiPreferences *settings, QQuickWindow *window);
