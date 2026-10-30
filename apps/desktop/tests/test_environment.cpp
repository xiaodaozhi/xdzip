#include "test_environment.h"
#include <QCoreApplication>
#include <QDir>
#include <QFile>
#include <QLockFile>
#include <QSaveFile>
#include <QSettings>
#include <QThread>
#ifdef Q_OS_WIN
#include <qt_windows.h>
#endif

namespace test_environment {
namespace {
bool onFixtureThread() {
    return !QCoreApplication::instance() ||
           QThread::currentThread() == QCoreApplication::instance()->thread();
}
#ifdef Q_OS_WIN
// Qt 6.8 reads the wide CRT environment; child processes read the Win32 block.
// Update both without ANSI conversion. These calls require the serial fixture
// lifetime above, not concurrent Qt/native environment mutation.
bool writeCrt(const QByteArray &name, const QString &value) {
    return _wputenv_s(QString::fromLatin1(name).toStdWString().c_str(),
                      value.isNull() ? L"" : value.toStdWString().c_str()) == 0;
}
bool writeProcess(const QByteArray &name, const QString &value) {
    return SetEnvironmentVariableW(QString::fromLatin1(name).toStdWString().c_str(),
                                   value.isNull() ? nullptr : value.toStdWString().c_str());
}
#endif
bool sameValue(const QString &left, const QString &right) {
    return left.isNull() == right.isNull() && left == right;
}
} // namespace

QString processValue(const char *name) {
#ifdef Q_OS_WIN
    const auto wide = QString::fromLatin1(name).toStdWString();
    SetLastError(ERROR_SUCCESS);
    const auto size = GetEnvironmentVariableW(wide.c_str(), nullptr, 0);
    if (!size) {
        const auto error = GetLastError();
        if (error == ERROR_ENVVAR_NOT_FOUND)
            return QString();
        if (error != ERROR_SUCCESS)
            qFatal("Fixture precondition: cannot read process environment");
        return QStringLiteral("");
    }
    QString value(qsizetype(size), Qt::Uninitialized);
    const auto length =
        GetEnvironmentVariableW(wide.c_str(), reinterpret_cast<wchar_t *>(value.data()), size);
    if (length >= size)
        qFatal("Fixture precondition: concurrent environment mutation");
    value.resize(length);
    return value;
#else
    const auto value = qgetenv(name);
    return value.isNull() ? QString() : QString::fromLocal8Bit(value);
#endif
}

ScopedVariable::ScopedVariable(const char *name, const QString &value) : m_name(name) {
    if (!onFixtureThread()) {
        m_error =
            QStringLiteral("Fixture precondition: environment scope requires the main thread");
        return;
    }
    m_processValue = processValue(name);
    m_crtValue = qEnvironmentVariable(name);
    m_bytes = qgetenv(name);
    m_restore = true;
#ifdef Q_OS_WIN
    const bool saved = writeCrt(m_name, value) && writeProcess(m_name, value);
#else
    const bool saved = value.isNull() ? qunsetenv(name) : qputenv(name, value.toUtf8());
#endif
    if (!saved || !sameValue(processValue(name), value) ||
        (!value.isEmpty() && qEnvironmentVariable(name) != value))
        m_error = QStringLiteral("Fixture precondition: Unicode environment update failed");
}

ScopedVariable::~ScopedVariable() {
    if (!m_restore)
        return;
    if (!onFixtureThread())
        qFatal("Fixture precondition: environment scope destroyed outside the main thread");
#ifdef Q_OS_WIN
    // The CRT removes empty values. Restore the native block separately to retain
    // the distinction between an inherited empty value and an absent variable.
    const bool restored = writeCrt(m_name, m_crtValue) && writeProcess(m_name, m_processValue);
    // MSVC cannot set a CRT entry to empty: _wputenv_s deletes it. The native
    // environment block above preserves empty presence for processValue and
    // QProcessEnvironment. Qt's CRT reader retains the empty value, while its
    // null/empty presence distinction is not a Windows restoration guarantee.
    const bool crtRestored = m_crtValue.isEmpty()
                                 ? qEnvironmentVariable(m_name.constData()).isEmpty()
                                 : sameValue(qEnvironmentVariable(m_name.constData()), m_crtValue);
#else
    const bool restored =
        m_bytes.isNull() ? qunsetenv(m_name.constData()) : qputenv(m_name.constData(), m_bytes);
    const bool crtRestored = sameValue(qEnvironmentVariable(m_name.constData()), m_crtValue);
#endif
    if (!restored || !sameValue(processValue(m_name.constData()), m_processValue) || !crtRestored)
        qFatal("Fixture precondition: environment restoration failed");
}

bool checkStorage(const QString &directory, QString *error) {
    const auto fail = [&](const char *step) {
        *error = QStringLiteral("Fixture precondition: temporary storage failed at %1")
                     .arg(QString::fromLatin1(step));
        return false;
    };
    QFile canary(QDir(directory).filePath("fixture-canary"));
    if (!canary.open(QIODevice::WriteOnly | QIODevice::NewOnly) || canary.write("owned") != 5)
        return fail("child file write");
    canary.close();
    if (!canary.remove())
        return fail("child file cleanup");
    QLockFile lock(QDir(directory).filePath("fixture-lock"));
    if (!lock.tryLock(0))
        return fail("lock");
    QTemporaryDir stage(QDir(directory).filePath("fixture-stage-XXXXXX"));
    if (!stage.isValid())
        return fail("staging directory");
    const auto path = stage.filePath("settings.ini");
    QSaveFile output(path);
    output.setDirectWriteFallback(false);
    const QByteArray bytes("[General]\nowned=fixture\n");
    if (!output.open(QIODevice::WriteOnly) ||
        !output.setPermissions(QFileDevice::ReadOwner | QFileDevice::WriteOwner) ||
        output.write(bytes) != bytes.size() || !output.commit())
        return fail("atomic commit");
    QSettings settings(path, QSettings::IniFormat);
    if (settings.status() != QSettings::NoError || settings.value("owned") != "fixture")
        return fail("settings readback");
    return true;
}

Runtime::Runtime(const QString &buildRoot) {
    auto root = qEnvironmentVariable("XDZIP_BROWSER_TEST_TEMP_ROOT");
    if (!root.isEmpty())
        root = QDir::cleanPath(QDir::fromNativeSeparators(root));
    if (root.isEmpty()) {
        QDir().mkpath(buildRoot);
        m_directory = std::make_unique<QTemporaryDir>(QDir(buildRoot).filePath("browser-XXXXXX"));
        if (!m_directory->isValid()) {
            m_error = QStringLiteral("Fixture precondition: cannot create owned temporary root");
            return;
        }
        root = m_directory->path();
    } else if (!root.startsWith(QDir::cleanPath(QDir::fromNativeSeparators(buildRoot)) + "/")) {
        m_error =
            QStringLiteral("Fixture precondition: inherited temporary root is outside the build");
        return;
    }
    // Auxiliary IPC/restart processes retain the parent's temp identity. Their
    // probes are private so concurrent clients cannot contend on the canary/lock.
    QTemporaryDir probe(QDir(root).filePath("preflight-XXXXXX"));
    if (!probe.isValid() || !checkStorage(probe.path(), &m_error)) {
        if (m_error.isEmpty())
            m_error = QStringLiteral("Fixture precondition: cannot create temporary preflight");
        return;
    }
    const auto add = [&](const char *name, const QString &value) {
        auto variable = std::make_unique<ScopedVariable>(name, value);
        if (!variable->valid())
            m_error = variable->error();
        m_variables.push_back(std::move(variable));
    };
    for (const auto *name : {"TEMP", "TMP", "TMPDIR"})
        add(name, root);
    add("XDZIP_BROWSER_TEST_TEMP_ROOT", root);
    // Explicit profiles are intentionally shared with settings restart children.
    if (qEnvironmentVariableIsEmpty("XDZIP_SETTINGS_PATH"))
        add("XDZIP_SETTINGS_PATH", QDir(root).filePath("settings.ini"));
}

void Runtime::close() {
    // Run after the test object and its children have gone, while Qt's application
    // services still exist. The destructor then has no Qt filesystem work left.
    m_variables.clear();
    m_directory.reset();
}
} // namespace test_environment
