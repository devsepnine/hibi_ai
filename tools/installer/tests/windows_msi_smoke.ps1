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
