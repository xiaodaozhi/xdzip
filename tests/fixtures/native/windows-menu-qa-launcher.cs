// tests/fixtures/native/windows-menu-qa-launcher.cs

using System;
using System.Collections;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Text;
using System.Text.RegularExpressions;
using System.Web.Script.Serialization;
using System.Windows.Forms;

namespace XdzipOwnedMenuQa {
    // Native actions use only this Form's HWND and the PID/handle of its directly
    // started child. Foreground reads inspect identity, never foreign content.
    // Query/grant failures are recorded; samples still require real foreground.
    // Empty paths fail closed, and an already-exited child is never terminated.
    internal static class Native {
        [DllImport("user32.dll")] internal static extern IntPtr GetForegroundWindow();
        [DllImport("user32.dll", SetLastError = true)] internal static extern bool SetForegroundWindow(IntPtr hwnd);
        [DllImport("user32.dll", SetLastError = true)] internal static extern bool AllowSetForegroundWindow(uint pid);
        [DllImport("user32.dll")] internal static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
        [DllImport("kernel32.dll", SetLastError = true)] internal static extern bool TerminateProcess(IntPtr process, uint code);
        [DllImport("kernel32.dll", EntryPoint = "QueryFullProcessImageNameW", CharSet = CharSet.Unicode, SetLastError = true)]
        internal static extern bool QueryFullProcessImageName(IntPtr process, uint flags, StringBuilder image, ref uint size);
    }

    public sealed class MenuQaOptions {
        public string Root, Tests, Source, Hash, Output, ContractPath;
        public bool CoreOnly;
        public MenuQaContract Contract;
        public string QtBin;

        public static MenuQaOptions Parse(string[] arguments, bool fixture) {
            var values = new Dictionary<string, string>();
            bool core = false;
            for (int i = 0; i < arguments.Length; ++i) {
                string name = arguments[i];
                if (fixture && name == "--core-only") {
                    if (core) throw new ArgumentException("Duplicate core-only option.");
                    core = true; continue;
                }
                if (name != "--root" && name != "--tests" && name != "--source" && name != "--hash" &&
                    name != "--output" && name != "--contract") throw new ArgumentException("Unknown option.");
                if (values.ContainsKey(name) || ++i >= arguments.Length || String.IsNullOrWhiteSpace(arguments[i]))
                    throw new ArgumentException("Missing or duplicate option.");
                values.Add(name, arguments[i]);
            }
            foreach (string name in new[] {"--root", "--tests", "--source", "--hash", "--output"})
                if (!values.ContainsKey(name)) throw new ArgumentException("Required option is missing.");
            var options = new MenuQaOptions {
                Root = Path.GetFullPath(values["--root"]), Tests = Path.GetFullPath(values["--tests"]),
                Output = Path.GetFullPath(values["--output"]), Source = values["--source"], Hash = values["--hash"], CoreOnly = core
            };
            if (IntPtr.Size != 8 || !Regex.IsMatch(options.Source, "^[0-9a-f]{40}$") || !Regex.IsMatch(options.Hash, "^[0-9a-f]{64}$"))
                throw new ArgumentException("x64, full source SHA and binary SHA256 are required.");
            if (!Directory.Exists(options.Root) || (!Directory.Exists(Path.Combine(options.Root, ".git")) && !File.Exists(Path.Combine(options.Root, ".git"))))
                throw new ArgumentException("Repository root is required.");
            RequireBuildChild(options.Root, options.Tests);
            RequireBuildChild(options.Root, options.Output);
            if (!String.Equals(Path.GetFileName(options.Tests), "xdzip-browser-tests.exe", StringComparison.OrdinalIgnoreCase))
                throw new ArgumentException("Use the browser test executable.");
            string canonicalContract = Path.Combine(options.Root, "tests", "fixtures", "native", "menu-qa-contract.json");
            options.ContractPath = values.ContainsKey("--contract") ? Path.GetFullPath(values["--contract"]) : canonicalContract;
            if (!String.Equals(options.ContractPath, canonicalContract, StringComparison.OrdinalIgnoreCase))
                throw new ArgumentException("Use the tracked repository contract.");
            options.Contract = MenuQaContract.Load(options.ContractPath);
            return options;
        }

