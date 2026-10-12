$ErrorActionPreference = 'Stop'
$serverRoot = Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$jars = @((Get-ChildItem "$serverRoot/reference/kotlin/api/cache/build/libs" -Filter '*.jar' | Where-Object Name -NotMatch 'sources' | ForEach-Object FullName))
$jars += @(Get-ChildItem "$serverRoot/reference/kotlin/engine" -Recurse -Filter '*.jar' | Where-Object { $_.FullName -match '[\\/]build[\\/]libs[\\/]' -and $_.Name -notmatch 'sources' } | ForEach-Object FullName)
$jars += @(Get-ChildItem 'C:/Users/backup/.gradle/caches/modules-2/files-2.1/org.openrs2' -Recurse -Filter '*.jar' | ForEach-Object FullName)
$classpath = "$serverRoot/work/rsprot/*;" + ($jars -join ';')
$jdk = 'C:/Program Files/Eclipse Adoptium/jdk-21.0.12.101-hotspot/bin'
$output = "$serverRoot/work/item-identity-oracle"
New-Item -ItemType Directory -Force -Path $output | Out-Null
& "$jdk/javac.exe" '-J-Xmx256m' '-J-XX:ActiveProcessorCount=2' '-cp' $classpath '-d' $output "$serverRoot/reference/ItemIdentityOracle.java"
if ($LASTEXITCODE -ne 0) { throw 'Item identity oracle compilation failed.' }
& "$jdk/java.exe" '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' "$output;$classpath" ItemIdentityOracle "$serverRoot/crates/cache/tests/fixtures/item-identity.expected"
if ($LASTEXITCODE -ne 0) { throw 'Item identity oracle execution failed.' }
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-cache','--test','kotlin_item_identity')

