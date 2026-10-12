$ErrorActionPreference = 'Stop'
$serverRoot = Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$fastutil = @(Get-ChildItem -LiteralPath 'C:/Users/backup/.gradle/caches/modules-2/files-2.1/it.unimi.dsi/fastutil/8.5.16' -Recurse -Filter 'fastutil-8.5.16.jar')
if ($fastutil.Count -ne 1) { throw 'Expected one cached fastutil jar.' }
if (-not (Test-Path "$serverRoot/work/rsprot/fastutil-8.5.16.jar")) { Copy-Item -LiteralPath $fastutil[0].FullName -Destination "$serverRoot/work/rsprot/fastutil-8.5.16.jar" }
$jdk = 'C:/Program Files/Eclipse Adoptium/jdk-21.0.12.101-hotspot/bin'
$output = "$serverRoot/work/npc-oracle"
New-Item -ItemType Directory -Force -Path $output | Out-Null
& "$jdk/javac.exe" '-J-Xmx256m' '-J-XX:ActiveProcessorCount=2' '-cp' "$serverRoot/work/rsprot/*" '-d' $output "$serverRoot/reference/NpcWireOracle.java"
if ($LASTEXITCODE -ne 0) { throw 'RSProt wire oracle compilation failed.' }
& "$jdk/java.exe" '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' "$output;$serverRoot/work/rsprot/*" NpcWireOracle "$serverRoot/crates/protocol/tests/fixtures/npcs.expected"
if ($LASTEXITCODE -ne 0) { throw 'RSProt wire oracle execution failed.' }
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-protocol','--test','rsprot_npcs')

