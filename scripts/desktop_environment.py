"""Read-only, stage-specific preflight for the supported desktop toolchains.

Results deliberately contain bounded tool labels and version numbers rather
than command output or absolute paths, so the same report is safe in CI logs.
Tool discovery never installs a toolchain or changes the caller's PATH.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
QT_VERSION = "6.8.2"
STAGES = {"prepare", "build", "test", "package", "verify"}
PYTHON_PACKAGES = {"jsonschema": "4.23.0", "referencing": "0.36.2"}
RUST_HOSTS = {
    "macos": "aarch64-apple-darwin",
    "windows": "x86_64-pc-windows-msvc",
    "linux": "x86_64-unknown-linux-gnu",
}


def current_system() -> str:
    return {"Darwin": "macos", "Windows": "windows", "Linux": "linux"}.get(
        platform.system(), "unsupported")


def locked_rust_version() -> str:
    # tomllib is unavailable in the minimum supported Python 3.10. The repository
    # intentionally pins a release channel, not an arbitrary TOML expression.
    text = (ROOT / "rust-toolchain.toml").read_text(encoding="utf-8")
    match = re.search(r'^\s*channel\s*=\s*"(\d+\.\d+\.\d+)"\s*(?:#.*)?$', text, re.M)
    if not match:
        raise ValueError("rust-toolchain.toml must pin a numeric Rust release")
    return match[1]


def _version(text: str) -> str | None:
    # Git for Windows appends .windows.1 to its numeric release. Permit a
    # textual suffix, while refusing to truncate an additional numeric part.
    match = re.search(r"(?<![\d.])(\d+\.\d+(?:\.\d+){0,2})(?!\d|\.\d)", text)
    return match[1] if match else None


def _numbers(version: str) -> tuple[int, ...]:
    return tuple(int(part) for part in version.split("."))


def _run(command: list[str], *, allowed: tuple[int, ...] | None = (0,),
         timeout: int = 20, env: dict[str, str] | None = None,
         include_stderr: bool = True) -> str | None:
    # rustup shims otherwise attempt network installation while merely reading
    # a version. Offline metadata must also fail before any deployment copying.
    environment = dict(os.environ if env is None else env)
    environment.update(RUSTUP_AUTO_INSTALL="0", CARGO_NET_OFFLINE="true")
    try:
        result = subprocess.run(command, cwd=ROOT, env=environment, capture_output=True,
                                text=True, errors="replace", timeout=timeout)
    except (OSError, subprocess.SubprocessError):
        return None
    if allowed is not None and result.returncode not in allowed:
        return None
    return result.stdout + ("\n" + result.stderr if include_stderr else "")


def check_environment(qt_prefix: Path | None = None, *, stage: str,
                      system: str | None = None, python_executable: Path | str | None = None,
                      env: dict[str, str] | None = None) -> dict:
    """Check dependencies needed by one stage, aggregating actionable failures.

    ``prepare`` permits an absent Qt SDK and an uninstalled pinned Rust channel,
    but requires Rust launchers in PATH. ``verify`` needs Python schema
    dependencies and deployment tools, not the Rust/C++ build toolchain. No CI variables
    are required. An explicit system is useful for deterministic policy tests;
    it does not enable cross-compilation.
    """
    if stage not in STAGES:
        raise ValueError("Unknown desktop stage: " + stage)
    system = system or current_system()
    checks: list[dict] = []
    environment = dict(os.environ if env is None else env)

    def run(command: list[str], **kwargs) -> str | None:
        return _run(command, env=environment, **kwargs)

    def which(name: str) -> str | None:
        return shutil.which(name, path=environment.get("PATH", ""))

    def add(name: str, passed: bool, hint: str = "", *, version: str | None = None,
            optional: bool = False) -> bool:
        record = {"name": name, "status": "passed" if passed else "warning" if optional else "failed"}
        if version is not None:
            record["version"] = version
        if not passed and hint:
            record["hint"] = hint
        checks.append(record)
        return passed

    def tool(name: str, arguments: list[str] | None = None, *, minimum: tuple[int, ...] | None = None,
             hint: str | None = None, allowed: tuple[int, ...] = (0,)) -> tuple[str | None, str | None]:
        executable = which(name)
        if not add("tool." + name, executable is not None,
                   hint or "Install " + name + " and expose it in PATH."):
            return None, None
        if arguments is None:
            return executable, None
        output = run([executable, *arguments], allowed=allowed)
        version = _version(output or "")
        valid = version is not None and (minimum is None or _numbers(version) >= minimum)
        add("version." + name, valid,
            hint or "Check that " + name + " runs and meets the documented toolchain version.", version=version)
        return executable, output

    python_version = ".".join(str(part) for part in sys.version_info[:3])
    add("python", sys.version_info >= (3, 10), "Use Python 3.10 or newer.", version=python_version)
    if not add("platform", system in RUST_HOSTS,
               "Use native macOS arm64, Windows x64 or Linux x86_64."):
        return {"status": "failed", "checks": checks}
    machine = platform.machine().lower()
    expected = {"macos": {"arm64", "aarch64"}, "windows": {"amd64", "x86_64"},
                "linux": {"x86_64", "amd64"}}[system]
    add("host.architecture", machine in expected,
        "Run this stage natively on " + ("Apple Silicon arm64." if system == "macos" else "x86_64."))

    if stage != "verify":
        tool("git", ["--version"])
        tool("cmake", ["--version"], minimum=(3, 24), hint="Install CMake 3.24 or newer and expose it in PATH.")
        tool("ninja", ["--version"])
        if stage == "test":
            tool("ctest", ["--version"], minimum=(3, 24))

        try:
            rust_version = locked_rust_version()
        except (OSError, ValueError):
            rust_version = None
            add("rust.lock", False, "Restore the numeric release pin in rust-toolchain.toml.")
        rustc = which("rustc")
        cargo = which("cargo")
        for name, executable in [("rustc", rustc), ("cargo", cargo)]:
            add("tool." + name, executable is not None,
                "Install rustup and expose its cargo bin directory in PATH before this stage.")
        if stage == "prepare":
            tool("rustup", ["--version"])
        rust_hint = ("Install the pinned Rust channel with rustup toolchain install " +
                     (rust_version or "<pinned-version>") + " --profile minimal --component rustfmt --component clippy.")
        rust_output = run([rustc, "-vV"]) if rustc else None
        cargo_output = run([cargo, "--version"]) if cargo else None
        release = re.search(r"^release:\s*(\S+)", rust_output or "", re.M)
        host = re.search(r"^host:\s*(\S+)", rust_output or "", re.M)
        actual_rust = _version(release[1]) if release else None
        actual_cargo = _version(cargo_output or "")
        if rustc:
            add("version.rustc", rust_version is not None and release is not None and release[1] == rust_version,
                rust_hint, version=actual_rust, optional=stage == "prepare")
            add("rust.host", host is not None and host[1] == RUST_HOSTS[system],
                "Use the native " + RUST_HOSTS[system] + " Rust toolchain.", optional=stage == "prepare")
        if cargo:
            add("version.cargo", rust_version is not None and actual_cargo == rust_version,
                rust_hint, version=actual_cargo, optional=stage == "prepare")
        if stage == "test" and cargo:
            for component, arguments in [("rustfmt", ["fmt", "--version"]), ("clippy", ["clippy", "--version"])]:
                add("rust." + component, run([cargo, *arguments]) is not None, rust_hint)

        if system == "macos":
            compiler, output = tool("clang++", ["--version"], hint="Install or select Xcode Command Line Tools with a macOS 14+ SDK.")
            target = run([compiler, "-dumpmachine"]) if compiler else None
            add("compiler.target", target is not None and re.search(r"\b(arm64|aarch64)-apple-", target) is not None,
                "Use the native Apple Silicon clang++ compiler, not a Rosetta/x86_64 shell.")
            sdk_tool, _ = tool("xcrun")
            sdk_version = _version(run([sdk_tool, "--sdk", "macosx", "--show-sdk-version"]) or "") if sdk_tool else None
            sdk_path = run([sdk_tool, "--sdk", "macosx", "--show-sdk-path"], include_stderr=False) if sdk_tool else None
            add("macos.sdk", sdk_version is not None and _numbers(sdk_version) >= (14, 0) and
                sdk_path is not None and Path(sdk_path.strip()).is_dir(),
                "Select an installed macOS 14+ SDK with xcode-select; accept any pending Xcode setup first.", version=sdk_version)
        elif system == "windows":
            compiler, output = tool("cl", ["/Bv"], minimum=(19, 30), allowed=(0, 2),
                                    hint="Open the MSVC 2022 x64 Native Tools shell before running this stage.")
            target = environment.get("VSCMD_ARG_TGT_ARCH", "").lower()
            add("compiler.target", output is not None and re.search(r"\bx64\b", output, re.I) is not None and
                target in {"", "x64", "amd64"}, "Select the x64 MSVC developer environment, not x86 or ARM64.")
            linker, _ = tool("link")
            if linker:
                # Microsoft LINK's read-only help query has nonzero statuses on
                # some SDKs. Its banner, not that status or its filename, proves
                # this is the MSVC linker rather than a GNU utility in PATH.
                link_output = run([linker, "/?"], allowed=None)
                link_version = _version(link_output or "")
                microsoft = re.search(r"\bMicrosoft\b[^\r\n]*\bIncremental\s+Linker\b",
                                      link_output or "", re.I) is not None
                add("version.link", microsoft and link_version is not None,
                    "Select the Microsoft Incremental Linker from the MSVC x64 Native Tools environment, not GNU link.",
                    version=link_version)
            tool("rc", ["/?"])
            # Windows os.environ normalizes keys to uppercase; a plain copied
            # dictionary no longer provides its case-insensitive lookup.
            sdk_version = _version(environment.get("WINDOWSSDKVERSION", environment.get("WindowsSDKVersion", "")))
            sdk_dir = environment.get("WINDOWSSDKDIR", environment.get("WindowsSdkDir"))
            add("windows.sdk", bool(sdk_dir and Path(sdk_dir).is_dir() and sdk_version),
                "Install the Windows SDK and enter the MSVC x64 developer environment.", version=sdk_version)
        else:
            compiler, _ = tool("c++", ["--version"], hint="Install an x86_64 GCC or Clang C++17 build toolchain.")
            target = run([compiler, "-dumpmachine"]) if compiler else None
            add("compiler.target", target is not None and re.search(r"\bx86_64-.*linux", target) is not None,
                "Use the native x86_64 Linux C++ toolchain.")
            pkg_config, _ = tool("pkg-config", ["--version"],
                                 hint="Install pkg-config and Ubuntu OpenGL/EGL/xkbcommon development packages.")
            graphics = run([pkg_config, "--exists", "gl", "egl", "xkbcommon"]) if pkg_config else None
            add("linux.graphics-devel", graphics is not None,
                "Install pkg-config libgl1-mesa-dev libegl1-mesa-dev libxkbcommon-dev before configuring the Qt desktop build.")

        if stage != "prepare":
            _check_qt(Path(qt_prefix) if qt_prefix is not None else None, system, stage, add, run, which)

        if stage == "package":
            if cargo and host and host[1] == RUST_HOSTS[system]:
                metadata = run([cargo, "metadata", "--locked", "--offline", "--format-version", "1",
                                 "--filter-platform", RUST_HOSTS[system]], timeout=120, include_stderr=False)
                try:
                    parsed = json.loads(metadata or "")
                    valid_metadata = isinstance(parsed.get("packages"), list) and bool(parsed["packages"])
                except (ValueError, AttributeError):
                    valid_metadata = False
                add("cargo.metadata", valid_metadata,
                    "Prepare the pinned Rust toolchain and fetch locked Cargo dependencies before packaging; cargo metadata --locked --offline must succeed.")
            else:
                add("cargo.metadata", False, "Restore native rustc/cargo in PATH before packaging, including license collection.")

    if stage in {"package", "verify"}:
        _check_python_packages(python_executable, add, run)
        _check_cached_materials(stage, add)
        tools = {
            "macos": ["hdiutil", "codesign", "lipo", "otool", "install_name_tool"],
            "windows": ["dumpbin", "pwsh"] + (["powershell.exe"] if stage == "package" else []),
            "linux": ["ldd", "readelf"] + (["patchelf", "dpkg-query", "dpkg-deb"] if stage == "package" else ["docker"]),
        }[system]
        for name in tools:
            tool(name)
        if system == "linux" and stage == "verify":
            docker = which("docker")
            daemon = run([docker, "info", "--format", "{{.OSType}}/{{.Architecture}}"],
                         include_stderr=False) if docker else None
            add("docker.daemon", (daemon or "").strip().lower() in {"linux/x86_64", "linux/amd64"},
                "Start a reachable Docker daemon with Linux x86_64 containers for installer/runtime verification.")

    return {"status": "failed" if any(check["status"] == "failed" for check in checks) else "passed",
            "checks": checks}


def _check_qt(qt: Path | None, system: str, stage: str, add, run, which) -> None:
    if not add("qt.prefix", qt is not None and qt.is_dir(),
               "Install Qt 6.8.2 desktop SDK and pass its platform directory with --qt-prefix."):
        return
    suffix = ".exe" if system == "windows" else ""
    qmake = qt / "bin" / ("qmake" + suffix)
    output = run([str(qmake), "-query", "QT_VERSION"], include_stderr=False) if qmake.is_file() else None
    actual = (output or "").strip()
    add("qt.version", actual == QT_VERSION,
        "Use the complete pinned Qt " + QT_VERSION + " desktop SDK, not a different Qt installation.",
        version=_version(actual))
    directories = ["lib/cmake/Qt6", "qml/QtQuick", "qml/QtQuick/Controls/Basic", "qml/QtQml", "plugins/platforms"]
    for relative in directories:
        add("qt." + relative, (qt / relative).is_dir(), "Reinstall the Qt Quick desktop SDK; required " + relative + " directory is missing.")
    # The shared build always enables BUILD_TESTING. Test and QuickTest must be
    # present before CMake runs, not discovered only after a costly SDK install.
    for module in ["Core", "Gui", "Qml", "Quick", "QuickControls2", "Widgets", "Test", "QuickTest"]:
        config = qt / "lib/cmake" / ("Qt6" + module) / ("Qt6" + module + "Config.cmake")
        add("qt.cmake." + module, config.is_file(),
            "Install the complete Qt 6.8.2 desktop SDK, including the " + module + " development module.")
    for relative in ["bin/qmllint" + suffix, "bin/qmlimportscanner.exe" if system == "windows" else "libexec/qmlimportscanner"]:
        add("qt." + relative, (qt / relative).is_file(), "Reinstall Qt declarative tools; required " + relative + " executable is missing.")
    plugin = {"macos": "libqcocoa.dylib", "windows": "qwindows.dll", "linux": "libqxcb.so"}[system]
    add("qt.platform-plugin", (qt / "plugins/platforms" / plugin).is_file(),
        "Install the native desktop Qt platform plugin for this host.")
    core = {"macos": "lib/QtCore.framework/Versions/A/QtCore", "windows": "bin/Qt6Core.dll",
            "linux": "lib/libQt6Core.so.6"}[system]
    add("qt.core", (qt / core).is_file(), "Install a complete native Qt desktop runtime, including QtCore.")
    if system in {"windows", "linux"}:
        add("qt.architecture", _native_x64(qt / core, system),
            "Install the x86_64 Qt desktop SDK matching the native compiler.")
    if system == "macos":
        lipo = which("lipo")
        archs = run([lipo, "-archs", str(qt / core)]) if lipo and (qt / core).is_file() else None
        add("qt.architecture", archs is not None and "arm64" in archs.split(),
            "Install a Qt SDK containing native arm64 QtCore and expose Xcode tools in PATH.")
    if stage == "package" and system in {"macos", "windows"}:
        deploy = "macdeployqt" if system == "macos" else "windeployqt.exe"
        add("qt." + deploy, (qt / "bin" / deploy).is_file(), "Install the Qt desktop deployment tool " + deploy + ".")
    if stage == "package":
        add("qt.translations", (qt / "translations/qtbase_zh_CN.qm").is_file(),
            "Install Qt base translation catalogs before packaging.")
        try:
            from runtime_attribution import load_qt
            _, _, _, documents, _ = load_qt(qt)
            valid_documents = {"qtbase", "qtdeclarative", "qtsvg"} <= {item["repository"] for item in documents}
        except (OSError, ValueError, KeyError, TypeError, AttributeError):
            valid_documents = False
        add("qt.spdx", valid_documents,
            "Install the complete Qt 6.8.2 SDK with valid qtbase, qtdeclarative and qtsvg SPDX build documents.")


def _native_x64(path: Path, system: str) -> bool:
    """Read only the bounded ELF/PE header, without executing an SDK library."""
    try:
        with path.open("rb") as stream:
            header = stream.read(64)
            if system == "linux":
                if len(header) < 20 or header[:5] != b"\x7fELF\x02" or header[5] not in (1, 2):
                    return False
                return int.from_bytes(header[18:20], "little" if header[5] == 1 else "big") == 62
            if len(header) != 64 or header[:2] != b"MZ":
                return False
            offset = int.from_bytes(header[60:64], "little")
            if not 64 <= offset <= 1024 * 1024:
                return False
            stream.seek(offset)
            pe = stream.read(6)
            return pe[:4] == b"PE\0\0" and pe[4:] == b"\x64\x86"
    except OSError:
        return False


def _check_python_packages(python_executable, add, run) -> None:
    # Probe the exact interpreter used by subsequent packaging scripts,
    # not the import environment of a different orchestration Python.
    probe = """import importlib.metadata as m, json, sys