        public static void RequireBuildChild(string root, string path) {
            string build = Path.GetFullPath(Path.Combine(root, "build"));
            string full = Path.GetFullPath(path);
            if (!full.StartsWith(build + Path.DirectorySeparatorChar, StringComparison.OrdinalIgnoreCase))
                throw new ArgumentException("Tests and evidence must stay below the repository build directory.");
            string current = full;
            while (current != null && current.Length >= build.Length) {
                if ((File.Exists(current) || Directory.Exists(current)) &&
                    (File.GetAttributes(current) & FileAttributes.ReparsePoint) != 0)
                    throw new ArgumentException("Build paths must not traverse a reparse point.");
                if (String.Equals(current, build, StringComparison.OrdinalIgnoreCase)) break;
                current = Path.GetDirectoryName(current);
            }
        }

        public static string Relative(string root, string path) {
            string prefix = Path.GetFullPath(root).TrimEnd(Path.DirectorySeparatorChar) + Path.DirectorySeparatorChar;
            string full = Path.GetFullPath(path);
            if (!full.StartsWith(prefix, StringComparison.OrdinalIgnoreCase)) throw new ArgumentException("Path is outside repository.");
            return full.Substring(prefix.Length).Replace('\\', '/');
        }

        private static string ReadGit(string root, string arguments) {
            var info = new ProcessStartInfo("git", arguments) {
                WorkingDirectory = root, UseShellExecute = false, CreateNoWindow = true,
                RedirectStandardOutput = true, RedirectStandardError = true
            };
            using (Process process = Process.Start(info)) {
                if (!process.WaitForExit(5000)) { process.Kill(); throw new InvalidOperationException("Source validation timed out."); }
                string text = process.StandardOutput.ReadToEnd();
                if (process.ExitCode != 0) throw new InvalidOperationException("Source validation failed.");
                return text.Trim();
            }
        }

        public void ValidateFull() {
            if (CoreOnly) throw new InvalidOperationException("Core-only configuration cannot start a launcher.");
            RequireBuildChild(Root, Tests); RequireBuildChild(Root, Output);
            if (ReadGit(Root, "rev-parse HEAD") != Source || ReadGit(Root, "status --porcelain").Length != 0)
                throw new InvalidOperationException("The expected source must be HEAD and clean.");
            if (!File.Exists(Tests) || MenuQaLauncher.HashFile(Tests) != Hash)
                throw new InvalidOperationException("Browser test SHA256 differs from the explicit binding.");
            string parent = Path.GetDirectoryName(Tests);
            string version = File.ReadAllText(Path.Combine(parent, "generated", "BuildVersion.h"));
            if (!Regex.IsMatch(version, "(?m)^#define XDZIP_REVISION \"" + Regex.Escape(Source.Substring(0, 12)) + "\"\\s*$") ||
                !Regex.IsMatch(version, "(?m)^#define XDZIP_DIRTY false\\s*$") ||
                !Regex.IsMatch(version, "(?m)^#define XDZIP_CONFIGURATION \"Release\"\\s*$"))
                throw new InvalidOperationException("Generated build metadata differs from the explicit binding.");
            QtBin = FindQtBin(parent);
        }

        private string FindQtBin(string testsParent) {
            if (File.Exists(Path.Combine(testsParent, "Qt6Core.dll"))) return testsParent;
            string build = Path.GetFullPath(Path.Combine(Root, "build"));
            for (string current = testsParent; current != null && current.StartsWith(build, StringComparison.OrdinalIgnoreCase); current = Path.GetDirectoryName(current)) {
                string cache = Path.Combine(current, "CMakeCache.txt");
                if (File.Exists(cache)) {
                    foreach (string line in File.ReadAllLines(cache)) {
                        if (!line.StartsWith("Qt6_DIR:PATH=", StringComparison.Ordinal)) continue;
                        string qt6 = Path.GetFullPath(line.Substring("Qt6_DIR:PATH=".Length));
                        string prefix = Path.GetFullPath(Path.Combine(qt6, "..", "..", ".."));
                        string bin = Path.Combine(prefix, "bin");
                        if (File.Exists(Path.Combine(bin, "Qt6Core.dll"))) return bin;
                    }
                    break;
                }
            }
            throw new InvalidOperationException("Qt DLLs were not found beside tests or through its CMake cache.");
        }

