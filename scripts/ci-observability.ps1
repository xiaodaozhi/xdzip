# CI timing records contain command labels, never arguments or host paths.
function Initialize-CiTiming {
    param([string]$Scope, [string]$OutputPath)
    if ($Scope -notmatch '^[a-z][a-z0-9-]{0,40}$') { throw 'Invalid CI timing scope' }
    $root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
    $target = [IO.Path]::GetFullPath((Join-Path $root $OutputPath))
    $comparison = if ($IsWindows) { [StringComparison]::OrdinalIgnoreCase } else { [StringComparison]::Ordinal }
    if (!$target.StartsWith((Join-Path $root 'build') + [IO.Path]::DirectorySeparatorChar, $comparison) -or [IO.Path]::GetExtension($target) -ne '.json') {
        throw 'CI timing output must be a dedicated build JSON file'
    }
    $directory = [IO.Path]::GetDirectoryName($target)
    for ($cursor = $directory; $cursor.Length -gt $root.Length; $cursor = [IO.Path]::GetDirectoryName($cursor)) {
        if (Test-Path -LiteralPath $cursor) {
            if ((Get-Item -LiteralPath $cursor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) {
                throw 'CI timing output cannot traverse a link'
            }
        }
    }
    [IO.Directory]::CreateDirectory($directory) | Out-Null
    $revision = (& git -C $root rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or $revision -notmatch '^[a-f0-9]{40}$') { throw 'Cannot identify CI timing source' }
    $script:CiTimingPath = $target
    $script:CiTimingScope = $Scope
    $script:CiTimingRevision = $revision
    $script:CiTimingPhase = 'initial'
    $script:CiTimingCommands = [Collections.Generic.List[object]]::new()
}

function Set-CiPhase {
    param([string]$Name)
    if ($Name -notmatch '^[a-z][a-z0-9-]{0,60}$') { throw 'Invalid CI phase label' }
    $script:CiTimingPhase = $Name
}

function Invoke-CiCommand {
    param([string]$Program, [string[]]$Arguments, [scriptblock]$AfterSuccess)
    if (!$script:CiTimingPath) { throw 'CI timing is not initialized' }
    $started = [DateTime]::UtcNow.ToString('o')
    $watch = [Diagnostics.Stopwatch]::StartNew()
    $outcome = 'failed'
    $code = $null
    try {
        & $Program @Arguments
        $code = $LASTEXITCODE
        if ($code -ne 0) { throw "CI command failed with exit code $code" }
        if ($AfterSuccess) { & $AfterSuccess }
        $outcome = 'passed'
    } finally {
        $watch.Stop()
        $label = [IO.Path]::GetFileName($Program)
        if ($Arguments.Count -gt 0 -and $Arguments[0] -match '\.(py|ps1)$') {
            $label = [IO.Path]::GetFileName($Arguments[0])
        }
        if ($label -notmatch '^[a-zA-Z0-9_.+-]{1,100}$') { $label = 'native-command' }
        $script:CiTimingCommands.Add([ordered]@{
            phase = $script:CiTimingPhase; command = $label; started_utc = $started
            elapsed_ms = [Math]::Round($watch.Elapsed.TotalMilliseconds, 3)
            status = $outcome; exit_code = $code
        })
        $document = [ordered]@{
            schema_version = 1; scope = $script:CiTimingScope; revision = $script:CiTimingRevision
            run_id = $env:GITHUB_RUN_ID; run_attempt = $env:GITHUB_RUN_ATTEMPT
            commands = $script:CiTimingCommands.ToArray()
        }
        $temporary = $script:CiTimingPath + '.tmp'
        [IO.File]::WriteAllText($temporary, (($document | ConvertTo-Json -Depth 6) + "`n"), [Text.UTF8Encoding]::new($false))
        [IO.File]::Move($temporary, $script:CiTimingPath, $true)
    }
}
