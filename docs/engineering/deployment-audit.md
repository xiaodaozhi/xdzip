# Deployment dependency verification and SBOM

Deployment verification generates development-review materials by default, explicitly marked `incomplete`; it does not automatically establish release readiness. Official Release separately collects materials for the same candidate and supplies them with the package after release-process review. The format uses the [official CycloneDX 1.6 schema](https://github.com/CycloneDX/specification/blob/55343ba19dee1785acf1ce9191540d5fd7b590db/schema/bom-1.6.schema.json). The upstream commit and SHA-256 of three schema files are pinned in [configuration](../../scripts/sbom-schemas.json). Validation downloads and parses only these resources, caching them in build/; it does not download schemas from arbitrary URLs in an SBOM.

| Artifact | Data source and purpose | What it cannot establish |
| --- | --- | --- |
| source-build.cdx.json | Cargo host-resolved workspace dependency graph including development/build tools; declared Qt modules; font and brand source hashes | That a Rust package is necessarily statically linked into the product, or that every Qt plugin/embedded third party is identified |
| deployment-audit.json | File SHA-256 under the specified actual artifact root, native binary imports, SDK/system/unresolved dependencies, and actual CLI build identity | Every dynamically named QML/plugin load, clean installation, system ABI, signatures, or license compliance |
| artifact-files.cdx.json | Actual artifacts and observed external SDK files, known edges from native imports, bound to source inventory and actual executable version/commit/dirty state | Complete deployed component identity, static-link attribution, Qt embedded-library licensing, or a completed formal product SBOM |
| Optional desktop records | Executed desktop checks and limitations according to the change and available environment | Unexecuted actions, other platforms, or release approval |

Source records do not include Cargo's path-based opaque IDs or fabricate crates.io purls for workspace packages. Artifact paths are relative to the specified root or Qt SDK. Other external dependencies are identified by content hash and filename, without private SDK absolute paths. Fonts use OFL-1.1; original brand sources retain the asset library's Apache-2.0. Declaring Qt modules does not infer licenses of every actually deployed component.

## Automated verification

`deployment_audit.py` checks only explicitly specified trusted project builds, not user archives. macOS uses otool for imports and RPATH; Windows uses MSVC dumpbin for ordinary and delayed DLL imports; Linux uses ldd, which must not be used on executables of unknown origin. A file symlink escaping the specified artifact root fails verification. Reports must be written outside the root to avoid including their own output in the inventory.

- `--mode sdk` permits resolution to an external Qt SDK and explicitly shows that the build is not standalone. Unresolved imports still fail. Recursion covers the artifact and declared Qt SDK; other external libraries are recorded only by hash and untraversed count, not considered closure-verified.
- `--mode deployed` disallows non-system external dependencies and unresolved imports. Windows VC++ imports such as MSVCP and VCRUNTIME must come from the package; developer-machine System32 cannot hide missing files. System UCRT and API contracts remain system-provided. Continue native startup smoke tests with Qt development search paths cleared.
- Unresolved imports, source versus actual binary version/commit/dirty mismatch, official schema hash/format mismatch, and duplicate or dangling component references block the relevant check.
- Native import closure does not replace coverage for plugins loaded by name or clean-machine checks. Passing imports from a deployed Windows directory is not installer acceptance.

The on-demand three-platform Qt workflow saves two root cdx.json files and deployment-audit.json without automatically generating blank manual forms. macOS reads only the locally built .app and SDK; ordinary checks do not run macdeployqt, sign, compress, or upload app packages. Linux supplies Qt/QML/plugins through an [independent runtime directory](linux-deployment.md), using deployed verification and an SDK-free container; downloaded tar.gz retains executable bits. Only the Ubuntu 24.04, x86_64, X11 baseline is currently declared, not a universal Linux installer.

For trusted project-owned Windows artifacts, independently extract the installer and compare its app directory with the same candidate's portable ZIP using `python scripts/check-windows-package-payload.py --installer-app <extracted-app-directory> --portable <portable-package.zip> --output build/review/payload-comparison.json`. The check compares per-file hashes, the installation manifest, and SDK graphics DLLs that must not be included. The output path must be new. This static comparison does not run the installer or an update and does not replace actual installation verification.

Local macOS example, after rebuilding from consistent source:

```sh
python3 scripts/collect-notices.py build/review/wp12/notices
python3 scripts/deployment_audit.py \
  --root build/ui/apps/desktop/xdzip.app \
  --app build/ui/apps/desktop/xdzip.app/Contents/MacOS/xdzip \
  --qt-prefix build/toolchain/Qt/6.8.2/macos --mode sdk \
  --output build/review/wp12/deployment-audit.json
python3 scripts/sbom.py \
  --audit build/review/wp12/deployment-audit.json \
  --source build/review/wp12/notices/source-build.cdx.json \
  --output build/review/wp12/artifact-files.cdx.json
```

After installing jsonschema==4.23.0 in an isolated verification environment, use that environment's Python to run `scripts/validate-sbom.py` with the two cdx.json files above. See the [jsonschema 4.23 documentation](https://python-jsonschema.readthedocs.io/en/v4.23.0/referencing/) for registry usage. Verification tools are not deployed into the product.

## Next release gate

[Actual component attribution](runtime-attribution.md) is integrated: SDK SPDX associates actual files, Qt modules, and embedded third parties; pinned-hash official license texts are delivered; SDK/upstream hash differences and unconfirmed attribution are recorded. Ordinary Rust runtime candidates are separated from build/development/proc-macro tools. New component relationships bind the actual audit and are merged into the artifact SBOM, which remains explicitly incomplete.

After obtaining an actual installed directory in an approved platform deployment environment, reverify and associate each DLL/framework/plugin with its upstream component, version, license, and corresponding source. Complete Qt embedded third parties, static Rust link attribution, relinking materials, and toolchain provenance. Release review also checks the same candidate's version, commit, artifact hashes, applicable clean-install evidence, and known-defect outcomes. Record actual conclusions under the [license review process](../qa/release-review.md). Historical `v0.1.0` candidate records are in [#4](https://github.com/xiaodaozhi/xdzip/issues/4) and [#53](https://github.com/xiaodaozhi/xdzip/issues/53); later candidates cannot inherit them automatically. Delivery follows the [personal open-source release policy](release-runbook.md) without requiring paid signing identity. Scripts do not automatically issue release conclusions or reopen completed WP-12 development acceptance.
