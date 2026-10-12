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
$oracleDir = "$serverRoot/work/ground-oracle"
New-Item -ItemType Directory -Force -Path $oracleDir | Out-Null
$mapSources = @(Get-ChildItem "$serverRoot/reference/kotlin/engine/map/src/main" -Recurse -Filter '*.kt' | ForEach-Object FullName)
$gameRoot = "$serverRoot/reference/kotlin/engine/game/src/main/kotlin/org/rsmod/game"
$registryRoot = "$serverRoot/reference/kotlin/api/registry/src/main/kotlin/org/rsmod/api/registry/obj"
$extraSources = @(
    "$serverRoot/reference/kotlin/engine/annotations/src/main/kotlin/org/rsmod/annotations/InternalApi.kt"
    "$gameRoot/MapClock.kt"
    "$gameRoot/obj/Obj.kt"
    "$gameRoot/obj/ObjEntity.kt"
    "$gameRoot/obj/ObjScope.kt"
    "$registryRoot/ObjRegistry.kt"
    "$registryRoot/ObjRegistryResult.kt"
    "$registryRoot/ObjStackEntryList.kt"
    "$registryRoot/ZoneObjMap.kt"
    "$serverRoot/reference/kotlin/api/repo/src/main/kotlin/org/rsmod/api/repo/obj/ObjRepository.kt"
)
$stubs = @(Get-ChildItem "$serverRoot/reference/ground-oracle" -Filter '*.kt' | ForEach-Object FullName)
$fastutil = Find-ReferenceJar 'it.unimi.dsi' 'fastutil' '8.5.16'
$inject = Find-ReferenceJar 'jakarta.inject' 'jakarta.inject-api' '2.0.1'
$compileClasspath = $jars[1] + ';' + $jars[-1] + ';' + $fastutil + ';' + $inject
& $java '-Xmx512m' '-XX:ActiveProcessorCount=2' '-cp' ($jars -join ';') org.jetbrains.kotlin.cli.jvm.K2JVMCompiler '-no-stdlib' '-no-reflect' '-opt-in=kotlin.contracts.ExperimentalContracts' '-jvm-target' '21' '-classpath' $compileClasspath '-d' "$oracleDir/oracle.jar" @mapSources @extraSources @stubs
if ($LASTEXITCODE -ne 0) { throw 'Ground-object reference compilation failed.' }
& $java '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' ("$oracleDir/oracle.jar;" + $compileClasspath) GroundOracleKt "$serverRoot/crates/world/tests/fixtures/ground.expected"
if ($LASTEXITCODE -ne 0) { throw 'Ground-object reference execution failed.' }
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-world','--test','kotlin_ground')