<#
.SYNOPSIS
  Regression test for agents/tools/run-guards.ps1: which checkout each repo's guards run in.

.DESCRIPTION
  Builds a throwaway <Work> tree in the temp folder - two fake craft-ai-manager checkouts (a "main"
  one under <Work> and a "task worktree" under <Work>\wt), one fake photocraft, each with a .git
  marker and one guard line - and runs the real runner against it. Each fake guard line is
  `cmd /c cd > <marker>.txt`, which writes the folder the command actually ran in, so the test can
  say which checkout was chosen rather than trusting a printed path.

  Four cases: the single-repository override (-Repo <repo> -RepoPath <worktree>) must guard the
  worktree and not the checkout under -Work; the same override with the default -Repo both must keep
  both repositories and still use the worktree; a -RepoPath with no checkout must fail the run; and a
  -RepoPath that is not named after a repo must ask for -Repo when both repos are selected.

  Exit code 0 when every case passes, 1 otherwise. Nothing is written to the real repos: the only
  writes are under the temp folder, which is removed at the end unless -Keep names it.

.EXAMPLE
  .\agents\tools\test-run-guards.ps1
  .\agents\tools\test-run-guards.ps1 -Runner E:\path\to\another\run-guards.ps1
#>
param(
    [string]$Runner = '',
    [string]$Keep = ''
)

$ErrorActionPreference = 'Continue'

$runnerPath = if ($Runner -ne '') { $Runner } else { Join-Path $PSScriptRoot 'run-guards.ps1' }
if (-not (Test-Path -LiteralPath $runnerPath)) {
    Write-Host "No runner to test at $runnerPath" -ForegroundColor Red
    exit 1
}

$root = Join-Path ([System.IO.Path]::GetTempPath()) ('pc07-run-guards-' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + $PID)
$main = Join-Path $root 'craft-ai-manager'
$worktree = Join-Path $root 'wt\craft-ai-manager'
$photo = Join-Path $root 'photocraft'

function New-FakeCheckout {
    param([string]$Folder, [string]$RepoName, [string]$Marker)
    New-Item -ItemType Directory -Force -Path (Join-Path $Folder 'agents') | Out-Null
    # A .git marker: the runner only asks whether it is there, so a plain folder is enough and keeps
    # the fake checkout out of git's way (the runner's own "which repo am I in" probe fails here).
    New-Item -ItemType Directory -Force -Path (Join-Path $Folder '.git') | Out-Null
    Set-Content -LiteralPath (Join-Path $Folder 'agents\guards.txt') -Value @(
        '# one line, and it writes where it ran rather than building anything',
        ('TESTG | {0} | cmd /c cd > {1}' -f $RepoName, $Marker)
    ) -Encoding ASCII
}

New-FakeCheckout -Folder $main -RepoName 'craft-ai-manager' -Marker 'caim-cwd.txt'
New-FakeCheckout -Folder $worktree -RepoName 'craft-ai-manager' -Marker 'caim-cwd.txt'
New-FakeCheckout -Folder $photo -RepoName 'photocraft' -Marker 'pc-cwd.txt'
# The runner copy sits inside the fake photocraft, so its own checkout probe fails and every repo is
# resolved from -Work. That is what makes a fake <Work> tree enough to test the resolution.
New-Item -ItemType Directory -Force -Path (Join-Path $photo 'agents\tools') | Out-Null
Copy-Item -LiteralPath $runnerPath -Destination (Join-Path $photo 'agents\tools\run-guards.ps1')
$copied = Join-Path $photo 'agents\tools\run-guards.ps1'

$failures = @()
$checksRun = 0
function Check {
    param([string]$Name, [bool]$Condition, [string]$Detail)
    $script:checksRun++
    if ($Condition) {
        Write-Host ('PASS  {0}' -f $Name) -ForegroundColor Green
    }
    else {
        Write-Host ('FAIL  {0} - {1}' -f $Name, $Detail) -ForegroundColor Red
        $script:failures += ('{0}: {1}' -f $Name, $Detail)
    }
}

function Clear-Markers {
    foreach ($f in @((Join-Path $main 'caim-cwd.txt'), (Join-Path $worktree 'caim-cwd.txt'), (Join-Path $photo 'pc-cwd.txt'))) {
        Remove-Item -LiteralPath $f -Force -ErrorAction SilentlyContinue
    }
}

function Ran-In {
    param([string]$MarkerPath)
    if (-not (Test-Path -LiteralPath $MarkerPath)) { return '' }
    return (@(Get-Content -LiteralPath $MarkerPath) -join ' ').Trim()
}

