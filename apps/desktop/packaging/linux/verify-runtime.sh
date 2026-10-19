#!/bin/sh
# CI only: isolated runtime image, read-only relocated application, no network.
set -eu
for tool in gcc g++ rustc cmake qmlimportscanner; do
    if command -v "$tool" >/dev/null 2>&1; then
        echo "Unexpected development tool in clean runtime image" >&2
        exit 1
    fi
done
if dpkg-query -W -f='${binary:Package}\n' | grep -E '^(libqt[56]|qt[56]-)' >/dev/null; then
    echo "Unexpected system Qt package in clean runtime image" >&2
    exit 1
fi
mkdir -p /tmp/home /tmp/config /tmp/cache /tmp/runtime /tmp/probe
chmod 700 /tmp/runtime
export HOME=/tmp/home XDG_CONFIG_HOME=/tmp/config XDG_CACHE_HOME=/tmp/cache XDG_RUNTIME_DIR=/tmp/runtime
export LANG=zh_CN.UTF-8 LC_ALL=zh_CN.UTF-8
export QT_QPA_PLATFORM=xcb QT_QUICK_BACKEND=software
unset LD_LIBRARY_PATH QT_PLUGIN_PATH QML_IMPORT_PATH QML2_IMPORT_PATH
xdzip_app="/opt/xdzip runtime/xdzip"
"$xdzip_app" --version
xvfb-run -a "$xdzip_app" --smoke-test
XDZIP_SETTINGS_PATH=/tmp/probe/settings.ini xvfb-run -a "$xdzip_app" --desktop-probe /fixture.zip
