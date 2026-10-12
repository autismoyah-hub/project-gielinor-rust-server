$ErrorActionPreference = 'Stop'
$serverRoot = Split-Path $PSScriptRoot -Parent
$integration = Join-Path $serverRoot 'work/integration'
$clientSource = 'C:/Projects/OSRSRSPS/rust-client'
$referenceData = 'C:/Projects/OSRSRSPS/Project-Gielinor/.data'
New-Item -ItemType Directory -Force -Path "$integration/client/bin","$integration/client/source","$integration/keys","$integration/runs","$integration/a-kotlin/.data/saves","$integration/b-rust" | Out-Null
$binaries = @('gielinor-client.exe','examples/live_session.exe','examples/live_actions.exe','examples/reconnect_session.exe')
$manifest = @()
foreach ($relative in $binaries) {
    $source = Join-Path "$clientSource/target/release" $relative
    $destination = Join-Path "$integration/client/bin" ([IO.Path]::GetFileName($relative))
    if (-not (Test-Path -LiteralPath $destination)) {
        $before = (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash
        Copy-Item -LiteralPath $source -Destination $destination
        if ((Get-FileHash -LiteralPath $source).Hash -ne $before -or (Get-FileHash -LiteralPath $destination).Hash -ne $before) { throw "Client binary changed during snapshot: $relative" }
    }
    $manifest += [pscustomobject]@{name=$relative;sha256=(Get-FileHash -LiteralPath $destination).Hash;sourceLastWriteUtc=(Get-Item -LiteralPath $source).LastWriteTimeUtc.ToString('o')}
}
# Source is a recorded observational snapshot; the existing binary is authoritative.
# No claim is made that uncommitted source was exactly the release build input.
foreach ($entry in @('src','examples','Cargo.toml','Cargo.lock','PROGRESS.md','PARITY_GAPS.md')) {
    $target=Join-Path "$integration/client/source" $entry
    if (-not (Test-Path -LiteralPath $target)) { Copy-Item -LiteralPath "$clientSource/$entry" -Destination $target -Recurse }
}
foreach ($environment in @('a-kotlin/.data','b-rust')) {
    foreach ($cache in @('game','js5')) {
        $target="$integration/$environment/cache/$cache"
        if (-not (Test-Path -LiteralPath $target)) {
            New-Item -ItemType Directory -Force -Path "$integration/$environment/cache" | Out-Null
            Copy-Item -LiteralPath "$referenceData/cache/$cache" -Destination $target -Recurse
        }
    }
    $target="$integration/$environment/symbols"
    if (-not (Test-Path -LiteralPath $target)) { Copy-Item -LiteralPath "$referenceData/symbols" -Destination $target -Recurse }
}
if (-not (Test-Path -LiteralPath "$integration/keys/game.key")) {
    $rsa=[Security.Cryptography.RSA]::Create(1024)
    try {
        $key=$rsa.ExportParameters($true)
        # Kotlin's generator labels a PKCS#8 payload RSA PRIVATE KEY.
        $pem=$rsa.ExportPkcs8PrivateKeyPem().Replace('BEGIN PRIVATE KEY','BEGIN RSA PRIVATE KEY').Replace('END PRIVATE KEY','END RSA PRIVATE KEY')
        [IO.File]::WriteAllText("$integration/keys/game.key",$pem)
        [IO.File]::WriteAllBytes("$integration/keys/exponent.bin",$key.D)
        [IO.File]::WriteAllBytes("$integration/keys/modulus.bin",$key.Modulus)
        [IO.File]::WriteAllText("$integration/keys/client.key","Exponent: 10001`nModulus: $([Convert]::ToHexString($key.Modulus).ToLowerInvariant())`n")
    } finally { $rsa.Dispose() }
}
foreach ($part in @('game.key','client.key')) { Copy-Item -LiteralPath "$integration/keys/$part" -Destination "$integration/a-kotlin/.data/$part" }
if (-not (Test-Path -LiteralPath "$integration/test-account.password")) {
    [IO.File]::WriteAllText("$integration/test-account.password",[Convert]::ToHexString([Security.Cryptography.RandomNumberGenerator]::GetBytes(24)))
}
[ordered]@{createdUtc=[DateTime]::UtcNow.ToString('o');revision='239.1';clientBinaries=$manifest;clientSourceProvenance='Contemporaneous source copy; binary SHA256 pins comparison identity, exact release source mapping is not attested';kotlinSource='cb6e66cc0f16a25b60ba6705d8f157cc22ab4869 with documented isolated build overlays';rustServerSha256=(Get-FileHash "$serverRoot/work/target/debug/gielinor-server.exe").Hash;environments=@{kotlin='a-kotlin';rust='b-rust';execution='sequential';account='integration239'}} | ConvertTo-Json -Depth 6 | Set-Content "$integration/versions.json"
Write-Host "Prepared isolated integration files at $integration"