        public MenuQaOptions ForOutput(string directory) {
            RequireBuildChild(Root, directory);
            return new MenuQaOptions { Root=Root, Tests=Tests, Source=Source, Hash=Hash, Output=directory,
                ContractPath=ContractPath, Contract=Contract, CoreOnly=CoreOnly, QtBin=QtBin };
        }
    }

    public sealed class MenuQaContract {
        public string[] Cases, GenerationFields;
        public string ReferenceGeneration, CaptureStage, Configuration, Platform;
        public int Repetitions, GraphicsApi;
        public double P95Limit, MaxLimit;

        public static MenuQaContract Load(string path) {
            var data = new JavaScriptSerializer().Deserialize<Dictionary<string, object>>(File.ReadAllText(path));
            if (Convert.ToInt32(data["schema_version"]) != 1) throw new ArgumentException("Unsupported menu contract.");
            var cases = new List<string>();
            foreach (object item in (IEnumerable)data["cases"]) {
                string value = Convert.ToString(item);
                if (!Regex.IsMatch(value, "^[a-z0-9]+(?:-[a-z0-9]+)*$") || cases.Contains(value))
                    throw new ArgumentException("Invalid or duplicate menu case.");
                cases.Add(value);
            }
            var fields = new List<string>();
            foreach (object item in (IEnumerable)data["required_generation_fields"]) fields.Add(Convert.ToString(item));
            var limits = (Dictionary<string,object>)data["limits_ms"];
            var windows = (Dictionary<string,object>)data["windows"];
            var result = new MenuQaContract {
                Cases=cases.ToArray(), GenerationFields=fields.ToArray(), Repetitions=Convert.ToInt32(data["repetitions"]),
                ReferenceGeneration=Convert.ToString(data["reference_generation_field"]), P95Limit=Convert.ToDouble(limits["p95"]), MaxLimit=Convert.ToDouble(limits["max"]),
                CaptureStage=Convert.ToString(windows["capture_stage"]), GraphicsApi=Convert.ToInt32(windows["graphics_api"]),
                Configuration=Convert.ToString(windows["configuration"]), Platform=Convert.ToString(windows["platform"])
            };
            if (result.Cases.Length == 0 || result.Repetitions < 1 || result.Cases.Length * result.Repetitions > 1000 ||
                result.P95Limit <= 0 || result.P95Limit > 100 || result.MaxLimit < result.P95Limit || result.MaxLimit > 200 ||
                result.ReferenceGeneration != "first_visible_menu_sync_generation" || fields.Count != 3 ||
                !fields.Contains("captured_sync_generation") || !fields.Contains("capture_begin_sync_generation") || !fields.Contains("capture_end_sync_generation"))
                throw new ArgumentException("Menu contract weakens required frame or timing gates.");
            return result;
        }

        public bool Accepts(Dictionary<string,object> record, string source) {
            double reference = Convert.ToDouble(record[ReferenceGeneration]);
            foreach (string field in GenerationFields) if (Convert.ToDouble(record[field]) != reference) return false;
            double elapsed = Convert.ToDouble(record["trigger_to_complete_frame_ms"]);
            return reference > 0 && !Double.IsNaN(reference) && !Double.IsInfinity(reference) && reference == Math.Truncate(reference) &&
                Convert.ToBoolean(record["shadow_same_frame"]) && elapsed >= 0 && !Double.IsNaN(elapsed) && elapsed <= MaxLimit &&
                Convert.ToString(record["capture_stage"]) == CaptureStage && Convert.ToInt32(record["graphics_api"]) == GraphicsApi &&
                Convert.ToString(record["configuration"]) == Configuration && Convert.ToString(record["platform"]) == Platform &&
                Convert.ToString(record["revision"]) == source.Substring(0,12);
        }
    }

