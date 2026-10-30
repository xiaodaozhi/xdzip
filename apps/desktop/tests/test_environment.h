#pragma once

#include <QByteArray>
#include <QString>
#include <QTemporaryDir>
#include <memory>
#include <vector>

namespace test_environment {
// Null means absent; a non-null empty QString means an explicitly empty OS value.
// On Windows use this or QProcessEnvironment for presence: the CRT setter deletes
// empty entries even though the native environment block can preserve them.
QString processValue(const char *name);

// Fixture-only, serial main-thread mutation. Construct before the associated
// engines/workers and destroy after they have joined; never use in product code.
class ScopedVariable {
  public:
    ScopedVariable(const char *name, const QString &value);
    ~ScopedVariable();
    ScopedVariable(const ScopedVariable &) = delete;
    ScopedVariable &operator=(const ScopedVariable &) = delete;
    bool valid() const {
        return m_error.isEmpty();
    }
    QString error() const {
        return m_error;
    }

  private:
    QByteArray m_name;
    QString m_processValue, m_crtValue, m_error;
    QByteArray m_bytes;
    bool m_restore = false;
};

// The caller owns a fresh directory. Probe real child writes, locking, staging,
// atomic commit and readback; existence/isValid alone is insufficient.
bool checkStorage(const QString &directory, QString *error);

class Runtime {
  public:
    explicit Runtime(const QString &buildRoot);
    void close();
    bool valid() const {
        return m_error.isEmpty();
    }
    QString error() const {
        return m_error;
    }

  private:
    // Variables restore before the directory is removed, including early returns.
    std::unique_ptr<QTemporaryDir> m_directory;
    std::vector<std::unique_ptr<ScopedVariable>> m_variables;
    QString m_error;
};
} // namespace test_environment
