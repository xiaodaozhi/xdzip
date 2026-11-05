# Helpers for an already-running, test-owned Windows process. They neither
# launch an application nor send input to windows owned by another process.
function Initialize-XdzipSettingsUia {
    Add-Type -AssemblyName UIAutomationClient
    Add-Type -AssemblyName UIAutomationTypes
    if ('XdzipOwnedSettingsUia' -as [type]) { return }
    Add-Type @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public static class XdzipOwnedSettingsUia {
    private delegate bool Callback(IntPtr hwnd, IntPtr state);
    [DllImport("user32.dll")] private static extern bool EnumWindows(Callback callback, IntPtr state);
    [DllImport("user32.dll")] private static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
    [DllImport("user32.dll")] private static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] private static extern int GetWindowTextW(IntPtr hwnd, StringBuilder text, int length);
    [DllImport("user32.dll")] private static extern bool PostMessageW(IntPtr hwnd, uint message, IntPtr wParam, IntPtr lParam);
    public static IntPtr[] Windows(uint expected) {
        var result = new List<IntPtr>();
        EnumWindows((hwnd, state) => {
            uint pid; GetWindowThreadProcessId(hwnd, out pid);
            if (pid == expected) result.Add(hwnd);
            return true;
        }, IntPtr.Zero);
        return result.ToArray();
    }
    public static IntPtr VisibleTitledWindow(uint expected, string title) {
        IntPtr found = IntPtr.Zero;
        foreach (var hwnd in Windows(expected)) {
            var text = new StringBuilder(512);
            GetWindowTextW(hwnd, text, text.Capacity);
            if (IsWindowVisible(hwnd) && text.ToString() == title) {
                if (found != IntPtr.Zero) return IntPtr.Zero;
                found = hwnd;
            }
        }
        return found;
    }
    public static bool Space(IntPtr hwnd, uint expected) {
        uint pid; GetWindowThreadProcessId(hwnd, out pid);
        return hwnd != IntPtr.Zero && pid == expected &&
            PostMessageW(hwnd, 0x100, (IntPtr)0x20, IntPtr.Zero) &&
            PostMessageW(hwnd, 0x101, (IntPtr)0x20, IntPtr.Zero);
    }
}
'@
}

function Wait-XdzipOwnedUiElement($Process, [string]$Name, [string]$ControlType) {
    $condition = [System.Windows.Automation.PropertyCondition]::new(
        [System.Windows.Automation.AutomationElement]::NameProperty, $Name)
    for ($attempt = 0; $attempt -lt 150; $attempt++) {
        $Process.Refresh()
        if ($Process.HasExited) { throw 'Owned product exited during UI verification' }
        foreach ($handle in [XdzipOwnedSettingsUia]::Windows([uint32]$Process.Id)) {
            try {
                $window = [System.Windows.Automation.AutomationElement]::FromHandle($handle)
                if (!$window) { continue }
                $items = $window.FindAll([System.Windows.Automation.TreeScope]::Descendants, $condition)
                foreach ($item in $items) {
                    if ($item.Current.ProcessId -eq $Process.Id -and
                        $item.Current.ControlType.ProgrammaticName -eq $ControlType) { return $item }
                }
            } catch [System.Windows.Automation.ElementNotAvailableException] {}
        }
        Start-Sleep -Milliseconds 100
    }
    throw "Owned UI control not found: $Name ($ControlType)"
}

function Open-XdzipSettingsGeneral($Process, $Catalog) {
    $button = Wait-XdzipOwnedUiElement $Process $Catalog.messages.'settings.title' 'ControlType.Button'
    $button.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
    $tab = Wait-XdzipOwnedUiElement $Process $Catalog.messages.'settings.tabGeneral' 'ControlType.TabItem'
    $tab.GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern).Select()
}

function Select-XdzipUiLanguage($Process, $CurrentCatalog, [string]$Option) {
    $select = Wait-XdzipOwnedUiElement $Process $CurrentCatalog.messages.'settings.language' 'ControlType.ComboBox'
    $select.SetFocus()
    $window = [XdzipOwnedSettingsUia]::VisibleTitledWindow([uint32]$Process.Id, $CurrentCatalog.messages.'settings.title')
    if ($window -eq [IntPtr]::Zero -or ![XdzipOwnedSettingsUia]::Space($window, [uint32]$Process.Id)) {
        throw 'Cannot open the language menu in the owned settings window'
    }
    $entry = Wait-XdzipOwnedUiElement $Process $Option 'ControlType.MenuItem'
    $entry.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
}