    internal static class MenuQaEntry {
        [STAThread]
        private static int Main(string[] arguments) {
            MenuQaOptions options = null;
            try {
                options = MenuQaOptions.Parse(arguments, false);
                options.ValidateFull();
                Application.EnableVisualStyles();
                Application.SetCompatibleTextRenderingDefault(false);
                using (var form = new MenuQaLauncher(options)) Application.Run(form);
                return 0;
            } catch (Exception error) {
                string report = new JavaScriptSerializer().Serialize(new Dictionary<string,object> {
                    {"status","launcher_start_failed"}, {"exception_type",error.GetType().FullName},
                    {"child_processes_started",0}, {"automatic_sampling",false}
                });
                if (options != null) {
                    Directory.CreateDirectory(options.Output);
                    File.WriteAllText(Path.Combine(options.Output,"launcher-start-error.json"),report);
                }
                Console.Error.WriteLine(report);
                return 1;
            }
        }
    }

    public sealed class MenuQaLauncher : Form {
        private readonly MenuQaOptions options;
        private readonly MenuQaContract contract;
        private string[] Cases { get { return contract.Cases; } }
        private int TotalSamples { get { return contract.Cases.Length * contract.Repetitions; } }
        private readonly string root, binary, binaryHash, sourceSha, output;
        private readonly JavaScriptSerializer serializer = new JavaScriptSerializer();
        private readonly List<Dictionary<string, object>> samples = new List<Dictionary<string, object>>();
        private readonly List<Dictionary<string, object>> launches = new List<Dictionary<string, object>>();
        private readonly List<Dictionary<string, object>> failures = new List<Dictionary<string, object>>();
        private readonly Button start = new Button();
        private readonly Label status = new Label();
        private readonly ProgressBar progress = new ProgressBar();
        private readonly Timer timer = new Timer();
        private FileStream binaryLock;
        private Process child;
        private IntPtr childHandle;
        private uint childPid;
        private bool childForegroundSeen, started, finished;
        private Stopwatch childClock;
        private int index;
        private string currentDirectory, currentCase, startedAt;
        private string currentStage = "waiting_user_click", closingReason, lastExceptionType;
        private Dictionary<string, object> currentLaunch;

        internal sealed class ChildIdentityObservation {
            internal string ImagePath, ExceptionType;
            internal bool Exited;
            internal int? ExitCode;
            internal int Win32Error;
        }

        internal static ChildIdentityObservation ObserveOwnedChildIdentity(Process ownedChild) {
            ChildIdentityObservation observation = new ChildIdentityObservation();
            if (!ownedChild.HasExited) {
                try {
                    StringBuilder image = new StringBuilder(32768);
                    uint size = (uint)image.Capacity;
                    if (Native.QueryFullProcessImageName(ownedChild.Handle, 0, image, ref size))
                        observation.ImagePath = image.ToString();
                    else observation.Win32Error = Marshal.GetLastWin32Error();
                } catch (InvalidOperationException error) {
                    observation.ExceptionType = error.GetType().FullName;
                    if (!ownedChild.HasExited) throw;
                }
            }
            observation.Exited = ownedChild.HasExited;
            if (observation.Exited) observation.ExitCode = ownedChild.ExitCode;
            return observation;
        }

        internal static bool ValidateObservedChildIdentity(ChildIdentityObservation observation,
                string expectedBinary, Action<string, int?, string> reject) {
            if (observation.Exited || String.IsNullOrWhiteSpace(observation.ImagePath)) {
                reject(observation.Exited ? "child_exited_before_identity" : "child_identity_unavailable",
                    observation.ExitCode, observation.ExceptionType);
                return false;
            }
            if (!String.Equals(Path.GetFullPath(observation.ImagePath), expectedBinary,
                               StringComparison.OrdinalIgnoreCase)) {
                reject("child_identity", observation.ExitCode, observation.ExceptionType);
                return false;
            }
            return true;
        }

