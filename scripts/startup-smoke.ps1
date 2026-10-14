param(
    [string]$GameCache = 'C:/Projects/OSRSRSPS/Project-Gielinor/.data/cache/game',
    [string]$Js5Cache = 'C:/Projects/OSRSRSPS/Project-Gielinor/.data/cache/js5',
    [string]$Symbols = 'C:/Projects/OSRSRSPS/Project-Gielinor/.data/symbols',
    [ValidatePattern('^[a-z0-9-]+$')][string]$StateName = 'executable-smoke',
    [string]$ElementalSource
)
$ErrorActionPreference = 'Stop'
$serverRoot = Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$keys = Join-Path $serverRoot 'work/startup-smoke-keys'
New-Item -ItemType Directory -Force -Path $keys | Out-Null
# Publicly checked-in oracle keys, exclusively for this isolated smoke test.
foreach ($part in @('exponent', 'modulus')) {
    $hex = (Get-Content -LiteralPath "$serverRoot/reference/protocol/login-fixture/rsa-$part.hex" -Raw).Trim()
    [IO.File]::WriteAllBytes((Join-Path $keys "$part.bin"), [Convert]::FromHexString($hex))
}
$arguments = @(
    '-p', 'gielinor-server', '--',
    '--game-cache', $GameCache, '--js5-cache', $Js5Cache, '--symbols', $Symbols,
    '--rsa-exponent', (Join-Path $keys 'exponent.bin'),
    '--rsa-modulus', (Join-Path $keys 'modulus.bin'),
    '--argon2', "$serverRoot/work/native/argon2.dll",
    '--state-name', $StateName, '--bind', '127.0.0.1:0', '--ticks', '2'
)
if ($ElementalSource) { $arguments += @('--elemental-source', $ElementalSource) }
& "$serverRoot/build.ps1" -Action run -CargoArgs $arguments
