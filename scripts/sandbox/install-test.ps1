# install-test.ps1 - runs inside a fresh Windows Sandbox (started by
# scripts/sandbox_install_test.py): the whole Windows setup as a user gets
# it, then what is there. Everything lands in C:\tvty-home\install-test,
# which the host reads:
#   out.txt    what was said, step by step
#   setup.log  tvty-setup.ps1's own log
#   desk.png   the whole desktop at the end (error dialogs show there, and
#              on no window's screenshot)
#   done       written last
# With a `release` folder beside it, the release's files come from there
# (a local build); without, from the published release. With an
# `aiball-ref.txt`, aiball is installed at that tag or branch.
$here = 'C:\tvty-home\install-test'
Remove-Item "$here\done" -ErrorAction SilentlyContinue
Start-Transcript -Path "$here\out.txt" -Force | Out-Null
$ProgressPreference = 'SilentlyContinue'
function Step($text) { "=== $text  [$(Get-Date -Format HH:mm:ss)]" }

Step 'network'
try { "github: " + (Invoke-WebRequest -UseBasicParsing -Uri 'https://github.com' -TimeoutSec 20).StatusCode } catch { "github: NO NETWORK ($_)" }

# The sandbox has no winget. Its packages are downloaded by their release
# links: Microsoft's own way there (Repair-WinGetPackageManager) asks
# GitHub's API, which refuses after a few runs from one address.
Step 'winget'
if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        $temp = Join-Path $env:TEMP 'winget-install'
        New-Item -ItemType Directory -Force $temp | Out-Null
        $release = 'https://github.com/microsoft/winget-cli/releases/latest/download'
        Invoke-WebRequest -UseBasicParsing -Uri "$release/DesktopAppInstaller_Dependencies.zip" -OutFile "$temp\dependencies.zip"
        Expand-Archive -Force "$temp\dependencies.zip" "$temp\dependencies"
        Get-ChildItem "$temp\dependencies\x64\*.appx" | ForEach-Object { Add-AppxPackage -Path $_.FullName }
        Invoke-WebRequest -UseBasicParsing -Uri "$release/Microsoft.DesktopAppInstaller_8wekyb3d8bbwe.msixbundle" -OutFile "$temp\winget.msixbundle"
        Add-AppxPackage -Path "$temp\winget.msixbundle"
    } catch { "winget did not install: $_" }
    $env:Path += ";$env:LOCALAPPDATA\Microsoft\WindowsApps"
}
"winget: " + $(if (Get-Command winget -ErrorAction SilentlyContinue) { winget --version } else { 'MISSING' })

Step 'tvty-setup.ps1, through iex'
if (Test-Path "$here\release") { $env:TVTY_SETUP_FROM = "$here\release" }
if (Test-Path "$here\aiball-ref.txt") {
    $env:TVTY_AIBALL_REF = (Get-Content "$here\aiball-ref.txt" -Raw).Trim()
    "aiball at: $env:TVTY_AIBALL_REF"
}
Get-Content -Raw "$here\tvty-setup.ps1" | Invoke-Expression

Step 'what is there'
$env:Path = [Environment]::GetEnvironmentVariable('Path', 'Machine') + ';' + [Environment]::GetEnvironmentVariable('Path', 'User')
foreach ($command in 'git', 'node', 'pwsh', 'psmux', 'aiball', 'tvty', 'tvty-updater') {
    $found = Get-Command $command -ErrorAction SilentlyContinue
    "{0,-13} {1}" -f $command, $(if ($found) { $found.Source } else { 'MISSING' })
}
"shortcuts: " + ((Get-ChildItem "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Terminal Velocity*" -ErrorAction SilentlyContinue | ForEach-Object Name) -join ', ')
"tvty: " + (& "$HOME\.local\bin\tvty.exe" --version | Out-String).Trim()
"aiball: " + (cmd /c "aiball --json version" 2>&1 | Out-String).Trim()
Copy-Item "$env:LOCALAPPDATA\tvty\setup.log" "$here\setup.log" -ErrorAction SilentlyContinue

Step 'aiball, once it had time to start'
Start-Sleep -Seconds 20
"daemon: " + $(try { (Invoke-WebRequest -UseBasicParsing -Uri 'http://127.0.0.1:7777/api/health' -TimeoutSec 5).Content.Substring(0, 60) } catch { "no answer ($_)" })
"processes: " + ((Get-Process node, conhost, wscript, powershell, pwsh -ErrorAction SilentlyContinue | Group-Object ProcessName | ForEach-Object { "$($_.Name) x$($_.Count)" }) -join ', ')
"task: " + ((Get-ScheduledTask -TaskName aiball-daemon -ErrorAction SilentlyContinue | ForEach-Object { "$($_.State): $($_.Actions[0].Execute) $($_.Actions[0].Arguments)" }) -join '; ')
"vbs left: " + ((Get-ChildItem "$env:LOCALAPPDATA\aiball\*.vbs" -ErrorAction SilentlyContinue | ForEach-Object Name) -join ', ')

Step 'the desktop'
try {
    Add-Type -AssemblyName System.Windows.Forms, System.Drawing
    $bounds = [System.Windows.Forms.SystemInformation]::VirtualScreen
    $bitmap = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
    [System.Drawing.Graphics]::FromImage($bitmap).CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
    $bitmap.Save("$here\desk.png", [System.Drawing.Imaging.ImageFormat]::Png)
    "desk.png: $($bounds.Width)x$($bounds.Height)"
} catch { "no picture of the desktop: $_" }

Step 'end'
Stop-Transcript | Out-Null
'done' | Out-File "$here\done"
