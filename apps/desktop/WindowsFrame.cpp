#if defined(_WIN32)
#ifndef NOMINMAX
#define NOMINMAX
#endif
#endif
#include "WindowsFrame.h"
#include <QCoreApplication>
#include <QGuiApplication>
#include <QPlatformSurfaceEvent>
#include <QScopedValueRollback>
#include <QTimer>
#include <cmath>
#include <limits>

#ifdef Q_OS_WIN
#include <dwmapi.h>
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
bool ordinaryCaptionWindow(HWND hwnd, const QWindow *window) {
    return window && window->windowState() == Qt::WindowNoState &&
           window->visibility() != QWindow::FullScreen && !IsZoomed(hwnd) && !IsIconic(hwnd) &&
           (GetWindowLongPtrW(hwnd, GWL_STYLE) & WS_CAPTION) == WS_CAPTION;
}
bool nativeOuterSize(HWND hwnd, UINT dpi, qint64 clientWidth, qint64 clientHeight, SIZE *size) {
    if (!dpi || clientWidth <= 0 || clientHeight <= 0 ||
        clientWidth > std::numeric_limits<int>::max() ||
        clientHeight > std::numeric_limits<int>::max() || GetMenu(hwnd))
        return false;
    SetLastError(0);
    const auto style = GetWindowLongPtrW(hwnd, GWL_STYLE);
    if (!style && GetLastError())
        return false;
    SetLastError(0);
    const auto exStyle = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
    if (!exStyle && GetLastError())
        return false;
    RECT frame{};
    if (!AdjustWindowRectExForDpi(&frame, DWORD(style), FALSE, DWORD(exStyle), dpi))
        return false;
    const qint64 left = -qint64(frame.left), right = frame.right, bottom = frame.bottom;
    if (left < 0 || right < 0 || bottom < 0)
        return false;
    const qint64 width = clientWidth + left + right;
    const qint64 height = clientHeight + 1 + bottom;
    if (width <= 0 || height <= 0 || width > std::numeric_limits<int>::max() ||
        height > std::numeric_limits<int>::max())
        return false;
    *size = SIZE{LONG(width), LONG(height)};
    return true;
}
bool compatibleNativeScaling(HWND hwnd) {
    if (!AreDpiAwarenessContextsEqual(GetWindowDpiAwarenessContext(hwnd),
                                      DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) ||
        !AreDpiAwarenessContextsEqual(GetThreadDpiAwarenessContext(),
                                      DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) ||
        QGuiApplication::highDpiScaleFactorRoundingPolicy() !=
            Qt::HighDpiScaleFactorRoundingPolicy::PassThrough)
        return false;
    for (const auto *name : {"QT_SCREEN_SCALE_FACTORS", "QT_USE_PHYSICAL_DPI", "QT_FONT_DPI",
                             "QT_DEVICE_PIXEL_RATIO", "QT_AUTO_SCREEN_SCALE_FACTOR"}) {
        if (qEnvironmentVariableIsSet(name))
            return false;
    }
    if (qEnvironmentVariableIsSet("QT_ENABLE_HIGHDPI_SCALING")) {
        bool valid = false;
        const int enabled = qEnvironmentVariableIntValue("QT_ENABLE_HIGHDPI_SCALING", &valid);
        if (!valid || enabled <= 0)
            return false;
    }
    return true;
}
bool scaledNativeExtent(qint64 extent, UINT currentDpi, UINT targetDpi, qint64 *result) {
    if (extent <= 0 || extent > std::numeric_limits<int>::max() || !currentDpi || !targetDpi)
        return false;
    // INT_MAX times UINT_MAX plus rounding fits quint64. Reject before narrowing.
    const quint64 scaled = (quint64(extent) * targetDpi + currentDpi / 2) / currentDpi;
    if (!scaled || scaled > quint64(std::numeric_limits<int>::max()))
        return false;
    *result = qint64(scaled);
    return true;
}
bool nativeSizeConstraints(HWND hwnd, const QWindow *window, const MINMAXINFO &incoming,
                           MINMAXINFO *response) {
    DWORD owner = 0;
    if (GetWindowThreadProcessId(hwnd, &owner) != GetCurrentThreadId() ||
        owner != GetCurrentProcessId() || !ordinaryCaptionWindow(hwnd, window) ||
        !compatibleNativeScaling(hwnd) || qEnvironmentVariableIsSet("QT_SCALE_FACTOR"))
        return false;
    const UINT dpi = GetDpiForWindow(hwnd);
    SIZE framedUnit{};
    if (!nativeOuterSize(hwnd, dpi, 1, 1, &framedUnit))
        return false;
    const auto minimum = window->minimumSize();
    const auto maximum = window->maximumSize();
    auto adjusted = incoming;
    const auto axis = [dpi](int axisMinimum, int axisMaximum, LONG border, LONG *minTrack,
                            LONG *maxTrack) {
        // QWindow's default maximum (2^24 - 1) means no application limit.
        constexpr int unlimited = (1 << 24) - 1;
        qint64 scaled = 0;
        if (axisMinimum < 0 || axisMinimum >= unlimited || axisMaximum < 0)
            return false;
        if (axisMinimum > 0) {
            if (!scaledNativeExtent(axisMinimum, 96, dpi, &scaled) ||
                scaled + border > std::numeric_limits<LONG>::max())
                return false;
            *minTrack = LONG(scaled + border);
        }
        if (axisMaximum < unlimited) {
            if (!scaledNativeExtent(qMax(axisMaximum, axisMinimum), 96, dpi, &scaled) ||
                scaled + border > std::numeric_limits<LONG>::max())
                return false;
            *maxTrack = LONG(scaled + border);
        }
        return true;
    };
    if (!axis(minimum.width(), maximum.width(), framedUnit.cx - 1, &adjusted.ptMinTrackSize.x,
              &adjusted.ptMaxTrackSize.x) ||
        !axis(minimum.height(), maximum.height(), framedUnit.cy - 1, &adjusted.ptMinTrackSize.y,
              &adjusted.ptMaxTrackSize.y))
        return false;
    *response = adjusted;
    return true;
}
bool dpiOuterSize(HWND hwnd, const QWindow *window, WPARAM target, const SIZE &incoming,
                  SIZE *size) {
    DWORD owner = 0;
    if (GetWindowThreadProcessId(hwnd, &owner) != GetCurrentThreadId() ||
        owner != GetCurrentProcessId() || !ordinaryCaptionWindow(hwnd, window) ||
        !compatibleNativeScaling(hwnd) || target > std::numeric_limits<UINT>::max() ||
        incoming.cx <= 0 || incoming.cy <= 0)
        return false;
    const UINT currentDpi = GetDpiForWindow(hwnd), targetDpi = UINT(target);
    RECT client{}, outer{};
    if (!GetClientRect(hwnd, &client) || !GetWindowRect(hwnd, &outer))
        return false;
    const qint64 currentOuterWidth = qint64(outer.right) - outer.left;
    const qint64 currentOuterHeight = qint64(outer.bottom) - outer.top;
    const qint64 clientWidth = qint64(client.right) - client.left;
    const qint64 clientHeight = qint64(client.bottom) - client.top;
    if (currentOuterWidth <= 0 || currentOuterHeight <= 0 || clientWidth <= 0 || clientHeight <= 0)
        return false;
    // The pending outer SIZE may contain resize intent in the old-DPI coordinates.
    const qint64 pendingWidth = clientWidth + qint64(incoming.cx) - currentOuterWidth;
    const qint64 pendingHeight = clientHeight + qint64(incoming.cy) - currentOuterHeight;
    qint64 width = 0, height = 0;
    return scaledNativeExtent(pendingWidth, currentDpi, targetDpi, &width) &&
           scaledNativeExtent(pendingHeight, currentDpi, targetDpi, &height) &&
           nativeOuterSize(hwnd, targetDpi, width, height, size);
}
bool initialOuterSize(HWND hwnd, const QSize &logicalSize, qreal dpr, SIZE *size) {
    if (!std::isfinite(dpr) || dpr <= 0 || logicalSize.width() <= 0 || logicalSize.height() <= 0)
        return false;
    const qreal width = logicalSize.width() * dpr;
    const qreal height = logicalSize.height() * dpr;
    if (!std::isfinite(width) || !std::isfinite(height) || width < .5 || height < .5 ||
        width > std::numeric_limits<int>::max() || height > std::numeric_limits<int>::max())
        return false;
    return nativeOuterSize(hwnd, GetDpiForWindow(hwnd), qRound64(width), qRound64(height), size);
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
#ifdef Q_OS_WIN
    m_frameAttachAttemptId = 0;
#endif
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
    if (m_applying || !m_window || QGuiApplication::platformName() != QStringLiteral("windows"))
        return;
    QScopedValueRollback<bool> applying(m_applying, true);
    const auto logicalSize = m_window->size();
    const qreal dpr = m_window->devicePixelRatio();
    const auto id = m_window->winId();
    const auto hwnd = reinterpret_cast<HWND>(id);
    DWORD owner = 0;
    const auto thread = GetWindowThreadProcessId(hwnd, &owner);
    if (!IsWindow(hwnd) || owner != GetCurrentProcessId() || thread != GetCurrentThreadId())
        return;
    if (m_frameAttachAttemptId == id && !m_id)
        return;
    const bool firstAttach = m_frameAttachAttemptId != id;
    m_frameAttachAttemptId = id;
    SIZE initial{};
    const bool preserveInitialSize = firstAttach && ordinaryCaptionWindow(hwnd, m_window);
    if (preserveInitialSize && !initialOuterSize(hwnd, logicalSize, dpr, &initial)) {
        // Leave the standard frame on an unsupported first attachment; do not retry
        // a geometry correction on a later theme/DPI event for this same HWND.
        m_id = 0;
        return;
    }
    // The owned HWND must be bound before synchronous NCCALCSIZE callbacks.
    m_id = id;
    const auto policy = DWMNCRP_ENABLED;
    DwmSetWindowAttribute(hwnd, DWMWA_NCRENDERING_POLICY, &policy, sizeof(policy));
    const BOOL dark = m_dark;
    DwmSetWindowAttribute(hwnd, darkModeAttribute, &dark, sizeof(dark));
    // Unsupported attributes (for example corners on Windows 10) leave the OS default.
    DwmSetWindowAttribute(hwnd, cornerPreferenceAttribute, &roundCorners, sizeof(roundCorners));
    const MARGINS margins{1, 1, 1, 1};
    DwmExtendFrameIntoClientArea(hwnd, &margins);
    const UINT flags = SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED |
                       (preserveInitialSize ? 0u : SWP_NOSIZE);
    if (!SetWindowPos(hwnd, nullptr, 0, 0, preserveInitialSize ? initial.cx : 0,
                      preserveInitialSize ? initial.cy : 0, flags))
        m_id = 0;
#endif
}
bool WindowsFrame::eventFilter(QObject *watched, QEvent *event) {
    if (watched == m_window && event->type() == QEvent::PlatformSurface) {
        const auto *surface = static_cast<QPlatformSurfaceEvent *>(event);
        if (surface->surfaceEventType() == QPlatformSurfaceEvent::SurfaceCreated)
            apply();
        else {
            m_id = 0;
#ifdef Q_OS_WIN
            m_frameAttachAttemptId = 0;
#endif
        }
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
    if (msg->message == WM_GETMINMAXINFO && msg->lParam && result) {
        const auto incoming = *reinterpret_cast<const MINMAXINFO *>(msg->lParam);
        MINMAXINFO response{};
        if (nativeSizeConstraints(hwnd, m_window, incoming, &response)) {
            // During a real drag the HWND already has its destination DPI while
            // Qt can still reference the previous screen. Consume this query so
            // Qt cannot clamp the pending size with the previous DPI's limits.
            // Borrow the writable payload only for this synchronous callback;
            // keep Windows' maximize position/size and commit tracking limits once.
            *reinterpret_cast<MINMAXINFO *>(msg->lParam) = response;
            *result = 0;
            return true;
        }
    }
    if (msg->message == WM_GETDPISCALEDSIZE && msg->lParam && result) {
        const auto incoming = *reinterpret_cast<const SIZE *>(msg->lParam);
        SIZE response{};
        if (dpiOuterSize(hwnd, m_window, msg->wParam, incoming, &response)) {
            // Qt borrows this writable SIZE only for the synchronous callback.
            // Commit once after every native metric and overflow check succeeds.
            *reinterpret_cast<SIZE *>(msg->lParam) = response;
            *result = TRUE;
            return true;
        }
    }
    if (msg->message == WM_NCCALCSIZE && msg->wParam) {
        auto *params = reinterpret_cast<NCCALCSIZE_PARAMS *>(msg->lParam);
        const auto outer = params->rgrc[0];
        DefWindowProcW(hwnd, msg->message, msg->wParam, msg->lParam);
        // Keep native side/bottom resize borders and a DWM top edge. At maximum
        // size Windows hides the resize band outside the monitor work area.
        // Fullscreen has no caption. Keep DefWindowProc's zero-inset client
        // rectangle there, or Qt mistakes the restored window for normal mode.
        if ((GetWindowLongPtrW(hwnd, GWL_STYLE) & WS_CAPTION) == WS_CAPTION)
            params->rgrc[0].top =
                outer.top + (IsZoomed(hwnd) ? resizeBorder(hwnd, SM_CYSIZEFRAME) : 1);
        *result = 0;
        return true;
    }
    if (msg->message == WM_NCHITTEST && !IsZoomed(hwnd) &&
        (GetWindowLongPtrW(hwnd, GWL_STYLE) & WS_THICKFRAME)) {
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
