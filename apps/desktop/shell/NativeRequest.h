#pragma once
// The file-manager host never loads Qt or an archive backend. Only this bounded
// selection manifest crosses into the elected desktop process.
#include <array>
#include <filesystem>
#include <fstream>
#include <string>
#include <vector>
#ifdef _WIN32
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>

#include <bcrypt.h>
#include <sddl.h>
#include <winver.h>
#else
#include <cerrno>
#include <fcntl.h>
#include <spawn.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <thread>
#include <unistd.h>
#ifdef __APPLE__
#include <stdlib.h>
#else
#include <sys/random.h>
#endif
extern char **environ;
#endif

namespace xdzip_shell {
namespace fs = std::filesystem;
inline bool protocolMarker(const fs::path &path) {
    std::error_code error;
    if (fs::symlink_status(path, error).type() != fs::file_type::regular)
        return false;
    std::ifstream file(path, std::ios::binary);
    char value[3]{};
    file.read(value, sizeof(value));
    return file.gcount() == 2 && value[0] == '1' && value[1] == '\n';
}
#ifdef _WIN32
inline bool commandTargetsExecutable(const std::wstring &command, const fs::path &executable) {
    if (command.empty() || command.front() != L'"')
        return false;
    const auto end = command.find(L'"', 1);
    if (end == std::wstring::npos || command.substr(end + 1) != L" -- \"%1\"")
        return false;
    std::error_code error;
    return fs::equivalent(fs::path(command.substr(1, end - 1)), executable, error) && !error;
}
inline bool registeredTargetMatches(const fs::path &executable) {
    // A legacy rollback may retain the former root executable. Only the
    // self-owned application command identifies the active installation target.
    constexpr auto key = L"Software\\Classes\\Applications\\xdzip.exe\\shell\\open\\command";
    DWORD size = 0;
    if (RegGetValueW(HKEY_CURRENT_USER, key, nullptr, RRF_RT_REG_SZ, nullptr, nullptr, &size) !=
            ERROR_SUCCESS ||
        size < sizeof(wchar_t) || size > 65536 || size % sizeof(wchar_t))
        return false;
    std::wstring command(size / sizeof(wchar_t), L'\0');
    if (RegGetValueW(HKEY_CURRENT_USER, key, nullptr, RRF_RT_REG_SZ, nullptr, command.data(),
                     &size) != ERROR_SUCCESS ||
        size < sizeof(wchar_t) || size % sizeof(wchar_t))
        return false;
    command.resize(size / sizeof(wchar_t));
    if (command.back() != 0)
        return false;
    command.pop_back();
    if (command.find(L'\0') != std::wstring::npos)
        return false;
    return commandTargetsExecutable(command, executable);
}
inline bool supportsShellProtocol(const fs::path &executable) {
    DWORD ignored = 0;
    const auto size = GetFileVersionInfoSizeW(executable.c_str(), &ignored);
    if (!size || size > 1024 * 1024)
        return false;
    std::vector<unsigned char> bytes(size);
    if (!GetFileVersionInfoW(executable.c_str(), 0, size, bytes.data()))
        return false;
    void *value = nullptr;
    UINT length = 0;
    return VerQueryValueW(bytes.data(), L"\\StringFileInfo\\040904b0\\XdzipShellProtocol", &value,
                          &length) &&
           value && length >= 2 && length <= 4 && static_cast<const wchar_t *>(value)[0] == L'1' &&
           static_cast<const wchar_t *>(value)[1] == 0;
}
#endif
inline std::string jsonString(const std::string &value) {
    const char *hex = "0123456789abcdef";
    std::string out = "\"";
    for (unsigned char c : value) {
        if (c == '"' || c == '\\') {
            out += '\\';
            out += char(c);
        } else if (c < 32) {
            out += "\\u00";
            out += hex[c >> 4];
            out += hex[c & 15];
        } else
            out += char(c);
    }
    return out + '"';
}
inline std::string randomHex() {
    std::array<unsigned char, 16> bytes{};
#ifdef _WIN32
    if (BCryptGenRandom(nullptr, bytes.data(), ULONG(bytes.size()),
                        BCRYPT_USE_SYSTEM_PREFERRED_RNG) != 0)
        return {};
#elif defined(__APPLE__)
    arc4random_buf(bytes.data(), bytes.size());
#else
    if (getrandom(bytes.data(), bytes.size(), 0) != ssize_t(bytes.size()))
        return {};
#endif
    std::string result;
    for (auto b : bytes) {
        result += "0123456789abcdef"[b >> 4];
        result += "0123456789abcdef"[b & 15];
    }
    return result;
}
inline bool validRequestIdentity(const std::string &value) {
    if (value.size() != 36)
        return false;
    bool nonzero = false;
    for (size_t i = 0; i < value.size(); ++i) {
        if (i == 8 || i == 13 || i == 18 || i == 23) {
            if (value[i] != '-')
                return false;
        } else {
            const auto c = value[i];
            if (!((c >= '0' && c <= '9') || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F')))
                return false;
            nonzero |= c != '0';
        }
    }
    return nonzero;
}
inline fs::path requestFile(const std::string &action, const std::vector<fs::path> &paths,
                            bool archiveCandidates = false,
                            const std::string &requestIdentity = {}) {
    if (paths.empty() || paths.size() > 128)
        return {};
    if (!requestIdentity.empty() && !validRequestIdentity(requestIdentity))
        return {};
    const auto token = randomHex();
    if (token.empty())
        return {};
    const auto id = requestIdentity.empty() ? token.substr(0, 8) + '-' + token.substr(8, 4) + '-' +
                                                  token.substr(12, 4) + '-' + token.substr(16, 4) +
                                                  '-' + token.substr(20)
                                            : requestIdentity;
    std::string bytes =
        "{\"schema_version\":1,\"id\":" + jsonString(id) + ",\"action\":" + jsonString(action) +
        ",\"archive_candidates\":" + (archiveCandidates ? "true" : "false") + ",\"paths\":[";
    for (const auto &path : paths) {
        if (!path.is_absolute() || path.u8string().size() > 4096)
            return {};
        if (bytes.back() != '[')
            bytes += ',';
        bytes += jsonString(path.u8string());
    }
    bytes += "]}";
    if (bytes.size() > 256 * 1024)
        return {};
    const auto directory = fs::temp_directory_path() / ("xdzip-shell-" + token);
#ifdef _WIN32
    // An explicit protected owner/system DACL is applied at creation, before
    // any selected path is written. All native handles remain locally owned.
    HANDLE processToken = nullptr;
    if (!OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &processToken))
        return {};
    DWORD size = 0;
    GetTokenInformation(processToken, TokenUser, nullptr, 0, &size);
    std::vector<unsigned char> buffer(size);
    const bool tokenOk =
        GetTokenInformation(processToken, TokenUser, buffer.data(), size, &size) != 0;
    CloseHandle(processToken);
    if (!tokenOk)
        return {};
    LPWSTR sid = nullptr;
    if (!ConvertSidToStringSidW(reinterpret_cast<TOKEN_USER *>(buffer.data())->User.Sid, &sid))
        return {};
    const std::wstring dacl = L"D:P(A;OICI;FA;;;SY)(A;OICI;FA;;;" + std::wstring(sid) + L")";
    LocalFree(sid);
    PSECURITY_DESCRIPTOR descriptor = nullptr;
    if (!ConvertStringSecurityDescriptorToSecurityDescriptorW(dacl.c_str(), SDDL_REVISION_1,
                                                              &descriptor, nullptr))
        return {};
    SECURITY_ATTRIBUTES attributes{sizeof(SECURITY_ATTRIBUTES), descriptor, FALSE};
    const bool made = CreateDirectoryW(directory.c_str(), &attributes) != 0;
    LocalFree(descriptor);
    if (!made)
        return {};
    const auto file = directory / L"request.json";
    HANDLE handle = CreateFileW(file.c_str(), GENERIC_WRITE, 0, nullptr, CREATE_NEW,
                                FILE_ATTRIBUTE_NORMAL, nullptr);
    DWORD written = 0;
    const bool ok = handle != INVALID_HANDLE_VALUE &&
                    WriteFile(handle, bytes.data(), DWORD(bytes.size()), &written, nullptr) &&
                    size_t(written) == bytes.size();
    if (handle != INVALID_HANDLE_VALUE)
        CloseHandle(handle);
#else
    if (mkdir(directory.c_str(), 0700) != 0)
        return {};
    const auto file = directory / "request.json";
    const int handle =
        open(file.c_str(), O_WRONLY | O_CREAT | O_EXCL | O_NOFOLLOW | O_CLOEXEC, 0600);
    bool ok = handle >= 0;
    size_t offset = 0;
    while (ok && offset < bytes.size()) {
        const auto n = write(handle, bytes.data() + offset, bytes.size() - offset);
        if (n <= 0)
            ok = false;
        else
            offset += size_t(n);
    }
    if (handle >= 0 && close(handle) != 0)
        ok = false;
#endif
    if (!ok) {
        std::error_code error;
        fs::remove(file, error);
        fs::remove(directory, error);
        return {};
    }
    return file;
}
inline bool launch(const fs::path &executable, const std::string &action,
                   const std::vector<fs::path> &paths, bool archiveCandidates = false,
                   const std::string &requestIdentity = {}) {
    const auto file = requestFile(action, paths, archiveCandidates, requestIdentity);
    if (file.empty())
        return false;
#ifdef _WIN32
    std::wstring command =
        L"\"" + executable.wstring() + L"\" --shell-request \"" + file.wstring() + L"\"";
    STARTUPINFOW startup{};
    startup.cb = sizeof(startup);
    PROCESS_INFORMATION process{};
    const bool ok = CreateProcessW(executable.c_str(), command.data(), nullptr, nullptr, FALSE, 0,
                                   nullptr, nullptr, &startup, &process) != 0;
    if (ok) {
        CloseHandle(process.hThread);
        CloseHandle(process.hProcess);
    }
#else
    std::string app = executable.string(), manifest = file.string();
    char flag[] = "--shell-request";
    char *args[] = {app.data(), flag, manifest.data(), nullptr};
    pid_t pid = 0;
    const bool ok = posix_spawn(&pid, app.c_str(), nullptr, nullptr, args, environ) == 0;
    if (ok) {
        // File managers keep this module loaded. Reap forwarded clients as well
        // as the long-lived primary process without changing the host's SIGCHLD.
        try {
            std::thread([pid] {
                int status = 0;
                while (waitpid(pid, &status, 0) < 0 && errno == EINTR) {
                }
            }).detach();
        } catch (...) {
            // The operation was already launched. Never report a rejection that
            // could cause it to be submitted twice when thread resources run out.
        }
    }
#endif
    if (!ok) {
        std::error_code error;
        fs::remove(file, error);
        fs::remove(file.parent_path(), error);
    }
    return ok;
}
} // namespace xdzip_shell
