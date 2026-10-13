param([ValidateSet('test','build','check','fmt','run')][string]$Action = 'test', [Parameter(ValueFromRemainingArguments=$true)][string[]]$CargoArgs)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot/scripts/resources.ps1"
Assert-ServerBuildCapacity
# Existing compiler binaries are read-only; all Cargo state belongs to Agent 2.
$toolchain = 'C:\Projects\OSRSRSPS\rust-client\work\rustup\toolchains\stable-x86_64-pc-windows-gnu'
$env:CARGO_HOME = "$PSScriptRoot/work/cargo"
$env:CARGO_TARGET_DIR = "$PSScriptRoot/work/target"
$env:CARGO_BUILD_JOBS = '1'
$env:RUSTC = "$toolchain/bin/rustc.exe"
$env:RUSTDOC = "$toolchain/bin/rustdoc.exe"
$env:PATH = "$toolchain/bin;$toolchain/lib/rustlib/x86_64-pc-windows-gnu/bin/self-contained;$env:PATH"
Push-Location $PSScriptRoot
try {
    if ($Action -eq 'fmt') { & "$toolchain/bin/cargo.exe" fmt @CargoArgs }
    else { & "$toolchain/bin/cargo.exe" $Action --offline @CargoArgs }
    if ($LASTEXITCODE -ne 0) { throw "Cargo $Action failed ($LASTEXITCODE)." }
} finally { Pop-Location }
