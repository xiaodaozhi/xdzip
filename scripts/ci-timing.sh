# The command remains a direct shell child. Record failure before bash -e exits.
ci_run() {
  local phase="$1"
  shift
  local label
  label="$(basename "${2:-$1}")"
  local started ended result=0
  started="$(date +%s%N)"
  "$@" || result=$?
  ended="$(date +%s%N)"
  build/ci-html-tools/bin/python scripts/ci-timing.py --phase "$phase" --label "$label" \
    --started-ns "$started" --ended-ns "$ended" --exit-code "$result" || return $?
  return "$result"
}
