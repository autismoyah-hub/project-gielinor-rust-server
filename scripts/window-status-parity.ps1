$ErrorActionPreference = 'Stop'
$serverRoot = Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$jdk = 'C:/Program Files/Eclipse Adoptium/jdk-21.0.12.101-hotspot/bin'
$output = "$serverRoot/work/window-status-oracle"
New-Item -ItemType Directory -Force -Path $output | Out-Null
& "$jdk/javac.exe" '-J-Xmx256m' '-J-XX:ActiveProcessorCount=2' '-cp' "$serverRoot/work/rsprot/*" '-d' $output "$serverRoot/reference/WindowStatusOracle.java"
if ($LASTEXITCODE -ne 0) { throw 'Window status oracle compilation failed.' }
& "$jdk/java.exe" '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' "$output;$serverRoot/work/rsprot/*" WindowStatusOracle "$serverRoot/crates/protocol/tests/fixtures/window-status.expected"
if ($LASTEXITCODE -ne 0) { throw 'Window status oracle execution failed.' }
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-protocol','--test','rsprot_window_status')

