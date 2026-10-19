# Qt 6.8.2 development build notices

The developer review application dynamically links Qt 6.8.2 Core, Gui, Widgets,
Qml, Quick, QuickControls2 and the QML modules/plugins needed by these imports.
Qt copyright belongs to The Qt Company Ltd. and other contributors.

Qt is available under commercial terms or applicable LGPL/GPL terms. The review
build uses the LGPL option for the runtime modules above. The LGPL-3.0 and its
referenced GPL-3.0 texts accompany this notice. Qt testing/build tools are not
part of the application runtime feature set.

Corresponding Qt source: https://download.qt.io/archive/qt/6.8/6.8.2/single/qt-everywhere-src-6.8.2.tar.xz
Qt licensing: https://doc.qt.io/qt-6.8/licensing.html
Third-party Qt notices: https://doc.qt.io/qt-6.8/licenses-used-in-qt.html

The application source for the exact build accompanies the artifact. Rebuild
with the supplied CMake/Cargo files and a compatible modified Qt installation.
The macOS bundle has only an ad-hoc signature; it is not hardened, notarized or
subject to a vendor library-validation restriction. Modified bundles can be
ad-hoc signed again. No restriction on debugging library modifications is imposed.

These are private development-review artifacts, not a public release compliance
sign-off. Full module/plugin and third-party-source packaging remains a release gate.
