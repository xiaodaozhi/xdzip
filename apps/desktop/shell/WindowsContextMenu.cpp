#include "MenuModel.h"
#include <atomic>
#include <new>
#include <shlobj.h>

namespace {
// Stable identity belongs only to xdzip; registration is per current user.
const CLSID ClassId{0x731d9a38, 0x549e, 0x4cde, {0xb3, 0x5b, 0x6e, 0xb3, 0xa2, 0xc8, 0x8b, 0x71}};
std::atomic<long> objects{0};
HMODULE module = nullptr;
std::wstring wide(const std::string &text) {
    const auto size = MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, text.data(),
                                          int(text.size()), nullptr, 0);
    std::wstring result(size, L'\0');
    if (size)
        MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, text.data(), int(text.size()),
                            result.data(), size);
    return result;
}
bool english() {
    wchar_t language[16]{};
    DWORD bytes = sizeof(language);
    return RegGetValueW(HKEY_CURRENT_USER, L"Software\\xdzip\\Shell", L"Language", RRF_RT_REG_SZ,
                        nullptr, language, &bytes) == ERROR_SUCCESS &&
           std::wstring(language) == L"en-US";
}
class Menu final : public IShellExtInit, public IContextMenu {
    std::atomic<ULONG> refs{1};
    std::vector<xdzip_shell::fs::path> paths;
    std::vector<xdzip_shell::MenuEntry> entries;

  public:
    Menu() {
        ++objects;
    }
    ~Menu() {
        --objects;
    }
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID id, void **value) override {
        if (!value)
            return E_POINTER;
        *value = nullptr;
        if (id == IID_IUnknown || id == IID_IShellExtInit)
            *value = static_cast<IShellExtInit *>(this);
        else if (id == IID_IContextMenu)
            *value = static_cast<IContextMenu *>(this);
        if (!*value)
            return E_NOINTERFACE;
        AddRef();
        return S_OK;
    }
    ULONG STDMETHODCALLTYPE AddRef() override {
        return ++refs;
    }
    ULONG STDMETHODCALLTYPE Release() override {
        const auto n = --refs;
        if (!n)
            delete this;
        return n;
    }
    HRESULT STDMETHODCALLTYPE Initialize(PCIDLIST_ABSOLUTE, IDataObject *data, HKEY) override {
        paths.clear();
        if (!data)
            return E_INVALIDARG;
        FORMATETC format{CF_HDROP, nullptr, DVASPECT_CONTENT, -1, TYMED_HGLOBAL};
        STGMEDIUM medium{};
        if (FAILED(data->GetData(&format, &medium)))
            return E_INVALIDARG;
        // IDataObject owns its source. ReleaseStgMedium releases our returned
        // reference after bounded Unicode names have been copied.
        const auto drop = static_cast<HDROP>(medium.hGlobal);
        const auto count = DragQueryFileW(drop, 0xffffffff, nullptr, 0);
        if (count && count <= 128)
            for (UINT i = 0; i < count; ++i) {
                const auto size = DragQueryFileW(drop, i, nullptr, 0);
                if (!size || size > 4096) {
                    paths.clear();
                    break;
                }
                std::wstring path(size + 1, L'\0');
                DragQueryFileW(drop, i, path.data(), size + 1);
                path.resize(size);
                paths.emplace_back(path);
            }
        ReleaseStgMedium(&medium);
        return paths.empty() ? E_INVALIDARG : S_OK;
    }
    HRESULT STDMETHODCALLTYPE QueryContextMenu(HMENU parent, UINT index, UINT first, UINT last,
                                               UINT flags) override {
        if (flags & CMF_DEFAULTONLY)
            return MAKE_HRESULT(SEVERITY_SUCCESS, 0, 0);
        try {
            DWORD enabled = 1, size = sizeof(enabled);
            RegGetValueW(HKEY_CURRENT_USER, L"Software\\xdzip\\Shell", L"Enabled", RRF_RT_REG_DWORD,
                         nullptr, &enabled, &size);
            if (!enabled)
                return MAKE_HRESULT(SEVERITY_SUCCESS, 0, 0);
            entries = xdzip_shell::menu(paths, english());
            if (entries.empty() || last < first || last - first + 1 < entries.size())
                return E_FAIL;
            const auto submenu = CreatePopupMenu();
            if (!submenu)
                return E_OUTOFMEMORY;
            for (size_t i = 0; i < entries.size(); ++i) {
                auto label = wide(entries[i].label);
                for (size_t at = 0; (at = label.find(L'&', at)) != std::wstring::npos; at += 2)
                    label.insert(at, 1, L'&');
                AppendMenuW(submenu, MF_STRING, first + UINT(i), label.c_str());
            }
            MENUITEMINFOW item{};
            item.cbSize = sizeof(item);
            item.fMask = MIIM_STRING | MIIM_SUBMENU;
            item.hSubMenu = submenu;
            auto title = wide(xdzip_shell::text(english(), "shell.title"));
            item.dwTypeData = title.data();
            if (!InsertMenuItemW(parent, index, TRUE, &item)) {
                DestroyMenu(submenu);
                return E_FAIL;
            }
            return MAKE_HRESULT(SEVERITY_SUCCESS, 0, USHORT(entries.size()));
        } catch (...) {
            return E_FAIL;
        }
    }
    HRESULT STDMETHODCALLTYPE InvokeCommand(CMINVOKECOMMANDINFO *info) override {
        if (!info)
            return E_POINTER;
        try {
            size_t index = entries.size();
            if (!HIWORD(info->lpVerb))
                index = LOWORD(info->lpVerb);
            else
                for (size_t i = 0; i < entries.size(); ++i)
                    if ("xdzip." + entries[i].action == info->lpVerb)
                        index = i;
            if (index >= entries.size())
                return E_INVALIDARG;
            std::wstring path(32768, L'\0');
            const auto size = GetModuleFileNameW(module, path.data(), DWORD(path.size()));
            if (!size || size >= path.size())
                return E_FAIL;
            path.resize(size);
            auto root = xdzip_shell::fs::path(path).parent_path();
            if (root.filename() == L"shell")
                root = root.parent_path();
            return xdzip_shell::launch(root / L"xdzip.exe", entries[index].action, paths) ? S_OK
                                                                                          : E_FAIL;
        } catch (...) {
            return E_FAIL;
        }
    }
    HRESULT STDMETHODCALLTYPE GetCommandString(UINT_PTR id, UINT flags, UINT *, LPSTR buffer,
                                               UINT count) override {
        if (flags == GCS_VALIDATEA || flags == GCS_VALIDATEW)
            return id < entries.size() ? S_OK : E_INVALIDARG;
        if (id >= entries.size() || !buffer || !count)
            return E_INVALIDARG;
        const auto verb = "xdzip." + entries[id].action;
        if (flags == GCS_VERBW) {
            const auto value = wide(verb);
            if (count <= value.size())
                return E_INVALIDARG;
            std::copy(value.begin(), value.end(), reinterpret_cast<wchar_t *>(buffer));
            reinterpret_cast<wchar_t *>(buffer)[value.size()] = 0;
        } else if (flags == GCS_VERBA) {
            if (count <= verb.size())
                return E_INVALIDARG;
            std::copy(verb.begin(), verb.end(), buffer);
            buffer[verb.size()] = 0;
        } else
            return E_NOTIMPL;
        return S_OK;
    }
};
class Factory final : public IClassFactory {
    std::atomic<ULONG> refs{1};

