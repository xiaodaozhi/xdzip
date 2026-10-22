// apps/desktop/tests/mac_product_update_fixture.h

#pragma once
#include <QJsonObject>
#include <QString>
class QObject;
class QNetworkAccessManager;
class UpdateController;
class ApplicationSessions;
class UiPreferences;
class QQuickWindow;
QNetworkAccessManager *macProductUpdateFixtureTransport(QObject *parent);
void installMacProductUpdateFixture(UpdateController *updates, ApplicationSessions *sessions,
                                    UiPreferences *settings, QQuickWindow *window);
QString macProductUpdateFixtureFeed(const QJsonObject &authenticatedCandidate);
QString macProductUpdateFixtureAsset(const QJsonObject &authenticatedCandidate,
                                     const QString &requestUrl);
void macProductUpdateFixtureEvent(const QString &stage, const QJsonObject &details);
