#pragma once
#include <QObject>
#include <functional>
#include <optional>
class UiPreferences;
class ShellIntegration final : public QObject {
    Q_OBJECT
    Q_PROPERTY(bool available READ available CONSTANT)
    Q_PROPERTY(bool enabled READ enabled NOTIFY changed)
    Q_PROPERTY(QString error READ error NOTIFY changed)
  public:
    explicit ShellIntegration(UiPreferences *settings, QObject *parent = nullptr);
    bool available() const;
    bool enabled() const;
    QString error() const {
        return m_error;
    }
    Q_INVOKABLE void setEnabled(bool value);
    void refresh();
    bool registerExplicitly(bool value);
  signals:
    void changed();

  private:
    friend class ShellTest;
    std::optional<bool> m_availableOverride;
    std::function<bool(bool)> m_applyOverride;
    bool apply(bool value);
    UiPreferences *m_settings;
    QString m_error;
};
