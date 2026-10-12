$ErrorActionPreference = 'Stop'
$serverRoot = Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$output = "$serverRoot/work/js5-store-oracle"
New-Item -ItemType Directory -Force -Path $output,"$serverRoot/work/openrs" | Out-Null
# Private copies avoid the shared Gradle JAR ZipFS close/access failure.
Get-ChildItem 'C:/Users/backup/.gradle/caches/modules-2/files-2.1/org.openrs2' -Recurse -Filter '*.jar' | ForEach-Object {
    Copy-Item -LiteralPath $_.FullName -Destination (Join-Path "$serverRoot/work/openrs" $_.Name)
}
$crypto = @(Get-ChildItem 'C:/Users/backup/.gradle/caches/modules-2/files-2.1/org.bouncycastle/bcprov-jdk18on' -Recurse -Filter '*.jar')
if ($crypto.Count -ne 1) { throw 'Expected one cached BC provider jar.' }
Copy-Item -LiteralPath $crypto[0].FullName -Destination (Join-Path "$serverRoot/work/openrs" $crypto[0].Name)
$guava = @(Get-ChildItem 'C:/Users/backup/.gradle/caches/modules-2/files-2.1/com.google.guava/guava/33.4.8-jre' -Recurse -Filter '*.jar')
if ($guava.Count -ne 1) { throw 'Expected cached Guava 33.4.8.' }
Copy-Item -LiteralPath $guava[0].FullName -Destination (Join-Path "$serverRoot/work/openrs" $guava[0].Name)
$net = @(Get-ChildItem "$serverRoot/reference/kotlin/api/net/build/libs" -Filter '*.jar' | Where-Object Name -NotMatch 'sources' | ForEach-Object FullName)
$classpath = "$serverRoot/work/rsprot/*;$serverRoot/work/openrs/*;" + ($net -join ';')
$jdk = 'C:/Program Files/Eclipse Adoptium/jdk-21.0.12.101-hotspot/bin'
& "$jdk/javac.exe" '-J-Xmx256m' '-J-XX:ActiveProcessorCount=2' '-cp' $classpath '-d' $output "$serverRoot/reference/Js5StoreOracle.java"
if ($LASTEXITCODE -ne 0) { throw 'JS5 store oracle compilation failed.' }
& "$jdk/java.exe" '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' "$output;$classpath" Js5StoreOracle "$serverRoot/crates/cache/tests/fixtures/js5-store.expected"
if ($LASTEXITCODE -ne 0) { throw 'JS5 store oracle execution failed.' }
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-cache','--test','kotlin_js5_store')