function Run-Runner {
    param([string]$Script, [string[]]$Params)
    $out = @(& powershell -NoProfile -ExecutionPolicy Bypass -File $Script @Params 2>&1 | ForEach-Object { ('{0}' -f $_) })
    return [pscustomobject]@{ Code = $LASTEXITCODE; Text = ($out -join "`n") }
}

Write-Host "runner under test: $runnerPath" -ForegroundColor DarkGray
Write-Host "throwaway tree:    $root" -ForegroundColor DarkGray

# 1. The single-repository override: -Repo names the repo, -RepoPath names the checkout.
Clear-Markers
$run = Run-Runner -Script $copied -Params @('-Work', $root, '-Repo', 'craft-ai-manager', '-RepoPath', $worktree)
Check 'the single-repository override exits 0' ($run.Code -eq 0) ('exit ' + $run.Code + ': ' + $run.Text)
Check 'it guards the -RepoPath checkout' ((Ran-In (Join-Path $worktree 'caim-cwd.txt')) -ieq $worktree) ('the guard ran in ' + (Ran-In (Join-Path $worktree 'caim-cwd.txt')))
Check 'it does not guard the checkout under -Work' (-not (Test-Path -LiteralPath (Join-Path $main 'caim-cwd.txt'))) 'the checkout under -Work was guarded as well'
$log = @(Get-ChildItem -LiteralPath (Join-Path $root 'logs') -Filter 'guards-*.txt' -ErrorAction SilentlyContinue | Sort-Object LastWriteTime | Select-Object -Last 1)
Check 'the saved log names the checkout it chose' ($log.Count -eq 1 -and ((@(Get-Content -LiteralPath $log[0].FullName)) -join "`n") -match ([regex]::Escape('checkouts: craft-ai-manager=' + $worktree))) 'the log does not say craft-ai-manager=' + $worktree

# 2. The same override with the default -Repo both: both repos still run, the override still wins.
Clear-Markers
$run = Run-Runner -Script $copied -Params @('-Work', $root, '-RepoPath', $worktree)
Check '-Repo both with a repo-named -RepoPath exits 0' ($run.Code -eq 0) ('exit ' + $run.Code + ': ' + $run.Text)
Check 'it still guards the -RepoPath checkout' ((Ran-In (Join-Path $worktree 'caim-cwd.txt')) -ieq $worktree) ('the guard ran in ' + (Ran-In (Join-Path $worktree 'caim-cwd.txt')))
Check 'it keeps the other repository' ((Ran-In (Join-Path $photo 'pc-cwd.txt')) -ieq $photo) ('photocraft ran in ' + (Ran-In (Join-Path $photo 'pc-cwd.txt')))
Check 'and it does not fall back to the checkout under -Work' (-not (Test-Path -LiteralPath (Join-Path $main 'caim-cwd.txt'))) 'the checkout under -Work was guarded as well'

# 3. A requested repo whose checkout is not there fails the run rather than passing without it.
Clear-Markers
$run = Run-Runner -Script $copied -Params @('-Work', $root, '-Repo', 'craft-ai-manager', '-RepoPath', (Join-Path $root 'no-such-checkout'))
Check 'a missing -RepoPath checkout fails the run' ($run.Code -ne 0) ('exit ' + $run.Code)
Check 'and says which repo could not run' ($run.Text -match 'No craft-ai-manager checkout at') ('the run said: ' + $run.Text)

# 4. A -RepoPath that is not named after a repo needs -Repo while both repos are selected.
Clear-Markers
$run = Run-Runner -Script $copied -Params @('-Work', $root, '-RepoPath', (Join-Path $root 'wt'))
Check 'an unnamed -RepoPath with both repos asks for -Repo' ($run.Code -ne 0 -and $run.Text -match 'needs -Repo photocraft or -Repo craft-ai-manager') ('exit ' + $run.Code + ': ' + $run.Text)

if ($Keep -ne '') {
    Copy-Item -LiteralPath $root -Destination $Keep -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "copy of the tree: $Keep" -ForegroundColor DarkGray
}
Remove-Item -LiteralPath $root -Recurse -Force -ErrorAction SilentlyContinue

if ($failures.Count -gt 0) {
    Write-Host ''
    Write-Host ('run-guards checkout selection: {0} of {1} checks failed:' -f $failures.Count, $checksRun) -ForegroundColor Red
    foreach ($f in $failures) { Write-Host ('  ' + $f) }
    exit 1
}
Write-Host ''
Write-Host 'run-guards checkout selection: every check passed.' -ForegroundColor Green
exit 0