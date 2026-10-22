#!/bin/sh
# Generated maintainer entry points pass their hook name as the first argument.
set -eu
hook=$1
shift
case "$hook:$1" in
  preinst:*|prerm:*)
    if test "$hook" = preinst; then
      schema=$(dpkg-query -W -f='${X-Xdzip-Settings-Schema}' xdzip 2>/dev/null || :)
      case "$schema" in ''|1) ;; *) echo 'Installed settings schema is newer; downgrade refused.' >&2; exit 1;; esac
    fi
    # A root-owned directory prevents symlink replacement by desktop users.
    test ! -L /run/xdzip
    if test -e /run/xdzip; then
      test "$(stat -c %u:%a /run/xdzip)" = 0:755
    else
      install -d -m 0755 -o root -g root /run/xdzip
    fi
    test ! -L /run/xdzip/usage.lock
    if ! test -e /run/xdzip/usage.lock; then
      (umask 022; : > /run/xdzip/usage.lock)
    fi
    test "$(stat -c %u:%a /run/xdzip/usage.lock)" = 0:644
    # The updater holds this lock across its complete handoff. Only its matching
    # root-owned transaction may enter dpkg; unrelated manual installs refuse it.
    test ! -L /run/xdzip/update.lock
    if ! test -e /run/xdzip/update.lock; then
      (umask 077; : > /run/xdzip/update.lock)
    fi
    test "$(stat -c %u:%a /run/xdzip/update.lock)" = 0:600
    if test -e /run/xdzip/updating; then
      test ! -L /run/xdzip/updating
      test "$(stat -c %u:%a /run/xdzip/updating)" = 0:644
      test -n "${XDZIP_UPDATE_TRANSACTION:-}"
      test "$(cat /run/xdzip/updating)" = "$XDZIP_UPDATE_TRANSACTION"
    else
      exec 8</run/xdzip/update.lock
      if ! flock -n -x 8; then
        echo 'An xdzip update is already being prepared.' >&2
        exit 1
      fi
    fi
    exec 9</run/xdzip/usage.lock
    if ! flock -n -x 9; then
      echo 'Close xdzip before installing, updating or removing it.' >&2
      exit 1
    fi
    # Prevent new launches between dpkg hooks. Failures are cleared by dpkg's
    # abort hooks; interrupted dpkg transactions require dpkg --configure -a.
    install -m 0644 /dev/null /run/xdzip/installing
    ;;
  postinst:configure)
    update-desktop-database -q /usr/share/applications
    rm -f /run/xdzip/installing
    ;;
  postinst:abort-*|postrm:remove|postrm:purge|postrm:abort-*)
    rm -f /run/xdzip/installing
    if command -v update-desktop-database >/dev/null 2>&1; then
      update-desktop-database -q /usr/share/applications
    fi
    ;;
esac
