# W01 main-window welcome page, high-fidelity candidate

The high-fidelity design is confirmed and integrated into the [official HTML review entry](../../design/html/index.html?view=welcome&zoom=1). Qt implements real opening and creation; desktop manual acceptance is recorded separately. See the [welcome page state design](../../design/hifi/sources/welcome.html) and [welcome-reference.json](../../specs/visual/welcome-reference.json). Colors and main-window density retain existing tokens and layout-scale without changing global standards. Four PNGs are exports of code-drawn design originals, not product runtime screenshots.

## Information and layout

The start-page headline is “打包与解压，都简单一点。” in Chinese and “Packing and unpacking, made simple.” in English. Supporting copy describes browsing first, extracting as needed and packing to share.

Default client area is 1024×622, minimum 832×544; title area 48, capsule 32 and bottom bar 32. The content group is 608 wide, centered horizontally only and starts a fixed 28 logical pixels below the available body top, not vertically centered. Headline size is 30, or 28 in a narrow window; description 13, action title 16 and card description 12. Two action cards are 296 wide by 124 high, with 16 gap, 20 padding and radius 8. These are welcome-page entry cards, not replacements for shared operation-window buttons.

The title bar fully reuses the main window's existing components, retaining branding, start tab, add tab and appearance settings; the welcome page changes none of its layout, dimensions, fonts or icons. The body does not display an empty file tree, table header, search box or extraction toolbar; browsing layout appears after opening an archive. The primary action is “Open archive”, the secondary “Create archive”; a drop area and ZIP, TAR, TAR.GZ appear separately below. “Files are processed locally” in the bottom bar describes processing location, not a safety guarantee. The task count uses real history; zero in the design represents an empty first-use history.

Do not show example recent files, sample sizes or format capabilities the product lacks. Traffic lights in the title area indicate macOS decoration placement only; the implementation uses platform window facilities. Windows and Linux retain platform decorations without copying traffic lights.

## Interaction design

- The welcome page may appear at first launch, after closing the last archive tab, or after opening fails with no browsable archive. Background tasks continue and are not reset by the welcome page.
- Open invokes the system file picker; cancellation stays on the welcome page. Success switches to the real archive tab; failure displays a recoverable error and must not fabricate a successful list.
- Create opens the existing creation window; cancellation retains the welcome page. Candidate shortcuts are ⌘O/⌘N on macOS and Ctrl+O/Ctrl+N on Windows/Linux. Check actual application bindings before implementation; this design does not claim they are integrated.
- When one supported archive is dragged in, the dashed border becomes a 2-pixel theme accent, fill becomes selection and copy says “Release to open archive”. Leaving restores the default. Directories, multiple files and unsupported formats must not appear openable; dropping them gives clear feedback and never implicitly creates an archive.
- The entire card is clickable. Keyboard order after the title bar is open, create, tasks. Keyboard focus adds no outline and follows the window-wide focus-loss rule. Hover and pressed use shared accentHover and accentPressed, not opacity alone.
- While opening, the open card replaces its title with a busy indicator and “Opening…” and temporarily prevents repeated opening and drops; disabled appearance uses accentDisabled and accentDisabledText. Whether the creation window can run concurrently follows host rules; the welcome page does not change task semantics. Busy, disabled and error are implementation contracts not depicted in the four static originals.
- Settings and task popups retain width 240, font 13 and minimum row height 32, a 1:1 ratio with body font 13; the welcome page introduces no separate menu style. Card font 16 is an entry heading, not a menu-density comparison baseline.

## Verification and boundaries

Cover light/dark themes, minimum window, drag states and menu/title-bar proportions, checking DPR 1, 1.25 and 2 and actual browser page zoom. Do not substitute a downscaled screenshot for layout or CSS scaling for browser page zoom. Results, images and geometry reports belong only in local/CI artifacts; status is recorded in the corresponding issue.

HTML interaction checks use `scripts/check-html-welcome.py`; Qt uses welcome-page and browser regressions. HTML drag-in only selects examples and does not read user files; Qt opens files through the real product entry. A complete dark welcome page does not imply complete dark originals for every page, and automated checks do not replace native drag-and-drop, screen-reader or IME verification.

## Real Qt integration

WelcomePage and WelcomeAction express layout and intent only. The welcome page appears on startup and after closing an archive; successful opening enters the appropriate archive session. An empty archive remains a valid browsing state. Initial loading disables repeated opening and creation, displays “Opening…” and allows load cancellation. Failure retains the welcome page and actual error for reselection. Closing a creation draft creates no task. Background task lists are not reset by the welcome page.

Open and system shortcuts reuse chooseArchive; creation reuses CreateWindow. The native file picker remains a system window and cancellation creates no session. Qt routes by the saved tab/window mode, preferentially reuses an empty start page and activates existing sessions for duplicate archives.

Drop handling accepts only one local regular file whose name ends in ZIP, TAR, TAR.GZ or TGZ. QFileInfo checks input shape only and does not read archive content. openDroppedUrls reuses openPath; the existing Rust core still checks format, budgets, errors and source changes. A dropped extension does not prove valid contents. Directories, multiple entries and network URLs are explicitly rejected; no archive is implicitly created.

Dimensions come from welcome-reference.json into Theme through generate-qml-theme.py. Larger text grows cards and body; the body scrolls and automatically brings focused keyboard entries into view. Status and title areas grow independently without Item.scale. Qt uses shared PNG icons, with the existing dark/light-ink graphic for the primary action; runtime assets are not cut from originals.

Automation entries are tst_welcome.qml, welcomeOpensRealArchivesAndReturns in browser_test.cpp and scripts/check-qt-welcome.py. On-demand three-platform Qt workflows check real opening, closing, recovery from failure and 36 geometry items at DPR 1, 1.25 and 2, while also checking menu width 240 and font 13. Native drop, file picker, screen reader and IME are checked as needed; offscreen results are not presented as unperformed desktop operations.

Title-bar revision: welcome-specific title-bar styling from the initial version was removed. Qt ArchiveTabs was restored unchanged to its pre-integration component; HTML reuses the main window's existing source canvas and scale. The welcome page changes only the body. Old PNGs and hashes remain historical records; the revision is `screens/welcome-titlebar-*.png`, sourced from design/hifi/sources/welcome.html. Regression compares title bar, tabs and appearance-button geometry before and after opening an archive to prevent the empty state from changing the application frame again.

Loading state: the initial-open cancellation entry moved into a floating body overlay. Unknown progress uses an active progress bar without simulated percentages; see the [loading overlay contract](welcome-opening.md). Original welcome bitmaps are retained; the dynamic design provides `opening=1`.

Error/warning feedback appears below format hints using the shared danger color and status-warning icon. Showing, wrapping, clearing or replacing an error does not change the top coordinates of the headline and entry cards; long content scrolls in the body. Reopening the main window after closing it to the tray shows only one start page and does not cancel background tasks. The editable source adds an `error=1` state; old welcome PNGs retain historical bytes and must not replace the top-positioning contract with their former centered layout.
