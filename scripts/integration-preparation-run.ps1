param(
 [Parameter(Mandatory)][ValidateSet('kotlin','rust')][string]$Environment,
 [Parameter(Mandatory)][ValidatePattern('^[a-z0-9-]{1,48}$')][string]$Run,
 [ValidateSet('login','actions','dialogue-ui','inventory','inventory-verify','render','floor','floor-content')][string]$Mode='actions',
 [string]$ReuseStateFrom
)
$ErrorActionPreference='Stop'
$serverRoot=Split-Path $PSScriptRoot -Parent
$python='C:/Users/backup/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe'
$root="$serverRoot/work/integration"
$client="$serverRoot/integration/client-builds/client-239.1-baseline-001-build-001"
$serverHash='57917C5CE4204CDC52329F1395ED1B84250AE96090FE34DF39DA9CF84B09AAE7'
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
 try {
  if ($Environment -eq 'rust') { & "$PSScriptRoot/integration-rust-login.ps1" -Run $Run -Mode $Mode -Ticks 300 -StateName "integration-prep-$stateRun" -ClientRoot $client -ServerExecutable $server -CacheRoot "$root/a-kotlin/.data" }
  else { & "$PSScriptRoot/integration-kotlin-login.ps1" -Run $Run -Mode $Mode -Port 43728 -ClientRoot $client -EnvironmentPath $environmentPath }
 } finally {
  if (Test-Path "$runDir/shutdown.json") {
   Copy-Item $before "$runDir/before-state.json"
   & $python "$PSScriptRoot/integration-preparation-state.py" export --run $stateRun --environment $Environment --output "$runDir/state.json"
   if ($ReuseStateFrom) {$fixture="continued-$Environment-$ReuseStateFrom"}
   $afterCache=(& $python "$PSScriptRoot/integration-resources.py" verify --environment kotlin | Out-String).Trim()
   if ($LASTEXITCODE -ne 0 -or $afterCache -ne $cacheId) {throw 'Cache changed during run; preserve evidence and review before comparison'}
   & $python "$PSScriptRoot/integration_telemetry.py" import $runDir "$runDir/events.jsonl" --environment $Environment --scenario $Mode --baseline 'client-239.1-baseline-001-build-001' --fixture $fixture --cache $cacheId
   & $python "$PSScriptRoot/integration-assess.py" $runDir "$runDir/scenario-result.json"
   [ordered]@{baseline='client-239.1-baseline-001-build-001';fixture=$fixture;stateRun=$stateRun;environment=$Environment;mode=$Mode;cacheProvenance='Both environments use the hashed owned Kotlin cache and symbols';reproduction="scripts/integration-preparation-run.ps1 -Environment $Environment -Run new-unique-id -Mode $Mode"} | ConvertTo-Json | Set-Content "$runDir/preparation.json"
  }
 }
} finally {
 $lock.Dispose()
 Remove-Item -LiteralPath $lockPath
}