  public:
    Factory() {
        ++objects;
    }
    ~Factory() {
        --objects;
    }
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID id, void **value) override {
        if (!value)
            return E_POINTER;
        *value = nullptr;
        if (id != IID_IUnknown && id != IID_IClassFactory)
            return E_NOINTERFACE;
        *value = static_cast<IClassFactory *>(this);
        AddRef();
        return S_OK;
    }
    ULONG STDMETHODCALLTYPE AddRef() override {
        return ++refs;
    }
    ULONG STDMETHODCALLTYPE Release() override {
        const auto n = --refs;
        if (!n)
            delete this;
        return n;
    }
    HRESULT STDMETHODCALLTYPE CreateInstance(IUnknown *outer, REFIID id, void **value) override {
        if (outer)
            return CLASS_E_NOAGGREGATION;
        auto *object = new (std::nothrow) Menu;
        if (!object)
            return E_OUTOFMEMORY;
        const auto result = object->QueryInterface(id, value);
        object->Release();
        return result;
    }
    HRESULT STDMETHODCALLTYPE LockServer(BOOL lock) override {
        objects += lock ? 1 : -1;
        return S_OK;
    }
};
} // namespace
extern "C" BOOL WINAPI DllMain(HINSTANCE instance, DWORD reason, LPVOID) {
    if (reason == DLL_PROCESS_ATTACH) {
        module = instance;
        DisableThreadLibraryCalls(instance);
    }
    return TRUE;
}
extern "C" __declspec(dllexport) HRESULT WINAPI DllGetClassObject(REFCLSID id, REFIID iid,
                                                                  void **value) {
    if (id != ClassId)
        return CLASS_E_CLASSNOTAVAILABLE;
    auto *factory = new (std::nothrow) Factory;
    if (!factory)
        return E_OUTOFMEMORY;
    const auto result = factory->QueryInterface(iid, value);
    factory->Release();
    return result;
}
extern "C" __declspec(dllexport) HRESULT WINAPI DllCanUnloadNow() {
    return objects == 0 ? S_OK : S_FALSE;
}
