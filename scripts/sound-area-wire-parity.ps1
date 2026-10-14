$ErrorActionPreference = 'Stop'
$serverRoot = Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$output = "$serverRoot/work/sound-area-wire-oracle"
New-Item -ItemType Directory -Force -Path $output | Out-Null
$classpath = "$serverRoot/work/rsprot/*"
$jdk = 'C:/Program Files/Eclipse Adoptium/jdk-21.0.12.101-hotspot/bin'
& "$jdk/javac.exe" '-J-Xmx256m' '-J-XX:ActiveProcessorCount=2' '-cp' $classpath '-d' $output "$serverRoot/reference/SoundAreaWireOracle.java"
if ($LASTEXITCODE -ne 0) { throw 'Sound area wire oracle compilation failed.' }
& "$jdk/java.exe" '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' "$output;$classpath" SoundAreaWireOracle "$serverRoot/crates/protocol/tests/fixtures/sound-area-wire.expected"
if ($LASTEXITCODE -ne 0) { throw 'Sound area wire oracle execution failed.' }
[ordered]@{
    method='Pinned RSProt SOUND_AREA through actual zone enclosure builder:4096 cases including field narrowing.'
    oracleSha256=(Get-FileHash "$serverRoot/reference/SoundAreaWireOracle.java").Hash
    jars=@(Get-ChildItem "$serverRoot/work/rsprot" -Filter '*.jar' | Sort-Object Name | ForEach-Object { [ordered]@{name=$_.Name;sha256=(Get-FileHash -LiteralPath $_.FullName).Hash} })
    fixtureSha256=(Get-FileHash "$serverRoot/crates/protocol/tests/fixtures/sound-area-wire.expected").Hash
} | ConvertTo-Json -Depth 5 | Set-Content "$output/provenance.json" -Encoding utf8
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-protocol','--test','rsprot_sound_area_wire')
