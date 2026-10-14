param(
 [Parameter(Mandatory)][ValidateSet('kotlin','rust')][string]$Environment,
 [Parameter(Mandatory)][ValidatePattern('^[a-z0-9-]{1,48}$')][string]$Run,
 [ValidateSet('tower-diary','login','actions','dialogue-ui','arrival','prayer','inventory','inventory-verify','render','floor','floor-content')][string]$Mode='actions',
 [string]$ReuseStateFrom,
 [ValidatePattern('^[a-z0-9-]+$')][string]$TowerDriver,
 [switch]$CreatorResume,
 [switch]$KnownEmptyDiary,
 [ValidatePattern('^[a-z0-9-]+$')][string]$DialogueDriver,
 [ValidatePattern('^[a-z0-9-]+$')][string]$ArrivalDriver,
 [ValidatePattern('^[a-z0-9-]+$')][string]$InventoryDriver,
 [ValidatePattern('^[a-z0-9-]+$')][string]$PrayerDriver,
 [ValidatePattern('^[A-Fa-f0-9]{64}$')][string]$ServerHash='57917C5CE4204CDC52329F1395ED1B84250AE96090FE34DF39DA9CF84B09AAE7'
)
$ErrorActionPreference='Stop'
if ($CreatorResume -and ($ReuseStateFrom -or $Mode -notin @('login','render'))) {throw 'CreatorResume requires a new login/render fixture'}
if ($KnownEmptyDiary -and $ReuseStateFrom) {throw 'KnownEmptyDiary requires a new fixture; existing saves are preserved'}
$serverRoot=Split-Path $PSScriptRoot -Parent
$python='C:/Users/backup/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe'
$root="$serverRoot/work/integration"
$client="$serverRoot/integration/client-builds/client-239.1-baseline-001-build-001"
$driverArgs=@{}
if ($Mode -eq 'tower-diary' -and (-not $KnownEmptyDiary -or $ReuseStateFrom -or -not $TowerDriver)) {throw 'Tower diary requires a new KnownEmptyDiary fixture and explicitly pinned TowerDriver'}
if ($TowerDriver) {
 if ($Mode -ne 'tower-diary' -or $DialogueDriver -or $ArrivalDriver -or $InventoryDriver -or $PrayerDriver) {throw 'Tower driver only applies to tower-diary'}
 $driverRoot="$serverRoot/integration/scenario-drivers/$TowerDriver"
 $driverManifest=Get-Content "$driverRoot/manifest.json" -Raw | ConvertFrom-Json -AsHashtable
 if ($driverManifest.baseline -ne 'client-239.1-baseline-001-build-001' -or $driverManifest.scenario -ne 'tower') {throw 'Tower driver provenance mismatch'}
 foreach ($entry in $driverManifest.files.GetEnumerator()) {
  if ((Get-FileHash "$driverRoot/$($entry.Key)").Hash -ne $entry.Value) {throw 'Scenario driver modified'}
 }
 $driverArgs.ScenarioExecutable="$driverRoot/server_tower.exe"
}

