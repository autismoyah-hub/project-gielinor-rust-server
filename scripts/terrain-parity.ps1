$ErrorActionPreference = 'Stop'
$serverRoot = Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$cacheRoot = 'C:\Users\backup\.gradle\caches\modules-2\files-2.1'
function Find-ReferenceJar([string]$group, [string]$artifact, [string]$version) {
    $jar = @(Get-ChildItem -LiteralPath "$cacheRoot/$group/$artifact/$version" -Recurse -Filter "$artifact-$version.jar")
    if ($jar.Count -ne 1) { throw "Expected one cached $artifact $version jar." }
    return $jar[0].FullName
}
$jars = @(
    Find-ReferenceJar 'org.jetbrains.kotlin' 'kotlin-compiler-embeddable' '2.2.0'
    Find-ReferenceJar 'org.jetbrains.kotlin' 'kotlin-stdlib' '2.2.0'
    Find-ReferenceJar 'org.jetbrains.kotlin' 'kotlin-script-runtime' '2.2.0'
    Find-ReferenceJar 'org.jetbrains.kotlin' 'kotlin-reflect' '1.6.10'
    Find-ReferenceJar 'org.jetbrains.kotlin' 'kotlin-daemon-embeddable' '2.2.0'
    Find-ReferenceJar 'org.jetbrains.kotlinx' 'kotlinx-coroutines-core-jvm' '1.8.0'
    Find-ReferenceJar 'org.jetbrains' 'annotations' '13.0'
)
$java = 'C:\Program Files\Eclipse Adoptium\jdk-21.0.12.101-hotspot\bin\java.exe'
$oracleDir = "$serverRoot/work/terrain-oracle"
New-Item -ItemType Directory -Force -Path $oracleDir,"$serverRoot/crates/cache/tests/fixtures" | Out-Null
$mapSources = @(Get-ChildItem "$serverRoot/reference/kotlin/engine/map/src/main" -Recurse -Filter '*.kt' | ForEach-Object FullName)
$sourceRoot = "$serverRoot/reference/kotlin/api/cache/src/main/kotlin/org/rsmod/api/cache"
$terrainSources = @('util/InlineByteBuf.kt','map/tile/MapTileDecoder.kt','map/tile/MapTileDefinition.kt','map/tile/TileOverlay.kt','map/tile/TileUnderlay.kt') | ForEach-Object { "$sourceRoot/$_" }
$compileClasspath = ($jars[1] + ';' + $jars[-1] + ';' + "$serverRoot/work/rsprot/netty-buffer-4.2.2.Final.jar;$serverRoot/work/rsprot/netty-common-4.2.2.Final.jar")
& $java '-Xmx512m' '-XX:ActiveProcessorCount=2' '-cp' ($jars -join ';') org.jetbrains.kotlin.cli.jvm.K2JVMCompiler '-no-stdlib' '-no-reflect' '-jvm-target' '21' '-classpath' $compileClasspath '-d' "$oracleDir/oracle.jar" @mapSources @terrainSources "$serverRoot/reference/TerrainOracle.kt"
if ($LASTEXITCODE -ne 0) { throw 'Terrain reference compilation failed.' }
& $java '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' ("$oracleDir/oracle.jar;" + $compileClasspath) TerrainOracleKt "$serverRoot/crates/cache/tests/fixtures/terrain.expected"
if ($LASTEXITCODE -ne 0) { throw 'Terrain reference execution failed.' }
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-cache','--test','kotlin_terrain')
