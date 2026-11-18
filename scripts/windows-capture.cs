// Native, lossless capture and scoped UI Automation for Windows capture recipes.
// HWNDs are borrowed and revalidated before/after readback. A failed guard never
// persists screen pixels; all GDI objects and thread DPI changes have scoped owners.
using Microsoft.Win32.SafeHandles;
using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Drawing;
using System.Drawing.Imaging;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using System.Security.Cryptography;
using System.Windows.Automation;

namespace Xdzip.WindowsCapture {
    public sealed class Result {
        public long Window;
        public string Title;
        public int Dpi, ClientWidth, ClientHeight, WindowWidth, WindowHeight;
        public int ClientLeft, ClientTop;
        public string ClientFile, WindowFile;
    }

    // Resolve fresh elements inside the exact HWND and process owned by this run.
    public static class Driver {
        private static AutomationElement Root(int process, long handle) {
            var root = AutomationElement.FromHandle(new IntPtr(handle));
            if (root == null || root.Current.ProcessId != process) throw new InvalidOperationException("Foreign UI Automation root");
            return root;
        }
        private static AutomationElement Find(int process, long handle, string name, string role) {
            var elements = Root(process, handle).FindAll(TreeScope.Descendants, new PropertyCondition(AutomationElement.NameProperty, name));
            AutomationElement match = null;
            foreach (AutomationElement element in elements) {
                var state = element.Current;
                if (state.ProcessId != process || state.IsOffscreen || !state.IsEnabled || state.BoundingRectangle.IsEmpty || (role != "*" && state.ControlType.ProgrammaticName != "ControlType." + role)) continue;
                if (match != null) throw new InvalidOperationException("Ambiguous owned control: " + name + " (" + role + ")");
                match = element;
            }
            return match;
        }
        public static bool Ready(int process, long handle, string name, string role) { return Find(process, handle, name, role) != null; }
        public static void SetValue(int process, long handle, string name, string value) {
            var element = Find(process, handle, name, "Edit");
            if (element == null) throw new InvalidOperationException("Missing owned editable field: " + name);
            element.SetFocus();
            ((ValuePattern)element.GetCurrentPattern(ValuePattern.Pattern)).SetValue(value);
        }
        public static void Focus(int process, long handle, string name, string role) {
            var element = Find(process, handle, name, role);
            if (element == null) throw new InvalidOperationException("Missing owned focus target: " + name);
            element.SetFocus();
        }
        public static bool ReadyObject(int process, long handle, string name, string objectName) {
            var elements = Root(process, handle).FindAll(TreeScope.Descendants, new PropertyCondition(AutomationElement.NameProperty, name));
            int matches = 0;
            foreach (AutomationElement element in elements) {
                var state = element.Current;
                if (state.ProcessId == process && !state.IsOffscreen && !state.BoundingRectangle.IsEmpty && state.AutomationId.EndsWith("." + objectName, StringComparison.Ordinal)) matches++;
            }
            return matches == 1;
        }
        public static void Invoke(int process, long handle, string name, string role) {
            var element = Find(process, handle, name, role);
            if (element == null) throw new InvalidOperationException("Missing owned control: " + name);
            object pattern;
            if (element.TryGetCurrentPattern(InvokePattern.Pattern, out pattern)) ((InvokePattern)pattern).Invoke();
            else if (element.TryGetCurrentPattern(SelectionItemPattern.Pattern, out pattern)) ((SelectionItemPattern)pattern).Select();
            else throw new InvalidOperationException("Owned control has no invoke/selection action: " + name);
        }
        public static void ClickNamed(int process, long handle, string name, string role) {
            var element = Find(process, handle, name, role);
            if (element == null) throw new InvalidOperationException("Missing owned control: " + name);
            var bounds = element.Current.BoundingRectangle;
            Native.Click(process, handle, (int)(bounds.Left + bounds.Width / 2), (int)(bounds.Top + bounds.Height / 2));
        }
        public static bool Selected(int process, long handle, string name, string role) {
            var element = Find(process, handle, name, role);
            if (element == null) return false;
            object pattern;
            if (element.TryGetCurrentPattern(TogglePattern.Pattern, out pattern)) return ((TogglePattern)pattern).Current.ToggleState == ToggleState.On;
            if (element.TryGetCurrentPattern(SelectionItemPattern.Pattern, out pattern)) return ((SelectionItemPattern)pattern).Current.IsSelected;
            return false;
        }
        public static string Snapshot(int process, long handle) {
            var text = new StringBuilder();
            var elements = Root(process, handle).FindAll(TreeScope.Descendants, Condition.TrueCondition);
            foreach (AutomationElement element in elements) {
                var state = element.Current;
                if (state.ProcessId == process && !state.IsOffscreen) text.AppendLine(state.ControlType.ProgrammaticName + " | " + state.Name + " | " + state.AutomationId + " | enabled=" + state.IsEnabled + " | " + state.BoundingRectangle);
            }
            return text.ToString();
        }
    }

