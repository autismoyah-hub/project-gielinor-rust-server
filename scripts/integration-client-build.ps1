$ErrorActionPreference='Stop'
$serverRoot=Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$source="$serverRoot/work/integration/client/source"
if (-not (Test-Path "$source/vendor")) { throw 'Prepare the isolated vendor snapshot first' }
$manifest=Get-Content "$source/Cargo.toml" -Raw
if (-not $manifest.Contains('[workspace]')) { Add-Content "$source/Cargo.toml" "`n[workspace]" }
Copy-Item "$serverRoot/reference/NativeDialogueScenario.rs" "$source/examples/server_dialogue.rs"
Copy-Item "$serverRoot/reference/NativeInventoryScenario.rs" "$source/examples/server_inventory.rs"
Copy-Item "$serverRoot/reference/NativeFloorScenario.rs" "$source/examples/server_floor.rs"
$toolchain='C:/Projects/OSRSRSPS/rust-client/work/rustup/toolchains/stable-x86_64-pc-windows-gnu'
$env:CARGO_HOME="$serverRoot/work/integration/client/cargo"
$env:CARGO_TARGET_DIR="$serverRoot/work/integration/client/target"
$env:CARGO_BUILD_JOBS='1'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_DEV_INCREMENTAL='false'
$env:CARGO_PROFILE_DEV_CODEGEN_UNITS='2'
$env:RUSTC="$toolchain/bin/rustc.exe"
$env:RUSTDOC="$toolchain/bin/rustdoc.exe"
$env:PATH="$toolchain/bin;$toolchain/lib/rustlib/x86_64-pc-windows-gnu/bin/self-contained;$env:PATH"
Push-Location $source
try {
    & "$toolchain/bin/cargo.exe" build --offline --example server_dialogue --example server_inventory --example server_floor
    if ($LASTEXITCODE -ne 0) { throw 'Owned native scenario build failed' }
} finally { Pop-Location }
$files=@(Get-ChildItem "$source/src","$source/vendor" -File -Recurse)+@(Get-Item "$source/Cargo.toml","$source/Cargo.lock","$source/examples/server_dialogue.rs","$source/examples/server_inventory.rs","$source/examples/server_floor.rs")
@($files | ForEach-Object { [ordered]@{path=$_.FullName.Substring($source.Length);sha256=(Get-FileHash $_.FullName).Hash} }) | ConvertTo-Json | Set-Content "$serverRoot/work/integration/client/scenario-source-hashes.json"
foreach ($scenario in @('server_dialogue','server_inventory','server_floor')) {
    $binary="$env:CARGO_TARGET_DIR/debug/examples/$scenario.exe"
    $hash=(Get-FileHash $binary).Hash
    $archive="$serverRoot/work/integration/client/scenarios/$hash"
    New-Item -ItemType Directory -Force -Path $archive | Out-Null
    if (-not (Test-Path "$archive/$scenario.exe")) { Copy-Item $binary "$archive/$scenario.exe" }
    Copy-Item "$serverRoot/work/integration/client/scenario-source-hashes.json" "$archive/source-hashes.json"
}
Get-FileHash "$env:CARGO_TARGET_DIR/debug/examples/server_dialogue.exe" | Select-Object Hash,Path
Get-FileHash "$env:CARGO_TARGET_DIR/debug/examples/server_inventory.exe" | Select-Object Hash,Path
Get-FileHash "$env:CARGO_TARGET_DIR/debug/examples/server_floor.exe" | Select-Object Hash,Path
