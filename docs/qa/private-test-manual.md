# On-demand desktop verification

Choose desktop verification for the current change; there is no requirement to complete a fixed three-platform manual acceptance form for every commit, merge or candidate. Prefer relevant automated tests, then operate the actual application and inspect actual windows or screenshots in available environments. When a device, system or desktop permission is unavailable, record unperformed scope and why. Do not present offscreen, synthetic-input or virtual-screen results as human or physical-device passes.

First determine necessary scenarios under [acceptance scope, evidence and closure](README.md#验收范围证据和收口). Actual window inspection, screenshots, event assertions and recordings have different evidence boundaries and cannot impersonate one another; recording is not a default desktop-acceptance step. Stop once required scenarios are sufficiently verified and limitations recorded; do not add fixed manual matrices due to historical comments or executor preference. Scenarios explicitly required by the user or applicable specification must still be performed and cannot be skipped merely to shorten the process.

## Select checks

| Change affects | Actual scenarios to check as needed |
| --- | --- |
| UI layout and styling | Actual windows and menus, light/dark themes, both languages and font sizes, window decorations and system scaling. Automated QML window contracts, shared controls and bilingual multiple-font-size layouts remain pre-commit checks. |
| System interaction | Confirming and cancelling file pickers, file-manager drag-and-drop, window ownership and file associations. |
| Input and accessibility | IME pre-edit, keyboard focus and real screen readers. Automated regressions check Qt accessibility actions and selection models for radio buttons and checkboxes. |
| Installation, updates and performance | Check installation or updates using owned test data according to change scope; use reproducible workloads and actual measurements for performance. On-demand workflows can cover full installation lifecycles and three platforms. |

Observe only application-owned windows and owned test files. Do not upload passwords, user archives or full private paths. Record actual results and limitations in the corresponding issue; put screenshots and logs under `build/`, `dist/` or temporary Actions artifacts, not source control.

## Optional tools

- [Windows native visual diagnostics](../../.github/workflows/windows-native-visual.yml) automatically captures owned windows on a disposable hosted runner. It requires no maintainer clicks and does not establish physical-display verification.
- Native menu diagnostics can run `scripts/check-windows-menu-launcher.ps1` on a suitable Windows desktop; see [menu-qa-contract.json](../../tests/fixtures/native/menu-qa-contract.json) for sampling and thresholds. Full sampling need not be repeated for unrelated changes.
- `scripts/prepare-private-test.py` and `scripts/private-test-matrix.json` remain voluntary historical-record tools. Ordinary CI, on-demand Qt checks and release builds do not automatically generate or require this form.

Existing completed desktop acceptance remains in its corresponding issue and is not repeated because workflows change. At release, describe actual verification and known limitations under the [release protocol](../engineering/release-protocol.md); licensing review and release decisions remain separate.
