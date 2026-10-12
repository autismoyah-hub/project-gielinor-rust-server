$ErrorActionPreference = 'Stop'
$serverRoot = Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$output = "$serverRoot/work/potions-oracle"
New-Item -ItemType Directory -Force -Path $output | Out-Null
$content = @(Get-ChildItem "$serverRoot/reference/kotlin/content/skills/herblore/build/libs" -Filter '*.jar' | Where-Object Name -NotMatch 'sources' | ForEach-Object FullName)
if ($content.Count -ne 1) { throw 'Expected one compiled isolated herblore reference JAR.' }
$classpath = "$serverRoot/work/rsprot/*;" + ($content -join ';')
$jdk = 'C:/Program Files/Eclipse Adoptium/jdk-21.0.12.101-hotspot/bin'
& "$jdk/javac.exe" '-J-Xmx256m' '-J-XX:ActiveProcessorCount=2' '-cp' $classpath '-d' $output "$serverRoot/reference/PotionOracle.java"
if ($LASTEXITCODE -ne 0) { throw 'Potion oracle compilation failed.' }
& "$jdk/java.exe" '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' "$output;$classpath" PotionOracle "$serverRoot/crates/world/tests/fixtures/potions.expected"
if ($LASTEXITCODE -ne 0) { throw 'Potion oracle execution failed.' }
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-world','--test','kotlin_potions')
