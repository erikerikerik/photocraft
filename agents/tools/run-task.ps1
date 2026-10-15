<#
.SYNOPSIS
  Start a local-agent task: bring both repos to an up-to-date main, then launch Codex on the task's prompt.

.DESCRIPTION
  Stops before touching anything if a repo has uncommitted changes, so no work is lost.
  The prompt is agents\prompts\local-<task>.md in the PhotoCraft fork.

.EXAMPLE
  .\run-task.ps1 mg-04           # interactive Codex, same as typing the one-liner yourself
.EXAMPLE
  .\run-task.ps1 mg-04 -Exec     # unattended (codex exec); output saved in E:\CraftML\work\logs
#>
param(
    [Parameter(Mandatory = $true, Position = 0)][string]$Task,
    [switch]$Exec,
    [string]$Work = 'E:\CraftML\work',
    [string]$CodexProfile = 'strata'
)

$repos = @('photocraft', 'craft-ai-manager')

foreach ($name in $repos) {
    $repo = Join-Path $Work $name
    if (-not (Test-Path (Join-Path $repo '.git'))) {
        Write-Host "Skipping $name (no repo at $repo)." -ForegroundColor Yellow
        continue
    }
    $dirty = git -C $repo status --porcelain
    if ($dirty) {
        Write-Host "$name has uncommitted changes, so I'm stopping before anything is lost:" -ForegroundColor Red
        $dirty | ForEach-Object { Write-Host "  $_" }
        Write-Host 'Send this list to Claude before doing anything else.'
        exit 1
    }
    git -C $repo checkout --quiet main
    if ($LASTEXITCODE -ne 0) { Write-Host "Could not switch $name to main." -ForegroundColor Red; exit 1 }
    git -C $repo pull --ff-only --quiet origin main
    if ($LASTEXITCODE -ne 0) { Write-Host "Could not pull $name (see the message above)." -ForegroundColor Red; exit 1 }
    $head = git -C $repo log -1 --format='%h %s'
    Write-Host ('{0,-18} main @ {1}' -f $name, $head)
}

$id = ($Task.ToLower() -replace '^local-', '') -replace '\.md$', ''
$prompts = Join-Path $Work 'photocraft\agents\prompts'
$prompt = Join-Path $prompts "local-$id.md"
if (-not (Test-Path $prompt)) {
    Write-Host "There is no prompt at $prompt. These exist:" -ForegroundColor Red
    Get-ChildItem $prompts -Name | ForEach-Object { Write-Host "  $_" }
    exit 1
}

# Network on inside the sandbox (crates.io, model and runtime downloads, git push). Without this,
# Codex's sandbox can block the network, as it did on MG-04's first run.
$sandbox = @('--sandbox', 'workspace-write', '-c', 'sandbox_workspace_write.network_access=true')

# No double quotes inside: Windows PowerShell 5.1 mangles them when passing arguments to programs.
$ask = "Read $prompt and carry out the task below the --- line exactly."
Write-Host "Starting Codex ($CodexProfile) on local-$id.md ..." -ForegroundColor Cyan
Set-Location $Work

if ($Exec) {
    $logs = Join-Path $Work 'logs'
    New-Item -ItemType Directory -Force $logs | Out-Null
    $log = Join-Path $logs ('{0}-{1:yyyyMMdd-HHmm}.log' -f $id, (Get-Date))
    codex exec --profile $CodexProfile @sandbox $ask 2>&1 | Tee-Object -FilePath $log
    Write-Host "Log: $log"
}
else {
    codex --profile $CodexProfile @sandbox $ask
}
