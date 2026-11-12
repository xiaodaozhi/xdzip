// Test-only activation of an installed package with an owned archive fixture.
using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;

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

    [ComImport, Guid("973810ae-9599-4b88-9e4d-6ee98c9552da"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    private interface IEnumAssocHandlers {
        [PreserveSig] int Next(uint count, out IAssocHandler handler, out uint fetched);
    }
    [ComImport, Guid("f04061ac-1659-4a3f-a954-775aa57fc083"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    private interface IAssocHandler {
        [PreserveSig] int GetName(out IntPtr name);
        [PreserveSig] int GetUIName(out IntPtr name);
        [PreserveSig] int GetIconLocation(out IntPtr path, out int index);
        [PreserveSig] int IsRecommended();
        [PreserveSig] int MakeDefault([MarshalAs(UnmanagedType.LPWStr)] string description);
        [PreserveSig] int Invoke(IntPtr data);
        [PreserveSig] int CreateInvoker(IntPtr data, out IntPtr invoker);
    }
    [ComImport, Guid("36db0196-9665-46d1-9ba7-d3709eecf9ed"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    private interface IObjectWithAppUserModelID {
        [PreserveSig] int SetAppID([MarshalAs(UnmanagedType.LPWStr)] string appId);
        [PreserveSig] int GetAppID(out IntPtr appId);
    }
    [ComImport, Guid("b63ea76d-1f85-456f-a19c-48159efa858b"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    private interface IShellItemArray {
        [PreserveSig] int BindToHandler(IntPtr bindContext, ref Guid handler, ref Guid iid, out IntPtr value);
    }

    [DllImport("shell32.dll", CharSet=CharSet.Unicode, PreserveSig=true)]
    private static extern int SHCreateItemFromParsingName(string path, IntPtr bindContext,
        ref Guid iid, out IntPtr item);
    [DllImport("shell32.dll", PreserveSig=true)]
    private static extern int SHCreateShellItemArrayFromShellItem(IntPtr item, ref Guid iid,
        out IntPtr items);
    [DllImport("shell32.dll", CharSet=CharSet.Unicode, PreserveSig=true)]
    private static extern int SHAssocEnumHandlers(string extension, uint filter, out IEnumAssocHandlers handlers);
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode)]
    private static extern int GetPackageFullName(IntPtr process, ref uint length, StringBuilder name);

    private static IApplicationActivationManager CreateManager() {
        if (manager != null) throw new InvalidOperationException("An activation is already in progress");
        var type = Type.GetTypeFromCLSID(new Guid("45ba127d-10a8-46ea-8ab7-56ea9078943c"), true);
        manager = Activator.CreateInstance(type);
        return (IApplicationActivationManager)manager;
    }

    public static uint Start(string appId, string arguments) {
        try {
            uint processId;
            Marshal.ThrowExceptionForHR(CreateManager().ActivateApplication(appId, arguments, 0, out processId));
            if (processId == 0) throw new InvalidOperationException("Activation returned no process");
            return processId;
        } catch {
            Close();
            throw;
        }
    }

    public static string PackageName(IntPtr process) {
        uint length = 0;
        int result = GetPackageFullName(process, ref length, null);
        if (result == 15700) return ""; // APPMODEL_ERROR_NO_PACKAGE
        if (result != 122 || length == 0) throw new System.ComponentModel.Win32Exception(result);
        var name = new StringBuilder((int)length);
        result = GetPackageFullName(process, ref length, name);
        if (result != 0) throw new System.ComponentModel.Win32Exception(result);
        return name.ToString();
    }

    public static void OpenAssociated(string appId, string executable, string path) {
        var itemIid = new Guid("43826d1e-e718-42ee-bc55-a1e261c37bfe");
        var arrayIid = new Guid("b63ea76d-1f85-456f-a19c-48159efa858b");
        var dataHandler = new Guid("b8c0bd9f-ed24-455c-83e6-d5390c4fe8c4");
        var dataIid = new Guid("0000010e-0000-0000-c000-000000000046");
        IntPtr item = IntPtr.Zero, items = IntPtr.Zero, data = IntPtr.Zero;
        IEnumAssocHandlers handlers = null;
        IAssocHandler selected = null;
        IShellItemArray array = null;
        try {
            Marshal.ThrowExceptionForHR(SHCreateItemFromParsingName(path, IntPtr.Zero, ref itemIid, out item));
            Marshal.ThrowExceptionForHR(SHCreateShellItemArrayFromShellItem(item, ref arrayIid, out items));
            array = (IShellItemArray)Marshal.GetObjectForIUnknown(items);
            Marshal.ThrowExceptionForHR(array.BindToHandler(IntPtr.Zero, ref dataHandler, ref dataIid, out data));
            Marshal.ThrowExceptionForHR(SHAssocEnumHandlers(Path.GetExtension(path), 0, out handlers));
            IAssocHandler handler;
            uint fetched;
            while (handlers.Next(1, out handler, out fetched) == 0 && fetched == 1) {
                IntPtr rawName = IntPtr.Zero, rawId = IntPtr.Zero;
                bool match = false;
                try {
                    if (handler.GetName(out rawName) == 0 && rawName != IntPtr.Zero) {
                        string name = Marshal.PtrToStringUni(rawName);
                        match = String.Equals(name, executable, StringComparison.OrdinalIgnoreCase);
                    }
                    var withId = handler as IObjectWithAppUserModelID;
                    if (withId != null && withId.GetAppID(out rawId) == 0 && rawId != IntPtr.Zero)
                        match = match || String.Equals(Marshal.PtrToStringUni(rawId), appId, StringComparison.OrdinalIgnoreCase);
                } finally {
                    if (rawName != IntPtr.Zero) Marshal.FreeCoTaskMem(rawName);
                    if (rawId != IntPtr.Zero) Marshal.FreeCoTaskMem(rawId);
                }
                if (match) {
                    if (selected != null) { Marshal.ReleaseComObject(handler); throw new InvalidOperationException("Multiple matching file handlers"); }
                    selected = handler;
                } else {
                    Marshal.ReleaseComObject(handler);
                }
            }
            if (selected == null) throw new InvalidOperationException("Installed package has no matching file handler");
            Marshal.ThrowExceptionForHR(selected.Invoke(data));
        } finally {
            if (selected != null) Marshal.ReleaseComObject(selected);
            if (handlers != null) Marshal.ReleaseComObject(handlers);
            if (array != null) Marshal.ReleaseComObject(array);
            if (data != IntPtr.Zero) Marshal.Release(data);
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
