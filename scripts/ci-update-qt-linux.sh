#!/usr/bin/env bash
set -euo pipefail

mode=$1
result=$2
publish=${3:-0}
token=${GITHUB_TOKEN:-}
unset GITHUB_TOKEN
dockerfile=tests/fixtures/native/update-qt-build.Dockerfile
prefix=/opt/Qt/6.8.2/gcc_64
mkdir -p "$result/incoming"
docker build --target sdk-base -f "$dockerfile" -t xdzip-update-qt-base .
docker run --rm xdzip-update-qt-base python3 scripts/ci-qt-network.py inputs \
    --qt-prefix "$prefix" > "$result/inputs.json"
GITHUB_TOKEN=$token python3 scripts/ci-qt-network.py fetch --inputs "$result/inputs.json" \
    --output "$result/incoming" --mode "$mode"
docker build --target workflow-prepared -f "$dockerfile" -t xdzip-update-qt-build \
    --build-context "qt-network-incoming=$result/incoming" --build-arg "XDZIP_QT_NETWORK_PUBLISH=$publish" .
container=$(docker create xdzip-update-qt-build)
trap 'docker rm "$container" >/dev/null' EXIT
docker cp "$container:/qt-network-result/." "$result/"
if [[ -n ${GITHUB_OUTPUT:-} && -f $result/qt-network.zip ]]; then
    python3 -c 'import json, sys; print("bundle=" + json.load(open(sys.argv[1]))["key"])' \
        "$result/qt-network-prepare.json" >> "$GITHUB_OUTPUT"
fi
