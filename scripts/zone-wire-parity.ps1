$ErrorActionPreference = 'Stop'
$serverRoot = Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$output = "$serverRoot/work/zone-wire-oracle"
New-Item -ItemType Directory -Force -Path $output | Out-Null
$classpath = "$serverRoot/work/rsprot/*"
$jdk = 'C:/Program Files/Eclipse Adoptium/jdk-21.0.12.101-hotspot/bin'
& "$jdk/javac.exe" '-J-Xmx256m' '-J-XX:ActiveProcessorCount=2' '-cp' $classpath '-d' $output "$serverRoot/reference/ZoneWireOracle.java"
if ($LASTEXITCODE -ne 0) { throw 'Zone wire oracle compilation failed.' }
& "$jdk/java.exe" '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' "$output;$classpath" ZoneWireOracle "$serverRoot/crates/protocol/tests/fixtures/zone-wire.expected"
if ($LASTEXITCODE -ne 0) { throw 'Zone wire oracle execution failed.' }
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-protocol','--test','rsprot_zone_wire')
