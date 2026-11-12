// Test-only activation of an installed package with an owned archive fixture.
using System;
using System.Runtime.InteropServices;

public static class XdzipMsixActivation {
    private static object manager;
    [ComImport, Guid("2e941141-7f97-4756-ba1d-9decde894a3d"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    private interface IApplicationActivationManager {
        [PreserveSig]
        int ActivateApplication([MarshalAs(UnmanagedType.LPWStr)] string appId,
            [MarshalAs(UnmanagedType.LPWStr)] string arguments, uint options, out uint processId);
        [PreserveSig]
        int ActivateForFile([MarshalAs(UnmanagedType.LPWStr)] string appId, IntPtr items,
            [MarshalAs(UnmanagedType.LPWStr)] string verb, out uint processId);
        [PreserveSig]
        int ActivateForProtocol([MarshalAs(UnmanagedType.LPWStr)] string appId, IntPtr items,
            out uint processId);
    }

    [DllImport("shell32.dll", CharSet=CharSet.Unicode, PreserveSig=true)]
    private static extern int SHCreateItemFromParsingName(string path, IntPtr bindContext,
        ref Guid iid, out IntPtr item);
    [DllImport("shell32.dll", PreserveSig=true)]
    private static extern int SHCreateShellItemArrayFromShellItem(IntPtr item, ref Guid iid,
        out IntPtr items);

    public static uint Open(string appId, string path) {
        if (manager != null) throw new InvalidOperationException("An activation is already in progress");
        var itemIid = new Guid("43826d1e-e718-42ee-bc55-a1e261c37bfe");
        var arrayIid = new Guid("b63ea76d-1f85-456f-a19c-48159efa858b");
        IntPtr item = IntPtr.Zero, items = IntPtr.Zero;
        try {
            Marshal.ThrowExceptionForHR(SHCreateItemFromParsingName(path, IntPtr.Zero, ref itemIid, out item));
            Marshal.ThrowExceptionForHR(SHCreateShellItemArrayFromShellItem(item, ref arrayIid, out items));
            var type = Type.GetTypeFromCLSID(new Guid("45ba127d-10a8-46ea-8ab7-56ea9078943c"), true);
            manager = Activator.CreateInstance(type);
            uint processId;
            Marshal.ThrowExceptionForHR(((IApplicationActivationManager)manager).ActivateForFile(appId, items, "open", out processId));
            if (processId == 0) throw new InvalidOperationException("File activation returned no process");
            return processId;
        } catch {
            Close();
            throw;
        } finally {
            if (items != IntPtr.Zero) Marshal.Release(items);
            if (item != IntPtr.Zero) Marshal.Release(item);
        }
    }

    public static void Close() {
        if (manager != null) {
            Marshal.ReleaseComObject(manager);
            manager = null;
        }
    }
}
