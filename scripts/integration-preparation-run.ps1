param(
 [Parameter(Mandatory)][ValidateSet('kotlin','rust')][string]$Environment,
 [Parameter(Mandatory)][ValidatePattern('^[a-z0-9-]{1,48}$')][string]$Run,
 [ValidateSet('login','actions','dialogue-ui','arrival','prayer','inventory','inventory-verify','render','floor','floor-content')][string]$Mode='actions',
 [string]$ReuseStateFrom,
 [ValidatePattern('^[a-z0-9-]+$')][string]$DialogueDriver,
 [ValidatePattern('^[a-z0-9-]+$')][string]$ArrivalDriver,
 [ValidatePattern('^[a-z0-9-]+$')][string]$InventoryDriver,
 [ValidatePattern('^[a-z0-9-]+$')][string]$PrayerDriver,
 [ValidatePattern('^[A-Fa-f0-9]{64}$')][string]$ServerHash='57917C5CE4204CDC52329F1395ED1B84250AE96090FE34DF39DA9CF84B09AAE7'
)
$ErrorActionPreference='Stop'
$serverRoot=Split-Path $PSScriptRoot -Parent
$python='C:/Users/backup/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe'
$root="$serverRoot/work/integration"
$client="$serverRoot/integration/client-builds/client-239.1-baseline-001-build-001"
$driverArgs=@{}
if ($PrayerDriver) {
 if ($Mode -ne 'prayer' -or $DialogueDriver -or $ArrivalDriver -or $InventoryDriver) {throw 'Prayer driver only applies to prayer'}
 $driverRoot="$serverRoot/integration/scenario-drivers/$PrayerDriver"
 $driverManifest=Get-Content "$driverRoot/manifest.json" -Raw | ConvertFrom-Json -AsHashtable
 if ($driverManifest.baseline -ne 'client-239.1-baseline-001-build-001' -or $driverManifest.scenario -ne 'prayer') {throw 'Prayer driver provenance mismatch'}
 foreach ($entry in $driverManifest.files.GetEnumerator()) {
  if ((Get-FileHash "$driverRoot/$($entry.Key)").Hash -ne $entry.Value) {throw 'Scenario driver modified'}
 }
 $driverArgs.ScenarioExecutable="$driverRoot/server_prayer.exe"
}
if ($Mode -eq 'prayer' -and -not $PrayerDriver) {throw 'Prayer requires an explicitly pinned driver'}
if ($InventoryDriver) {
 if ($Mode -notin @('inventory','inventory-verify') -or $DialogueDriver -or $ArrivalDriver) {throw 'Inventory driver only applies to inventory modes'}
 $driverRoot="$serverRoot/integration/scenario-drivers/$InventoryDriver"
 $driverManifest=Get-Content "$driverRoot/manifest.json" -Raw | ConvertFrom-Json -AsHashtable
 if ($driverManifest.baseline -ne 'client-239.1-baseline-001-build-001' -or $driverManifest.scenario -ne 'inventory') {throw 'Inventory driver provenance mismatch'}
 foreach ($entry in $driverManifest.files.GetEnumerator()) {
  if ((Get-FileHash "$driverRoot/$($entry.Key)").Hash -ne $entry.Value) {throw 'Scenario driver modified'}
 }
 $driverArgs.ScenarioExecutable="$driverRoot/server_inventory.exe"
}
if ($ArrivalDriver) {
 if ($Mode -ne 'arrival' -or $DialogueDriver) {throw 'Arrival driver only applies to arrival'}
 $driverRoot="$serverRoot/integration/scenario-drivers/$ArrivalDriver"
 $driverManifest=Get-Content "$driverRoot/manifest.json" -Raw | ConvertFrom-Json -AsHashtable
 if ($driverManifest.baseline -ne 'client-239.1-baseline-001-build-001' -or $driverManifest.scenario -ne 'arrival') {throw 'Arrival driver provenance mismatch'}
 foreach ($entry in $driverManifest.files.GetEnumerator()) {
  if ((Get-FileHash "$driverRoot/$($entry.Key)").Hash -ne $entry.Value) {throw 'Scenario driver modified'}
 }
 $driverArgs.ScenarioExecutable="$driverRoot/server_arrival.exe"
}
if ($Mode -eq 'arrival' -and -not $ArrivalDriver) {throw 'Arrival requires an explicitly pinned driver'}
if ($DialogueDriver) {
 if ($Mode -ne 'dialogue-ui') {throw 'Dialogue driver override only applies to dialogue-ui'}
 $driverRoot="$serverRoot/integration/scenario-drivers/$DialogueDriver"
 $driverManifest=Get-Content "$driverRoot/manifest.json" -Raw | ConvertFrom-Json -AsHashtable
 if ($driverManifest.baseline -ne 'client-239.1-baseline-001-build-001') {throw 'Scenario driver baseline mismatch'}
 foreach ($entry in $driverManifest.files.GetEnumerator()) {
  if ((Get-FileHash "$driverRoot/$($entry.Key)").Hash -ne $entry.Value) {throw 'Scenario driver modified'}
 }
 $driverArgs.ScenarioExecutable="$driverRoot/server_dialogue.exe"
}
$server="$root/server/$serverHash/gielinor-server.exe"
. "$PSScriptRoot/integration-capacity.ps1"
Assert-IntegrationRunCapacity
if (Test-Path "$root/runs/$Run") {throw 'Run ID already exists'}
if ($ReuseStateFrom -and $ReuseStateFrom -notmatch '^[a-z0-9-]{1,48}$') {throw 'Invalid prior run ID'}
$manifest=Get-Content "$client/manifest.json" -Raw | ConvertFrom-Json -AsHashtable
foreach ($entry in $manifest.files.GetEnumerator()) {
 if ((Get-FileHash "$client/$($entry.Key)").Hash.ToLowerInvariant() -ne $entry.Value) {throw "Pinned client modified: $($entry.Key)"}
}
if ((Get-FileHash $server).Hash -ne $serverHash) {throw 'Pinned server mismatch'}
if ($Mode -like 'floor*') {
 $floorInput=Get-Content "$serverRoot/integration/floor-scenario-input.json" -Raw | ConvertFrom-Json
 if ((Get-FileHash "$serverRoot/$($floorInput.path)").Hash.ToLowerInvariant() -ne $floorInput.sha256) {throw 'Tower scene reference changed'}
}
$cacheId=(& $python "$PSScriptRoot/integration-resources.py" verify --environment kotlin | Out-String).Trim()
if ($LASTEXITCODE -ne 0) {throw 'Shared owned cache validation failed'}
New-Item -ItemType Directory -Force -Path "$serverRoot/work/preparation" | Out-Null
$lockPath="$serverRoot/work/preparation/session.lock"
$lock=[IO.File]::Open($lockPath,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::None)
$stateRun=if ($ReuseStateFrom) {$ReuseStateFrom} else {$Run}
$runDir="$root/runs/$Run"
try {
 if ($ReuseStateFrom) {
  $fixture=(& $python "$PSScriptRoot/integration_continuation.py" "$root/runs/$ReuseStateFrom" $Environment | Out-String).Trim()
  if ($LASTEXITCODE -ne 0) {throw 'Predecessor scenario is not eligible for continuation'}
  $predecessor=Get-Content "$root/runs/$ReuseStateFrom/preparation.json" -Raw | ConvertFrom-Json
  $stateRun=$predecessor.stateRun
  if (-not $stateRun -or $stateRun -notmatch '^[a-z0-9-]{1,48}$') {throw 'Predecessor state directory is invalid'}
 }
 if (-not $ReuseStateFrom) {
  $fixturePreset=if ($Mode -like 'floor*') {'tower'} else {'basic'}
  $provision=(& $python "$PSScriptRoot/integration-preparation-state.py" clone --run $Run --environment $Environment --fixture $fixturePreset | Out-String).Trim()
  if ($LASTEXITCODE -ne 0) {throw 'Fixture provisioning failed'}
  $fixture=($provision | ConvertFrom-Json).fixture_id
 }
 $environmentPath="$root/preparation/$stateRun"
 if ($Environment -eq 'kotlin' -and -not $ReuseStateFrom) {
  New-Item -ItemType Directory -Force -Path "$environmentPath/.data/cache" | Out-Null
  foreach ($cache in @('game','js5','vanilla','enriched')) {
   New-Item -ItemType Junction -Path "$environmentPath/.data/cache/$cache" -Target "$root/a-kotlin/.data/cache/$cache" | Out-Null
  }
  New-Item -ItemType Junction -Path "$environmentPath/.data/symbols" -Target "$root/a-kotlin/.data/symbols" | Out-Null
  foreach ($name in @('server.toml','game.key','client.key')) {Copy-Item "$root/a-kotlin/.data/$name" "$environmentPath/.data/$name"}
 }
 $before="$serverRoot/work/preparation/$Run-before.json"
 & $python "$PSScriptRoot/integration-preparation-state.py" export --run $stateRun --environment $Environment --output $before
 if ($LASTEXITCODE -ne 0) {throw 'Initial state export failed'}
 $towerSource=if ($Environment -eq 'rust') {"$serverRoot/work/state/integration-prep-$stateRun/accounts.sqlite"} else {"$environmentPath/.data/saves/tower-floor1/integration239"}
 $towerBefore="$serverRoot/work/preparation/$Run-tower-before.json"
 & $python "$PSScriptRoot/integration_tower_state.py" $Environment $towerSource $towerBefore
 if ($LASTEXITCODE -ne 0) {throw 'Initial Tower state export failed'}
 try {
  if ($Environment -eq 'rust') { & "$PSScriptRoot/integration-rust-login.ps1" -Run $Run -Mode $Mode -Ticks 300 -StateName "integration-prep-$stateRun" -ClientRoot $client -ServerExecutable $server -CacheRoot "$root/a-kotlin/.data" @driverArgs }
  else { & "$PSScriptRoot/integration-kotlin-login.ps1" -Run $Run -Mode $Mode -Port 43728 -ClientRoot $client -EnvironmentPath $environmentPath @driverArgs }
 } finally {
  if (Test-Path "$runDir/shutdown.json") {
   Copy-Item $before "$runDir/before-state.json"
   Copy-Item $towerBefore "$runDir/before-tower-state.json"
   & $python "$PSScriptRoot/integration_tower_state.py" $Environment $towerSource "$runDir/tower-state.json"
   if ($LASTEXITCODE -ne 0) {throw 'Tower state export failed'}
   & $python "$PSScriptRoot/integration-preparation-state.py" export --run $stateRun --environment $Environment --output "$runDir/state.json"
   $afterCache=(& $python "$PSScriptRoot/integration-resources.py" verify --environment kotlin | Out-String).Trim()
   if ($LASTEXITCODE -ne 0 -or $afterCache -ne $cacheId) {throw 'Cache changed during run; preserve evidence and review before comparison'}
   & $python "$PSScriptRoot/integration_telemetry.py" import $runDir "$runDir/events.jsonl" --environment $Environment --scenario $Mode --baseline 'client-239.1-baseline-001-build-001' --fixture $fixture --cache $cacheId
   & $python "$PSScriptRoot/integration-assess.py" $runDir "$runDir/scenario-result.json"
   $reproduce="scripts/integration-preparation-run.ps1 -Environment $Environment -Run new-unique-id -Mode $Mode -ServerHash $ServerHash"
   if ($DialogueDriver) {$reproduce+=" -DialogueDriver $DialogueDriver"}
   if ($ArrivalDriver) {$reproduce+=" -ArrivalDriver $ArrivalDriver"}
   if ($InventoryDriver) {$reproduce+=" -InventoryDriver $InventoryDriver"}
   if ($PrayerDriver) {$reproduce+=" -PrayerDriver $PrayerDriver"}
   if ($ReuseStateFrom) {$reproduce+=" -ReuseStateFrom $ReuseStateFrom"}
   [ordered]@{baseline='client-239.1-baseline-001-build-001';serverHash=$ServerHash;dialogueDriver=$DialogueDriver;arrivalDriver=$ArrivalDriver;inventoryDriver=$InventoryDriver;prayerDriver=$PrayerDriver;fixture=$fixture;stateRun=$stateRun;environment=$Environment;mode=$Mode;cacheProvenance='Both environments use the hashed owned Kotlin cache and symbols';reproduction=$reproduce} | ConvertTo-Json | Set-Content "$runDir/preparation.json"
  }
 }
} finally {
 $lock.Dispose()
 Remove-Item -LiteralPath $lockPath
}
