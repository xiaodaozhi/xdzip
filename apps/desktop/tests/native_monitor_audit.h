// apps/desktop/tests/native_monitor_audit.h

#pragma once
#include <QJsonArray>
#include <QJsonObject>
#include <QQuickWindow>

#ifdef Q_OS_WIN
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <qt_windows.h>

namespace native_monitor_audit {
inline bool monitorRecord(HMONITOR monitor, QJsonObject *record) {
    MONITORINFOEXW info{};
    info.cbSize = sizeof(info);
    DEVMODEW mode{};
    mode.dmSize = sizeof(mode);
    // MONITORINFOEXW has the documented MONITORINFO prefix. These APIs only
    // read the active display; any failure rejects the record and its capture.
    if (!GetMonitorInfoW(monitor, reinterpret_cast<MONITORINFO *>(&info)) ||
        !EnumDisplaySettingsW(info.szDevice, ENUM_CURRENT_SETTINGS, &mode) || !mode.dmPelsWidth ||
        !mode.dmPelsHeight)
        return false;
    *record = QJsonObject{{"monitor_device", QString::fromWCharArray(info.szDevice)},
                          {"primary", bool(info.dwFlags & MONITORINFOF_PRIMARY)},
                          {"physical_width", int(mode.dmPelsWidth)},
                          {"physical_height", int(mode.dmPelsHeight)},
                          {"refresh_hz", int(mode.dmDisplayFrequency)}};
    return true;
}
struct Enumeration {
    QJsonArray records;
    bool valid = true;
};
inline BOOL CALLBACK enumerate(HMONITOR monitor, HDC, LPRECT, LPARAM data) {
    auto &state = *reinterpret_cast<Enumeration *>(data);
    QJsonObject record;
    state.valid = monitorRecord(monitor, &record);
    if (state.valid)
        state.records.append(record);
    return state.valid;
}
inline bool activeDisplays(QJsonArray *records) {
    Enumeration state;
    // EnumDisplayMonitors calls synchronously; the callback never retains this
    // stack context or a monitor handle after enumeration returns.
    if (!EnumDisplayMonitors(nullptr, nullptr, enumerate, reinterpret_cast<LPARAM>(&state)) ||
        !state.valid)
        return false;
    *records = state.records;
    return true;
}
inline bool snapshot(QQuickWindow *window, QJsonObject *record) {
    const auto owned = reinterpret_cast<HWND>(window->winId());
    DWORD pid = 0;
    // Read only the HWND owned by this live QQuickWindow and this process.
    // Invalid ownership, monitor/DPI or frame diagnostics rejects all evidence.
    if (!IsWindow(owned) || !GetWindowThreadProcessId(owned, &pid) ||
        pid != GetCurrentProcessId() ||
        !monitorRecord(MonitorFromWindow(owned, MONITOR_DEFAULTTONULL), record))
        return false;
    const auto dpi = GetDpiForWindow(owned);
    QVariantMap frame;
    auto *adapter = window->findChild<QObject *>("windowsNativeFrame");
    if (!dpi || !adapter ||
        !QMetaObject::invokeMethod(adapter, "diagnostics", Q_RETURN_ARG(QVariantMap, frame)))
        return false;
    record->insert("native_dpi", int(dpi));
    record->insert("qt_dpr", window->devicePixelRatio());
    record->insert("logical_width", window->width());
    record->insert("logical_height", window->height());
    record->insert("frame", QJsonObject::fromVariantMap(frame));
    return true;
}
} // namespace native_monitor_audit
#endif
