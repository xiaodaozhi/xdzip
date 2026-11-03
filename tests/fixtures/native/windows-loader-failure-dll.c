#include <windows.h>

__declspec(dllexport) int owned_dependency(void) {
    return 42;
}

BOOL APIENTRY DllMain(HMODULE module, DWORD reason, LPVOID reserved) {
    (void)module;
    (void)reserved;
    return reason == DLL_PROCESS_ATTACH ? FALSE : TRUE;
}
