param([ValidatePattern('^[a-z0-9-]+$')][string]$Driver='dialogue-menu-001',[ValidateSet('dialogue','arrival','inventory','prayer')][string]$Scenario='dialogue')
$ErrorActionPreference='Stop'
$serverRoot=Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$pin="$serverRoot/integration/client-baselines/client-239.1-baseline-001"
$working="$serverRoot/work/preparation/client-239.1-baseline-001"
$out="$serverRoot/integration/scenario-drivers/$Driver"
if (Test-Path $out) {throw 'Driver pin already exists; use a new explicit version'}
$manifest=Get-Content "$pin/manifest.json" -Raw | ConvertFrom-Json -AsHashtable
foreach ($entry in $manifest.files.GetEnumerator() | Where-Object {$_.Key.StartsWith('source/src/') -or $_.Key.StartsWith('source/vendor/') -or $_.Key -eq 'source/Cargo.lock'}) {
 if ((Get-FileHash "$working/$($entry.Key)").Hash.ToLowerInvariant() -ne $entry.Value) {throw 'Working client source differs from immutable baseline'}
}
$source=switch ($Scenario) { 'prayer' {'NativePrayerScenario.rs'} 'arrival' {'NativeArrivalScenario.rs'} 'inventory' {'NativeInventoryScenario.rs'} default {'NativeDialogueScenario.rs'} }
$exampleName="server_${Scenario}_scenario"
$example="$working/source/examples/$exampleName.rs"
Copy-Item "$serverRoot/reference/$source" $example
$toolchain='C:/Projects/OSRSRSPS/rust-client/work/rustup/toolchains/stable-x86_64-pc-windows-gnu'
$env:CARGO_HOME="$working/cargo";$env:CARGO_TARGET_DIR="$working/target";$env:CARGO_BUILD_JOBS='1'
$env:CARGO_PROFILE_DEV_DEBUG='0';$env:CARGO_PROFILE_DEV_INCREMENTAL='false';$env:CARGO_PROFILE_DEV_CODEGEN_UNITS='2'
$env:RUSTC="$toolchain/bin/rustc.exe";$env:RUSTDOC="$toolchain/bin/rustdoc.exe"
$env:PATH="$toolchain/bin;$toolchain/lib/rustlib/x86_64-pc-windows-gnu/bin/self-contained;$env:PATH"
Push-Location "$working/source"
try {
 & "$toolchain/bin/cargo.exe" build --offline --example $exampleName
 if ($LASTEXITCODE -ne 0) {throw 'Scenario driver compilation failed'}
} finally {Pop-Location}
New-Item -ItemType Directory -Path $out | Out-Null
Copy-Item "$working/target/debug/examples/$exampleName.exe" "$out/server_$Scenario.exe"
Copy-Item $example "$out/$source"
$files=@{}
foreach ($name in @("server_$Scenario.exe",$source)) {$files[$name]=(Get-FileHash "$out/$name").Hash}
[ordered]@{driver=$Driver;scenario=$Scenario;baseline='client-239.1-baseline-001-build-001';reason='Owned integration scenario; pinned client library unchanged';files=$files} | ConvertTo-Json -Depth 4 | Set-Content "$out/manifest.json"
Get-ChildItem $out -File | ForEach-Object {$_.IsReadOnly=$true}
