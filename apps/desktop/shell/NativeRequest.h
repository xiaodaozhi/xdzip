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
#else
#include <fcntl.h>
#include <spawn.h>
#include <sys/stat.h>
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
inline fs::path requestFile(const std::string &action, const std::vector<fs::path> &paths) {
    if (paths.empty() || paths.size() > 128)
        return {};
    const auto token = randomHex();
    if (token.empty())
        return {};
    const auto id = token.substr(0, 8) + '-' + token.substr(8, 4) + '-' + token.substr(12, 4) +
                    '-' + token.substr(16, 4) + '-' + token.substr(20);
    std::string bytes = "{\"schema_version\":1,\"id\":" + jsonString(id) +
                        ",\"action\":" + jsonString(action) + ",\"paths\":[";
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
                   const std::vector<fs::path> &paths) {
    const auto file = requestFile(action, paths);
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
#endif
    if (!ok) {
        std::error_code error;
        fs::remove(file, error);
        fs::remove(file.parent_path(), error);
    }
    return ok;
}
} // namespace xdzip_shell
