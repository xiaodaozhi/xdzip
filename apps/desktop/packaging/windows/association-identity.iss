{ Inno's ValueType none creates only a key, not REG_NONE. These native calls
  write the documented opt-out marker on our already-created ProgIDs only.
  The installer is x64-only and explicitly opens the matching registry view.
  Failure aborts Setup; the handle is released even when the value write fails.
  No extension default or protected user choice is read or written here. }
function AssociationOpenKey(Root: THandle; Subkey: String; Options, Access: LongWord;
  var Key: THandle): Longint;
external 'RegOpenKeyExW@advapi32.dll stdcall';
function AssociationSetNone(Key: THandle; Name: String; Reserved, ValueType: LongWord;
  Data: THandle; Size: LongWord): Longint;
external 'RegSetValueExW@advapi32.dll stdcall';
function AssociationCloseKey(Key: THandle): Longint;
external 'RegCloseKey@advapi32.dll stdcall';

procedure PreventSilentDefault(const ProgId: String);
var Key: THandle;
begin
  if (ProgId <> 'xdzip.Archive') and (Pos('xdzip.Archive.', ProgId) <> 1) then
    RaiseException('Invalid xdzip association identity');
  if AssociationOpenKey(HKEY_CURRENT_USER, 'Software\Classes\' + ProgId,
       0, $0102, Key) <> 0 then
    RaiseException('Cannot open xdzip association identity');
  try
    if AssociationSetNone(Key, 'AllowSilentDefaultTakeOver', 0, 0, 0, 0) <> 0 then
      RaiseException('Cannot preserve user-controlled default selection');
  finally
    AssociationCloseKey(Key);
  end;
end;

{ Qt uses LocalAppData/xdzip/xdzip/settings.ini for this application's profile.
  Only a supported schema and the known English locale change the default.
  No profile is created or modified by the installer. }
function AssociationDescription(Param: String): String;
var Profile, Language: String;
begin
  Profile := ExpandConstant('{localappdata}\xdzip\xdzip\settings.ini');
  Language := 'zh-CN';
  if GetIniString('General', 'schemaVersion', '1', Profile) = '1' then
    Language := GetIniString('locale', 'language', 'zh-CN', Profile);
  if Language = 'en-US' then
    Result := @ASSOCIATION_DESCRIPTION_EN@
  else
    Result := @ASSOCIATION_DESCRIPTION_ZH@;
end;
