$ErrorActionPreference = 'Stop'
$serverRoot = Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$output = "$serverRoot/work/door-effects-oracle"
New-Item -ItemType Directory -Force -Path $output | Out-Null
$classpath = "$serverRoot/work/rsprot/*"
$jdk = 'C:/Program Files/Eclipse Adoptium/jdk-21.0.12.101-hotspot/bin'
& "$jdk/javac.exe" '-J-Xmx256m' '-J-XX:ActiveProcessorCount=2' '-cp' $classpath '-d' $output "$serverRoot/reference/DoorEffectsOracle.java"
if ($LASTEXITCODE -ne 0) { throw 'Location input oracle compilation failed.' }
& "$jdk/java.exe" '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' "$output;$classpath" DoorEffectsOracle "$serverRoot/crates/protocol/tests/fixtures/door-effects.expected"
if ($LASTEXITCODE -ne 0) { throw 'Location input oracle execution failed.' }
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-protocol','--test','rsprot_door_effects')