        private void StartNextSampleSafely() {
            RunDeferred(StartNextSample, delegate(Exception error) {
                Fail("launcher_error", "验收入口发生异常，已停止并保留诊断。", error.GetType().FullName);
            });
        }

        public static string HashFile(string path) {
            using (FileStream stream = new FileStream(path, FileMode.Open, FileAccess.Read, FileShare.Read))
            using (SHA256 hash = SHA256.Create())
                return BitConverter.ToString(hash.ComputeHash(stream)).Replace("-", "").ToLowerInvariant();
        }

        internal static void RunDeferred(Action action, Action<Exception> reject) {
            try { action(); } catch (Exception error) { reject(error); }
        }

        public MenuQaLauncher(MenuQaOptions configuration) {
            options = configuration;
            options.ValidateFull();
            contract = options.Contract;
            root = options.Root; binary = options.Tests; binaryHash = options.Hash;
            sourceSha = options.Source; output = options.Output;
            if (File.Exists(Path.Combine(output, "review.json")))
                throw new InvalidOperationException("Use a fresh evidence directory.");
            Directory.CreateDirectory(output);
            // The read lease prevents replacement or modification while awaiting the user's click.
            binaryLock = new FileStream(binary, FileMode.Open, FileAccess.Read, FileShare.Read);
            serializer.MaxJsonLength = Int32.MaxValue;
            Text = "xdzip 菜单验收"; ClientSize = new Size(590, 280);
            AutoScaleMode = AutoScaleMode.Dpi; StartPosition = FormStartPosition.CenterScreen;
            FormBorderStyle = FormBorderStyle.FixedDialog; MaximizeBox = false;
            Font = new Font("Microsoft YaHei UI", 10);
            Label explanation = new Label();
            explanation.SetBounds(24, 20, 542, 84);
            explanation.Text = "点击下方按钮开始原生菜单验收，约 1–2 分钟。\r\n只会依次打开本任务的测试窗口，请等待完成。\r\n关闭本窗口可停止。";
            Controls.Add(explanation);
            Label identity = new Label(); identity.SetBounds(24, 108, 542, 28);
            identity.Font = new Font("Segoe UI", 9);
            identity.Text = "Source " + sourceSha.Substring(0, 12) + "   Binary " + binaryHash.Substring(0, 12);
            Controls.Add(identity);
            status.SetBounds(24, 143, 542, 32); status.Text = "已就绪，等待你点击。"; Controls.Add(status);
            progress.SetBounds(24, 182, 542, 16); progress.Maximum = TotalSamples; Controls.Add(progress);
            start.Text = "开始菜单验收"; start.SetBounds(178, 218, 234, 40);
            start.MouseClick += OnRealStartClick; Controls.Add(start);
            timer.Interval = 25; timer.Tick += OnTick;
            FormClosing += OnClosing;
            Shown += delegate {
                try {
                    using (Bitmap image = new Bitmap(Width, Height)) {
                        DrawToBitmap(image, new Rectangle(0, 0, Width, Height));
                        image.Save(Path.Combine(output, "launcher.png"), System.Drawing.Imaging.ImageFormat.Png);
                    }
                } catch (Exception error) {
                    File.WriteAllText(Path.Combine(output, "launcher-preview-unavailable.json"),
                        serializer.Serialize(new Dictionary<string, object> {{"error_type", error.GetType().FullName}, {"window_is_own", true}}));
                }
            };
            WriteReport("ready_waiting_for_user_click");
        }

        private void OnRealStartClick(object sender, MouseEventArgs e) {
            if (started || e.Button != MouseButtons.Left || Native.GetForegroundWindow() != Handle) return;
            try {
                options.ValidateFull();
                started = true; startedAt = DateTimeOffset.Now.ToString("o"); start.Enabled = false;
                WriteReport("running"); StartNextSampleSafely();
            } catch (Exception error) { Fail("start_validation", "启动核验未通过，已停止。", error.GetType().FullName); }
        }

