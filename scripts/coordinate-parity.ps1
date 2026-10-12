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
foreach ($x in @(-1,0,1,63,64,3200,16383,16384)) {
    foreach ($z in @(-1,0,1,63,64,3200,16383,16384)) {
        foreach ($level in @(-1,0,1,3,4)) { $cases.Add("new $x $z $level") }
    }
}
foreach ($raw in @(-2147483648,-1,0,1,1073741823,2147483647)) {
    $cases.Add("raw $raw")
    foreach ($delta in @(-2147483648,-16384,-1,0,1,16384,2147483647)) {
        $cases.Add("move $raw $delta 0 0")
        $cases.Add("move $raw 0 $delta 0")
        $cases.Add("move $raw 0 0 $delta")
    }
}
$rng = [System.Random]::new(2391)
for ($i=0; $i -lt 1000; $i++) {
    $cases.Add(('new {0} {1} {2}' -f $rng.Next(16384),$rng.Next(16384),$rng.Next(4)))
    $raw = $rng.Next(-2147483648,2147483647)
    $cases.Add("raw $raw")
    $cases.Add(('move {0} {1} {2} {3}' -f $raw,$rng.Next(-32,33),$rng.Next(-32,33),$rng.Next(-1,2)))
    $cases.Add(('distance {0} {1}' -f $raw,$rng.Next(-2147483648,2147483647)))
}
$cases | Set-Content "$oracleDir/input.txt"
$expected = @($cases | & $java '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' ("$oracleDir/oracle.jar;" + $jars[1]) CoordOracleKt)
if ($LASTEXITCODE -ne 0) { throw 'Kotlin oracle execution failed.' }
$expected | Set-Content "$oracleDir/kotlin.txt"
# Kotlin process is finished before starting Rust.
& "$serverRoot/build.ps1" -Action build -CargoArgs @('-p', 'gielinor-world', '--example', 'coord_oracle')
$actual = @($cases | & "$serverRoot/work/target/debug/examples/coord_oracle.exe")
if ($LASTEXITCODE -ne 0) { throw 'Rust oracle execution failed.' }
$actual | Set-Content "$oracleDir/rust.txt"
if ($actual.Count -ne $expected.Count -or $actual.Count -ne $cases.Count) { throw 'Fixture output count mismatch.' }
for ($i=0; $i -lt $cases.Count; $i++) {
    if ($actual[$i] -cne $expected[$i]) { throw "Parity mismatch on case ${i}: $($cases[$i]); Kotlin=$($expected[$i]); Rust=$($actual[$i])" }
}
$summary = "PASS: $($cases.Count) Kotlin/Rust coordinate cases match. Source commit cb6e66cc0f16a25b60ba6705d8f157cc22ab4869."
$summary | Set-Content "$oracleDir/result.txt"
Write-Host $summary
