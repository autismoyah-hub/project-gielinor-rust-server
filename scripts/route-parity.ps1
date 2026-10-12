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
$oracleDir = "$serverRoot/work/coord-oracle"
New-Item -ItemType Directory -Force -Path $oracleDir | Out-Null
$sources = @(Get-ChildItem "$serverRoot/reference/kotlin/engine/map/src/main" -Recurse -Filter '*.kt' | ForEach-Object { $_.FullName })
& $java '-Xmx768m' '-XX:ActiveProcessorCount=2' '-cp' ($jars -join ';') org.jetbrains.kotlin.cli.jvm.K2JVMCompiler '-no-stdlib' '-no-reflect' '-jvm-target' '21' '-classpath' ($jars[1] + ';' + $jars[-1]) '-d' "$oracleDir/oracle.jar" @sources "$serverRoot/reference/CoordOracle.kt"
if ($LASTEXITCODE -ne 0) { throw 'Kotlin reference compilation failed.' }
$cases = [System.Collections.Generic.List[string]]::new()
$oracleDir = "$serverRoot/work/route-oracle"
New-Item -ItemType Directory -Force -Path $oracleDir | Out-Null
$sources = @(Get-ChildItem "$serverRoot/reference/kotlin/engine/routefinder/src/main" -Recurse -Filter '*.kt' | ForEach-Object { $_.FullName })
& $java '-Xmx512m' '-XX:ActiveProcessorCount=2' '-cp' ($jars -join ';') org.jetbrains.kotlin.cli.jvm.K2JVMCompiler '-no-stdlib' '-no-reflect' '-opt-in=kotlin.contracts.ExperimentalContracts' '-jvm-target' '21' '-classpath' ($jars[1] + ';' + $jars[-1]) '-d' "$oracleDir/oracle.jar" @sources "$serverRoot/reference/RouteOracle.kt"
if ($LASTEXITCODE -ne 0) { throw 'Route reference compilation failed.' }
& $java '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' ("$oracleDir/oracle.jar;" + $jars[1]) RouteOracleKt "$serverRoot/crates/world/tests/fixtures/routes.expected"
if ($LASTEXITCODE -ne 0) { throw 'Route reference execution failed.' }
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-world','--test','kotlin_routes')

