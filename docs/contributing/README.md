# Build and contribute to XDZip

English | [简体中文](../zh-CN/contributing/README.md)

1. Read the [contribution policy](../../CONTRIBUTING.md) and find the relevant issue or open one. Describe the intended change, affected platform and a small scenario you can verify.
2. Follow the [build guide](../engineering/developer-build.md) to prepare the pinned tools. Run `python3 scripts/desktop.py doctor`, then build and test the desktop target. For a CLI change use the separate `scripts/cli.py` entry point. On Windows, use `python` in the documented MSVC environment.
3. Work from the appropriate contract and its source entry below. Keep behavioral changes tied to an owned regression sample or a focused test that can reveal the defect.
4. In your PR, explain the change, exact scope tested and any outstanding platform checks. Update both language versions of affected reader docs.

| Change | Start with the implementation and contract | Verify |
| --- | --- | --- |
| Interface, settings or translation | `apps/desktop/ui`, `resources/i18n` and `specs/visual`. Read the [visual specification](../visual/design-standard.md) and [design workflow](../visual/design-workflow.md) before editing a window. | Run the matching component and locale checks. For native window behavior, exercise the affected window on the relevant platform and record it separately. |
| Format support | Start at `specs/engineering/archive-formats.json`, the matching `xdzip-codec-*` crate and runtime routing. Read the [format registry contract](../engineering/format-registry.md). | Use an owned fixture and check scanning, the claimed operation, and at least one format-specific rejection. Regenerate and verify registry outputs when the registry changes. |
| Archive task operation | Follow `BrowserController` and `RuntimeConnection` to `xdzip-runtime` and `xdzip-io`. Read the [runtime](../engineering/product-runtime.md) and [file security](../design/file-security.md) contracts. | Cover the affected result and its failure path, such as cancellation, conflicts or writes already committed. Use the [runtime worker tests](../../developers/qa/runtime-worker-tests.md) when a real worker is part of the path. |

For a map of the packages and runtime boundaries, see [Technical reference](../reference/README.md). For CI and release procedures used by the team, see the [maintainer handbook](../../developers/README.md).
