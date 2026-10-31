# Shared by the SDK-free HTTPS check and its owned probe fixtures.
run_https_probe() {
    probe_status=0
    "$1" --update-probe "$2" || probe_status=$?
    if [ ! -s "$2" ]; then
        printf 'Update probe did not write a report (exit %s)\n' "$probe_status" >&2
        if [ "$probe_status" -ne 0 ]; then
            return "$probe_status"
        fi
        return 1
    fi
    if ! python3 -c 'import json, sys; value = json.load(open(sys.argv[1], encoding="utf-8")); isinstance(value, dict) or sys.exit("Expected a JSON object"); print("XDZIP_UPDATE_HTTPS " + json.dumps(value))' "$2"; then
        echo 'Update probe report is invalid' >&2
        if [ "$probe_status" -ne 0 ]; then
            printf 'Update probe failed (exit %s)\n' "$probe_status" >&2
            return "$probe_status"
        fi
        return 1
    fi
    if [ "$probe_status" -ne 0 ]; then
        printf 'Update probe failed (exit %s)\n' "$probe_status" >&2
        return "$probe_status"
    fi
}
