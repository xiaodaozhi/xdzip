#include "NativeRequest.h"
#include <iostream>
#include <shlobj.h>

int wmain(int argc, wchar_t **argv) {
    if (argc != 2)
        return 2;
    const auto initialized = CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);
    if (FAILED(initialized))
        return 1;
    const auto library = LoadLibraryW(argv[1]);
    if (!library)
        return 1;
    // The built DLL is a trusted test input; the symbol has the documented COM
    // class-factory ABI. Every acquired COM reference and PIDL is released below.
    using GetClass = HRESULT(WINAPI *)(REFCLSID, REFIID, void **);
    const auto getClass = reinterpret_cast<GetClass>(GetProcAddress(library, "DllGetClassObject"));
    const CLSID id{0x731d9a38, 0x549e, 0x4cde, {0xb3, 0x5b, 0x6e, 0xb3, 0xa2, 0xc8, 0x8b, 0x71}};
    IClassFactory *factory = nullptr;
    if (!getClass || FAILED(getClass(id, IID_PPV_ARGS(&factory))))
        return 1;
    const auto marker =
        xdzip_shell::requestFile("choose", {xdzip_shell::fs::temp_directory_path()});
    if (marker.empty())
        return 1;
    const auto root = marker.parent_path();
    const auto one = root / L"中文 & one.zip", two = root / L"two.7z", text = root / L"notes.txt";
    for (const auto &path : {one, two, text}) {
        std::ofstream file(path);
        file << "fixture";
    }
    int checks = 0;
    const auto check = [&](const std::vector<xdzip_shell::fs::path> &paths, UINT expected) {
        std::vector<PIDLIST_ABSOLUTE> ids;
        for (const auto &path : paths)
            ids.push_back(ILCreateFromPathW(path.c_str()));
        std::vector<PCIDLIST_ABSOLUTE> views(ids.begin(), ids.end());
        IShellItemArray *selection = nullptr;
        IDataObject *data = nullptr;
        IContextMenu *menu = nullptr;
        IShellExtInit *init = nullptr;
        bool ok = SUCCEEDED(
            SHCreateShellItemArrayFromIDLists(UINT(ids.size()), views.data(), &selection));
        if (ok)
            ok = SUCCEEDED(selection->BindToHandler(nullptr, BHID_DataObject, IID_PPV_ARGS(&data)));
        if (ok)
            ok = SUCCEEDED(factory->CreateInstance(nullptr, IID_PPV_ARGS(&menu)));
        if (ok)
            ok = SUCCEEDED(menu->QueryInterface(IID_PPV_ARGS(&init)));
        if (ok)
            ok = SUCCEEDED(init->Initialize(nullptr, data, nullptr));
        const auto popup = CreatePopupMenu();
        if (ok) {
            const auto result = menu->QueryContextMenu(popup, 0, 1000, 2000, 0);
            ok = SUCCEEDED(result) && HRESULT_CODE(result) == expected &&
                 GetMenuItemCount(popup) == 1;
            wchar_t verb[64]{};
            ok = ok &&
                 SUCCEEDED(menu->GetCommandString(0, GCS_VERBW, nullptr,
                                                  reinterpret_cast<LPSTR>(verb), 64)) &&
                 std::wstring(verb) == L"xdzip.zip";
        }
        DestroyMenu(popup);
        if (init)
            init->Release();
        if (menu)
            menu->Release();
        if (data)
            data->Release();
        if (selection)
            selection->Release();
        for (auto *pidl : ids)
            CoTaskMemFree(pidl);
        ++checks;
        return ok;
    };
    const bool passed =
        check({text}, 3) && check({one}, 7) && check({one, two}, 8) && check({one, text}, 4);
    for (const auto &path : {one, two, text, marker})
        xdzip_shell::fs::remove(path);
    xdzip_shell::fs::remove(root);
    factory->Release();
    FreeLibrary(library);
    CoUninitialize();
    if (passed)
        std::cout << "Windows native COM menu: " << checks << " cases passed\n";
    return passed ? 0 : 1;
}
