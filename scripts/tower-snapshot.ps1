param([string]$Run='rust-tower-snapshot-001')
$ErrorActionPreference='Stop'
$serverRoot=Split-Path $PSScriptRoot -Parent
if ($Run -notmatch '^[a-zA-Z0-9_-]+$') { throw 'Invalid snapshot run name' }
$output="$serverRoot/work/integration/runs/$Run"
if (Test-Path $output) { throw 'Snapshot run already exists' }
& "$serverRoot/build.ps1" -Action build -CargoArgs @('-p','gielinor-server','--example','tower_snapshot')
if ($LASTEXITCODE -ne 0) { throw 'Snapshot build failed' }
& "$serverRoot/work/target/debug/examples/tower_snapshot.exe" "$serverRoot/work/integration/b-rust/cache/game" "$serverRoot/work/integration/b-rust/symbols" $output
if ($LASTEXITCODE -ne 0) { throw 'Snapshot preparation failed' }
Get-FileHash "$serverRoot/work/target/debug/examples/tower_snapshot.exe" | ConvertTo-Json | Set-Content "$output/binary.json"
