$ErrorActionPreference = 'Stop'
$serverRoot = Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$jdk = 'C:/Program Files/Eclipse Adoptium/jdk-21.0.12.101-hotspot/bin'
$output = "$serverRoot/work/interface-oracle"
New-Item -ItemType Directory -Force -Path $output | Out-Null
& "$jdk/javac.exe" '-J-Xmx256m' '-J-XX:ActiveProcessorCount=2' '-cp' "$serverRoot/work/rsprot/*" '-d' $output "$serverRoot/reference/InterfaceOracle.java"
if ($LASTEXITCODE -ne 0) { throw 'Interface oracle compilation failed.' }
& "$jdk/java.exe" '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' "$output;$serverRoot/work/rsprot/*" InterfaceOracle "$serverRoot/crates/protocol/tests/fixtures/interfaces.expected"
if ($LASTEXITCODE -ne 0) { throw 'Interface oracle execution failed.' }
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-protocol','--test','rsprot_interfaces')
