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
$oracleDir = "$serverRoot/work/tutorial-rules-oracle"
New-Item -ItemType Directory -Force -Path $oracleDir | Out-Null
$classpath = Find-ReferenceJar 'org.jetbrains.kotlin' 'kotlin-stdlib' '2.2.0'
$rules="$serverRoot/reference/kotlin/content/areas/tutorial-island/src/main/kotlin/org/rsmod/content/areas/tutorialisland/scripts/rules/TutorialIslandRulesScript.kt"
$table="$serverRoot/reference/kotlin/engine/game/src/main/kotlin/org/rsmod/game/stat/PlayerSkillXPTable.kt"
$stubs=@(Get-ChildItem "$serverRoot/reference/tutorial-rules-stubs" -Filter '*.kt' | Sort-Object Name | ForEach-Object FullName)
$stubs+=@('Inject.kt','Plugins.kt') | ForEach-Object {"$serverRoot/reference/tutorial-guide-stubs/$_"}
$compileArgs=@('-no-stdlib','-no-reflect','-jvm-target','21','-classpath',$classpath,'-d',"$oracleDir/oracle.jar","$serverRoot/reference/TutorialRulesOracle.kt",$rules,$table)+$stubs
$quoted=$compileArgs | ForEach-Object {'"'+$_.Replace('\','/')+'"'}
Set-Content "$oracleDir/compile.args" $quoted -Encoding utf8
& $java '-Xmx512m' '-XX:ActiveProcessorCount=2' '-cp' ($jars -join ';') org.jetbrains.kotlin.cli.jvm.K2JVMCompiler "@$oracleDir/compile.args"
if ($LASTEXITCODE -ne 0) {throw 'Tutorial rules oracle compilation failed'}
& $java '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' "$oracleDir/oracle.jar;$classpath" TutorialRulesOracleKt "$serverRoot/crates/world/tests/fixtures/tutorial-rules.expected"
if ($LASTEXITCODE -ne 0) {throw 'Tutorial rules oracle execution failed'}
[ordered]@{
 method='Original TutorialIslandRulesScript and PlayerSkillXPTable11776 handler cases; explicit stat storage, access and completion doubles; not full engine scheduling'
 sources=@(@($rules,$table)+$stubs | ForEach-Object {[ordered]@{name=(Split-Path $_ -Leaf);sha256=(Get-FileHash $_).Hash}})
 oracleSha256=(Get-FileHash "$serverRoot/reference/TutorialRulesOracle.kt").Hash
 fixtureSha256=(Get-FileHash "$serverRoot/crates/world/tests/fixtures/tutorial-rules.expected").Hash
} | ConvertTo-Json -Depth 5 | Set-Content "$oracleDir/provenance.json"
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-world','--test','kotlin_tutorial_rules')