if ($CreatorResume) {$driverArgs.CreatorResume=$true}
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
if ($KnownEmptyDiary -and $Environment -eq 'rust') {
 $sourceManifest=Get-Content "$root/server/$serverHash/source-hashes.json" -Raw | ConvertFrom-Json -AsHashtable
 if (-not @($sourceManifest.Keys | Where-Object { $_ -match 'V11__tower_bones_offered\.sql$' }).Count) {throw 'KnownEmptyDiary requires a V11-capable archived server'}
}
if ($Mode -like 'floor*' -or $Mode -eq 'tower-diary') {
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
  $fixturePreset=if ($CreatorResume) {'creator-resume'} elseif ($Mode -eq 'tower-diary') {'tower-tanning'} elseif ($Mode -like 'floor*') {'tower'} else {'basic'}
  [string[]]$diaryArgs=@()
  if ($KnownEmptyDiary) {$diaryArgs=@('--known-empty-diary')}
  $provision=(& $python "$PSScriptRoot/integration-preparation-state.py" clone --run $Run --environment $Environment --fixture $fixturePreset @diaryArgs | Out-String).Trim()
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
 $diaryBefore="$serverRoot/work/preparation/$Run-diary-before.json"
 & $python "$PSScriptRoot/integration_diary_state.py" $Environment $towerSource $diaryBefore
 if ($LASTEXITCODE -ne 0) {throw 'Initial diary state export failed'}
 try {
  if ($Environment -eq 'rust') { & "$PSScriptRoot/integration-rust-login.ps1" -Run $Run -Mode $Mode -Ticks 300 -StateName "integration-prep-$stateRun" -ClientRoot $client -ServerExecutable $server -CacheRoot "$root/a-kotlin/.data" @driverArgs }
  else { & "$PSScriptRoot/integration-kotlin-login.ps1" -Run $Run -Mode $Mode -Port 43728 -ClientRoot $client -EnvironmentPath $environmentPath @driverArgs }
 } finally {
  if (Test-Path "$runDir/shutdown.json") {
   Copy-Item $before "$runDir/before-state.json"
   Copy-Item $towerBefore "$runDir/before-tower-state.json"
   Copy-Item $diaryBefore "$runDir/before-diary-state.json"
   & $python "$PSScriptRoot/integration_diary_state.py" $Environment $towerSource "$runDir/diary-state.json"
   if ($LASTEXITCODE -ne 0) {throw 'Final diary state export failed'}
   & $python "$PSScriptRoot/integration_tower_state.py" $Environment $towerSource "$runDir/tower-state.json"
   if ($LASTEXITCODE -ne 0) {throw 'Tower state export failed'}
   & $python "$PSScriptRoot/integration-preparation-state.py" export --run $stateRun --environment $Environment --output "$runDir/state.json"
   $afterCache=(& $python "$PSScriptRoot/integration-resources.py" verify --environment kotlin | Out-String).Trim()
   if ($LASTEXITCODE -ne 0 -or $afterCache -ne $cacheId) {throw 'Cache changed during run; preserve evidence and review before comparison'}
   & $python "$PSScriptRoot/integration_telemetry.py" import $runDir "$runDir/events.jsonl" --environment $Environment --scenario $Mode --baseline 'client-239.1-baseline-001-build-001' --fixture $fixture --cache $cacheId
   & $python "$PSScriptRoot/integration-assess.py" $runDir "$runDir/scenario-result.json"
   $reproduce="scripts/integration-preparation-run.ps1 -Environment $Environment -Run new-unique-id -Mode $Mode -ServerHash $ServerHash"
   if ($TowerDriver) {$reproduce+=" -TowerDriver $TowerDriver"}
   if ($DialogueDriver) {$reproduce+=" -DialogueDriver $DialogueDriver"}
   if ($ArrivalDriver) {$reproduce+=" -ArrivalDriver $ArrivalDriver"}
   if ($InventoryDriver) {$reproduce+=" -InventoryDriver $InventoryDriver"}
   if ($PrayerDriver) {$reproduce+=" -PrayerDriver $PrayerDriver"}
   if ($ReuseStateFrom) {$reproduce+=" -ReuseStateFrom $ReuseStateFrom"}
   if ($CreatorResume) {$reproduce+=' -CreatorResume'}
   if ($KnownEmptyDiary) {$reproduce+=' -KnownEmptyDiary'}
   [ordered]@{baseline='client-239.1-baseline-001-build-001';serverHash=$ServerHash;dialogueDriver=$DialogueDriver;arrivalDriver=$ArrivalDriver;inventoryDriver=$InventoryDriver;prayerDriver=$PrayerDriver;towerDriver=$TowerDriver;fixture=$fixture;stateRun=$stateRun;environment=$Environment;mode=$Mode;creatorResume=[bool]$CreatorResume;knownEmptyDiary=[bool]$KnownEmptyDiary;cacheProvenance='Both environments use the hashed owned Kotlin cache and symbols';reproduction=$reproduce} | ConvertTo-Json | Set-Content "$runDir/preparation.json"
  }
 }
} finally {
 $lock.Dispose()
 Remove-Item -LiteralPath $lockPath
}
