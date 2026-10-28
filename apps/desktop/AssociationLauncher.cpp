#include <Windows.h>
#include <shellapi.h>

// The format-specific ProgIDs keep their own icons but delegate opening to the
// stable legacy ProgID. This preserves the correct executable after rollback
// to an older installer that only updates xdzip.Archive.
int WINAPI wWinMain(HINSTANCE, HINSTANCE, PWSTR, int) {
    int count = 0;
    wchar_t **arguments = CommandLineToArgvW(GetCommandLineW(), &count);
    if (!arguments)
        return 1;
    if (count != 3 || lstrcmpW(arguments[1], L"--") != 0 || !arguments[2][0]) {
        LocalFree(arguments);
        return 2;
    }
    // Shell execution may load COM-based handlers; this process owns an STA.
    // A failed initialization prevents dispatch and never changes associations.
    const HRESULT initialized =
        CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE);
    if (FAILED(initialized)) {
        LocalFree(arguments);
        return 4;
    }
    SHELLEXECUTEINFOW request{};
    request.cbSize = sizeof(request);
    request.fMask = SEE_MASK_CLASSNAME | SEE_MASK_NOASYNC | SEE_MASK_FLAG_NO_UI;
    request.lpVerb = L"open";
    request.lpFile = arguments[2];
    request.lpClass = L"xdzip.Archive";
    request.nShow = SW_SHOWNORMAL;
    const bool accepted = ShellExecuteExW(&request) != FALSE;
    CoUninitialize();
    LocalFree(arguments);
    return accepted ? 0 : 3;
}
