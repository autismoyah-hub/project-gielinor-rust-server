$ErrorActionPreference = 'Stop'
$serverRoot = Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$jdk = 'C:/Program Files/Eclipse Adoptium/jdk-21.0.12.101-hotspot/bin'
$output = "$serverRoot/work/outgoing-category-oracle"
New-Item -ItemType Directory -Force -Path $output | Out-Null
$jars = @(Get-ChildItem "$serverRoot/work/rsprot/*.jar" | Where-Object { $_.Name -ne 'compression-1.0.0-ALPHA-20260624.jar' } | Sort-Object Name)
$classpath = ($jars.FullName -join ';')
& "$jdk/javac.exe" '-J-Xmx256m' '-J-XX:ActiveProcessorCount=2' '-cp' $classpath '-d' $output "$serverRoot/reference/OutgoingCategoryOracle.java"
if ($LASTEXITCODE -ne 0) { throw 'Outgoing category oracle compilation failed.' }
& "$jdk/java.exe" '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' "$output;$classpath" OutgoingCategoryOracle "$serverRoot/work/rsprot/osrs-239-desktop-1.0.0-ALPHA-20260630.jar" "$output/outgoing-categories.expected" "$serverRoot/work/rsprot/osrs-239-model-1.0.0-ALPHA-20260630.jar"
if ($LASTEXITCODE -ne 0) { throw 'Outgoing category oracle execution failed.' }
@{ source_sha256 = (Get-FileHash "$serverRoot/reference/OutgoingCategoryOracle.java").Hash; jars = @($jars | ForEach-Object { @{name=$_.Name; sha256=(Get-FileHash $_.FullName).Hash} }); fixture_sha256=(Get-FileHash "$output/outgoing-categories.expected").Hash } | ConvertTo-Json -Depth 6 | Set-Content "$output/provenance.json" -Encoding utf8
