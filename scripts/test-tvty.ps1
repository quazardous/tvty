# test-tvty.ps1 — runs tvty apart from the user's own, on Windows: what
# scripts/test-env and scripts/test-tvty do on Linux.
# - its own settings (XDG_CONFIG_HOME), memory (XDG_STATE_HOME) and fonts
#   (XDG_DATA_HOME), under dev/tvty-wbox-win: the user's are never read nor
#   written;
# - never the user's aiball: AIBALL_SOCK points at a socket that is not
#   there (no throwaway aiball on Windows yet);
# - its own psmux server (TVTY_MUX_SERVER, psmux's -L): the user's loops are
#   not even listed, so never attached nor resized. psmux ignores
#   TMUX_TMPDIR, which does it on Linux;
# - one more tvty, whatever else runs (TVTY_NEW_INSTANCE).
# Arguments go to tvty (a session to open).
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$dev = Join-Path $root 'dev\tvty-wbox-win'

# Nothing of the caller's own loop (see scripts/test-env).
Get-ChildItem Env: | Where-Object { $_.Name -match '^(CL_|AIBALL_|CLAUDE_LOOP_)' } |
    ForEach-Object { Remove-Item "Env:$($_.Name)" }

$env:XDG_CONFIG_HOME = Join-Path $dev 'config'
$env:XDG_STATE_HOME = Join-Path $dev 'state'
$env:XDG_DATA_HOME = Join-Path $dev 'data'
$env:AIBALL_SOCK = Join-Path $dev 'no-aiball\sock'
$env:TVTY_MUX_SERVER = 'tvty-test'
$env:TVTY_NEW_INSTANCE = '1'
$env:TVTY_STATS = Join-Path $dev 'log\tvty-stats.log'
foreach ($dir in 'config\tvty', 'state\tvty', 'data\tvty', 'log') {
    New-Item -ItemType Directory -Force (Join-Path $dev $dir) | Out-Null
}
# The projects' list starts folded: nothing to click by accident.
$layout = Join-Path $dev 'state\tvty\layout.json'
if (-not (Test-Path $layout)) { Set-Content -Path $layout -Value '{"sidebar_open": false}' }

& (Join-Path $root 'target\debug\tvty.exe') @args
exit $LASTEXITCODE
