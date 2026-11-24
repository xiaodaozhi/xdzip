# XDZip Snap component sources and Qt replacement

The XDZip sources for this build are at @SOURCE_URL@. They include the locked Rust inputs and the QtNetwork and image-library patch recipes. `snap-source-access.json` lists the Qt source locations and Ubuntu source descriptors. The Qt SDK declarations and component license notices are alongside this file. Ubuntu package copyright files are under `usr/share/doc/` in the Snap. The `core24` base is supplied separately by snapd.

To try an ABI-compatible Qt replacement on Ubuntu 24.04 amd64, extract the Snap with `unsquashfs -d xdzip-local <original.snap>`. Replace shared objects under `xdzip-local/runtime/lib/` and matching plugins under `xdzip-local/runtime/plugins/`. Preserve the library SONAMEs and relative paths, and match the public and private Qt ABIs used by the application and its plugins.

Run `SNAP="$(realpath xdzip-local)" QT_QPA_PLATFORM=offscreen xdzip-local/bin/launch --smoke-test` to check startup. From a graphical session, omit `QT_QPA_PLATFORM` and `--smoke-test` to launch the extracted application. To try the updated package inside snapd confinement, run `snap pack xdzip-local` and install the output with `sudo snap install --dangerous <repacked.snap>` on a test system where the Store copy is absent.
