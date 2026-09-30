# tvty-setup.ps1 - Terminal Velocity on Windows, everything it needs in one go:
#
#   irm https://github.com/quazardous/tvty/releases/latest/download/tvty-setup.ps1 | iex
#
# 1. Terminal Velocity Updater, from the release (its own installer, into
#    ~\.local\bin, added to your PATH). It needs nothing.
# 2. The updater, without a window (`tvty-updater --install`), does all the
#    rest: what aiball and tvty need, through winget when it is missing
#    (Git, Node.js LTS, PowerShell 7, psmux, Claude Code: the updater keeps
#    that list), then aiball (its own install.ps1), Terminal Velocity itself,
#    and the Start menu's shortcuts.
#
# It tries everything and says what failed; run it again once that is fixed,
# it skips what is there. Everything it says and everything the installers
# say is also kept in %LOCALAPPDATA%/tvty/setup.log (your home folder
# written ~): the file to attach to an issue. TVTY_SETUP_FROM: where the release's files are
# (a URL, or a folder), to try a build before it is released.
#
# ASCII only: Windows PowerShell 5.1 reads a script without a BOM as ANSI.
#
# Run through iex, it lives in a block of its own: its settings stay in it,
# and it returns instead of exiting (which would close your window).

& {
    $ErrorActionPreference = 'Stop'
    $ProgressPreference = 'SilentlyContinue'

    $from = $env:TVTY_SETUP_FROM
    if (-not $from) { $from = 'https://github.com/quazardous/tvty/releases/latest/download' }
    $from = $from.TrimEnd('/', '\')
    $bin = Join-Path $HOME '.local\bin'
    $failed = @()

    # The log of this run: what the console shows, colour codes removed and
    # the home folder written ~ (it names the user).
    $logDir = Join-Path $env:LOCALAPPDATA 'tvty'
    $log = Join-Path $logDir 'setup.log'
    New-Item -ItemType Directory -Force $logDir | Out-Null
    Set-Content -LiteralPath $log -Value $null -Encoding UTF8
    $escape = [string][char]27 + '\[[0-9;]*[A-Za-z]'

    function Write-Line($text) {
        Write-Host $text
        $kept = ([string]$text -replace $escape, '').Replace($HOME, '~')
        Add-Content -LiteralPath $log -Value $kept -Encoding UTF8
    }
    function Say($text) { Write-Line "tvty-setup: $text" }

    # Runs a program, each line it writes (errors too) shown and logged.
    # What it writes on its error output is said, not thrown: under 'Stop',
    # PowerShell 5.1 would end the script on the first such line.
    # The pipe is on the program itself: through a script block, PowerShell
    # neither waits for a window program (the updater) nor reads what it says.
    function Invoke-Logged($program, [string[]]$arguments) {
        $ErrorActionPreference = 'Continue'
        & $program @arguments 2>&1 | ForEach-Object { Write-Line "  $_" }
    }

    Say "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss'), from $from"
    Say "Windows $([Environment]::OSVersion.Version), PowerShell $($PSVersionTable.PSVersion)"

    # The PATH as the registry has it now: what was just installed shows.
    function Update-Path {
        $machine = [Environment]::GetEnvironmentVariable('Path', 'Machine')
        $user = [Environment]::GetEnvironmentVariable('Path', 'User')
        $env:Path = (@($machine, $user, $bin) | Where-Object { $_ }) -join ';'
    }

    # A file of the release: downloaded, or copied from TVTY_SETUP_FROM's folder.
    function Get-ReleaseFile($name) {
        $to = Join-Path ([IO.Path]::GetTempPath()) $name
        if (Test-Path -LiteralPath $from -PathType Container) {
            Copy-Item -LiteralPath (Join-Path $from $name) -Destination $to -Force
        } else {
            Invoke-WebRequest -UseBasicParsing -Uri "$from/$name" -OutFile $to
        }
        return $to
    }

    # 1. The updater, by its own installer.
    foreach ($app in @('tvty-updater')) {
        Say "installing $app..."
        try {
            $installer = Get-ReleaseFile "$app-installer.ps1"
            $prefix = ($app -replace '-', '_').ToUpper()
            if (Test-Path -LiteralPath $from -PathType Container) {
                # A folder: the installer takes its archive from there too.
                Set-Item "Env:${prefix}_DOWNLOAD_URL" ([Uri](Resolve-Path -LiteralPath $from).Path).AbsoluteUri
            }
            Invoke-Logged powershell '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', $installer
            if ($LASTEXITCODE) { throw "its installer said $LASTEXITCODE" }
        } catch {
            Say "$app did not install: $_"
            $failed += $app
        }
    }
    Update-Path

    # 2. Everything else, by the updater.
    $updater = Join-Path $bin 'tvty-updater.exe'
    if (Test-Path -LiteralPath $updater) {
        Say 'installing what is missing, aiball, Terminal Velocity and the Start menu shortcuts...'
        # Where the release's files are, for the updater too.
        if ($env:TVTY_SETUP_FROM) { $env:TVTY_SETUP_FROM = $from }
        Invoke-Logged $updater '--install'
        if ($LASTEXITCODE) { $failed += 'what the updater installs (the lines above say which)' }
    } else {
        $failed += 'what the updater installs (it is not there to do it)'
    }

    if ($failed.Count) {
        Say "not everything is in place: $($failed -join ', '). Fix it and run this again: what is there is skipped."
        Say "the log of this run, to attach to an issue (https://github.com/quazardous/tvty/issues): $log"
        return
    }
    Say 'done. Terminal Velocity is in the Start menu; its updater keeps it and aiball up to date.'
    Say 'Claude Code asks you to sign in the first time: run `claude` once in a terminal before starting a loop.'
    Say "the log of this run: $log"
}
