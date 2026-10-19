#include "WindowsFrame.h"
#include <QCoreApplication>
#include <QGuiApplication>
#include <QPlatformSurfaceEvent>
#include <QTimer>

#ifdef Q_OS_WIN
#include <dwmapi.h>
#include <qpa/qplatformwindow_p.h>
#include <qt_windows.h>
#include <windowsx.h>

namespace {
// Stable DWM ABI values keep this buildable with the older Windows 10 SDK.
constexpr DWORD cornerPreferenceAttribute = 33;
constexpr DWORD visibleBorderAttribute = 37;
constexpr DWORD darkModeAttribute = 20;
constexpr int roundCorners = 2;
int resizeBorder(HWND hwnd, int metric) {
    const UINT dpi = GetDpiForWindow(hwnd);
    return GetSystemMetricsForDpi(metric, dpi) + GetSystemMetricsForDpi(SM_CXPADDEDBORDER, dpi);
}
} // namespace
#endif

WindowsFrame::WindowsFrame(QObject *parent) : QObject(parent) {
    QCoreApplication::instance()->installNativeEventFilter(this);
}
WindowsFrame::~WindowsFrame() {
    QCoreApplication::instance()->removeNativeEventFilter(this);
    if (m_window)
        m_window->removeEventFilter(this);
}
void WindowsFrame::setWindow(QWindow *window) {
    if (m_window == window)
        return;
    if (m_window)
        m_window->removeEventFilter(this);
    m_window = window;
    m_id = 0;
    if (m_window) {
        m_window->installEventFilter(this);
        if (m_window->handle())
            apply();
    }
    emit windowChanged();
}
void WindowsFrame::setDark(bool dark) {
    if (m_dark == dark)
        return;
    m_dark = dark;
    if (m_id)
        apply();
    emit darkChanged();
}
void WindowsFrame::apply() {
#ifdef Q_OS_WIN
    if (!m_window || QGuiApplication::platformName() != QStringLiteral("windows"))
        return;
    // The QWindow owns this HWND. Never destroy or cache it past SurfaceAboutToBeDestroyed.
    m_id = m_window->winId();
    const auto hwnd = reinterpret_cast<HWND>(m_id);
    // Keep Qt's client/frame geometry model consistent with NCCALCSIZE. This
    // private native interface is isolated to the pinned Qt Windows adapter.
    RECT frame{0, 0, 0, 0};
    auto *native = m_window->nativeInterface<QNativeInterface::Private::QWindowsWindow>();
    if (!native || !AdjustWindowRectExForDpi(&frame, DWORD(GetWindowLongPtrW(hwnd, GWL_STYLE)),
                                             FALSE, DWORD(GetWindowLongPtrW(hwnd, GWL_EXSTYLE)),
                                             GetDpiForWindow(hwnd))) {
        // Keep the standard system frame if the native adapter cannot bind.
        m_id = 0;
        return;
    }
    native->setCustomMargins(QMargins(0, int(frame.top) + 1, 0, 0));
    const auto policy = DWMNCRP_ENABLED;
    DwmSetWindowAttribute(hwnd, DWMWA_NCRENDERING_POLICY, &policy, sizeof(policy));
    const BOOL dark = m_dark;
    DwmSetWindowAttribute(hwnd, darkModeAttribute, &dark, sizeof(dark));
    // Unsupported attributes (for example corners on Windows 10) leave the OS default.
    DwmSetWindowAttribute(hwnd, cornerPreferenceAttribute, &roundCorners, sizeof(roundCorners));
    // DWM margins are native device pixels, never QML component dimensions.
    const MARGINS margins{1, 1, 1, 1};
    DwmExtendFrameIntoClientArea(hwnd, &margins);
    SetWindowPos(hwnd, nullptr, 0, 0, 0, 0,
                 SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED);
#endif
}
bool WindowsFrame::eventFilter(QObject *watched, QEvent *event) {
    if (watched == m_window && event->type() == QEvent::PlatformSurface) {
        const auto *surface = static_cast<QPlatformSurfaceEvent *>(event);
        if (surface->surfaceEventType() == QPlatformSurfaceEvent::SurfaceCreated)
            apply();
        else
            m_id = 0;
    }
    return false;
}
bool WindowsFrame::nativeEventFilter(const QByteArray &type, void *message, qintptr *result) {
    Q_UNUSED(type)
#ifdef Q_OS_WIN
    // Qt supplies a borrowed MSG for this callback. NCCALCSIZE lParam is writable
    // only during the corresponding synchronous native call; retain no pointers.
    const auto *msg = static_cast<MSG *>(message);
    if (!m_id || msg->hwnd != reinterpret_cast<HWND>(m_id))
        return false;
    const auto hwnd = msg->hwnd;
    if (msg->message == WM_NCCALCSIZE && msg->wParam) {
        auto *params = reinterpret_cast<NCCALCSIZE_PARAMS *>(msg->lParam);
        const auto outer = params->rgrc[0];
        DefWindowProcW(hwnd, msg->message, msg->wParam, msg->lParam);
        // Keep native side/bottom resize borders and a DWM top edge. At maximum
        // size Windows hides the resize band outside the monitor work area.
        params->rgrc[0].top = outer.top + (IsZoomed(hwnd) ? resizeBorder(hwnd, SM_CYSIZEFRAME) : 1);
        *result = 0;
        return true;
    }
    if (msg->message == WM_NCHITTEST && !IsZoomed(hwnd)) {
        RECT bounds;
        if (!GetWindowRect(hwnd, &bounds))
            return false;
        const POINT point{GET_X_LPARAM(msg->lParam), GET_Y_LPARAM(msg->lParam)};
        const int x = resizeBorder(hwnd, SM_CXSIZEFRAME), y = resizeBorder(hwnd, SM_CYSIZEFRAME);
        const bool left = point.x < bounds.left + x, right = point.x >= bounds.right - x;
        const bool top = point.y < bounds.top + y, bottom = point.y >= bounds.bottom - y;
        if (top || bottom || left || right) {
            *result = top      ? (left    ? HTTOPLEFT
                                  : right ? HTTOPRIGHT
                                          : HTTOP)
                      : bottom ? (left    ? HTBOTTOMLEFT
                                  : right ? HTBOTTOMRIGHT
                                          : HTBOTTOM)
                      : left   ? HTLEFT
                               : HTRIGHT;
            return true;
        }
    }
    if (msg->message == WM_DWMCOMPOSITIONCHANGED || msg->message == WM_THEMECHANGED ||
        msg->message == WM_DPICHANGED)
        QTimer::singleShot(0, this, [this] { apply(); });
#else
    Q_UNUSED(message)
    Q_UNUSED(result)
#endif
    return false;
}
QVariantMap WindowsFrame::diagnostics() const {
    QVariantMap data{{QStringLiteral("attached"), m_id != 0}};
#ifdef Q_OS_WIN
    if (m_id) {
        const auto hwnd = reinterpret_cast<HWND>(m_id);
        const auto style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        BOOL composition = FALSE, rendering = FALSE;
        int corners = 0;
        UINT border = 0;
        DwmIsCompositionEnabled(&composition);
        DwmGetWindowAttribute(hwnd, DWMWA_NCRENDERING_ENABLED, &rendering, sizeof(rendering));
        const auto cornerStatus =
            DwmGetWindowAttribute(hwnd, cornerPreferenceAttribute, &corners, sizeof(corners));
        DwmGetWindowAttribute(hwnd, visibleBorderAttribute, &border, sizeof(border));
        POINT origin{0, 0};
        RECT outer{};
        ClientToScreen(hwnd, &origin);
        GetWindowRect(hwnd, &outer);
        const POINT center{outer.left + (outer.right - outer.left) / 2,
                           outer.top + (outer.bottom - outer.top) / 2};
        data.insert(QStringLiteral("visibleAtCenter"), WindowFromPoint(center) == hwnd);
        RECT visible{};
        if (SUCCEEDED(DwmGetWindowAttribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS, &visible,
                                            sizeof(visible)))) {
            data.insert(QStringLiteral("frameBoundsNative"),
                        QVariantList{int(visible.left), int(visible.top),
                                     int(visible.right - visible.left),
                                     int(visible.bottom - visible.top)});
        }
        MONITORINFO monitor{sizeof(MONITORINFO), {}, {}, 0};
        if (GetMonitorInfoW(MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST), &monitor))
            data.insert(QStringLiteral("monitorOriginNative"),
                        QVariantList{int(monitor.rcMonitor.left), int(monitor.rcMonitor.top)});
        data.insert(QStringLiteral("captionStyle"), (style & WS_CAPTION) == WS_CAPTION);
        data.insert(QStringLiteral("resizeStyle"), (style & WS_THICKFRAME) != 0);
        data.insert(QStringLiteral("composition"), composition != FALSE);
        data.insert(QStringLiteral("nonClientRendering"), rendering != FALSE);
        data.insert(QStringLiteral("cornerPreferenceSupported"), SUCCEEDED(cornerStatus));
        data.insert(QStringLiteral("cornerPreference"), corners);
        data.insert(QStringLiteral("visibleBorderPixels"), border);
        data.insert(QStringLiteral("topInsetPixels"), int(origin.y - outer.top));
    }
#endif
    return data;
}
