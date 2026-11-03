// Bounded, non-secret HTTP evidence for the SDK-free update probe.
#pragma once

#include <QDateTime>
#include <QJsonArray>
#include <QJsonObject>
#include <QNetworkAccessManager>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <algorithm>

class UpdateProbeDiagnostics {
  public:
    void recordRequest(const QNetworkRequest &request) {
        ++m_requests;
        m_authorizationPresent |= request.hasRawHeader("Authorization");
    }

    void recordResponse(QNetworkReply *reply) {
        const auto status = reply->attribute(QNetworkRequest::HttpStatusCodeAttribute).toInt();
        if (reply->url().scheme() == QStringLiteral("https") &&
            reply->error() == QNetworkReply::NoError && status == 200)
            ++m_httpsResponses;
        if (m_responses.size() >= 32) {
            m_truncated = true;
            return;
        }
        const bool api =
            reply->url().host().compare(QStringLiteral("api.github.com"), Qt::CaseInsensitive) == 0;
        QJsonObject headers{{"remaining", numericHeader(reply, "x-ratelimit-remaining")},
                            {"reset", numericHeader(reply, "x-ratelimit-reset")},
                            {"retry_after", numericHeader(reply, "retry-after")}};
        const auto date =
            QDateTime::fromString(QString::fromLatin1(reply->rawHeader("date")), Qt::RFC2822Date);
        m_responses.append(QJsonObject{
            {"host", api ? QStringLiteral("api.github.com") : QStringLiteral("other_allowed_host")},
            {"route", api && reply->url().path() == "/repos/xiaodaozhi/xdzip/releases"
                          ? QStringLiteral("release_list")
                          : QStringLiteral("other")},
            {"status", status},
            {"rate_headers", headers},
            {"server_date_epoch",
             date.isValid() ? QJsonValue(date.toSecsSinceEpoch()) : QJsonValue(QJsonValue::Null)}});
    }

    void recordEncrypted() {
        ++m_encrypted;
    }

    QJsonObject report(const QString &error, bool timeout) const {
        return {{"http_diagnostics_schema", 1},
                {"request_count", m_requests},
                {"http_responses", m_responses},
                {"response_records_truncated", m_truncated},
                {"authorization_present", m_authorizationPresent},
                {"rate_limit_confirmed", error == QStringLiteral("rate_limited")},
                {"confirmed_from_controller", true},
                {"timeout", timeout},
                {"encrypted_connections", m_encrypted},
                {"https_responses", m_httpsResponses}};
    }

  private:
    static QJsonValue numericHeader(QNetworkReply *reply, const char *name) {
        if (!reply->hasRawHeader(name))
            return QJsonValue::Null;
        const auto value = reply->rawHeader(name).trimmed();
        if (value.isEmpty() || value.size() > 20 ||
            !std::all_of(value.cbegin(), value.cend(),
                         [](char digit) { return digit >= '0' && digit <= '9'; }))
            return QStringLiteral("invalid");
        return QString::fromLatin1(value);
    }

    int m_requests = 0, m_encrypted = 0, m_httpsResponses = 0;
    bool m_authorizationPresent = false, m_truncated = false;
    QJsonArray m_responses;
};

class UpdateProbeNetwork final : public QNetworkAccessManager {
  public:
    UpdateProbeNetwork() {
        connect(this, &QNetworkAccessManager::encrypted, this,
                [this](QNetworkReply *) { evidence.recordEncrypted(); });
        connect(this, &QNetworkAccessManager::finished, this,
                [this](QNetworkReply *reply) { evidence.recordResponse(reply); });
    }
    UpdateProbeDiagnostics evidence;

  protected:
    QNetworkReply *createRequest(Operation operation, const QNetworkRequest &request,
                                 QIODevice *outgoingData = nullptr) override {
        evidence.recordRequest(request);
        return QNetworkAccessManager::createRequest(operation, request, outgoingData);
    }
};