    public static class Native {
        [StructLayout(LayoutKind.Sequential)] private struct Rect {
            public int Left, Top, Right, Bottom;
            public Rectangle Box { get { return Rectangle.FromLTRB(Left, Top, Right, Bottom); } }
        }
        [StructLayout(LayoutKind.Sequential)] private struct Point { public int X, Y; }
        [StructLayout(LayoutKind.Sequential)] private struct Message {
            public IntPtr Window;
            public uint Id;
            public UIntPtr WParam;
            public IntPtr LParam;
            public uint Time;
            public Point Position;
            public uint Private;
        }
        [StructLayout(LayoutKind.Sequential)] private struct Monitor {
            public int Size;
            public Rect Bounds, Work;
            public uint Flags;
        }
        private delegate bool WindowCallback(IntPtr window, IntPtr state);
        [DllImport("user32.dll")] private static extern IntPtr GetForegroundWindow();
        [DllImport("user32.dll")] private static extern bool SetForegroundWindow(IntPtr window);
        [DllImport("user32.dll")] private static extern bool BringWindowToTop(IntPtr window);
        [DllImport("user32.dll")] private static extern bool SetWindowPos(IntPtr window, IntPtr after, int x, int y, int width, int height, uint flags);
        [DllImport("user32.dll")] private static extern bool AttachThreadInput(uint first, uint second, bool attach);
        [DllImport("kernel32.dll")] private static extern uint GetCurrentThreadId();
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern SafeWaitHandle OpenProcess(uint access, bool inherit, uint process);
        [DllImport("kernel32.dll")] private static extern uint WaitForSingleObject(SafeWaitHandle handle, uint milliseconds);
        [DllImport("user32.dll")] private static extern bool SetCursorPos(int x, int y);
        [DllImport("user32.dll")] private static extern bool GetCursorPos(out Point point);
        [DllImport("user32.dll")] private static extern IntPtr WindowFromPoint(Point point);
        [StructLayout(LayoutKind.Sequential)] private struct MouseInput { public int X, Y; public uint Data, Flags, Time; public UIntPtr Extra; }
        [StructLayout(LayoutKind.Sequential)] private struct Input { public uint Type; public MouseInput Mouse; }
        [DllImport("user32.dll", SetLastError = true)] private static extern uint SendInput(uint count, Input[] inputs, int size);
        [DllImport("user32.dll")] private static extern uint GetWindowThreadProcessId(IntPtr window, out uint process);
        [DllImport("user32.dll")] private static extern bool EnumWindows(WindowCallback callback, IntPtr state);
        [DllImport("user32.dll")] private static extern bool IsWindowVisible(IntPtr window);
        [DllImport("user32.dll")] private static extern bool IsWindowEnabled(IntPtr window);
        [DllImport("user32.dll")] private static extern bool IsIconic(IntPtr window);
        [DllImport("user32.dll")] private static extern bool IsZoomed(IntPtr window);
        [DllImport("user32.dll")] private static extern bool GetClientRect(IntPtr window, out Rect rect);
        [DllImport("user32.dll")] private static extern bool ClientToScreen(IntPtr window, ref Point point);
        [DllImport("user32.dll")] private static extern bool GetWindowRect(IntPtr window, out Rect rect);
        [DllImport("user32.dll")] private static extern IntPtr GetWindow(IntPtr window, uint command);
        [DllImport("user32.dll")] private static extern uint GetDpiForWindow(IntPtr window);
        [DllImport("user32.dll", SetLastError = true)] private static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
        [DllImport("user32.dll", CharSet = CharSet.Unicode)] private static extern int GetWindowTextW(IntPtr window, StringBuilder text, int count);
        [DllImport("user32.dll")] private static extern IntPtr MonitorFromWindow(IntPtr window, uint flags);
        [DllImport("user32.dll", CharSet = CharSet.Unicode)] private static extern bool GetMonitorInfoW(IntPtr monitor, ref Monitor info);
        [DllImport("user32.dll")] private static extern bool GetLayeredWindowAttributes(IntPtr window, out uint key, out byte alpha, out uint flags);
        [DllImport("user32.dll", SetLastError = true)] private static extern bool RegisterHotKey(IntPtr window, int id, uint modifiers, uint key);
        [DllImport("user32.dll")] private static extern bool UnregisterHotKey(IntPtr window, int id);
        [DllImport("user32.dll")] private static extern bool PeekMessageW(out Message message, IntPtr window, uint first, uint last, uint remove);
        [DllImport("dwmapi.dll")] private static extern int DwmGetWindowAttribute(IntPtr window, uint attribute, out Rect rect, int size);
        [DllImport("dwmapi.dll")] private static extern int DwmGetWindowAttribute(IntPtr window, uint attribute, out uint value, int size);
        [DllImport("dwmapi.dll")] private static extern int DwmFlush();
        private const int CancelKey = 0x5845;

