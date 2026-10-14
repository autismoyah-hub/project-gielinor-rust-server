$ErrorActionPreference = 'Stop'
$serverRoot = Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$jdk = 'C:/Program Files/Eclipse Adoptium/jdk-21.0.12.101-hotspot/bin'
$output = "$serverRoot/work/location-target-oracle"
New-Item -ItemType Directory -Force -Path $output | Out-Null
& "$jdk/javac.exe" '-J-Xmx256m' '-J-XX:ActiveProcessorCount=2' '-cp' "$serverRoot/work/rsprot/*" '-d' $output "$serverRoot/reference/LocationTargetOracle.java"
if ($LASTEXITCODE -ne 0) { throw 'Location target oracle compilation failed.' }
& "$jdk/java.exe" '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' "$output;$serverRoot/work/rsprot/*" LocationTargetOracle "$serverRoot/crates/protocol/tests/fixtures/location-target.expected"
if ($LASTEXITCODE -ne 0) { throw 'Location target oracle execution failed.' }
[ordered]@{
    method='Original RSProt OpLocT decoder: 2048 transformed-wire cases'
    oracleSha256=(Get-FileHash "$serverRoot/reference/LocationTargetOracle.java").Hash
    jars=@(Get-ChildItem -LiteralPath "$serverRoot/work/rsprot" -Filter '*.jar' | Sort-Object Name | ForEach-Object {
        [ordered]@{name=$_.Name;sha256=(Get-FileHash $_.FullName).Hash}
    })
    fixtureSha256=(Get-FileHash "$serverRoot/crates/protocol/tests/fixtures/location-target.expected").Hash
} | ConvertTo-Json -Depth 5 | Set-Content "$output/provenance.json" -Encoding utf8
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-protocol','--test','rsprot_location_target')


