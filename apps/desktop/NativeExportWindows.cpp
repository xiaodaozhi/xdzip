// apps/desktop/NativeExportWindows.cpp

#include "NativeExport.h"
#include <QCoreApplication>
#include <QDir>
#include <QThread>
#include <QMap>
#define NOMINMAX
#include <cstring>
#include <shlobj.h>
#include <shlwapi.h>
#include <vector>
#include <windows.h>

namespace {
class ExportStream final : public IStream {
    std::atomic<ULONG> refs{1};
    IStream *inner;
    IUnknown *marshaller = nullptr;
    std::shared_ptr<NativeExportState> state;

  public:
    ExportStream(IStream *stream, std::shared_ptr<NativeExportState> value)
        : inner(stream), state(std::move(value)) {
        CoCreateFreeThreadedMarshaler(static_cast<IStream *>(this), &marshaller);
    }
    ~ExportStream() {
        if (marshaller)
            marshaller->Release();
        inner->Release();
    }
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID iid, void **out) override {
        if (!out)
            return E_POINTER;
        *out = nullptr;
        if (iid == IID_IUnknown || iid == IID_IStream || iid == IID_ISequentialStream)
            *out = static_cast<IStream *>(this);
        else if (iid == IID_IMarshal && marshaller)
            return marshaller->QueryInterface(iid, out);
        else
            return E_NOINTERFACE;
        AddRef();
        return S_OK;
    }
    ULONG STDMETHODCALLTYPE AddRef() override {
        return ++refs;
    }
    ULONG STDMETHODCALLTYPE Release() override {
        const auto left = --refs;
        if (!left)
            delete this;
        return left;
    }
    HRESULT STDMETHODCALLTYPE Read(void *bytes, ULONG count, ULONG *read) override {
        if (state->cancelled.load()) {
            if (read)
                *read = 0;
            return E_ABORT;
        }
        return inner->Read(bytes, count, read);
    }
    HRESULT STDMETHODCALLTYPE Write(const void *, ULONG, ULONG *) override {
        return STG_E_ACCESSDENIED;
    }
    HRESULT STDMETHODCALLTYPE Seek(LARGE_INTEGER move, DWORD origin,
                                   ULARGE_INTEGER *position) override {
        return state->cancelled.load() ? E_ABORT : inner->Seek(move, origin, position);
    }
    HRESULT STDMETHODCALLTYPE SetSize(ULARGE_INTEGER) override {
        return STG_E_ACCESSDENIED;
    }
    HRESULT STDMETHODCALLTYPE CopyTo(IStream *target, ULARGE_INTEGER requested,
                                     ULARGE_INTEGER *read, ULARGE_INTEGER *written) override {
        if (!target)
            return E_POINTER;
        if (read)
            read->QuadPart = 0;
        if (written)
            written->QuadPart = 0;
        char buffer[65536];
        ULONGLONG left = requested.QuadPart;
        while (left) {
            ULONG got = 0;
            const auto result =
                Read(buffer, ULONG(std::min<ULONGLONG>(left, sizeof(buffer))), &got);
            if (FAILED(result))
                return result;
            if (!got)
                return S_FALSE;
            ULONG saved = 0;
            const auto writeResult = target->Write(buffer, got, &saved);
            if (read)
                read->QuadPart += got;
            if (written)
                written->QuadPart += saved;
            if (FAILED(writeResult))
                return writeResult;
            if (saved != got)
                return STG_E_MEDIUMFULL;
            left -= got;
        }
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE Commit(DWORD) override {
        return STG_E_ACCESSDENIED;
    }
    HRESULT STDMETHODCALLTYPE Revert() override {
        return STG_E_ACCESSDENIED;
    }
    HRESULT STDMETHODCALLTYPE LockRegion(ULARGE_INTEGER, ULARGE_INTEGER, DWORD) override {
        return STG_E_INVALIDFUNCTION;
    }
    HRESULT STDMETHODCALLTYPE UnlockRegion(ULARGE_INTEGER, ULARGE_INTEGER, DWORD) override {
        return STG_E_INVALIDFUNCTION;
    }
    HRESULT STDMETHODCALLTYPE Stat(STATSTG *stat, DWORD flags) override {
        return inner->Stat(stat, flags);
    }
    HRESULT STDMETHODCALLTYPE Clone(IStream **out) override {
        if (!out)
            return E_POINTER;
        IStream *copy = nullptr;
        const auto result = inner->Clone(&copy);
        if (FAILED(result))
            return result;
        *out = new ExportStream(copy, state);
        return S_OK;
    }
};
class ExportData final : public IDataObject, public IDataObjectAsyncCapability {
    std::atomic<ULONG> refs{1};
    std::shared_ptr<NativeExportState> state;
    IUnknown *marshaller = nullptr;
    bool asyncMode = true;
    std::mutex feedbackMutex;
    QMap<CLIPFORMAT, QByteArray> feedbackFormats;
    IStream *dragContext = nullptr;
    UINT dragContextFormat = RegisterClipboardFormatW(L"DragContext");
    UINT descriptors = RegisterClipboardFormatW(CFSTR_FILEDESCRIPTORW);
    UINT contents = RegisterClipboardFormatW(CFSTR_FILECONTENTS);
    UINT preferred = RegisterClipboardFormatW(CFSTR_PREFERREDDROPEFFECT);

