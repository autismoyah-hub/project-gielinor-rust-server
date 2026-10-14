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
$oracleDir = "$serverRoot/work/smithing-make-oracle"
$fixture = "$serverRoot/crates/world/tests/fixtures/smithing-make.expected"
if (Test-Path -LiteralPath $fixture) { throw 'Smithing fixture already exists; preserve prior evidence.' }
New-Item -ItemType Directory -Force -Path $oracleDir | Out-Null
$classpath = Find-ReferenceJar 'org.jetbrains.kotlin' 'kotlin-stdlib' '2.2.0'
$source = "$serverRoot/reference/kotlin/content/skills/smithing/src/main/kotlin/org/rsmod/content/skills/smithing/scripts/MakeLoop.kt"
$oracle = "$serverRoot/reference/SmithingMakeOracle.kt"
$stubs = @(Get-ChildItem "$serverRoot/reference/smithing-make-stubs" -Filter '*.kt' | Sort-Object Name | ForEach-Object FullName)
$compileArgs = @('-no-stdlib','-no-reflect','-jvm-target','21','-classpath',$classpath,'-d',"$oracleDir/oracle.jar",$oracle,$source) + $stubs
$quoted = $compileArgs | ForEach-Object { '"' + $_.Replace('\','/') + '"' }
Set-Content "$oracleDir/compile.args" $quoted -Encoding utf8
& $java '-Xmx512m' '-XX:ActiveProcessorCount=2' '-cp' ($jars -join ';') org.jetbrains.kotlin.cli.jvm.K2JVMCompiler "@$oracleDir/compile.args"
if ($LASTEXITCODE -ne 0) { throw 'Smithing MakeLoop oracle compilation failed.' }
$runArgs = @('-Xmx128m','-XX:ActiveProcessorCount=2','-cp',"$oracleDir/oracle.jar;$classpath",'SmithingMakeOracleKt',$fixture)
$quoted = $runArgs | ForEach-Object { '"' + $_.Replace('\','/') + '"' }
Set-Content "$oracleDir/run.args" $quoted -Encoding utf8
& $java "@$oracleDir/run.args"
if ($LASTEXITCODE -ne 0) { throw 'Smithing MakeLoop oracle execution failed.' }
[ordered]@{
    method='Unchanged MakeLoop affordableCount/hasInputs:12500 cases; explicit invTotal double, not Inventory.count/transactions/SmithingActions/queue validation'
    sourceSha256=(Get-FileHash $source).Hash
    oracleSha256=(Get-FileHash $oracle).Hash
    stubs=@($stubs | ForEach-Object { [ordered]@{name=(Split-Path $_ -Leaf);sha256=(Get-FileHash $_).Hash} })
    fixtureSha256=(Get-FileHash $fixture).Hash
} | ConvertTo-Json -Depth 5 | Set-Content "$oracleDir/provenance.json"
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-world','--test','kotlin_smithing_make')