        private void StartNextSample() {
            if (finished) return;
            currentStage = "validating_next_sample";
            if (index == TotalSamples) { Complete(); return; }
            if (Native.GetForegroundWindow() != Handle) { Fail("launcher_not_foreground", "本任务窗口未获得前台，已停止。"); return; }
            if (HashFile(binary) != binaryHash) { Fail("binary_changed", "测试程序校验不符，已停止。"); return; }
            currentCase = Cases[index % Cases.Length];
            currentDirectory = Path.Combine(output, String.Format("{0:00}-{1}", index / Cases.Length + 1, currentCase));
            Directory.CreateDirectory(currentDirectory);
            ProcessStartInfo info = new ProcessStartInfo(binary);
            info.Arguments = "nativeMenuFirstFrame:" + currentCase + " -v1 -o \"" + Path.Combine(currentDirectory, "qt.log") + ",txt\"";
            info.WorkingDirectory = Path.GetDirectoryName(binary); info.UseShellExecute = false; info.CreateNoWindow = true;
            info.EnvironmentVariables["PATH"] = options.QtBin + ";" + Environment.GetEnvironmentVariable("PATH");
            string temporary = Path.Combine(output, "temporary");
            Directory.CreateDirectory(temporary);
            info.EnvironmentVariables["TEMP"] = info.EnvironmentVariables["TMP"] = temporary;
            info.EnvironmentVariables["QT_QPA_PLATFORM"] = "windows";
            info.EnvironmentVariables["QSG_RHI_BACKEND"] = "d3d11";
            info.EnvironmentVariables["QSG_INFO"] = "1";
            info.EnvironmentVariables["XDZIP_MENU_FIRST_FRAME"] = "1";
            info.EnvironmentVariables["XDZIP_MENU_FRAME_OUTPUT"] = currentDirectory;
            info.EnvironmentVariables["XDZIP_SETTINGS_PATH"] = Path.Combine(currentDirectory, "settings.ini");
            info.EnvironmentVariables.Remove("QT_QUICK_BACKEND");
            info.EnvironmentVariables.Remove("QT_SCALE_FACTOR");
            info.EnvironmentVariables.Remove("QT_SCREEN_SCALE_FACTORS");
            currentStage = "starting_owned_child";
            currentLaunch = new Dictionary<string, object> {
                {"sample", index + 1}, {"menu", currentCase}, {"stage", currentStage},
                {"owned_child_pid", null}, {"binary_path_verified", false}, {"binary_sha256", binaryHash}
            };
            launches.Add(currentLaunch);
            WriteReport("running");
            child = Process.Start(info);
            childPid = (uint)child.Id;
            currentLaunch["owned_child_pid"] = childPid;
            childHandle = child.Handle;
            if (childPid == 0) { Fail("child_identity", "无法核对本任务测试进程，已停止。"); return; }
            childForegroundSeen = false;
            currentStage = "validating_owned_child_identity";
            currentLaunch["stage"] = currentStage;
            WriteReport("running");
            ChildIdentityObservation observation = ObserveOwnedChildIdentity(child);
            currentLaunch["child_exited_before_identity"] = observation.Exited;
            currentLaunch["child_exit_code_before_identity"] = observation.ExitCode;
            currentLaunch["identity_exception_type"] = observation.ExceptionType;
            currentLaunch["identity_win32_error"] = observation.Win32Error;
            if (!ValidateObservedChildIdentity(observation, binary, delegate(string reason, int? code, string type) {
                currentLaunch["child_exit_code_before_identity"] = code;
                Fail(reason, "无法核对本任务测试进程，已停止并保留退出诊断。", type);
            })) return;
            currentLaunch["binary_path_verified"] = true;
            currentStage = "granting_owned_child_foreground";
            currentLaunch["stage"] = currentStage;
            // The foreground launcher grants only its directly created, validated child PID.
            // A foreground-created child may already have taken foreground by this point.
            bool exitedBeforeGrant = child.HasExited;
            bool launcherForegroundBeforeGrant = Native.GetForegroundWindow() == Handle;
            bool allowed = Native.AllowSetForegroundWindow(childPid);
            int grantError = allowed ? 0 : Marshal.GetLastWin32Error();
            bool exitedAfterGrant = child.HasExited;
            uint foregroundAfterGrant;
            Native.GetWindowThreadProcessId(Native.GetForegroundWindow(), out foregroundAfterGrant);
            childForegroundSeen = foregroundAfterGrant == childPid;
            currentLaunch["allow_foreground_exact_child_pid"] = allowed;
            currentLaunch["allow_foreground_win32_error"] = grantError;
            currentLaunch["child_exited_before_grant"] = exitedBeforeGrant;
            currentLaunch["child_exited_after_grant"] = exitedAfterGrant;
            currentLaunch["child_exit_code_at_grant"] = exitedAfterGrant ? (object)child.ExitCode : null;
            currentLaunch["launcher_os_foreground_before_grant"] = launcherForegroundBeforeGrant;
            currentLaunch["launcher_os_foreground_after_grant"] = Native.GetForegroundWindow() == Handle;
            currentLaunch["child_os_foreground_immediately_after_grant"] = childForegroundSeen;
            currentStage = "waiting_owned_child_result";
            currentLaunch["stage"] = currentStage;
            // Failed redundant grants are diagnostic, never evidence that the child lacked eligibility.
            // Actual child foreground, native occlusion, first-frame and timing gates remain mandatory.
            childClock = Stopwatch.StartNew();
            status.Text = String.Format("正在验收 {0}/{1}：{2}", index + 1, TotalSamples, currentCase);
            WriteReport("running");
            timer.Start();
        }

