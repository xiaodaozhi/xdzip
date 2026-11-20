#!/bin/sh
set -eu

export LD_LIBRARY_PATH="$SNAP/runtime/lib:$SNAP/usr/lib/x86_64-linux-gnu:$SNAP/lib/x86_64-linux-gnu"
export QT_PLUGIN_PATH="$SNAP/runtime/plugins"
export QML_IMPORT_PATH="$SNAP/runtime/qml"

# snapd gives each snap a private XDG_RUNTIME_DIR, while the compositor's
# socket remains in the parent directory. Keep existing paths intact and use
# X11 if a Wayland socket cannot be reached.
wayland_available=false
if [ -n "${WAYLAND_DISPLAY:-}" ]; then
    case "$WAYLAND_DISPLAY" in
        /*)
            if [ -S "$WAYLAND_DISPLAY" ]; then wayland_available=true; fi
            ;;
        */*|.|..)
            ;;
        *)
            if [ -n "${XDG_RUNTIME_DIR:-}" ] && [ -d "$XDG_RUNTIME_DIR" ]; then
                local_socket="$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY"
                compositor_socket="$XDG_RUNTIME_DIR/../$WAYLAND_DISPLAY"
                if [ ! -S "$local_socket" ] && [ -S "$compositor_socket" ] &&
                   [ ! -e "$local_socket" ] && [ ! -L "$local_socket" ]; then
                    ln -s "$compositor_socket" "$local_socket" 2>/dev/null || :
                fi
                if [ -S "$local_socket" ]; then wayland_available=true; fi
            fi
            ;;
    esac
fi

if [ -z "${QT_QPA_PLATFORM:-}" ]; then
    if [ "$wayland_available" = true ]; then
        QT_QPA_PLATFORM=wayland
    else
        QT_QPA_PLATFORM=xcb
    fi
    export QT_QPA_PLATFORM
fi
export QT_QUICK_BACKEND="${QT_QUICK_BACKEND:-software}"
export QT_XKB_CONFIG_ROOT="$SNAP/usr/share/X11/xkb"
export FONTCONFIG_FILE="$SNAP/meta/fonts.conf"
exec "$SNAP/runtime/bin/xdzip" "$@"