versions = {"python": list(sys.version_info[:3])}
for package in ("jsonschema", "referencing"):
    try:
        versions[package] = m.version(package)
    except m.PackageNotFoundError:
        versions[package] = None
print(json.dumps(versions))
"""
    payload = run([str(python_executable or sys.executable), "-c", probe], include_stderr=False)
    try:
        installed_packages = json.loads(payload or "")
        selected_version = installed_packages.get("python", [])
        valid_python = (isinstance(selected_version, list) and len(selected_version) == 3 and
                        all(isinstance(value, int) for value in selected_version))
    except (ValueError, AttributeError):
        installed_packages = {}
        valid_python = False
        selected_version = []
    add("python.packaging", valid_python and tuple(selected_version) >= (3, 10),
        "Use a working Python 3.10+ packaging interpreter.",
        version=".".join(str(value) for value in selected_version) if valid_python else None)
    for package, pinned in PYTHON_PACKAGES.items():
        installed = installed_packages.get(package)
        add("python." + package, installed == pinned,
            "Install " + package + "==" + pinned + " into the Python environment running packaging.",
            version=_version(installed) if isinstance(installed, str) else None)


def _check_cached_materials(stage: str, add) -> None:
    # Packaging and verification must not discover missing network inputs only
    # after copying a large Qt runtime. Preparation owns all downloads.
    groups = [("sbom-schemas", "sbom-schemas.json", "files", "name", "")]
    if stage == "package":
        groups.append(("qt-license-cache", "qt-license-sources.json", "licenses", "id", ".txt"))
    for label, config, collection, key, suffix in groups:
        valid = True
        try:
            records = json.loads((ROOT / "scripts" / config).read_text(encoding="utf-8"))[collection]
            if not isinstance(records, list) or not records:
                valid = False
            else:
                for record in records:
                    name, expected = record[key], record["sha256"]
                    if (not isinstance(name, str) or re.fullmatch(r"[A-Za-z0-9.+_-]{1,128}", name) is None or
                            not isinstance(expected, str) or re.fullmatch(r"[0-9a-f]{64}", expected) is None):
                        valid = False
                        break
                    path = ROOT / "build/review" / label / (name + suffix)
                    if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
                        valid = False
                        break
        except (OSError, ValueError, KeyError, TypeError):
            valid = False
        add("cache." + label, valid,
            "Run python scripts/desktop.py prepare --packaging to prepare the hash-pinned " + label + " inputs; remove any corrupted cache entry first.")
