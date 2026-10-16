<#
.SYNOPSIS
  Run the regression guards: every line of agents\guards.txt, for one repo or both, as a pass/fail table.

.DESCRIPTION
  Reads each repo's agents\guards.txt (lines ID | repo | command; blank lines and # comments are skipped),
  runs the lines for that repo from that repo's root with CARGO_TARGET_DIR=target/agent-guards, and prints
  one row per line: ID, repo, command, PASS or FAIL, seconds. A line with && in it runs its parts in order
  and stops at the first failure. The table is also saved to <Work>\logs\guards-<yyyyMMdd-HHmm>.txt.
  Each repo is resolved to a checkout first: the checkout this script sits in wins for its own repo, so a
  task worktree tests its own changes; the other repo comes from <Work>\<repo>, and -RepoPath overrides one.
  Exit code is 1 if any guard failed or a requested repo could not run, 0 if all passed. Nothing is written
  to the repos.

.EXAMPLE
  .\run-guards.ps1                      # both repos, every line
  .\run-guards.ps1 -Repo photocraft     # one repo
  .\run-guards.ps1 -Only PC-03          # every line with that ID
  .\run-guards.ps1 -Only BASE-1         # one line: the first BASE line (add -Repo when both repos run)
  .\run-guards.ps1 -Repo photocraft -RepoPath E:\CraftML\work\wt\pc-07   # guards for one checkout
#>
param(
    [ValidateSet('both', 'photocraft', 'craft-ai-manager')]
    [string]$Repo = 'both',
    [string]$Work = 'E:\CraftML\work',
    [string]$RepoPath = '',
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
            Repo    = $RepoName
            Command = $command
            Folder  = $Folder
        }
    }
    return $lines
}

$repos = if ($Repo -eq 'both') { @('photocraft', 'craft-ai-manager') } else { @($Repo) }

# Which checkout is this script sitting in? A linked worktree counts, so the guards test the changes
# under review rather than the separate main checkout under <Work>.
$selfRoot = ''
$selfName = ''
$top = ''
try {
    $top = @((& git -C $PSScriptRoot rev-parse --show-toplevel 2>&1) | ForEach-Object { ('{0}' -f $_) }) -join ''
}
catch {
    $top = ''
}
if ($LASTEXITCODE -eq 0 -and $top.Trim() -ne '') {
    $selfRoot = $top.Trim()
    $url = ''
    try {
        $url = @((& git -C $selfRoot remote get-url origin 2>&1) | ForEach-Object { ('{0}' -f $_) }) -join ''
    }
    catch {
        $url = ''
    }
    if ($LASTEXITCODE -eq 0) {
        if ($url -match 'photocraft') { $selfName = 'photocraft' }
        elseif ($url -match 'craft-ai-manager') { $selfName = 'craft-ai-manager' }
    }
}
if ($selfName -eq '') {
    Write-Host "Could not tell which checkout this script sits in, so every repo comes from $Work." -ForegroundColor Yellow
}