        public static bool ProcessMayBeRunning(int process) {
            const uint synchronize = 0x00100000;
            const int invalidParameter = 87;
            const uint waitObject = 0;
            using (var handle = OpenProcess(synchronize, false, checked((uint)process))) {
                // A signaled handle confirms exit even when the process is still listed.
                // Keep the guard in place when Windows denies access or the wait fails.
                if (handle.IsInvalid) return Marshal.GetLastWin32Error() != invalidParameter;
                return WaitForSingleObject(handle, 0) != waitObject;
            }
        }

        private sealed class DpiScope : IDisposable {
            private readonly IntPtr previous;
            public DpiScope() {
                previous = SetThreadDpiAwarenessContext(new IntPtr(-4));
                if (previous == IntPtr.Zero) throw new Win32Exception(Marshal.GetLastWin32Error(), "Per-monitor DPI context unavailable");
            }
            public void Dispose() { SetThreadDpiAwarenessContext(previous); }
        }
        private static string Title(IntPtr window) {
            var text = new StringBuilder(1024);
            GetWindowTextW(window, text, text.Capacity);
            return text.ToString();
        }
        private static bool Owned(IntPtr window, int process) {
            uint owner;
            return window != IntPtr.Zero && GetWindowThreadProcessId(window, out owner) != 0 && owner == process;
        }
        private static Rectangle Client(IntPtr window) {
            Rect rect;
            var point = new Point();
            if (!GetClientRect(window, out rect) || !ClientToScreen(window, ref point))
                throw new InvalidOperationException("Cannot read client geometry");
            return new Rectangle(point.X, point.Y, rect.Right - rect.Left, rect.Bottom - rect.Top);
        }
        private static Rectangle Frame(IntPtr window) {
            Rect rect;
            if (DwmGetWindowAttribute(window, 9, out rect, Marshal.SizeOf(typeof(Rect))) != 0)
                throw new InvalidOperationException("Cannot read visible DWM frame");
            return rect.Box;
        }
        private static bool Hidden(IntPtr window) {
            uint cloaked;
            if (!IsWindowVisible(window) || IsIconic(window)) return true;
            if (DwmGetWindowAttribute(window, 14, out cloaked, 4) == 0 && cloaked != 0) return true;
            uint key, flags;
            byte alpha;
            return GetLayeredWindowAttributes(window, out key, out alpha, out flags) && (flags & 2) != 0 && alpha == 0;
        }
        private static void Guard(IntPtr window, int process, Rectangle frame, Rectangle client, int dpi) {
            if (!Owned(window, process) || GetForegroundWindow() != window || Hidden(window) ||
                !IsWindowEnabled(window) || IsZoomed(window))
                throw new InvalidOperationException("The requested window is inactive, hidden, disabled or maximized");
            if (GetDpiForWindow(window) != dpi || Frame(window) != frame || Client(window) != client)
                throw new InvalidOperationException("Window geometry changed during capture; session stopped");
            var monitor = new Monitor { Size = Marshal.SizeOf(typeof(Monitor)) };
            if (!GetMonitorInfoW(MonitorFromWindow(window, 2), ref monitor) || !monitor.Work.Box.Contains(frame))
                throw new InvalidOperationException("The complete window must fit inside one monitor's work area");
            // Conservatively reject every intersecting visible window above ours,
            // including dialogs in the same process. No foreign pixels are saved.
            for (var above = GetWindow(window, 3); above != IntPtr.Zero; above = GetWindow(above, 3)) {
                if (Hidden(above)) continue;
                Rect other;
                if (DwmGetWindowAttribute(above, 9, out other, Marshal.SizeOf(typeof(Rect))) != 0 && !GetWindowRect(above, out other))
                    throw new InvalidOperationException("Cannot verify an overlapping window");
                if (frame.IntersectsWith(other.Box))
                    throw new InvalidOperationException("Another window covers the target; no pixels saved");
            }
        }
        public static bool DefaultSize(int width, int height, int dpi, int logicalWidth, int logicalHeight) {
            return dpi >= 96 && Math.Abs(width - logicalWidth * dpi / 96.0) <= 2 &&
                Math.Abs(height - logicalHeight * dpi / 96.0) <= 2;
        }
        public static long FindDefaultWindow(int process, int logicalWidth, int logicalHeight) {
            using (new DpiScope()) {
                var matches = new List<long>();
                EnumWindows((window, state) => {
                    if (Owned(window, process) && !Hidden(window) && GetWindow(window, 4) == IntPtr.Zero) {
                        var client = Client(window);
                        if (DefaultSize(client.Width, client.Height, (int)GetDpiForWindow(window), logicalWidth, logicalHeight))
                            matches.Add(window.ToInt64());
                    }
                    return true;
                }, IntPtr.Zero);
                return matches.Count == 1 ? matches[0] : 0;
            }
        }
        public static void RegisterKeys() {
            if (!RegisterHotKey(IntPtr.Zero, CancelKey, 0x4000, 0x79))
                throw new Win32Exception(Marshal.GetLastWin32Error(), "F10 is already registered by another app");
        }
        public static int PollKey() {
            Message message;
            while (PeekMessageW(out message, IntPtr.Zero, 0x0312, 0x0312, 1)) {
                if (message.WParam.ToUInt64() == CancelKey) return 2;
            }
            return 0;
        }
        public static void ReleaseKeys() {
            UnregisterHotKey(IntPtr.Zero, CancelKey);
        }
        public static void Activate(int process, long handle) {
            var window = new IntPtr(handle);
            if (!Owned(window, process) || Hidden(window) || IsZoomed(window))
                throw new InvalidOperationException("Owned default-size window unavailable");
            if (GetForegroundWindow() == window) return;
            SetForegroundWindow(window);
            if (GetForegroundWindow() == window) return;
            uint ignored;
            uint foregroundThread = GetWindowThreadProcessId(GetForegroundWindow(), out ignored);
            uint currentThread = GetCurrentThreadId();
            // Temporarily share input queues for activation only. Never send
            // input to the foreign foreground window, and always detach.
            bool attached = foregroundThread != 0 && foregroundThread != currentThread && AttachThreadInput(currentThread, foregroundThread, true);
            try { BringWindowToTop(window); SetForegroundWindow(window); }
            finally { if (attached) AttachThreadInput(currentThread, foregroundThread, false); }
            if (GetForegroundWindow() != window) throw new InvalidOperationException("Windows refused activation of the capture-owned window");
        }
        public static void Click(int process, long handle, int x, int y) {
            using (new DpiScope()) {
                var window = new IntPtr(handle);
                var point = new Point { X = x, Y = y };
                if (!Owned(window, process) || GetForegroundWindow() != window || !Client(window).Contains(x, y) || !Owned(WindowFromPoint(point), process))
                    throw new InvalidOperationException("Owned row is obscured or inactive; no click sent");
                if (!SetCursorPos(x, y)) throw new InvalidOperationException("Cannot position pointer on the owned row");
                if (GetForegroundWindow() != window || !Owned(WindowFromPoint(point), process)) throw new InvalidOperationException("Input target changed; no click sent");
                var inputs = new [] { new Input { Mouse = new MouseInput { Flags = 2 } }, new Input { Mouse = new MouseInput { Flags = 4 } } };
                if (SendInput(2, inputs, Marshal.SizeOf(typeof(Input))) != 2) throw new Win32Exception(Marshal.GetLastWin32Error(), "Owned row click was not delivered");
            }
        }
        public static void ParkPointer(int process, long handle) {
            using (new DpiScope()) {
                var window = new IntPtr(handle);
                if (!Owned(window, process)) throw new InvalidOperationException("Foreign window rejected");
                var monitor = new Monitor { Size = Marshal.SizeOf(typeof(Monitor)) };
                if (!GetMonitorInfoW(MonitorFromWindow(window, 2), ref monitor)) throw new InvalidOperationException("Monitor unavailable");
                var bounds = Frame(window);
                var work = monitor.Work.Box;
                var points = new [] { new System.Drawing.Point(work.Left + 2, work.Top + 2), new System.Drawing.Point(work.Right - 3, work.Bottom - 3) };
                foreach (var point in points) if (!bounds.Contains(point)) {
                    if (!SetCursorPos(point.X, point.Y)) throw new InvalidOperationException("Cannot move pointer outside capture");
                    return;
                }
                throw new InvalidOperationException("No pointer parking area outside the window");
            }
        }
        public static void FitPosition(int process, long handle) {
            using (new DpiScope()) {
                var window = new IntPtr(handle);
                if (!Owned(window, process)) throw new InvalidOperationException("Foreign window rejected");
                var monitor = new Monitor { Size = Marshal.SizeOf(typeof(Monitor)) };
                if (!GetMonitorInfoW(MonitorFromWindow(window, 2), ref monitor)) throw new InvalidOperationException("Monitor unavailable");
                var frame = Frame(window);
                var work = monitor.Work.Box;
                if (frame.Width > work.Width || frame.Height > work.Height) throw new InvalidOperationException("The default-size window is larger than the monitor work area");
                int x = Math.Max(work.Left, Math.Min(frame.Left, work.Right - frame.Width));
                int y = Math.Max(work.Top, Math.Min(frame.Top, work.Bottom - frame.Height));
                if (x == frame.X && y == frame.Y) return;
                Rect outer;
                if (!GetWindowRect(window, out outer) || !SetWindowPos(window, IntPtr.Zero, outer.Left + x - frame.Left, outer.Top + y - frame.Top, 0, 0, 0x15))
                    throw new InvalidOperationException("Cannot position the owned window inside its monitor");
            }
        }
        public static int[] Pointer() {
            using (new DpiScope()) {
                Point point;
                if (!GetCursorPos(out point)) throw new InvalidOperationException("Cannot read pointer position");
                return new [] { point.X, point.Y };
            }
        }
        public static void RestorePointer(int x, int y) { using (new DpiScope()) { SetCursorPos(x, y); } }
        public static long TitledWindow(int process, string title) {
            long match = 0;
            int count = 0;
            EnumWindows((window, state) => {
                if (Owned(window, process) && !Hidden(window) && Title(window) == title) { match = window.ToInt64(); count++; }
                return true;
            }, IntPtr.Zero);
            return count == 1 ? match : 0;
        }
        private static byte[] Png(Bitmap image) {
            using (var stream = new MemoryStream()) {
                image.Save(stream, ImageFormat.Png);
                return stream.ToArray();
            }
        }
        private static Bitmap ReadFrame(IntPtr window, int process, Rectangle frame, Rectangle client, int dpi) {
            Guard(window, process, frame, client, dpi);
            if (DwmFlush() != 0) throw new InvalidOperationException("DWM did not complete presentation");
            Guard(window, process, frame, client, dpi);
            // Screen GDI alpha is unspecified. RGB retains every color sample.
            var image = new Bitmap(frame.Width, frame.Height, PixelFormat.Format24bppRgb);
            try {
                using (var graphics = Graphics.FromImage(image))
                    graphics.CopyFromScreen(frame.Location, System.Drawing.Point.Empty, frame.Size, CopyPixelOperation.SourceCopy);
                Guard(window, process, frame, client, dpi);
                return image;
            } catch { image.Dispose(); throw; }
        }
        public static string Fingerprint(int process, long handle) {
            using (new DpiScope()) {
                var window = new IntPtr(handle);
                var frame = Frame(window);
                var client = Client(window);
                using (var image = ReadFrame(window, process, frame, client, (int)GetDpiForWindow(window)))
                using (var hash = SHA256.Create()) {
                    var data = image.LockBits(new Rectangle(0, 0, image.Width, image.Height), ImageLockMode.ReadOnly, PixelFormat.Format24bppRgb);
                    try {
                        var row = new byte[image.Width * 3];
                        for (int y = 0; y < image.Height; y++) {
                            Marshal.Copy(IntPtr.Add(data.Scan0, y * data.Stride), row, 0, row.Length);
                            hash.TransformBlock(row, 0, row.Length, row, 0);
                        }
                        hash.TransformFinalBlock(new byte[0], 0, 0);
                        return BitConverter.ToString(hash.Hash);
                    } finally { image.UnlockBits(data); }
                }
            }
        }
        private static void WriteNew(string path, byte[] bytes) {
            using (var file = new FileStream(path, FileMode.CreateNew, FileAccess.Write, FileShare.None))
                file.Write(bytes, 0, bytes.Length);
        }
        public static Result Capture(int process, long expectedWindow, string requiredTitle, int logicalWidth, int logicalHeight, string clientFile, string windowFile) {
            using (new DpiScope()) {
                var window = GetForegroundWindow();
                if (!Owned(window, process)) throw new InvalidOperationException("The capture-owned window is no longer active");
                var title = Title(window);
                if (window.ToInt64() != expectedWindow || (!String.IsNullOrEmpty(requiredTitle) && title != requiredTitle))
                    throw new InvalidOperationException("This is not the window requested for the current scene");
                var frame = Frame(window);
                var client = Client(window);
                int dpi = (int)GetDpiForWindow(window);
                if (client.Width <= 0 || client.Height <= 0 || !frame.Contains(client))
                    throw new InvalidOperationException("Invalid capture bounds");
                if ((logicalWidth > 0 || logicalHeight > 0) && !DefaultSize(client.Width, client.Height, dpi, logicalWidth, logicalHeight))
                    throw new InvalidOperationException("Window dimensions differ from the recipe's logical size");
                using (var full = ReadFrame(window, process, frame, client, dpi)) {
                    using (var ui = full.Clone(new Rectangle(client.X - frame.X, client.Y - frame.Y, client.Width, client.Height), PixelFormat.Format24bppRgb)) {
                        // No resize, JPEG, sharpening, cursor drawing or synthetic frame.
                        var uiBytes = Png(ui);
                        var frameBytes = Png(full);
                        Guard(window, process, frame, client, dpi);
                        WriteNew(clientFile, uiBytes);
                        WriteNew(windowFile, frameBytes);
                    }
                }
                return new Result { Window = window.ToInt64(), Title = title, Dpi = dpi,
                    ClientWidth = client.Width, ClientHeight = client.Height,
                    WindowWidth = frame.Width, WindowHeight = frame.Height,
                    ClientLeft = client.X - frame.X, ClientTop = client.Y - frame.Y,
                    ClientFile = Path.GetFileName(clientFile), WindowFile = Path.GetFileName(windowFile) };
            }
        }
        public static int[] VerifyPair(string clientFile, string windowFile, int[] crop) {
            using (var client = new Bitmap(clientFile))
            using (var window = new Bitmap(windowFile)) {
                if (client.RawFormat.Guid != ImageFormat.Png.Guid || window.RawFormat.Guid != ImageFormat.Png.Guid)
                    throw new InvalidOperationException("Capture is not an original PNG");
                if (crop == null || crop.Length != 4 || crop[0] < 0 || crop[1] < 0 ||
                    crop[2] != client.Width || crop[3] != client.Height ||
                    (long)crop[0] + crop[2] > window.Width || (long)crop[1] + crop[3] > window.Height)
                    throw new InvalidOperationException("Invalid client crop");
                for (int y = 0; y < client.Height; y++)
                    for (int x = 0; x < client.Width; x++)
                        if (client.GetPixel(x, y).ToArgb() != window.GetPixel(x + crop[0], y + crop[1]).ToArgb())
                            throw new InvalidOperationException("Client PNG differs from native window pixels");
                return new [] { client.Width, client.Height, window.Width, window.Height };
            }
        }
        public static void PairSelfTest(string directory) {
            var clientFile = Path.Combine(directory, "pair-client.png");
            var windowFile = Path.Combine(directory, "pair-window.png");
            var jpegFile = Path.Combine(directory, "pair-lossy.png");
            var crop = new [] { 3, 4, 11, 9 };
            using (var frame = new Bitmap(23, 19, PixelFormat.Format24bppRgb)) {
                for (int y = 0; y < frame.Height; y++)
                    for (int x = 0; x < frame.Width; x++) frame.SetPixel(x, y, Color.FromArgb(x * 11 % 256, y * 13 % 256, (x + y) * 17 % 256));
                WriteNew(windowFile, Png(frame));
                using (var client = frame.Clone(new Rectangle(3, 4, 11, 9), PixelFormat.Format24bppRgb)) {
                    var original = Png(client);
                    WriteNew(clientFile, original);
                    VerifyPair(clientFile, windowFile, crop);
                    client.SetPixel(0, 0, Color.Magenta);
                    File.WriteAllBytes(clientFile, Png(client));
                    bool rejected = false;
                    try { VerifyPair(clientFile, windowFile, crop); } catch (InvalidOperationException) { rejected = true; }
                    if (!rejected) throw new Exception("Changed capture pixels accepted");
                    File.WriteAllBytes(clientFile, original);
                    rejected = false;
                    try { VerifyPair(clientFile, windowFile, new [] { -1, 4, 11, 9 }); } catch (InvalidOperationException) { rejected = true; }
                    if (!rejected) throw new Exception("Invalid crop accepted");
                    client.Save(jpegFile, ImageFormat.Jpeg);
                    rejected = false;
                    try { VerifyPair(jpegFile, windowFile, crop); } catch (InvalidOperationException) { rejected = true; }
                    if (!rejected) throw new Exception("JPEG disguised as PNG accepted");
                }
            }
        }
        public static void CodecSelfTest(string path) {
            using (var image = new Bitmap(37, 23, PixelFormat.Format32bppArgb)) {
                for (int y = 0; y < image.Height; y++)
                    for (int x = 0; x < image.Width; x++)
                        image.SetPixel(x, y, Color.FromArgb((x * 13 + y * 7) % 256, x * 7 % 256, y * 11 % 256, (x * 3 + y * 5) % 256));
                var bytes = Png(image);
                using (var stream = new MemoryStream(bytes))
                using (var decoded = new Bitmap(stream)) {
                    if (decoded.Width != 37 || decoded.Height != 23) throw new Exception("PNG size changed");
                    for (int y = 0; y < image.Height; y++)
                        for (int x = 0; x < image.Width; x++)
                            if (image.GetPixel(x, y).ToArgb() != decoded.GetPixel(x, y).ToArgb())
                                throw new Exception("PNG RGBA pixels changed");
                }
                WriteNew(path, bytes);
            }
            if (!DefaultSize(1024, 622, 96, 1024, 622) || !DefaultSize(1280, 778, 120, 1024, 622) ||
                !DefaultSize(2048, 1244, 192, 1024, 622) || DefaultSize(1920, 1080, 96, 1024, 622))
                throw new Exception("Physical/logical size guard failed");
            bool rejected = false;
            try { WriteNew(path, new byte[] { 1 }); } catch (IOException) { rejected = true; }
            if (!rejected) throw new Exception("Existing output was overwritten");
        }
    }
}
