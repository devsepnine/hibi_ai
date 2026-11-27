param(
    [Parameter(Mandatory = $true)][string]$Directory,
    [Parameter(Mandatory = $true)][string]$Version
)

$ErrorActionPreference = 'Stop'
$Directory = (Resolve-Path -LiteralPath $Directory).Path
$hibi = Join-Path $env:ProgramFiles 'Hibi/hibi.exe'
$marker = Join-Path $env:ProgramFiles 'Hibi/upgrade-probe.txt'

function Install-TestPackage([string]$Name, [int]$ExpectedExitCode) {
    $package = Join-Path $Directory "$Name.msi"
    $log = Join-Path $Directory "$Name.log"
    $result = Start-Process -FilePath "$env:SystemRoot/System32/msiexec.exe" `
        -ArgumentList '/i', "`"$package`"", '/qn', '/norestart', '/l*v', "`"$log`"" -Wait -PassThru
    if ($result.ExitCode -ne $ExpectedExitCode) {
        throw "$Name returned $($result.ExitCode), expected $ExpectedExitCode. See $log."
    }
}

Install-TestPackage previous 0
if (!(Test-Path -LiteralPath $hibi) -or !(Test-Path -LiteralPath $marker)) {
    throw 'The previous package did not install its executable and marker.'
}

Install-TestPackage rollback 1603
if (!(Select-String -LiteralPath (Join-Path $Directory 'rollback.log') -SimpleMatch 'Requested rollback test failure')) {
    throw 'The failing upgrade did not reach the rollback fixture.'
}
if (!(Test-Path -LiteralPath $hibi) -or (Get-Content -LiteralPath $marker) -ne 'previous') {
    throw 'The previous installation was not restored.'
}

Install-TestPackage upgrade 0
if (Test-Path -LiteralPath $marker) {
    throw 'The successful upgrade left a file from the previous package.'
}
if ((& $hibi --version) -ne "hibi v$Version (Config Installer)" -or $LASTEXITCODE -ne 0) {
    throw 'The upgraded executable did not report the expected version.'
}

$existing = @(Get-Process hibi -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id)
function Get-LaunchedHibi {
    Get-Process hibi -ErrorAction SilentlyContinue |
        Where-Object { $_.Id -notin $existing -and $_.Path -eq $hibi }
}

# cmd waits for console executables. Its private console closes when cmd exits.
$shell = Start-Process -FilePath $env:ComSpec -ArgumentList "/d /c `"`"$hibi`"`"" -PassThru
try {
    if (!$shell.WaitForExit(15000) -or $shell.ExitCode -ne 0) {
        throw 'Launching the desktop installer did not return control to cmd.'
    }
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    do {
        $window = Get-LaunchedHibi | Where-Object { $_.MainWindowTitle -eq 'HIBI AI' }
        if ($window) { break }
        Start-Sleep -Milliseconds 150
    } while ([DateTime]::UtcNow -lt $deadline)
    if (!$window) { throw 'The HIBI AI window did not survive its launching console.' }
} finally {
    Get-LaunchedHibi | Stop-Process -Force
    if (!$shell.HasExited) { $shell.Kill() }
    $shell.Dispose()
}
