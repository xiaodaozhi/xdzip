#pragma once
#include <QObject>
#include <QtQml/qqmlregistration.h>

class UiPreferences : public QObject {
    Q_OBJECT
    QML_NAMED_ELEMENT(AppSettings)
    QML_SINGLETON
    Q_PROPERTY(QString theme READ theme NOTIFY appearanceChanged)
    Q_PROPERTY(double textScale READ textScale NOTIFY appearanceChanged)
    Q_PROPERTY(bool dark READ dark NOTIFY appearanceChanged)
    Q_PROPERTY(QString savedTheme READ savedTheme NOTIFY savedChanged)
    Q_PROPERTY(double savedTextScale READ savedTextScale NOTIFY savedChanged)
    Q_PROPERTY(QString language READ language NOTIFY savedChanged)
    Q_PROPERTY(QString activeLanguage READ activeLanguage CONSTANT)
    Q_PROPERTY(QString error READ error NOTIFY errorChanged)
    Q_PROPERTY(QString density READ density NOTIFY appearanceChanged)
    Q_PROPERTY(QString savedDensity READ savedDensity NOTIFY savedChanged)
    Q_PROPERTY(bool recentEnabled READ recentEnabled NOTIFY savedChanged)
    Q_PROPERTY(bool trayEnabled READ trayEnabled NOTIFY savedChanged)
    bool m_trayEnabled = true;
    QString m_density = QStringLiteral("standard");
    QString m_savedDensity = QStringLiteral("standard");
    bool m_recentEnabled = true;
    QString m_theme = QStringLiteral("system");
    QString m_savedTheme = QStringLiteral("system");
    double m_scale = 1;
    double m_savedScale = 1;
    QString m_language = QStringLiteral("zh-CN");
    QString m_error;
    void applyColorScheme();

  public:
    explicit UiPreferences(QObject *parent = nullptr);
    QString theme() const {
        return m_theme;
    }
    double textScale() const {
        return m_scale;
    }
    bool dark() const;
    QString density() const {
        return m_density;
    }
    QString savedDensity() const {
        return m_savedDensity;
    }
    bool recentEnabled() const {
        return m_recentEnabled;
    }
    bool trayEnabled() const {
        return m_trayEnabled;
    }
    Q_INVOKABLE QStringList recentArchives() const;
    Q_INVOKABLE void previewDensity(const QString &density);
    Q_INVOKABLE void rememberArchive(const QString &path);
    Q_INVOKABLE bool saveDetails(const QString &theme, double textScale, const QString &language,
                                 const QString &density, bool recentEnabled, bool clearRecent);
    Q_INVOKABLE bool saveDetails(const QString &theme, double textScale, const QString &language,
                                 const QString &density, bool recentEnabled, bool clearRecent,
                                 bool trayEnabled);
    QString savedTheme() const {
        return m_savedTheme;
    }
    double savedTextScale() const {
        return m_savedScale;
    }
    QString language() const {
        return m_language;
    }
    QString activeLanguage() const;
    QString error() const {
        return m_error;
    }
    Q_INVOKABLE void preview(const QString &theme, double textScale);
    Q_INVOKABLE void cancelPreview();
    Q_INVOKABLE bool save(const QString &theme, double textScale, const QString &language);
  signals:
    void recentChanged();
    void appearanceChanged();
    void savedChanged();
    void errorChanged();
};
