# apps/desktop/packaging/linux/verify-https.sh

# CI only: a separate network-enabled process in the same SDK-free runtime image.
set -eu
for tool in gcc g++ rustc cmake qmlimportscanner; do
    if command -v "$tool" >/dev/null 2>&1; then
        echo 'Unexpected development tool in clean HTTPS runtime image' >&2
        exit 1
    fi
done
if dpkg-query -W -f='${binary:Package}\n' | grep -E '^(libqt[56]|qt[56]-)' >/dev/null; then
    echo 'Unexpected system Qt package in clean HTTPS runtime image' >&2
    exit 1
fi
test -f '/opt/xdzip runtime/plugins/tls/libqopensslbackend.so'
test -s /etc/ssl/certs/ca-certificates.crt
test "$(dpkg-query -W -f='${Status}' libssl3t64)" = 'install ok installed'
test "$(dpkg-query -W -f='${Status}' ca-certificates)" = 'install ok installed'
mkdir -p /tmp/home /tmp/config /tmp/cache /tmp/runtime /tmp/probe
chmod 700 /tmp/runtime
export HOME=/tmp/home XDG_CONFIG_HOME=/tmp/config XDG_CACHE_HOME=/tmp/cache XDG_RUNTIME_DIR=/tmp/runtime
export LANG=C.UTF-8 QT_QPA_PLATFORM=offscreen QT_QUICK_BACKEND=software
export XDZIP_SETTINGS_PATH=/tmp/probe/https-settings.ini
unset LD_LIBRARY_PATH QT_PLUGIN_PATH QML_IMPORT_PATH QML2_IMPORT_PATH
'/opt/xdzip runtime/xdzip' --update-probe /tmp/probe/https.json
python3 -c 'import json; print("XDZIP_UPDATE_HTTPS "+json.dumps(json.load(open("/tmp/probe/https.json"))))'