if ($RepoPath -ne '') {
    $leaf = Split-Path -Leaf ($RepoPath.TrimEnd('\', '/'))
    if ($repos -contains $leaf) {
        $repos = @($leaf)
    }
    elseif ($repos.Count -eq 1) {
        # -Repo already named the repo, so -RepoPath may be any checkout of it (a worktree, another clone).
    }
    else {
        Write-Host "-RepoPath needs -Repo photocraft or -Repo craft-ai-manager when the folder is not named after the repo (here: $leaf)." -ForegroundColor Red
        exit 1
    }
}

$guards = @()
$missing = @()
$resolved = @()
foreach ($name in $repos) {
    if ($RepoPath -ne '') { $folder = $RepoPath }
    elseif ($name -eq $selfName) { $folder = $selfRoot }
    else { $folder = Join-Path $Work $name }
    if (-not (Test-Path -LiteralPath (Join-Path $folder '.git'))) {
        Write-Host "No $name checkout at $folder, so its guards cannot run." -ForegroundColor Red
        $missing += $name
        continue
    }
    $guardFile = Join-Path $folder 'agents\guards.txt'
    if (-not (Test-Path -LiteralPath $guardFile)) {
        Write-Host "No guards file at $guardFile, so $name guards cannot run." -ForegroundColor Red
        $missing += $name
        continue
    }
    $resolved += ('{0}={1}' -f $name, $folder)
    Write-Host ('{0} guards run in {1}' -f $name, $folder) -ForegroundColor DarkGray
    $guards += @(Get-GuardLines -Folder $folder -RepoName $name)
}

if ($Only -ne '') {
    $picked = @($guards | Where-Object { $_.Id -ieq $Only -or $_.Key -ieq $Only })
    if ($picked.Count -eq 0) {
        Write-Host "No guard line matches -Only $Only. The lines are:" -ForegroundColor Red
        foreach ($g in $guards) { Write-Host ('  {0,-16} {1,-10} {2}' -f $g.Repo, $g.Key, $g.Command) }
        exit 1
    }
    if (@(@($picked | Select-Object -ExpandProperty Repo) | Select-Object -Unique).Count -gt 1) {
        Write-Host "-Only $Only matches that line in both repos. Add -Repo photocraft or -Repo craft-ai-manager." -ForegroundColor Red
        foreach ($g in $picked) { Write-Host ('  {0,-16} {1,-10} {2}' -f $g.Repo, $g.Key, $g.Command) }
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
    $launchFail = ''
    foreach ($segment in ($g.Command -split '&&')) {
        $argv = @($segment.Trim() -split '\s+' | Where-Object { $_ -ne '' })
        if ($argv.Count -eq 0) { continue }
        $exe = $argv[0]
        $argvRest = @()
        if ($argv.Count -gt 1) { $argvRest = @($argv[1..($argv.Count - 1)]) }
        $segOut = @()
        $code = $null
        if ($null -eq (Get-Command $exe -ErrorAction SilentlyContinue)) {
            $launchFail = "cannot run this guard: there is no command named $exe"
        }
        else {
            Push-Location $g.Folder
            try {
                & $exe @argvRest 2>&1 | ForEach-Object { ('{0}' -f $_) } | Tee-Object -Variable segOut | Out-Host
                $code = $LASTEXITCODE
            }
            catch {
                $launchFail = "cannot run this guard: $($_.Exception.Message)"
            }
            finally {
                Pop-Location
            }
        }
        $tail = @($segOut | Select-Object -Last 25)
        if ($launchFail -ne '') {
            $ok = $false
            Write-Host "  $launchFail" -ForegroundColor Yellow
            break
        }
        if ($null -eq $code) { $code = 1 }
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
        Repo    = $g.Repo
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
if ($missing.Count -gt 0) {
    Write-Host ('No guards ran for: {0}' -f ($missing -join ', ')) -ForegroundColor Red
}

$logDir = Join-Path $Work 'logs'
New-Item -ItemType Directory -Force -Path $logDir | Out-Null
$log = Join-Path $logDir ('guards-{0:yyyyMMdd-HHmm}.txt' -f $started)
$header = @(
    ('guards run started {0} on {1}' -f $started.ToString('s'), $env:COMPUTERNAME),
    ('repos: {0}' -f ($repos -join ', ')),
    ('checkouts: {0}' -f ($resolved -join ', ')),
    ('could not run: {0}' -f $(if ($missing.Count -gt 0) { $missing -join ', ' } else { '(none)' })),
    ('CARGO_TARGET_DIR: {0}' -f $targetDir),
    $(if ($Only -ne '') { ('only: {0}' -f $Only) } else { 'only: (all lines)' }),
    ''
)
Set-Content -LiteralPath $log -Value ($header + @($table, '')) -Encoding UTF8
Write-Host "Table saved to $log"

if ($failed.Count -gt 0) { exit 1 }
if ($missing.Count -gt 0) {
    Write-Host 'A requested repo could not run its guards, so this run counts as a failure.' -ForegroundColor Red
    exit 1
}
exit 0
