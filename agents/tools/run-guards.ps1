<#
.SYNOPSIS
  Run the regression guards: every line of agents\guards.txt, for one repo or both, as a pass/fail table.

.DESCRIPTION
  Reads each repo's agents\guards.txt (lines ID | repo | command; blank lines and # comments are skipped),
  runs the lines for that repo from that repo's root with CARGO_TARGET_DIR=target/agent-guards, and prints
  one row per line: ID, command, PASS or FAIL, seconds. A line with && in it runs its parts in order and
  stops at the first failure. The table is also saved to <Work>\logs\guards-<yyyyMMdd-HHmm>.txt.
  Exit code is 1 if any guard failed, 0 if all passed. Nothing is written to the repos.

.EXAMPLE
  .\run-guards.ps1                      # both repos, every line
  .\run-guards.ps1 -Repo photocraft     # one repo
  .\run-guards.ps1 -Only PC-03          # every line with that ID
  .\run-guards.ps1 -Only BASE-1         # one line: the first BASE line in the file
#>
param(
    [ValidateSet('both', 'photocraft', 'craft-ai-manager')]
    [string]$Repo = 'both',
    [string]$Work = 'E:\CraftML\work',
    [string]$Only = ''
)

# Continue, not Stop: cargo writes progress to stderr, and PowerShell 5.1 turns redirected stderr
# into terminating errors when the preference is Stop. The exit code is what decides pass or fail.
$ErrorActionPreference = 'Continue'

$started = Get-Date
$targetDir = 'target/agent-guards'

function Get-GuardLines {
    param([string]$Folder, [string]$RepoName)
    $file = Join-Path $Folder 'agents\guards.txt'
    if (-not (Test-Path -LiteralPath $file)) {
        Write-Host "No guards file at $file, so nothing to run for $RepoName." -ForegroundColor Yellow
        return @()
    }
    $seen = @{}
    $lines = @()
    foreach ($raw in @(Get-Content -LiteralPath $file)) {
        $line = $raw.Trim()
        if ($line -eq '' -or $line.StartsWith('#')) { continue }
        $parts = $line -split '\|'
        if ($parts.Count -lt 3) {
            Write-Host "Skipping an unparseable line in $file : $line" -ForegroundColor Yellow
            continue
        }
        $id = $parts[0].Trim()
        $repoField = $parts[1].Trim()
        $command = (($parts[2..($parts.Count - 1)] -join '|')).Trim()
        if ($repoField -ne $RepoName) { continue }
        if (-not $seen.ContainsKey($id)) { $seen[$id] = 0 }
        $seen[$id] = $seen[$id] + 1
        $lines += [pscustomobject]@{
            Id      = $id
            Key     = ('{0}-{1}' -f $id, $seen[$id])
            Command = $command
            Folder  = $Folder
        }
    }
    return $lines
}

$repos = if ($Repo -eq 'both') { @('photocraft', 'craft-ai-manager') } else { @($Repo) }

$guards = @()
foreach ($name in $repos) {
    $folder = Join-Path $Work $name
    if (-not (Test-Path -LiteralPath (Join-Path $folder '.git'))) {
        Write-Host "No repo at $folder." -ForegroundColor Yellow
        continue
    }
    $guards += @(Get-GuardLines -Folder $folder -RepoName $name)
}

if ($Only -ne '') {
    $picked = @($guards | Where-Object { $_.Id -ieq $Only -or $_.Key -ieq $Only })
    if ($picked.Count -eq 0) {
        Write-Host "No guard line matches -Only $Only. The lines are:" -ForegroundColor Red
        foreach ($g in $guards) { Write-Host ('  {0,-10} {1}' -f $g.Key, $g.Command) }
        exit 1
    }
    $guards = $picked
}

if ($guards.Count -eq 0) {
    Write-Host 'No guard lines to run.' -ForegroundColor Red
    exit 1
}

Write-Host ('Running {0} guard line(s) on {1}, CARGO_TARGET_DIR={2}' -f $guards.Count, $env:COMPUTERNAME, $targetDir) -ForegroundColor Cyan

$rows = @()
$savedTarget = $env:CARGO_TARGET_DIR
$env:CARGO_TARGET_DIR = $targetDir
foreach ($g in $guards) {
    Write-Host ''
    Write-Host ('--- {0} ({1}): {2}' -f $g.Key, (Split-Path -Leaf $g.Folder), $g.Command) -ForegroundColor Cyan
    $watch = [System.Diagnostics.Stopwatch]::StartNew()
    $ok = $true
    $tail = @()
    foreach ($segment in ($g.Command -split '&&')) {
        $argv = @($segment.Trim() -split '\s+' | Where-Object { $_ -ne '' })
        if ($argv.Count -eq 0) { continue }
        $exe = $argv[0]
        $argvRest = @()
        if ($argv.Count -gt 1) { $argvRest = @($argv[1..($argv.Count - 1)]) }
        Push-Location $g.Folder
        try {
            & $exe @argvRest 2>&1 | ForEach-Object { ('{0}' -f $_) } | Tee-Object -Variable segOut | Out-Host
            $code = $LASTEXITCODE
        }
        finally {
            Pop-Location
        }
        $tail = @($segOut | Select-Object -Last 25)
        if ($code -ne 0) {
            $ok = $false
            Write-Host "  exit code $code - the last 25 lines of output:" -ForegroundColor Yellow
            foreach ($line in $tail) { Write-Host "  $line" }
            break
        }
    }
    $watch.Stop()
    $rows += [pscustomobject]@{
        ID      = $g.Key
        Command = $g.Command
        Result  = $(if ($ok) { 'PASS' } else { 'FAIL' })
        Seconds = [math]::Round($watch.Elapsed.TotalSeconds, 1)
    }
}
if ($savedTarget -eq $null -or $savedTarget -eq '') { Remove-Item Env:\CARGO_TARGET_DIR -ErrorAction SilentlyContinue }
else { $env:CARGO_TARGET_DIR = $savedTarget }

$failed = @($rows | Where-Object { $_.Result -eq 'FAIL' })
$table = ($rows | Format-Table -AutoSize | Out-String -Width 200).TrimEnd()
Write-Host ''
Write-Host $table
Write-Host ('{0} of {1} guard lines passed in {2} s.' -f ($rows.Count - $failed.Count), $rows.Count, [math]::Round(((Get-Date) - $started).TotalSeconds, 1))

$logDir = Join-Path $Work 'logs'
New-Item -ItemType Directory -Force -Path $logDir | Out-Null
$log = Join-Path $logDir ('guards-{0:yyyyMMdd-HHmm}.txt' -f $started)
$header = @(
    ('guards run started {0} on {1}' -f $started.ToString('s'), $env:COMPUTERNAME),
    ('repos: {0}' -f ($repos -join ', ')),
    ('CARGO_TARGET_DIR: {0}' -f $targetDir),
    $(if ($Only -ne '') { ('only: {0}' -f $Only) } else { 'only: (all lines)' }),
    ''
)
Set-Content -LiteralPath $log -Value ($header + @($table, '')) -Encoding UTF8
Write-Host "Table saved to $log"

if ($failed.Count -gt 0) { exit 1 }
exit 0