  public:
    explicit ExportData(std::shared_ptr<NativeExportState> value) : state(std::move(value)) {
        // COM owns the marshaler's reference. Aggregation delegates identity to
        // this IDataObject; it is released before the final object destruction.
        CoCreateFreeThreadedMarshaler(static_cast<IDataObject *>(this), &marshaller);
    }
    ~ExportData() {
        if (dragContext)
            dragContext->Release();
        if (marshaller)
            marshaller->Release();
    }
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID iid, void **out) override {
        if (!out)
            return E_POINTER;
        *out = nullptr;
        if (iid == IID_IUnknown || iid == IID_IDataObject)
            *out = static_cast<IDataObject *>(this);
        else if (iid == IID_IDataObjectAsyncCapability)
            *out = static_cast<IDataObjectAsyncCapability *>(this);
        else if (iid == IID_IMarshal && marshaller)
            return marshaller->QueryInterface(iid, out);
        else
            return E_NOINTERFACE;
        AddRef();
        return S_OK;
    }
    ULONG STDMETHODCALLTYPE AddRef() override {
        return ++refs;
    }
    ULONG STDMETHODCALLTYPE Release() override {
        const auto left = --refs;
        if (!left)
            delete this;
        return left;
    }
    HRESULT STDMETHODCALLTYPE GetData(FORMATETC *format, STGMEDIUM *medium) override {
        if (!format || !medium)
            return E_POINTER;
        std::memset(medium, 0, sizeof(*medium));
        if (format->dwAspect != DVASPECT_CONTENT)
            return DV_E_DVASPECT;
        if (format->cfFormat == dragContextFormat && (format->tymed & TYMED_ISTREAM) &&
            format->lindex == -1) {
            std::lock_guard<std::mutex> lock(feedbackMutex);
            if (!dragContext)
                return DV_E_FORMATETC;
            dragContext->AddRef();
            medium->tymed = TYMED_ISTREAM;
            medium->pstm = dragContext;
            return S_OK;
        }
        if ((format->tymed & TYMED_HGLOBAL) && format->lindex == -1) {
            QByteArray bytes;
            {
                std::lock_guard<std::mutex> lock(feedbackMutex);
                bytes = feedbackFormats.value(format->cfFormat);
            }
            if (!bytes.isEmpty()) {
                const auto block = GlobalAlloc(GMEM_MOVEABLE, SIZE_T(bytes.size()));
                if (!block)
                    return E_OUTOFMEMORY;
                auto *destination = GlobalLock(block);
                if (!destination) {
                    GlobalFree(block);
                    return E_OUTOFMEMORY;
                }
                std::memcpy(destination, bytes.constData(), size_t(bytes.size()));
                GlobalUnlock(block);
                medium->tymed = TYMED_HGLOBAL;
                medium->hGlobal = block;
                return S_OK;
            }
        }
        if (format->cfFormat == descriptors && (format->tymed & TYMED_HGLOBAL)) {
            const auto count = state->entries.size();
            const auto bytes = sizeof(UINT) + size_t(count) * sizeof(FILEDESCRIPTORW);
            HGLOBAL block = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, bytes);
            if (!block)
                return E_OUTOFMEMORY;
            auto *group = static_cast<FILEGROUPDESCRIPTORW *>(GlobalLock(block));
            if (!group) {
                GlobalFree(block);
                return E_OUTOFMEMORY;
            }
            group->cItems = UINT(count);
            for (qsizetype i = 0; i < count; ++i) {
                const auto &item = state->entries[i];
                auto &d = group->fgd[i];
                d.dwFlags = DWORD(FD_ATTRIBUTES) | DWORD(FD_FILESIZE) | DWORD(FD_UNICODE);
                d.dwFileAttributes =
                    item.directory ? FILE_ATTRIBUTE_DIRECTORY : FILE_ATTRIBUTE_NORMAL;
                d.nFileSizeHigh = DWORD(item.size >> 32);
                d.nFileSizeLow = DWORD(item.size);
                const auto name = QDir::toNativeSeparators(item.name);
                if (name.size() >= MAX_PATH) {
                    GlobalUnlock(block);
                    GlobalFree(block);
                    return DV_E_FORMATETC;
                }
                std::memcpy(d.cFileName, name.utf16(), size_t(name.size()) * sizeof(wchar_t));
            }
            GlobalUnlock(block);
            medium->tymed = TYMED_HGLOBAL;
            medium->hGlobal = block;
            return S_OK;
        }
        if (format->cfFormat == preferred && (format->tymed & TYMED_HGLOBAL)) {
            HGLOBAL block = GlobalAlloc(GMEM_MOVEABLE, sizeof(DWORD));
            if (!block)
                return E_OUTOFMEMORY;
            auto *effect = static_cast<DWORD *>(GlobalLock(block));
            if (!effect) {
                GlobalFree(block);
                return E_OUTOFMEMORY;
            }
            *effect = DROPEFFECT_COPY;
            GlobalUnlock(block);
            medium->tymed = TYMED_HGLOBAL;
            medium->hGlobal = block;
            return S_OK;
        }
        if (format->cfFormat == contents && (format->tymed & TYMED_ISTREAM)) {
            const auto index = format->lindex;
            if (index < 0 || index >= state->entries.size() || state->entries[index].directory)
                return DV_E_LINDEX;
            if (!state->inOperation.load())
                return E_PENDING;
            state->request();
            // Explorer's asynchronous extraction runs on its background thread.
            // An unsupported synchronous target receives a failure, never a GUI wait.
            if (!state->wait())
                return state->cancelled.load() ? E_ABORT : E_PENDING;
            QString path;
            {
                std::lock_guard<std::mutex> lock(state->mutex);
                path = QDir(state->directory).filePath(state->entries[index].name);
            }
            IStream *stream = nullptr;
            const auto hr = SHCreateStreamOnFileEx(reinterpret_cast<LPCWSTR>(path.utf16()),
                                                   STGM_READ | STGM_SHARE_DENY_WRITE,
                                                   FILE_ATTRIBUTE_NORMAL, FALSE, nullptr, &stream);
            if (FAILED(hr))
                return hr;
            medium->tymed = TYMED_ISTREAM;
            medium->pstm = new ExportStream(stream, state);
            return S_OK;
        }
        return DV_E_FORMATETC;
    }
    HRESULT STDMETHODCALLTYPE GetDataHere(FORMATETC *, STGMEDIUM *) override {
        return E_NOTIMPL;
    }
    HRESULT STDMETHODCALLTYPE QueryGetData(FORMATETC *f) override {
        if (!f)
            return E_POINTER;
        if (f->dwAspect != DVASPECT_CONTENT)
            return DV_E_DVASPECT;
        if (f->cfFormat == dragContextFormat && (f->tymed & TYMED_ISTREAM) && f->lindex == -1) {
            std::lock_guard<std::mutex> lock(feedbackMutex);
            return dragContext ? S_OK : DV_E_FORMATETC;
        }
        if ((f->tymed & TYMED_HGLOBAL) && f->lindex == -1) {
            std::lock_guard<std::mutex> lock(feedbackMutex);
            if (feedbackFormats.contains(f->cfFormat))
                return S_OK;
        }
        if ((f->cfFormat == descriptors || f->cfFormat == preferred) && (f->tymed & TYMED_HGLOBAL))
            return S_OK;
        if (f->cfFormat == contents && (f->tymed & TYMED_ISTREAM))
            return S_OK;
        return DV_E_FORMATETC;
    }
    HRESULT STDMETHODCALLTYPE GetCanonicalFormatEtc(FORMATETC *, FORMATETC *out) override {
        if (out)
            out->ptd = nullptr;
        return E_NOTIMPL;
    }
    HRESULT STDMETHODCALLTYPE SetData(FORMATETC *format, STGMEDIUM *medium, BOOL release) override {
        if (!format || !medium)
            return E_POINTER;
        // Shell's DragContext is a live COM context, not archive data. Retain
        // only this named stream without reading it; balance both ownership modes.
        if (format->cfFormat == dragContextFormat && format->dwAspect == DVASPECT_CONTENT &&
            format->lindex == -1 && (format->tymed & TYMED_ISTREAM) &&
            medium->tymed == TYMED_ISTREAM && medium->pstm) {
            medium->pstm->AddRef();
            IStream *previous = nullptr;
            {
                std::lock_guard<std::mutex> lock(feedbackMutex);
                previous = dragContext;
                dragContext = medium->pstm;
            }
            if (previous)
                previous->Release();
            if (release)
                ReleaseStgMedium(medium);
            return S_OK;
        }
        if (format->dwAspect != DVASPECT_CONTENT || format->lindex != -1 ||
            !(format->tymed & TYMED_HGLOBAL) || medium->tymed != TYMED_HGLOBAL ||
            format->cfFormat < 0xc000 || format->cfFormat == descriptors ||
            format->cfFormat == contents || format->cfFormat == preferred)
            return DV_E_FORMATETC;
        // Shell drag images use private HGLOBAL formats. Copy bounded bytes,
        // never accept file streams here, and take ownership only on success.
        const auto size = GlobalSize(medium->hGlobal);
        if (!size || size > 16 * 1024 * 1024)
            return STG_E_MEDIUMFULL;
        const auto *source = GlobalLock(medium->hGlobal);
        if (!source)
            return E_OUTOFMEMORY;
        const QByteArray bytes(static_cast<const char *>(source), qsizetype(size));
        GlobalUnlock(medium->hGlobal);
        {
            std::lock_guard<std::mutex> lock(feedbackMutex);
            if (!feedbackFormats.contains(format->cfFormat) && feedbackFormats.size() >= 16)
                return STG_E_MEDIUMFULL;
            feedbackFormats.insert(format->cfFormat, bytes);
        }
        if (release && medium)
            ReleaseStgMedium(medium);
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE EnumFormatEtc(DWORD direction, IEnumFORMATETC **out) override {
        if (direction != DATADIR_GET)
            return E_NOTIMPL;
        std::vector<FORMATETC> formats = {
            {CLIPFORMAT(descriptors), nullptr, DVASPECT_CONTENT, -1, TYMED_HGLOBAL},
            {CLIPFORMAT(contents), nullptr, DVASPECT_CONTENT, -1, TYMED_ISTREAM},
            {CLIPFORMAT(preferred), nullptr, DVASPECT_CONTENT, -1, TYMED_HGLOBAL}};
        {
            std::lock_guard<std::mutex> lock(feedbackMutex);
            for (auto it = feedbackFormats.cbegin(); it != feedbackFormats.cend(); ++it)
                formats.push_back({it.key(), nullptr, DVASPECT_CONTENT, -1, TYMED_HGLOBAL});
            if (dragContext)
                formats.push_back({CLIPFORMAT(dragContextFormat), nullptr, DVASPECT_CONTENT, -1,
                                   TYMED_ISTREAM});
        }
        return SHCreateStdEnumFmtEtc(UINT(formats.size()), formats.data(), out);
    }
    HRESULT STDMETHODCALLTYPE DAdvise(FORMATETC *, DWORD, IAdviseSink *, DWORD *) override {
        return OLE_E_ADVISENOTSUPPORTED;
    }
    HRESULT STDMETHODCALLTYPE DUnadvise(DWORD) override {
        return OLE_E_ADVISENOTSUPPORTED;
    }
    HRESULT STDMETHODCALLTYPE EnumDAdvise(IEnumSTATDATA **) override {
        return OLE_E_ADVISENOTSUPPORTED;
    }
    HRESULT STDMETHODCALLTYPE SetAsyncMode(BOOL mode) override {
        asyncMode = mode;
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE GetAsyncMode(BOOL *mode) override {
        if (!mode)
            return E_POINTER;
        *mode = asyncMode;
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE StartOperation(IBindCtx *) override {
        state->inOperation = true;
        state->request();
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE InOperation(BOOL *operation) override {
        if (!operation)
            return E_POINTER;
        *operation = state->inOperation.load();
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE EndOperation(HRESULT result, IBindCtx *, DWORD) override {
        state->inOperation = false;
        if (FAILED(result)) {
            state->cancelled = true;
            state->changed.notify_all();
        }
        return S_OK;
    }
};
class ExportSource final : public IDropSource {
    std::atomic<ULONG> refs{1};
    std::shared_ptr<NativeExportState> state;

  public:
    explicit ExportSource(std::shared_ptr<NativeExportState> value) : state(std::move(value)) {}
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID iid, void **out) override {
        if (!out)
            return E_POINTER;
        *out = nullptr;
        if (iid != IID_IUnknown && iid != IID_IDropSource)
            return E_NOINTERFACE;
        *out = static_cast<IDropSource *>(this);
        AddRef();
        return S_OK;
    }
    ULONG STDMETHODCALLTYPE AddRef() override {
        return ++refs;
    }
    ULONG STDMETHODCALLTYPE Release() override {
        const auto left = --refs;
        if (!left)
            delete this;
        return left;
    }
    HRESULT STDMETHODCALLTYPE QueryContinueDrag(BOOL escape, DWORD keys) override {
        return escape || state->cancelled.load() ? DRAGDROP_S_CANCEL
               : !(keys & MK_LBUTTON)            ? DRAGDROP_S_DROP
                                                 : S_OK;
    }
    HRESULT STDMETHODCALLTYPE GiveFeedback(DWORD) override {
        return DRAGDROP_S_USEDEFAULTCURSORS;
    }
};
} // namespace
IDataObject *nativeExportData(const std::shared_ptr<NativeExportState> &state) {
    return new ExportData(state);
}
bool initializeNativeExportImage(IDataObject *data, const QImage &preview, QString *failure) {
    if (!data || preview.isNull())
        return false;
    IDragSourceHelper *helper = nullptr;
    const auto created = CoCreateInstance(CLSID_DragDropHelper, nullptr, CLSCTX_INPROC_SERVER,
                                          IID_PPV_ARGS(&helper));
    if (FAILED(created)) {
        if (failure) *failure = "CoCreateInstance: " + QString::number(quint32(created), 16);
        return false;
    }
    // The Shell multiplies RGB by alpha itself, so provide straight BGRA.
    const auto image = preview.convertToFormat(QImage::Format_ARGB32);
    BITMAPINFO info{};
    info.bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
    info.bmiHeader.biWidth = image.width();
    info.bmiHeader.biHeight = -image.height();
    info.bmiHeader.biPlanes = 1;
    info.bmiHeader.biBitCount = 32;
    info.bmiHeader.biCompression = BI_RGB;
    void *bits = nullptr;
    const auto bitmap = CreateDIBSection(nullptr, &info, DIB_RGB_COLORS, &bits, nullptr, 0);
    if (!bitmap || !bits) {
        if (bitmap)
            DeleteObject(bitmap);
        helper->Release();
        return false;
    }
    std::memcpy(bits, image.constBits(), size_t(image.sizeInBytes()));
    SHDRAGIMAGE dragImage{{image.width(), image.height()}, {0, 0}, bitmap, 0xffffffff};
    // InitializeFromBitmap takes the bitmap. If image setup fails, the data
    // transfer continues with the OS cursor and retains existing copy semantics.
    const auto result = helper->InitializeFromBitmap(&dragImage, data);
    if (failure && FAILED(result))
        *failure = "InitializeFromBitmap: " + QString::number(quint32(result), 16);
    helper->Release();
    return SUCCEEDED(result);
}
bool startNativeExport(QWindow *owner, const std::shared_ptr<NativeExportState> &state) {
    for (const auto &entry : state->entries)
        if (entry.name.size() >= MAX_PATH)
            return false;
    auto *data = nativeExportData(state);
    initializeNativeExportImage(data, nativeExportPreview(state->entries,
                                                         owner ? owner->devicePixelRatio() : 1));
    auto *source = new ExportSource(state);
    DWORD effect = 0;
    // Qt initializes OLE on the GUI STA. DoDragDrop pumps its native message loop;
    // payload extraction remains worker-owned through IDataObjectAsyncCapability.
    const auto result = DoDragDrop(data, source, DROPEFFECT_COPY, &effect);
    source->Release();
    data->Release();
    if (result != DRAGDROP_S_DROP && !state->inOperation.load()) {
        state->cancelled = true;
        state->changed.notify_all();
    }
    return result == DRAGDROP_S_DROP && effect == DROPEFFECT_COPY;
}
