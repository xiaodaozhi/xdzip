// Native, lossless capture for the user-operated Store screenshot assistant.
// HWNDs are borrowed and revalidated before/after readback. A failed guard never
// persists screen pixels; all GDI objects and thread DPI changes have scoped owners.
using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Drawing;
using System.Drawing.Imaging;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;

namespace Xdzip.StoreCapture {
    public sealed class Result {
        public long Window;
        public string Title;
        public int Dpi, ClientWidth, ClientHeight, WindowWidth, WindowHeight;
        public int ClientLeft, ClientTop;
        public string ClientFile, WindowFile;
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
        private const int CaptureKey = 0x5844, CancelKey = 0x5845;

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
                throw new InvalidOperationException("Activate the requested xdzip window at its default size");
            if (GetDpiForWindow(window) != dpi || Frame(window) != frame || Client(window) != client)
                throw new InvalidOperationException("Window geometry changed during capture; retry F8");
            var monitor = new Monitor { Size = Marshal.SizeOf(typeof(Monitor)) };
            if (!GetMonitorInfoW(MonitorFromWindow(window, 2), ref monitor) || !monitor.Work.Box.Contains(frame))
                throw new InvalidOperationException("The complete window must fit inside one monitor's work area");
            // Conservatively reject every intersecting visible window above ours,
            // including other xdzip dialogs. No foreign pixels are saved.
            for (var above = GetWindow(window, 3); above != IntPtr.Zero; above = GetWindow(above, 3)) {
                if (Hidden(above)) continue;
                Rect other;
                if (DwmGetWindowAttribute(above, 9, out other, Marshal.SizeOf(typeof(Rect))) != 0 && !GetWindowRect(above, out other))
                    throw new InvalidOperationException("Cannot verify an overlapping window");
                if (frame.IntersectsWith(other.Box))
                    throw new InvalidOperationException("Another window covers xdzip; move it away and retry F8");
            }
        }
        public static bool DefaultSize(int width, int height, int dpi, int logicalWidth, int logicalHeight) {
            return dpi >= 96 && Math.Abs(width - logicalWidth * dpi / 96.0) <= 2 &&
                Math.Abs(height - logicalHeight * dpi / 96.0) <= 2;
        }
        public static long FindMain(int process, int logicalWidth, int logicalHeight) {
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
            if (!RegisterHotKey(IntPtr.Zero, CaptureKey, 0x4000, 0x77))
                throw new Win32Exception(Marshal.GetLastWin32Error(), "F8 is already registered by another app");
            if (!RegisterHotKey(IntPtr.Zero, CancelKey, 0x4000, 0x79)) {
                UnregisterHotKey(IntPtr.Zero, CaptureKey);
                throw new Win32Exception(Marshal.GetLastWin32Error(), "F10 is already registered by another app");
            }
        }
        public static int PollKey() {
            Message message;
            while (PeekMessageW(out message, IntPtr.Zero, 0x0312, 0x0312, 1)) {
                if (message.WParam.ToUInt64() == CaptureKey) return 1;
                if (message.WParam.ToUInt64() == CancelKey) return 2;
            }
            return 0;
        }
        public static void ReleaseKeys() {
            UnregisterHotKey(IntPtr.Zero, CaptureKey);
            UnregisterHotKey(IntPtr.Zero, CancelKey);
        }
        private static byte[] Png(Bitmap image) {
            using (var stream = new MemoryStream()) {
                image.Save(stream, ImageFormat.Png);
                return stream.ToArray();
            }
        }
        private static void WriteNew(string path, byte[] bytes) {
            using (var file = new FileStream(path, FileMode.CreateNew, FileAccess.Write, FileShare.None))
                file.Write(bytes, 0, bytes.Length);
        }
        public static Result Capture(int process, long main, string requiredTitle, int logicalWidth, int logicalHeight, string clientFile, string windowFile) {
            using (new DpiScope()) {
                var window = GetForegroundWindow();
                if (!Owned(window, process)) throw new InvalidOperationException("Click the xdzip window before pressing F8");
                var title = Title(window);
                bool creation = !String.IsNullOrEmpty(requiredTitle);
                if (!creation ? window.ToInt64() != main : window.ToInt64() == main || title != requiredTitle)
                    throw new InvalidOperationException("This is not the window requested for the current scene");
                var frame = Frame(window);
                var client = Client(window);
                int dpi = (int)GetDpiForWindow(window);
                if (client.Width <= 0 || client.Height <= 0 || !frame.Contains(client))
                    throw new InvalidOperationException("Invalid capture bounds");
                if (!creation && !DefaultSize(client.Width, client.Height, dpi, logicalWidth, logicalHeight))
                    throw new InvalidOperationException("Main window was resized; restart the assistant to restore its default size");
                Guard(window, process, frame, client, dpi);
                if (DwmFlush() != 0) throw new InvalidOperationException("DWM did not complete presentation");
                Guard(window, process, frame, client, dpi);
                // Screen GDI alpha is unspecified. RGB avoids manufacturing
                // transparent/black pixels while retaining every color sample.
                using (var full = new Bitmap(frame.Width, frame.Height, PixelFormat.Format24bppRgb)) {
                    using (var graphics = Graphics.FromImage(full))
                        graphics.CopyFromScreen(frame.Location, System.Drawing.Point.Empty, frame.Size, CopyPixelOperation.SourceCopy | CopyPixelOperation.CaptureBlt);
                    Guard(window, process, frame, client, dpi);
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
