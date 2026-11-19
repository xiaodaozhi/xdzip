#include "LinuxFrame.h"

#include <QGuiApplication>
#include <QPlatformSurfaceEvent>
#ifdef Q_OS_LINUX
#include <X11/Xatom.h>
#include <X11/Xlib.h>
#include <X11/extensions/shape.h>

static Display *x11Display() {
    auto *xcb = qGuiApp->nativeInterface<QNativeInterface::QX11Application>();
    return xcb ? xcb->display() : nullptr;
}
#endif

LinuxFrame::LinuxFrame(QObject *parent) : QObject(parent) {
#ifdef Q_OS_LINUX
    // Check the session before reserving any transparent space. On an
    // uncomposited desktop the QML outline still uses the original geometry.
    if (auto *display = x11Display()) {
        const auto selection =
            QByteArray("_NET_WM_CM_S") + QByteArray::number(DefaultScreen(display));
        const auto atom = XInternAtom(display, selection.constData(), True);
        int major = 0;
        int minor = 0;
        m_shadowAvailable = atom != None && XGetSelectionOwner(display, atom) != None &&
                            XShapeQueryVersion(display, &major, &minor) &&
                            (major > 1 || (major == 1 && minor >= 1));
    }
#endif
}

void LinuxFrame::setWindow(QWindow *window) {
    if (m_window == window)
        return;
    if (m_window) {
        updateNativeFrame(0);
        m_window->removeEventFilter(this);
    }
    m_window = window;
    m_id = 0;
    m_nativeInset = 0;
    if (m_window && m_shadowAvailable) {
        m_window->installEventFilter(this);
        updateNativeFrame(m_inset);
    }
    emit windowChanged();
}

void LinuxFrame::setInset(int inset) {
    if (m_inset == inset)
        return;
    m_inset = inset;
    updateNativeFrame(inset);
    emit insetChanged();
}

bool LinuxFrame::eventFilter(QObject *watched, QEvent *event) {
    if (event->type() == QEvent::PlatformSurface) {
        const auto *surface = static_cast<QPlatformSurfaceEvent *>(event);
        if (surface->surfaceEventType() == QPlatformSurfaceEvent::SurfaceAboutToBeDestroyed) {
            m_id = 0;
            m_nativeInset = 0;
            return false;
        }
    }
    if (event->type() == QEvent::PlatformSurface || event->type() == QEvent::Show ||
        event->type() == QEvent::Resize || event->type() == QEvent::DevicePixelRatioChange)
        updateNativeFrame(m_inset);
    return QObject::eventFilter(watched, event);
}

void LinuxFrame::updateNativeFrame(int inset) {
#ifdef Q_OS_LINUX
    // Only touch Qt's existing XID. A destroyed/replaced native surface has
    // no frame properties to clear; the next surface is configured afresh.
    if (!m_shadowAvailable || !m_window || !m_window->handle())
        return;
    auto *display = x11Display();
    if (!display)
        return;
    const auto id = m_window->winId();
    const qreal scale = m_window->devicePixelRatio();
    const auto nativeInset = qRound(inset * scale);
    if (id != m_id) {
        m_id = id;
        m_nativeInset = 0;
    }
    if (nativeInset != m_nativeInset) {
        const auto extents = XInternAtom(display, "_GTK_FRAME_EXTENTS", False);
        if (nativeInset) {
            const auto margin = static_cast<unsigned long>(nativeInset);
            const unsigned long values[] = {margin, margin, margin, margin};
            XChangeProperty(display, id, extents, XA_CARDINAL, 32, PropModeReplace,
                            reinterpret_cast<const unsigned char *>(values), 4);
        } else {
            XDeleteProperty(display, id, extents);
            XShapeCombineMask(display, id, ShapeInput, 0, 0, None, ShapeSet);
        }
        m_nativeInset = nativeInset;
    }
    if (nativeInset) {
        // Shape input alone so the compositor can still see the shadow.
        const auto width = qMax(0, qRound(m_window->width() * scale) - 2 * nativeInset);
        const auto height = qMax(0, qRound(m_window->height() * scale) - 2 * nativeInset);
        XRectangle region{0, 0, static_cast<unsigned short>(width),
                          static_cast<unsigned short>(height)};
        XShapeCombineRectangles(display, id, ShapeInput, nativeInset, nativeInset, &region, 1,
                                ShapeSet, Unsorted);
    }
    XFlush(display);
#else
    Q_UNUSED(inset);
#endif
}
