param([scriptblock] $ReportFailure = {
    param($reason)
    Add-Type -AssemblyName PresentationFramework
    [void][System.Windows.MessageBox]::Show($reason, 'hibi update')
})
$ErrorActionPreference = 'Stop'
Remove-Item Env:HIBI_UPDATE_FAILURE -ErrorAction SilentlyContinue
$root = $env:HIBI_UPDATE_ROOT
$version = $env:HIBI_UPDATE_TO
$previous = $null
$promoted = $false
$lock = $null
try {
    $parent = Get-Process -Id $env:HIBI_UPDATE_PARENT -ErrorAction SilentlyContinue
    if ($parent -and !$parent.WaitForExit(30000)) { throw 'Close hibi and try again.' }
    if ($env:HIBI_UPDATE_KIND -eq 'scoop') {
        & scoop update
        if ($LASTEXITCODE -ne 0) { throw 'Scoop metadata refresh failed.' }
        $record = Join-Path $root 'scoop-install.json'
        if (!(Test-Path -LiteralPath $record)) { $record = Join-Path $root 'install.json' }
        $bucket = (Get-Content -LiteralPath $record -Raw | ConvertFrom-Json).bucket
        if ($bucket -notmatch '^[a-zA-Z0-9][a-zA-Z0-9._-]*$') { throw 'Unknown Scoop bucket.' }
        $manifest = (& scoop cat "$bucket/hibi-ai" | Out-String | ConvertFrom-Json)
        if ($manifest.version -ne $version -or $manifest.homepage -ne 'https://github.com/devsepnine/hibi_ai') {
            throw 'Scoop manifest changed. Check again.'
        }
        $apps = @(& scoop list '^hibi-ai$' 6>$null | Where-Object { $_.Name -eq 'hibi-ai' })
        if (!$apps.Count) { throw 'Scoop installation record unavailable.' }
        $local = @($apps | Where-Object { $_.Info -notlike '*Global install*' })
        if ($local.Count) { & scoop update hibi-ai } else { & scoop update --global hibi-ai }
        if ($LASTEXITCODE -ne 0) { throw 'Scoop update failed.' }
        $destination = (& scoop prefix hibi-ai | Out-String).Trim()
    } else {
        $lockPath = Join-Path (Split-Path -Parent $root) '.hibi-update.lock'
        $lock = [IO.File]::Open($lockPath, 'OpenOrCreate', 'Write', 'None')
        if ((Get-Content -Raw -LiteralPath (Join-Path $root '.hibi-bundle')).Trim() -ne $env:HIBI_UPDATE_FROM) {
            throw 'Installed version changed. Check again.'
        }
        $stage = $env:HIBI_UPDATE_STAGE
        $backup = Join-Path $stage 'previous'
        [IO.Directory]::Move($root, $backup)
        $previous = $backup
        [IO.Directory]::Move((Join-Path $stage 'bundle'), $root)
        $promoted = $true
        $destination = $root
    }
    $exe = Join-Path $destination 'hibi.exe'
    if ((& $exe --version) -ne ("hibi v$version (Config Installer)") -or $LASTEXITCODE -ne 0) {
        throw 'Updated executable version check failed.'
    }
    Start-Process -FilePath $exe -WorkingDirectory $destination
} catch {
    $reason = $_.Exception.Message
    if ($previous) {
        try {
            if ($promoted) { [IO.Directory]::Move($root, (Join-Path $stage 'failed')) }
            [IO.Directory]::Move($previous, $root)
        } catch { $reason += " Restore the previous installation from $previous." }
    }
    try {
        $env:HIBI_UPDATE_FAILURE = $reason
        Start-Process -FilePath (Join-Path $root 'hibi.exe') -WorkingDirectory $root
    } catch {
        & $ReportFailure $reason
    }
    exit 1
} finally {
    if ($lock) { $lock.Dispose() }
}
