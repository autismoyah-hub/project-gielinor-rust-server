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
$oracleDir = "$serverRoot/work/prayer-offering-oracle"
New-Item -ItemType Directory -Force -Path $oracleDir | Out-Null
$references = @(Get-ChildItem "$serverRoot/reference/kotlin/engine","$serverRoot/reference/kotlin/api","$serverRoot/reference/kotlin/content/interfaces" -Recurse -Filter '*.jar' | Where-Object { $_.FullName -match '[\\/]build[\\/]libs[\\/]' -and $_.Name -notmatch 'sources' } | ForEach-Object FullName)
$references += @(Get-ChildItem "$serverRoot/work/rsprot" -Filter '*.jar' | ForEach-Object FullName)
$references += @(Get-ChildItem "$cacheRoot/org.rsmod" -Recurse -Filter '*.jar' | ForEach-Object FullName)
$classpath = (Get-Content "$serverRoot/work/integration/kotlin-runtime-classpath.txt" -Raw).Trim()
$compileArgs = @('-Xskip-prerelease-check','-no-stdlib','-no-reflect','-jvm-target','21','-classpath',$classpath,'-d',"$oracleDir/oracle.jar","$serverRoot/reference/PrayerOfferingOracle.kt")
$quoted = $compileArgs | ForEach-Object { '"' + $_.Replace('\','/') + '"' }
Set-Content "$oracleDir/compile.args" $quoted -Encoding utf8
& $java '-Xmx512m' '-XX:ActiveProcessorCount=2' '-cp' ($jars -join ';') org.jetbrains.kotlin.cli.jvm.K2JVMCompiler "@$oracleDir/compile.args"
if ($LASTEXITCODE -ne 0) { throw 'Prayer offering oracle compilation failed.' }
$runArgs = @('-Xmx128m','-XX:ActiveProcessorCount=2','-cp',"$oracleDir/oracle.jar;$classpath",'PrayerOfferingOracleKt',"$serverRoot/crates/world/tests/fixtures/prayer-offering.expected")
$quoted = $runArgs | ForEach-Object { '"' + $_.Replace('\','/') + '"' }
Set-Content "$oracleDir/run.args" $quoted -Encoding utf8
& $java "@$oracleDir/run.args"
if ($LASTEXITCODE -ne 0) { throw 'Prayer offering oracle execution failed.' }
[ordered]@{
    method='2048 cases of original Player.statAdvance composed with chapel XP multiplier and worn/account modifiers; repeated offerings, cap, zero rate and negative modifier. Protected coroutine and inventory are not invoked.'
    sourceSha256=(Get-FileHash "$serverRoot/reference/kotlin/content/skills/prayer/src/main/kotlin/org/rsmod/content/skills/prayer/scripts/Prayer.kt").Hash
    statSourceSha256=(Get-FileHash "$serverRoot/reference/kotlin/api/player/src/main/kotlin/org/rsmod/api/player/stat/PlayerStatExtensions.kt").Hash
    oracleSha256=(Get-FileHash "$serverRoot/reference/PrayerOfferingOracle.kt").Hash
    jars=@($classpath.Split(';') | Sort-Object -Unique | ForEach-Object { [ordered]@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash} })
    fixtureSha256=(Get-FileHash "$serverRoot/crates/world/tests/fixtures/prayer-offering.expected").Hash
} | ConvertTo-Json -Depth 5 | Set-Content "$oracleDir/provenance.json" -Encoding utf8
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-world','--test','kotlin_prayer_offering')