        private void OnTick(object sender, EventArgs e) {
            try {
                uint foregroundPid;
                Native.GetWindowThreadProcessId(Native.GetForegroundWindow(), out foregroundPid);
                if (foregroundPid == childPid) childForegroundSeen = true;
                if (child.HasExited) {
                    timer.Stop(); int code = child.ExitCode; child.Dispose(); child = null; childHandle = IntPtr.Zero;
                    launches[launches.Count - 1]["child_os_foreground_seen"] = childForegroundSeen;
                    launches[launches.Count - 1]["exit_code"] = code;
                    string recordPath = Path.Combine(currentDirectory, currentCase + ".json");
                    if (code != 0 || !File.Exists(recordPath) || !childForegroundSeen) {
                        Fail("invalid_sample", "一项菜单验收未通过，已停止并保留记录。"); return;
                    }
                    Dictionary<string, object> record = serializer.Deserialize<Dictionary<string, object>>(File.ReadAllText(recordPath, Encoding.UTF8));
                    if (Convert.ToString(record["menu"]) != currentCase || !contract.Accepts(record, sourceSha)) {
                        Fail("record_contract", "菜单帧或版本核对未通过，已停止。"); return;
                    }
                    record["repetition"] = index / Cases.Length + 1; samples.Add(record);
                    ++index; progress.Value = index;
                    WriteReport("running");
                    BringToFront(); Native.SetForegroundWindow(Handle);
                    BeginInvoke(new Action(StartNextSampleSafely));
                } else if (childClock.ElapsedMilliseconds > 35000) {
                    Fail("child_timeout", "测试窗口等待超时，已停止。");
                }
            } catch (Exception error) { Fail("launcher_error", "验收入口发生异常，已停止并保留诊断。", error.GetType().FullName); }
        }

