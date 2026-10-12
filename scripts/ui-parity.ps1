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
$oracleDir = "$serverRoot/work/ui-oracle"
New-Item -ItemType Directory -Force -Path $oracleDir | Out-Null
$extraSources = @(
    "$serverRoot/reference/kotlin/engine/game/src/main/kotlin/org/rsmod/game/ui/collection/ComponentEventMap.kt"
) + @(Get-ChildItem "$serverRoot/reference/ui-oracle" -Filter '*.kt' | ForEach-Object FullName)
$inject = Find-ReferenceJar 'jakarta.inject' 'jakarta.inject-api' '2.0.1'
$compileClasspath = $jars[1] + ';' + $jars[-1] + ';' + $inject + ';' + ((Get-ChildItem -LiteralPath "$serverRoot/work/rsprot" -Filter '*.jar' | ForEach-Object FullName) -join ';')
& $java '-Xmx512m' '-XX:ActiveProcessorCount=2' '-cp' ($jars -join ';') org.jetbrains.kotlin.cli.jvm.K2JVMCompiler '-no-stdlib' '-no-reflect' '-jvm-target' '21' '-classpath' $compileClasspath '-d' "$oracleDir/oracle.jar" @extraSources
if ($LASTEXITCODE -ne 0) { throw 'UI range reference compilation failed.' }
& $java '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' ("$oracleDir/oracle.jar;" + $compileClasspath) UiOracleKt "$serverRoot/crates/world/tests/fixtures/ui.expected"
if ($LASTEXITCODE -ne 0) { throw 'UI range reference execution failed.' }
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-world','--test','kotlin_ui')

