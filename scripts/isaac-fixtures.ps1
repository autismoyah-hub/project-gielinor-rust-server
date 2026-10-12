$ErrorActionPreference = 'Stop'
$serverRoot = Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$cacheRoot = 'C:\Users\backup\.gradle\caches\modules-2\files-2.1'
function Find-IsaacJar([string]$group, [string]$artifact, [string]$version) {
    $jars = @(Get-ChildItem "$cacheRoot/$group/$artifact/$version" -Recurse -Filter "$artifact-$version.jar")
    if ($jars.Count -ne 1) { throw "Missing or ambiguous $artifact $version." }
    $jars[0].FullName
}
$jars = @(
    Find-IsaacJar 'org.jetbrains.kotlin' 'kotlin-compiler-embeddable' '2.2.0'
    Find-IsaacJar 'org.jetbrains.kotlin' 'kotlin-stdlib' '2.2.0'
    Find-IsaacJar 'org.jetbrains.kotlin' 'kotlin-script-runtime' '2.2.0'
    Find-IsaacJar 'org.jetbrains.kotlin' 'kotlin-reflect' '1.6.10'
    Find-IsaacJar 'org.jetbrains.kotlin' 'kotlin-daemon-embeddable' '2.2.0'
    Find-IsaacJar 'org.jetbrains.kotlinx' 'kotlinx-coroutines-core-jvm' '1.8.0'
    Find-IsaacJar 'org.jetbrains' 'annotations' '13.0'
    Find-IsaacJar 'net.rsprot' 'crypto' '1.0.0-ALPHA-20260630'
)
$oracleDir = "$serverRoot/work/isaac-oracle"
New-Item -ItemType Directory -Force -Path $oracleDir | Out-Null
$java = 'C:\Program Files\Eclipse Adoptium\jdk-21.0.12.101-hotspot\bin\java.exe'
& $java '-Xmx768m' '-XX:ActiveProcessorCount=2' '-cp' ($jars -join ';') org.jetbrains.kotlin.cli.jvm.K2JVMCompiler '-no-stdlib' '-no-reflect' '-jvm-target' '21' '-classpath' ($jars -join ';') '-d' "$oracleDir/oracle.jar" "$serverRoot/reference/IsaacOracle.kt"
if ($LASTEXITCODE -ne 0) { throw 'ISAAC oracle compilation failed.' }
$results = @(& $java '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' ("$oracleDir/oracle.jar;" + ($jars -join ';')) IsaacOracleKt)
if ($LASTEXITCODE -ne 0 -or $results.Count -ne 5125) { throw 'ISAAC reference execution failed.' }
$results | Set-Content "$oracleDir/rsprot.expected"
Get-FileHash $jars[-1], "$serverRoot/reference/IsaacOracle.kt", "$oracleDir/rsprot.expected" | Select-Object Path,Hash | ConvertTo-Json | Set-Content "$oracleDir/provenance.json"
Write-Host 'Generated 5,125 outputs from the pinned RSProt ISAAC implementation; Rust comparison is a separate test.'
