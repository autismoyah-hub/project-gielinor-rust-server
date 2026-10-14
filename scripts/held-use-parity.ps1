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
$oracleDir = "$serverRoot/work/held-use-oracle"
New-Item -ItemType Directory -Force -Path $oracleDir | Out-Null
$classpath = Find-ReferenceJar 'org.jetbrains.kotlin' 'kotlin-stdlib' '2.2.0'
$source="$serverRoot/reference/kotlin/api/player/src/main/kotlin/org/rsmod/api/player/interact/HeldUInteractions.kt"
$events="$serverRoot/reference/kotlin/api/player/src/main/kotlin/org/rsmod/api/player/events/interact/HeldEvents.kt"
$stubs=@(Get-ChildItem "$serverRoot/reference/held-use-stubs" -Filter '*.kt' | Sort-Object Name | ForEach-Object FullName)
$stubs+="$serverRoot/reference/tutorial-guide-stubs/Inject.kt"
$compileArgs=@('-no-stdlib','-no-reflect','-jvm-target','21','-classpath',$classpath,'-d',"$oracleDir/oracle.jar","$serverRoot/reference/HeldUseOracle.kt",$source,$events)+$stubs
$quoted=$compileArgs | ForEach-Object {'"'+$_.Replace('\','/')+'"'}
Set-Content "$oracleDir/compile.args" $quoted -Encoding utf8
& $java '-Xmx512m' '-XX:ActiveProcessorCount=2' '-cp' ($jars -join ';') org.jetbrains.kotlin.cli.jvm.K2JVMCompiler "@$oracleDir/compile.args"
if ($LASTEXITCODE -ne 0) {throw 'Held use oracle compilation failed'}
& $java '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' "$oracleDir/oracle.jar;$classpath" HeldUseOracleKt "$serverRoot/crates/world/tests/fixtures/held-use.expected"
if ($LASTEXITCODE -ne 0) {throw 'Held use oracle execution failed'}
[ordered]@{
 method='Original HeldUInteractions and HeldEvents4608 directional selection/verification cases with explicit event bus, access and inventory doubles; not full UI/coroutine acceptance'
 sources=@(@($source,$events)+$stubs | ForEach-Object {[ordered]@{name=(Split-Path $_ -Leaf);sha256=(Get-FileHash $_).Hash}})
 oracleSha256=(Get-FileHash "$serverRoot/reference/HeldUseOracle.kt").Hash
 fixtureSha256=(Get-FileHash "$serverRoot/crates/world/tests/fixtures/held-use.expected").Hash
} | ConvertTo-Json -Depth 5 | Set-Content "$oracleDir/provenance.json"
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-world','--test','kotlin_held_use')
