param([string]$Baseline='client-239.1-baseline-001')
$ErrorActionPreference='Stop'
$serverRoot=Split-Path $PSScriptRoot -Parent
if ($Baseline -notmatch '^[a-zA-Z0-9][a-zA-Z0-9.-]+$' -or $Baseline.Contains('..')) { throw 'Invalid baseline' }
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$python='C:/Users/backup/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe'
& $python "$PSScriptRoot/integration-baseline.py" verify $Baseline
if ($LASTEXITCODE -ne 0) {throw 'Baseline integrity failed'}
$pin="$serverRoot/integration/client-baselines/$Baseline"
$working="$serverRoot/work/preparation/$Baseline"
if (Test-Path "$working/build-attestation.json") {throw 'Build already attested; use existing binaries or a new candidate'}
New-Item -ItemType Directory -Force -Path "$working/bin","$working/cargo" | Out-Null
if (-not (Test-Path "$working/source")) {
    Copy-Item -LiteralPath "$pin/source" -Destination "$working/source" -Recurse
    Get-ChildItem "$working/source" -Recurse -File | ForEach-Object {$_.IsReadOnly=$false}
    Add-Content "$working/source/Cargo.toml" "`n[workspace]"
    foreach ($entry in @(@('NativeDialogueScenario.rs','server_dialogue.rs'),@('NativeInventoryScenario.rs','server_inventory.rs'),@('NativeFloorScenario.rs','server_floor.rs'))) {
        Copy-Item "$serverRoot/reference/$($entry[0])" "$working/source/examples/$($entry[1])"
    }
}
$toolchain='C:/Projects/OSRSRSPS/rust-client/work/rustup/toolchains/stable-x86_64-pc-windows-gnu'
$env:CARGO_HOME="$working/cargo";$env:CARGO_TARGET_DIR="$working/target";$env:CARGO_BUILD_JOBS='1'
$env:CARGO_PROFILE_DEV_DEBUG='0';$env:CARGO_PROFILE_DEV_INCREMENTAL='false';$env:CARGO_PROFILE_DEV_CODEGEN_UNITS='2'
$env:RUSTC="$toolchain/bin/rustc.exe";$env:RUSTDOC="$toolchain/bin/rustdoc.exe"
$env:PATH="$toolchain/bin;$toolchain/lib/rustlib/x86_64-pc-windows-gnu/bin/self-contained;$env:PATH"
$p=Start-Process "$toolchain/bin/cargo.exe" -ArgumentList @('build','--offline','--bin','gielinor-client','--example','live_session','--example','live_actions','--example','reconnect_session','--example','server_dialogue','--example','server_inventory','--example','server_floor') -WorkingDirectory "$working/source" -WindowStyle Hidden -PassThru -RedirectStandardOutput "$working/build.log" -RedirectStandardError "$working/build.err"
$p.PriorityClass='BelowNormal';$p.ProcessorAffinity=[IntPtr]3
try {
    if (-not $p.WaitForExit(600000)) {$p.Kill($true);$p.WaitForExit();throw 'Owned baseline build exceeded ten-minute bound'}
    if ($p.ExitCode -ne 0) {throw "Baseline build failed: $working/build.err"}
} finally {$p.Dispose()}
$binaries=@()
foreach ($name in @('gielinor-client','live_session','live_actions','reconnect_session','server_dialogue','server_inventory','server_floor')) {
    $source=if ($name -eq 'gielinor-client') {"$working/target/debug/$name.exe"} else {"$working/target/debug/examples/$name.exe"}
    Copy-Item $source "$working/bin/$name.exe"
    $binaries+=@{name=$name;sha256=(Get-FileHash $source).Hash}
}
$pinManifest=Get-Content "$pin/manifest.json" -Raw | ConvertFrom-Json -AsHashtable
foreach ($entry in $pinManifest.files.GetEnumerator() | Where-Object {$_.Key.StartsWith('source/src/') -or $_.Key.StartsWith('source/vendor/') -or $_.Key -eq 'source/Cargo.lock'}) {
    if ((Get-FileHash "$working/$($entry.Key)").Hash.ToLowerInvariant() -ne $entry.Value) {throw "Build source diverged: $($entry.Key)"}
}
[ordered]@{baseline=$Baseline;sourceIntegrity='PASS';build='PASS';runtimeValidation='UNVERIFIED';manifestSha256=(Get-FileHash "$pin/manifest.json").Hash;manifestOverlay='Add empty [workspace] to owned working manifest';scenarioOverlays=@('NativeDialogueScenario.rs','NativeInventoryScenario.rs','NativeFloorScenario.rs');binaries=$binaries} | ConvertTo-Json -Depth 5 | Set-Content "$working/build-attestation.json"
