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
$oracleDir = "$serverRoot/work/tutorial-bank-oracle"
New-Item -ItemType Directory -Force -Path $oracleDir | Out-Null
$classpath = Find-ReferenceJar 'org.jetbrains.kotlin' 'kotlin-stdlib' '2.2.0'
$guide = "$serverRoot/reference/kotlin/content/areas/tutorial-island/src/main/kotlin/org/rsmod/content/areas/tutorialisland/scripts/locs/TutorialBankScript.kt"
$steps = "$serverRoot/reference/kotlin/content/areas/tutorial-island/src/main/kotlin/org/rsmod/content/areas/tutorialisland/progression/TutorialSteps.kt"
$delegate = "$serverRoot/reference/kotlin/api/utils/utils-vars/src/main/kotlin/org/rsmod/api/utils/vars/VarEnumDelegate.kt"
$stubs = @(Get-ChildItem "$serverRoot/reference/tutorial-bank-stubs" -Filter '*.kt' | Sort-Object Name | ForEach-Object FullName)
$stubs += @("Inject.kt","Entity.kt","Output.kt","Config.kt","Refs.kt","Focus.kt","Vars.kt") | ForEach-Object {"$serverRoot/reference/tutorial-combat-death-stubs/$_"}
$stubs += "$serverRoot/reference/tutorial-guide-stubs/Plugins.kt"
$progression = "$serverRoot/reference/kotlin/content/areas/tutorial-island/src/main/kotlin/org/rsmod/content/areas/tutorialisland/progression/TutorialProgression.kt"
$journal = "$serverRoot/reference/kotlin/content/areas/tutorial-island/src/main/kotlin/org/rsmod/content/areas/tutorialisland/progression/LearningTheRopesJournal.kt"
$compileArgs = @('-no-stdlib','-no-reflect','-jvm-target','21','-classpath',$classpath,'-d',"$oracleDir/oracle.jar","$serverRoot/reference/TutorialBankOracle.kt",$guide,$steps,$delegate,$progression,$journal) + $stubs
$quoted = $compileArgs | ForEach-Object { '"' + $_.Replace('\','/') + '"' }
Set-Content "$oracleDir/compile.args" $quoted -Encoding utf8
& $java '-Xmx512m' '-XX:ActiveProcessorCount=2' '-cp' ($jars -join ';') org.jetbrains.kotlin.cli.jvm.K2JVMCompiler "@$oracleDir/compile.args"
if ($LASTEXITCODE -ne 0) { throw 'Tutorial bank oracle compilation failed.' }
$runArgs = @('-Xmx128m','-XX:ActiveProcessorCount=2','-cp',"$oracleDir/oracle.jar;$classpath",'TutorialBankOracleKt',"$serverRoot/crates/world/tests/fixtures/tutorial-bank.expected")
$quoted = $runArgs | ForEach-Object { '"' + $_.Replace('\','/') + '"' }
Set-Content "$oracleDir/run.args" $quoted -Encoding utf8
& $java "@$oracleDir/run.args"
if ($LASTEXITCODE -ne 0) { throw 'Tutorial bank oracle execution failed.' }
[ordered]@{
    method='Unchanged TutorialBankScript and TutorialProgression:2016 gates/bank/progression cases; explicit inventory and openBank doubles, not native bank UI acceptance'
    progressionSha256=(Get-FileHash $progression).Hash
    journalSha256=(Get-FileHash $journal).Hash
    sourceSha256=(Get-FileHash $guide).Hash
    delegateSha256=(Get-FileHash $delegate).Hash
    stepsSha256=(Get-FileHash $steps).Hash
    oracleSha256=(Get-FileHash "$serverRoot/reference/TutorialBankOracle.kt").Hash
    stubs=@($stubs | ForEach-Object { [ordered]@{name=(Split-Path $_ -Leaf);sha256=(Get-FileHash $_).Hash} })
    fixtureSha256=(Get-FileHash "$serverRoot/crates/world/tests/fixtures/tutorial-bank.expected").Hash
} | ConvertTo-Json -Depth 5 | Set-Content "$oracleDir/provenance.json"
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-world','--test','kotlin_tutorial_bank')












