// CI-only launcher: preserve the logged-on identity/session while removing
// administrator rights. No accounts, UAC settings or production tokens change.
#include <iostream>
#include <string>
#include <windows.h>
#include <winsafer.h>

int wmain(int argc, wchar_t **argv) {
    wchar_t ci[8]{};
    if (argc < 3 || !GetEnvironmentVariableW(L"GITHUB_ACTIONS", ci, DWORD(std::size(ci))) ||
        std::wstring(ci) != L"true")
        return 2;
    SAFER_LEVEL_HANDLE level = nullptr;
    HANDLE token = nullptr;
    const bool computed = SaferCreateLevel(SAFER_SCOPEID_USER, SAFER_LEVELID_NORMALUSER,
                                           SAFER_LEVEL_OPEN, &level, nullptr) &&
                          SaferComputeTokenFromLevel(level, nullptr, &token, 0, nullptr);
    const auto error = GetLastError();
    if (level)
        SaferCloseLevel(level);
    if (!computed) {
        if (token)
            CloseHandle(token);
        std::cerr << "Cannot compute ordinary-user token: " << error << '\n';
        return 1;
    }
    struct {
        TOKEN_MANDATORY_LABEL label{};
        BYTE sid[SECURITY_MAX_SID_SIZE]{};
    } integrity;
    DWORD length = sizeof(integrity.sid);
    integrity.label.Label.Sid = integrity.sid;
    integrity.label.Label.Attributes = SE_GROUP_INTEGRITY;
    const bool adjusted = CreateWellKnownSid(WinMediumLabelSid, nullptr, integrity.sid, &length) &&
                          SetTokenInformation(token, TokenIntegrityLevel, &integrity,
                                              DWORD(sizeof(TOKEN_MANDATORY_LABEL) + length));
    std::wstring command;
    for (int index = 1; index < argc; ++index) {
        if (std::wstring(argv[index]).find(L'"') != std::wstring::npos) {
            CloseHandle(token);
            return 2;
        }
        command += L"\"" + std::wstring(argv[index]) + L"\" ";
    }
    STARTUPINFOW startup{};
    startup.cb = sizeof(startup);
    PROCESS_INFORMATION process{};
    const bool started =
        adjusted && CreateProcessAsUserW(token, argv[1], command.data(), nullptr, nullptr, FALSE,
                                         CREATE_NO_WINDOW, nullptr, nullptr, &startup, &process);
    const auto startError = GetLastError();
    CloseHandle(token);
    if (!started) {
        std::cerr << "Cannot start ordinary-user probe: " << startError << '\n';
        return 1;
    }
    CloseHandle(process.hThread);
    DWORD exitCode = 1;
    if (WaitForSingleObject(process.hProcess, 60000) == WAIT_OBJECT_0)
        GetExitCodeProcess(process.hProcess, &exitCode);
    else {
        // Own child only. Never terminate another application or Explorer.
        TerminateProcess(process.hProcess, 1);
        WaitForSingleObject(process.hProcess, 5000);
    }
    CloseHandle(process.hProcess);
    return int(exitCode);
}
