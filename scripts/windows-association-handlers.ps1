# Read-only native shell enumeration shared by disposable installation checks.
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
[ComImport,Guid("F04061AC-1659-4a3f-A954-775AA57FC083"),InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
public interface XdzipAssocHandler {
 [PreserveSig] int GetName([MarshalAs(UnmanagedType.LPWStr)] out string name);
 [PreserveSig] int GetUIName([MarshalAs(UnmanagedType.LPWStr)] out string name);
 [PreserveSig] int GetIconLocation([MarshalAs(UnmanagedType.LPWStr)] out string path,out int index);
 [PreserveSig] int IsRecommended();
}
[ComImport,Guid("973810ae-9599-4b88-9e4d-6ee98c9552da"),InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
public interface XdzipEnumAssocHandlers {
 [PreserveSig] int Next(uint count,[MarshalAs(UnmanagedType.Interface)] out XdzipAssocHandler item,out uint fetched);
}
public class XdzipAssocRow {
 public string Name; public string UIName; public string Icon; public int IconIndex;
}
public static class XdzipAssociationHandlers {
 [DllImport("shell32.dll",CharSet=CharSet.Unicode,ExactSpelling=true)]
 static extern int SHAssocEnumHandlers(string extension,uint filter,out XdzipEnumAssocHandlers handlers);
 [DllImport("shlwapi.dll",CharSet=CharSet.Unicode)]
 static extern int AssocQueryString(uint flags,uint kind,string extension,string verb,StringBuilder value,ref uint size);
 public static string FallbackProgId(string extension) {
  uint size=0; AssocQueryString(0,20,extension,null,null,ref size);
  if(size==0||size>32768) return null;
  var value=new StringBuilder((int)size);
  return AssocQueryString(0,20,extension,null,value,ref size)==0 ? value.ToString() : null;
 }
 // Shell-owned COM references stay on this thread and are released on failure.
 // No handler is invoked; unexpected native failures abort the check.
 public static XdzipAssocRow[] Read(string extension) {
  XdzipEnumAssocHandlers handlers;
  Marshal.ThrowExceptionForHR(SHAssocEnumHandlers(extension,0,out handlers));
  var rows=new List<XdzipAssocRow>();
  try {
   for(int i=0;i<128;i++) {
    XdzipAssocHandler handler; uint fetched;
    Marshal.ThrowExceptionForHR(handlers.Next(1,out handler,out fetched));
    if(fetched==0) return rows.ToArray();
    try {
     string name,ui,icon; int index;
     Marshal.ThrowExceptionForHR(handler.GetName(out name));
     Marshal.ThrowExceptionForHR(handler.GetUIName(out ui));
     Marshal.ThrowExceptionForHR(handler.GetIconLocation(out icon,out index));
     rows.Add(new XdzipAssocRow {Name=name,UIName=ui,Icon=icon,IconIndex=index});
    } finally { if(handler!=null) Marshal.ReleaseComObject(handler); }
   }
   throw new InvalidOperationException("Unexpectedly many shell application handlers");
  } finally { if(handlers!=null) Marshal.ReleaseComObject(handlers); }
 }
}
'@
