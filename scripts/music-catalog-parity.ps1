param([Parameter(Mandatory)][ValidatePattern('^[a-z0-9-]+$')][string]$Run)
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
$oracleDir = "$serverRoot/work/music-catalog-oracle/$Run"
if (Test-Path $oracleDir) {throw 'Catalog run already exists'}
New-Item -ItemType Directory -Force -Path $oracleDir | Out-Null
& "$serverRoot/build.ps1" -Action run -CargoArgs @('--package','gielinor-cache','--example','music_catalog_parity','--',"$serverRoot/work/integration/a-kotlin/.data/cache/game","$serverRoot/work/integration/a-kotlin/.data/symbols","$oracleDir/raw.txt","$oracleDir/rust.txt")
Assert-ServerBuildCapacity
$classpath = (Get-Content "$serverRoot/work/integration/kotlin-runtime-classpath.txt" -Raw).Trim()
$compileArgs = @('-Xskip-prerelease-check','-no-stdlib','-no-reflect','-jvm-target','21','-classpath',$classpath,'-d',"$oracleDir/oracle.jar","$serverRoot/reference/MusicCatalogOracle.kt")
$quoted = $compileArgs | ForEach-Object { '"' + $_.Replace('\','/') + '"' }
Set-Content "$oracleDir/compile.args" $quoted -Encoding utf8
& $java '-Xmx512m' '-XX:ActiveProcessorCount=2' '-cp' ($jars -join ';') org.jetbrains.kotlin.cli.jvm.K2JVMCompiler "@$oracleDir/compile.args"
if ($LASTEXITCODE -ne 0) { throw 'Music catalog oracle compilation failed.' }
$runArgs = @('-Xmx128m','-XX:ActiveProcessorCount=2','-cp',"$oracleDir/oracle.jar;$classpath",'MusicCatalogOracleKt',"$oracleDir/raw.txt","$serverRoot/work/integration/a-kotlin/.data/symbols","$oracleDir/kotlin.txt")
$quoted = $runArgs | ForEach-Object { '"' + $_.Replace('\','/') + '"' }
Set-Content "$oracleDir/run.args" $quoted -Encoding utf8
& $java "@$oracleDir/run.args"
if ($LASTEXITCODE -ne 0) { throw 'Music catalog oracle execution failed.' }
[ordered]@{method='Original MusicRepository from raw pinned cache groups; full catalog metadata and ordering';oracleSha256=(Get-FileHash "$serverRoot/reference/MusicCatalogOracle.kt").Hash;runtimeManifestSha256=(Get-FileHash "$serverRoot/integration/resource-manifest.json").Hash;rawSha256=(Get-FileHash "$oracleDir/raw.txt").Hash;rustSha256=(Get-FileHash "$oracleDir/rust.txt").Hash;kotlinSha256=(Get-FileHash "$oracleDir/kotlin.txt").Hash} | ConvertTo-Json | Set-Content "$oracleDir/provenance.json"
if ((Get-FileHash "$oracleDir/rust.txt").Hash -ne (Get-FileHash "$oracleDir/kotlin.txt").Hash) {throw "Music catalogs differ; inspect $oracleDir"}
Write-Host "Music catalog differential PASS: $((Get-Content $oracleDir/kotlin.txt).Count) records"
