param([Parameter(Mandatory)][ValidatePattern('^[a-z0-9-]+$')][string]$Run)
$ErrorActionPreference = 'Stop'
$serverRoot = Split-Path $PSScriptRoot -Parent
$record = Join-Path $serverRoot "work/$Run-server-build.json"
if (Test-Path -LiteralPath $record) { throw 'Build record already exists.' }
Push-Location $serverRoot
try {
    function Get-ServerSources {
        $sources = [ordered]@{}
        $paths = @(& rg --files crates) + @('Cargo.toml', 'Cargo.lock', 'build.ps1')
        if ($LASTEXITCODE -ne 0) { throw 'Cannot enumerate server source.' }
        foreach ($path in ($paths | Sort-Object)) {
            $sources[$path] = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
        }
        return $sources
    }
    $before = Get-ServerSources
    & "$serverRoot/build.ps1" -Action build -CargoArgs @('-p', 'gielinor-server', '--bin', 'gielinor-server')
    $after = Get-ServerSources
    if (($before | ConvertTo-Json -Compress) -ne ($after | ConvertTo-Json -Compress)) {
        throw 'Source changed during build; refusing to archive ambiguous identity.'
    }
    $binary = Join-Path $serverRoot 'work/target/debug/gielinor-server.exe'
    $hash = (Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash
    $archive = Join-Path $serverRoot "work/integration/server/$hash"
    if (Test-Path -LiteralPath $archive) { throw 'Archive already exists; preserving it.' }
    New-Item -ItemType Directory -Path $archive | Out-Null
    Copy-Item -LiteralPath $binary -Destination (Join-Path $archive 'gielinor-server.exe')
    $after | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $archive 'source-hashes.json')
    if ((Get-FileHash -LiteralPath (Join-Path $archive 'gielinor-server.exe')).Hash -ne $hash) {
        throw 'Archived executable copy failed verification.'
    }
    [ordered]@{ run=$Run; sha256=$hash; sourceCount=$after.Count; archive=$archive } | ConvertTo-Json | Set-Content -LiteralPath $record
    Write-Host "Archived server $hash ($($after.Count) source files)."
} finally { Pop-Location }
