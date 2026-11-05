$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_OS -ne 'Windows' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted') { throw 'Hosted Windows runner required' }
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$env:PATH=(Join-Path $env:RUNNER_TEMP 'Qt/6.8.2/msvc2022_64/bin')+';'+$env:PATH
$env:QT_QPA_PLATFORM='windows'
$env:QT_QUICK_BACKEND='software'
$env:XDZIP_ISSUE165_UIA_HOLD='1'
$base=Join-Path (Get-Location) 'build/issue165-evidence'
New-Item -ItemType Directory -Force $base | Out-Null
$xml=Join-Path $base 'about-uia.xml'
$exe=(Resolve-Path 'build/issue165/apps/desktop/xdzip-browser-tests.exe').Path
$labels=@{}
foreach($language in @('zh-CN','en-US')) {
  $catalog=Get-Content -Raw -Encoding UTF8 "resources/i18n/$language.json" | ConvertFrom-Json
  foreach($id in @('update.checking','update.updated','update.verificationFailed')) { $labels["$language $id"]=$catalog.messages.$id }
}
if (@($labels.Values | Where-Object { -not $_ }).Count) { throw 'Missing translated control strings' }
$seen=@{}
$uiaErrors=[System.Collections.Generic.List[string]]::new()
$process=Start-Process -FilePath $exe -ArgumentList @('aboutUpdateStateLayoutAndReadOnlyActions','-o',"$xml,xml") -WindowStyle Hidden -PassThru
$deadline=[DateTime]::UtcNow.AddSeconds(180)
try {
 while (-not $process.HasExited -and [DateTime]::UtcNow -lt $deadline) {
  $condition=[System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::ProcessIdProperty,$process.Id)
  $windows=[System.Windows.Automation.AutomationElement]::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children,$condition)
  foreach($window in $windows) {
   try {
    foreach($key in $labels.Keys) {
     if($seen.ContainsKey($key)) {continue}
     $byName=[System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::NameProperty,$labels[$key])
     $element=$window.FindFirst([System.Windows.Automation.TreeScope]::Descendants,$byName)
     if($element) { $seen[$key]=@{name=$element.Current.Name;role=$element.Current.ControlType.ProgrammaticName;time=[DateTime]::UtcNow.ToString('o')} }
    }
   } catch [System.Windows.Automation.ElementNotAvailableException] {} catch { if ($uiaErrors.Count -lt 4) { $uiaErrors.Add($_.Exception.Message) } }
  }
  Start-Sleep -Milliseconds 30
  $process.Refresh()
 }
 if(-not $process.HasExited) { $process.Kill(); throw 'Test exceeded its deadline' }
 $process.WaitForExit()
 $report=@{exit_code=$process.ExitCode; observed=$seen; errors=@($uiaErrors); scope='Windows UI Automation, synthetic update states, test-only holds'}
 $report | ConvertTo-Json -Depth 4 | Set-Content -Encoding UTF8 (Join-Path $base 'uia-observation.json')
 $report | ConvertTo-Json -Depth 4
 if($process.ExitCode -ne 0) {throw 'Qt test failed'}
 if($seen.Count -ne $labels.Count) {throw "Observed $($seen.Count) of $($labels.Count) target labels"}
} finally { if(-not $process.HasExited) { $process.Kill() } }