        private void Fail(string reason, string message, string exceptionType = null) {
            timer.Stop(); finished = true; start.Enabled = false;
            lastExceptionType = exceptionType;
            int? childExitCode = null;
            bool terminationAttempted = false;
            string cleanupExceptionType = null;
            if (child != null) {
                try {
                    if (child.HasExited) childExitCode = child.ExitCode;
                    else if (childHandle != IntPtr.Zero) {
                        terminationAttempted = true;
                        Native.TerminateProcess(childHandle, 123);
                    }
                } catch (InvalidOperationException error) { cleanupExceptionType = error.GetType().FullName; }
                child.Dispose(); child = null; childHandle = IntPtr.Zero;
            }
            if (currentLaunch != null) {
                currentLaunch["failure_stage"] = currentStage;
                currentLaunch["exit_code_before_failure"] = childExitCode;
                currentLaunch["termination_attempted"] = terminationAttempted;
            }
            failures.Add(new Dictionary<string, object> {
                {"sample", index < TotalSamples ? (object)(index + 1) : null}, {"menu", currentCase ?? ""},
                {"reason", reason}, {"stage", currentStage}, {"exception_type", exceptionType},
                {"child_exit_code", childExitCode}, {"termination_attempted", terminationAttempted},
                {"cleanup_exception_type", cleanupExceptionType}, {"closing_reason", closingReason}
            });
            status.Text = message; start.Text = "已停止"; WriteReport("failed");
            BringToFront(); Native.SetForegroundWindow(Handle);
        }

        private void Complete() {
            List<double> values = new List<double>();
            foreach (Dictionary<string, object> sample in samples) values.Add(Convert.ToDouble(sample["trigger_to_complete_frame_ms"]));
            values.Sort(); double p95 = values[(int)Math.Ceiling(values.Count * .95) - 1];
            if (p95 > contract.P95Limit || values[values.Count - 1] > contract.MaxLimit) { Fail("timing_budget", "计时门槛未通过，已保留结果。"); return; }
            options.ValidateFull();
            finished = true; status.Text = String.Format("完成 {0} 项，p95 {1:F1} ms，最大 {2:F1} ms。", TotalSamples, p95, values[values.Count - 1]);
            start.Text = "验收完成"; WriteReport("passed");
        }

        private void WriteReport(string state) {
            List<double> values = new List<double>();
            foreach (Dictionary<string, object> sample in samples) values.Add(Convert.ToDouble(sample["trigger_to_complete_frame_ms"]));
            values.Sort();
            Dictionary<string, object> report = new Dictionary<string, object> {
                {"status", state}, {"current_stage", currentStage}, {"closing_reason", closingReason}, {"last_exception_type", lastExceptionType},
                {"full_source_sha", sourceSha}, {"binary_sha256", binaryHash},
                {"fixed_binary", MenuQaOptions.Relative(root, binary)}, {"contract_sha256", HashFile(options.ContractPath)},
                {"started_at", startedAt}, {"updated_at", DateTimeOffset.Now.ToString("o")},
                {"started_by_real_foreground_mouse_click", started}, {"fresh_process_per_sample", true},
                {"foreground_grant", "AllowSetForegroundWindow exact directly-created validated child PID only"},
                {"foreground_acceptance", "Actual owned child foreground required; ASFW failure alone does not terminate child"},
                {"menu_warmup", false}, {"backend", "native D3D11"}, {"valid_samples", samples.Count},
                {"limits_ms", new Dictionary<string, object> {{"p95",contract.P95Limit},{"max",contract.MaxLimit}}},
                {"p95_ms", values.Count == 0 ? (object)null : values[(int)Math.Ceiling(values.Count * .95) - 1]},
                {"max_ms", values.Count == 0 ? (object)null : values[values.Count - 1]},
                {"records", samples}, {"owned_child_launches", launches}, {"failures", failures}
            };
            File.WriteAllText(Path.Combine(output, "review.json"), serializer.Serialize(report), new UTF8Encoding(false));
        }

        private void OnClosing(object sender, FormClosingEventArgs e) {
            closingReason = e.CloseReason.ToString();
            if (started && !finished) Fail("launcher_closing", "验收入口正在关闭，已停止并保留诊断。");
            if (!started && !finished) { finished = true; WriteReport("closed_before_user_start"); }
            timer.Stop();
            if (binaryLock != null) { binaryLock.Dispose(); binaryLock = null; }
        }
    }
}
