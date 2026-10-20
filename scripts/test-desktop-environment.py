#!/usr/bin/env python3
"""Owned-fixture regressions for early, read-only desktop environment checks."""
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import desktop_environment as environment


class EnvironmentTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name)
        self.sdk = self.base / "sdk"
        self.sdk.mkdir()
        self.repository = self.base / "repository"
        (self.repository / "scripts").mkdir(parents=True)
        (self.repository / "rust-toolchain.toml").write_text('[toolchain]\nchannel = "1.99.0"\n')
        self.cache_materials()
        self.qt = self.base / "Qt"
        self.calls = []
        self.lookups = []
        self.missing = set()
        self.overrides = {}
        self.system = "macos"
        self.machine = "arm64"
        self.variables = {"PATH": "fixture-tools", "WindowsSdkDir": str(self.sdk),
                          "WindowsSDKVersion": "10.0.22621.0\\", "VSCMD_ARG_TGT_ARCH": "x64"}
        self.addCleanup(patch.stopall)
        patch.object(environment.shutil, "which", side_effect=self.which).start()
        patch.object(environment.subprocess, "run", side_effect=self.run_command).start()
        patch.object(environment.platform, "machine", side_effect=lambda: self.machine).start()
        patch.object(environment, "ROOT", self.repository).start()

    def cache_materials(self):
        for label, config, collection, key, suffix, names in [
            ("sbom-schemas", "sbom-schemas.json", "files", "name", "", ["bom-1.6.schema.json", "spdx.schema.json", "jsf-0.82.schema.json"]),
            ("qt-license-cache", "qt-license-sources.json", "licenses", "id", ".txt", ["Owned-1.0"]),
        ]:
            records = []
            for name in names:
                data = ("owned fixture: " + name).encode()
                path = self.repository / "build/review" / label / (name + suffix)
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(data)
                records.append({key: name, "sha256": hashlib.sha256(data).hexdigest()})
            (self.repository / "scripts" / config).write_text(json.dumps({collection: records}))

    def which(self, name, *, path=None):
        self.lookups.append((name, path))
        return None if name in self.missing else "/fixture-tools/" + name

    def run_command(self, command, **kwargs):
        self.calls.append((command, kwargs))
        name = Path(command[0]).name.removesuffix(".exe")
        arguments = tuple(command[1:])
        override = self.overrides.get((name, arguments))
        if override is not None:
            if isinstance(override, Exception):
                raise override
            return subprocess.CompletedProcess(command, *override)
        result = {
            "git": "git version 2.48.1", "cmake": "cmake version 3.31.6", "ctest": "ctest version 3.31.6",
            "ninja": "1.12.1", "rustup": "rustup 1.28.1", "qmake": "6.8.2",
            "clang++": "Apple clang version 16.0.0", "c++": "c++ (Ubuntu 13.3.0) 13.3.0",
            "cl": "Microsoft C/C++ Optimizing Compiler Version 19.44.35218 for x64",
            "link": "Microsoft Incremental Linker Version 14.44.35218.0",
            "rc": "Microsoft Windows Resource Compiler Version 10.0.22621.0",
            "lipo": "x86_64 arm64", "pkg-config": "1.8.1",
        }.get(name, "")
        if name == "rustc":
            result = "rustc 1.99.0 (owned fixture)\nrelease: 1.99.0\nhost: " + environment.RUST_HOSTS[self.system]
        elif name == "cargo":
            result = '{"packages":[{"name":"owned-fixture"}]}' if arguments[:1] == ("metadata",) else "cargo 1.99.0 (owned fixture)"
        elif name in {"clang++", "c++"} and arguments == ("-dumpmachine",):
            result = "arm64-apple-darwin24.0.0" if name == "clang++" else "x86_64-linux-gnu"
        elif name == "xcrun":
            result = str(self.sdk) if arguments[-1] == "--show-sdk-path" else "15.2"
        elif name == "docker":
            result = "linux/x86_64"
        elif arguments[:1] == ("-c",):
            compile(arguments[1], "<packaging-python-probe>", "exec")
            result = json.dumps({"python": [3, 12, 9], "jsonschema": "4.23.0", "referencing": "0.36.2"})
        # MSVC reports its version on stderr and returns 2 without an input file.
        return subprocess.CompletedProcess(command, 2 if name == "cl" else 0,
                                           "" if name == "cl" else result,
                                           result if name == "cl" else "")

    def qt_sdk(self):
        suffix = ".exe" if self.system == "windows" else ""
        for relative in ["lib/cmake/Qt6", "qml/QtQuick/Controls/Basic", "qml/QtQml", "plugins/platforms"]:
            (self.qt / relative).mkdir(parents=True, exist_ok=True)
        files = ["bin/qmake" + suffix, "bin/qmllint" + suffix, "translations/qtbase_zh_CN.qm",
                 "bin/qmlimportscanner.exe" if self.system == "windows" else "libexec/qmlimportscanner"]
        files += {
            "macos": ["bin/macdeployqt", "lib/QtCore.framework/Versions/A/QtCore", "plugins/platforms/libqcocoa.dylib"],
            "windows": ["bin/windeployqt.exe", "bin/Qt6Core.dll", "plugins/platforms/qwindows.dll"],
            "linux": ["lib/libQt6Core.so.6", "plugins/platforms/libqxcb.so"],
        }[self.system]
        for relative in files:
            path = self.qt / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"owned-fixture")
        for module in ["Core", "Gui", "Qml", "Quick", "QuickControls2", "Widgets", "Test", "QuickTest"]:
            path = self.qt / "lib/cmake" / ("Qt6" + module) / ("Qt6" + module + "Config.cmake")
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("# owned fixture\n")
        if self.system == "linux":
            header = bytearray(64)
            header[:6] = b"\x7fELF\x02\x01"
            header[18:20] = (62).to_bytes(2, "little")
            (self.qt / "lib/libQt6Core.so.6").write_bytes(header)
        elif self.system == "windows":
            header = bytearray(64)
            header[:2] = b"MZ"
            header[60:64] = (64).to_bytes(4, "little")
            (self.qt / "bin/Qt6Core.dll").write_bytes(header + b"PE\0\0\x64\x86")
        (self.qt / "sbom").mkdir(exist_ok=True)
        for repo in ["qtbase", "qtdeclarative", "qtsvg"]:
            doc = {"spdxVersion": "SPDX-2.3", "documentNamespace": "urn:owned:" + repo,
                   "packages": [{"name": repo, "versionInfo": "6.8.2", "SPDXID": "SPDXRef-" + repo}]}
            (self.qt / "sbom" / (repo + "-6.8.2.spdx.json")).write_text(json.dumps(doc))
        return self.qt

    def check(self, stage="build", **kwargs):
        return environment.check_environment(kwargs.pop("qt_prefix", self.qt), stage=stage,
                                             system=self.system, env=self.variables, **kwargs)

    def failed(self, report):
        return {check["name"] for check in report["checks"] if check["status"] == "failed"}

    def test_native_builds_need_no_ci_environment(self):
        for system, machine in [("macos", "arm64"), ("windows", "AMD64"), ("linux", "x86_64")]:
            with self.subTest(system=system):
                self.system, self.machine = system, machine
                self.qt = self.base / ("Qt-" + system)
                self.qt_sdk()
                report = self.check()
                self.assertEqual(report["status"], "passed", report)
                self.assertFalse(any("RUNNER" in key or key == "CI" for key in self.variables))
        self.assertTrue(all(path == "fixture-tools" for _, path in self.lookups))

    def test_prepare_allows_uninstalled_channel_and_qt_but_requires_launchers(self):
        self.overrides[("rustc", ("-vV",))] = (1, "", "toolchain not installed")
        self.overrides[("cargo", ("--version",))] = (1, "", "toolchain not installed")
        report = self.check("prepare", qt_prefix=None)
        self.assertEqual(report["status"], "passed", report)
        self.assertTrue(any(check["status"] == "warning" for check in report["checks"]))
        self.assertEqual(self.check("prepare")["status"], "passed")
        self.assertFalse(any(check["name"].startswith("qt.") for check in self.check("prepare")["checks"]))
        self.missing.update({"cargo", "rustc", "rustup"})
        report = self.check("prepare", qt_prefix=None)
        self.assertTrue({"tool.cargo", "tool.rustc", "tool.rustup"} <= self.failed(report))

    def test_package_catches_rust_path_before_license_collection(self):
        self.qt_sdk()
        self.missing.update({"cargo", "rustc"})
        report = self.check("package")
        self.assertEqual(report["status"], "failed")
        self.assertTrue({"tool.cargo", "tool.rustc", "cargo.metadata"} <= self.failed(report))
        self.assertFalse(any("collect-notices" in str(call) for call, _ in self.calls))
        self.assertFalse(any("macdeployqt" == Path(call[0]).name for call, _ in self.calls))

    def test_build_reports_wrong_rust_and_qt_versions_together(self):
        self.qt_sdk()
        self.overrides[("rustc", ("-vV",))] = (0, "release: 1.98.0\nhost: aarch64-apple-darwin", "")
        self.overrides[("cargo", ("--version",))] = (0, "cargo 1.98.0", "")
        self.overrides[("qmake", ("-query", "QT_VERSION"))] = (0, "6.9.0", "")
        self.assertTrue({"version.rustc", "version.cargo", "qt.version"} <= self.failed(self.check()))

    def test_architecture_and_sdk_mismatches_fail_before_configuration(self):
        self.qt_sdk()
        self.machine = "x86_64"
        self.overrides[("clang++", ("-dumpmachine",))] = (0, "x86_64-apple-darwin", "")
        self.overrides[("xcrun", ("--sdk", "macosx", "--show-sdk-version"))] = (0, "13.3", "")
        self.overrides[("lipo", ("-archs", str(self.qt / "lib/QtCore.framework/Versions/A/QtCore")))] = (0, "x86_64", "")
        self.assertTrue({"host.architecture", "compiler.target", "macos.sdk", "qt.architecture"} <= self.failed(self.check()))

    def test_windows_wrong_shell_and_qt_architecture_are_rejected(self):
        self.system, self.machine = "windows", "AMD64"
        self.qt_sdk()
        self.variables["VSCMD_ARG_TGT_ARCH"] = "x86"
        self.variables.pop("WindowsSdkDir")
        core = self.qt / "bin/Qt6Core.dll"
        data = core.read_bytes()
        core.write_bytes(data[:-2] + b"\x4c\x01")
        self.assertTrue({"compiler.target", "windows.sdk", "qt.architecture"} <= self.failed(self.check()))

    def test_copied_windows_environment_has_uppercase_sdk_keys(self):
        self.system, self.machine = "windows", "AMD64"
        self.variables = {key.upper(): value for key, value in self.variables.items()}
        self.qt_sdk()
        self.assertEqual(self.check()["status"], "passed")

    def test_missing_qml_or_platform_plugin_is_not_a_complete_sdk(self):
        self.qt_sdk()
        (self.qt / "libexec/qmlimportscanner").unlink()
        (self.qt / "plugins/platforms/libqcocoa.dylib").unlink()
        self.assertTrue({"qt.libexec/qmlimportscanner", "qt.platform-plugin"} <= self.failed(self.check()))

    def test_partial_qt_development_modules_fail_before_cmake(self):
        self.qt_sdk()
        (self.qt / "lib/cmake/Qt6Widgets/Qt6WidgetsConfig.cmake").unlink()
        (self.qt / "lib/cmake/Qt6QuickTest/Qt6QuickTestConfig.cmake").unlink()
        self.assertTrue({"qt.cmake.Widgets", "qt.cmake.QuickTest"} <= self.failed(self.check()))

    def test_linux_missing_graphics_development_packages_fail_before_configuration(self):
        self.system, self.machine = "linux", "x86_64"
        self.qt_sdk()
        self.overrides[("pkg-config", ("--exists", "gl", "egl", "xkbcommon"))] = (1, "", "missing .pc files")
        self.assertEqual(self.failed(self.check()), {"linux.graphics-devel"})
        self.missing.add("pkg-config")
        self.assertEqual(self.failed(self.check("prepare")), {"tool.pkg-config", "linux.graphics-devel"})

    def test_minimum_cmake_and_missing_compiler_fail_as_one_report(self):
        self.qt_sdk()
        self.overrides[("cmake", ("--version",))] = (0, "cmake version 3.23.5", "")
        self.missing.add("clang++")
        self.assertTrue({"version.cmake", "tool.clang++", "compiler.target"} <= self.failed(self.check()))

    def test_package_uses_selected_python_and_offline_locked_metadata(self):
        self.qt_sdk()
        python = self.base / "packaging-python"
        report = self.check("package", python_executable=python)
        self.assertEqual(report["status"], "passed", report)
        self.assertTrue(any(command[0] == str(python) and command[1] == "-c" for command, _ in self.calls))
        metadata = next(command for command, _ in self.calls if command[1:2] == ["metadata"])
        self.assertEqual(metadata[1:], ["metadata", "--locked", "--offline", "--format-version", "1",
                                        "--filter-platform", "aarch64-apple-darwin"])
        self.assertTrue(all(kwargs["env"]["RUSTUP_AUTO_INSTALL"] == "0" and
                            kwargs["env"]["CARGO_NET_OFFLINE"] == "true" for _, kwargs in self.calls))
        self.assertNotIn("RUSTUP_AUTO_INSTALL", self.variables)
        self.assertEqual(self.variables["PATH"], "fixture-tools")

    def test_missing_metadata_cache_and_python_packages_are_early_failures(self):
        self.qt_sdk()
        original = self.run_command
        def failing(command, **kwargs):
            if command[1:2] == ["metadata"]:
                return subprocess.CompletedProcess(command, 101, "", "offline cache missing: /private/home/secret")
            if command[1:2] == ["-c"]:
                return subprocess.CompletedProcess(command, 0, '{"python":[3,12,9],"jsonschema":"4.22.0"}', "")
            return original(command, **kwargs)
        with patch.object(environment.subprocess, "run", side_effect=failing):
            report = self.check("package")
        self.assertTrue({"cargo.metadata", "python.jsonschema", "python.referencing"} <= self.failed(report))
        self.assertNotIn("/private/home/secret", json.dumps(report))
        self.assertNotIn(str(self.base), json.dumps(report))

    def test_metadata_warning_does_not_corrupt_valid_json(self):
        self.qt_sdk()
        arguments = ("metadata", "--locked", "--offline", "--format-version", "1", "--filter-platform", "aarch64-apple-darwin")
        self.overrides[("cargo", arguments)] = (0, '{"packages":[{"name":"fixture"}]}', "warning: unused manifest key")
        self.assertEqual(self.check("package")["status"], "passed")

    def test_package_rejects_missing_or_corrupt_pinned_materials(self):
        self.qt_sdk()
        (self.repository / "build/review/qt-license-cache/Owned-1.0.txt").write_bytes(b"tampered")
        (self.repository / "build/review/sbom-schemas/spdx.schema.json").unlink()
        report = self.check("package")
        self.assertTrue({"cache.qt-license-cache", "cache.sbom-schemas"} <= self.failed(report))
        self.assertNotIn(str(self.repository), json.dumps(report))
        self.assertTrue(all("prepare --packaging" in check.get("hint", "") for check in report["checks"]
                            if check["name"].startswith("cache.") and check["status"] == "failed"))

    def test_package_validates_real_qt_spdx_parser_before_deployment(self):
        self.qt_sdk()
        (self.qt / "sbom/qtsvg-6.8.2.spdx.json").unlink()
        self.assertIn("qt.spdx", self.failed(self.check("package")))
        (self.qt / "sbom/qtsvg-6.8.2.spdx.json").write_text('{"spdxVersion":"SPDX-2.2"}')
        self.assertIn("qt.spdx", self.failed(self.check("package")))

    def test_verify_needs_schema_cache_but_not_qt_licenses_or_sdk(self):
        (self.repository / "build/review/qt-license-cache/Owned-1.0.txt").unlink()
        self.assertEqual(self.check("verify", qt_prefix=None)["status"], "passed")
        (self.repository / "build/review/sbom-schemas/bom-1.6.schema.json").write_bytes(b"corrupt")
        self.assertEqual(self.failed(self.check("verify", qt_prefix=None)), {"cache.sbom-schemas"})

    def test_verify_does_not_depend_on_build_compiler_rust_or_qt_sdk(self):
        self.missing.update({"cargo", "rustc", "rustup", "cmake", "ninja", "clang++", "git"})
        report = self.check("verify", qt_prefix=None)
        self.assertEqual(report["status"], "passed", report)
        self.assertTrue(all(command[1:2] == ["-c"] for command, _ in self.calls))
        self.missing.add("codesign")
        self.assertEqual(self.failed(self.check("verify", qt_prefix=None)), {"tool.codesign"})

    def test_linux_rejects_arm64_qt_without_running_the_sdk_library(self):
        self.system, self.machine = "linux", "x86_64"
        self.qt_sdk()
        core = self.qt / "lib/libQt6Core.so.6"
        data = bytearray(core.read_bytes())
        data[18:20] = (183).to_bytes(2, "little")
        core.write_bytes(data)
        self.assertEqual(self.failed(self.check()), {"qt.architecture"})
        self.assertFalse(any(command[0] == str(core) for command, _ in self.calls))

    def test_linux_verify_checks_reachable_x64_docker_without_build_tools(self):
        self.system, self.machine = "linux", "x86_64"
        self.assertEqual(self.check("verify", qt_prefix=None)["status"], "passed")
        self.overrides[("docker", ("info", "--format", "{{.OSType}}/{{.Architecture}}"))] = (1, "", "permission denied: /private/socket")
        report = self.check("verify", qt_prefix=None)
        self.assertEqual(self.failed(report), {"docker.daemon"})
        self.assertNotIn("/private/socket", json.dumps(report))

    def test_timeout_and_raw_tool_failures_remain_bounded(self):
        self.qt_sdk()
        self.overrides[("cmake", ("--version",))] = subprocess.TimeoutExpired("cmake /private/home/name", 20)
        self.overrides[("rustc", ("-vV",))] = FileNotFoundError("/private/home/name")
        report = self.check()
        self.assertTrue({"version.cmake", "version.rustc", "rust.host"} <= self.failed(report))
        self.assertNotIn("/private/home/name", json.dumps(report))

    def test_unknown_system_and_stage_are_explicit(self):
        report = environment.check_environment(stage="build", system="other", env={})
        self.assertEqual(self.failed(report), {"platform"})
        with self.assertRaisesRegex(ValueError, "Unknown desktop stage"):
            environment.check_environment(stage="install")


if __name__ == "__main__":
    unittest.main()
