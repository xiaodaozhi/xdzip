#!/usr/bin/env bash
set -euo pipefail

snap_tools=(desktop-file-utils squashfs-tools snapd)
apt_options=(-o Acquire::http::Timeout=30 -o Acquire::https::Timeout=30 -o Acquire::Retries=2)

case "$1" in
  mirror)
    for sources in /etc/apt/sources.list /etc/apt/sources.list.d/ubuntu.sources; do
      if [[ -f "$sources" ]]; then
        sudo sed -E -i \
          -e 's#https?://(azure|[a-z0-9-]+\.ec2)\.archive\.ubuntu\.com/ubuntu#https://archive.ubuntu.com/ubuntu#g' \
          -e 's|mirror\+file:/etc/apt/apt-mirrors\.txt|https://archive.ubuntu.com/ubuntu|g' "$sources"
      fi
    done
    ;;
  prepare)
    # Check and download packaging tools without changing the native Qt inputs.
    sudo apt-get "${apt_options[@]}" install -y --download-only --no-install-recommends "${snap_tools[@]}"
    curl --fail --location --retry 3 --output "$RUNNER_TEMP/core24.snap" \
      https://api.snapcraft.io/api/v1/snaps/download/dwTAh7MZZ01zyriOZErqd1JynQLiOGvM_2307.snap
    python3 -B scripts/build-linux-snap.py --prepare-inputs \
      --core24 "$RUNNER_TEMP/core24.snap" --package-cache "$RUNNER_TEMP/snap-debs"
    python3 -B scripts/test-snap-launch.py
    ;;
  pack)
    sudo apt-get "${apt_options[@]}" install -y --no-install-recommends "${snap_tools[@]}"
    qt_prefix="$RUNNER_TEMP/Qt/6.8.2/gcc_64"
    tools_python="$RUNNER_TEMP/desktop-tools/bin/python"
    "$tools_python" -B scripts/deploy_linux.py --experimental-wayland \
      --app build/desktop/apps/desktop/xdzip --qt-prefix "$qt_prefix" \
      --notices build/review/notices --output build/snap-runtime
    package=$("$tools_python" -B scripts/build-linux-snap.py --grade stable \
      --runtime build/snap-runtime --qt-prefix "$qt_prefix" \
      --core24 "$RUNNER_TEMP/core24.snap" --package-cache "$RUNNER_TEMP/snap-debs" \
      --output build/snap-store)
    extracted="$RUNNER_TEMP/xdzip-snap"
    unsquashfs -q -no-progress -d "$extracted" "$package"
    SNAP="$extracted" QT_QPA_PLATFORM=offscreen "$extracted/bin/launch" --smoke-test
    ;;
  *)
    echo 'Usage: ci-release-snap.sh {mirror|prepare|pack}' >&2
    exit 2
    ;;
esac
