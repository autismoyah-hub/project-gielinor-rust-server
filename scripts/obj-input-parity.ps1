$ErrorActionPreference = 'Stop'
$serverRoot = Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$output = "$serverRoot/work/obj-input-oracle"
New-Item -ItemType Directory -Force -Path $output | Out-Null
$classpath = "$serverRoot/work/rsprot/*"
$jdk = 'C:/Program Files/Eclipse Adoptium/jdk-21.0.12.101-hotspot/bin'
& "$jdk/javac.exe" '-J-Xmx256m' '-J-XX:ActiveProcessorCount=2' '-cp' $classpath '-d' $output "$serverRoot/reference/ObjInputOracle.java"
if ($LASTEXITCODE -ne 0) { throw 'Location input oracle compilation failed.' }
& "$jdk/java.exe" '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' "$output;$classpath" ObjInputOracle "$serverRoot/crates/protocol/tests/fixtures/obj-input.expected"
if ($LASTEXITCODE -ne 0) { throw 'Location input oracle execution failed.' }
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-protocol','--test','rsprot_obj_input')


