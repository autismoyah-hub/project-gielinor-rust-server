$ErrorActionPreference = 'Stop'
$serverRoot = Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$jdk = 'C:/Program Files/Eclipse Adoptium/jdk-21.0.12.101-hotspot/bin'
$output = "$serverRoot/work/hint-arrow-oracle"
New-Item -ItemType Directory -Force -Path $output | Out-Null
$classpath = (Get-Content "$serverRoot/work/integration/kotlin-runtime-classpath.txt" -Raw).Trim()
$compileArgs = @('-cp', $classpath, '-d', $output, "$serverRoot/reference/HintArrowOracle.java")
$quoted = $compileArgs | ForEach-Object { '"' + $_.Replace('\','/') + '"' }
Set-Content "$output/compile.args" $quoted -Encoding utf8
& "$jdk/javac.exe" '-J-Xmx256m' '-J-XX:ActiveProcessorCount=2' "@$output/compile.args"
if ($LASTEXITCODE -ne 0) { throw 'Hint arrow oracle compilation failed.' }
$runArgs = @('-Xmx128m','-XX:ActiveProcessorCount=2','-cp',"$output;$classpath",'HintArrowOracle',"$serverRoot/crates/protocol/tests/fixtures/hint-arrow.expected")
$quoted = $runArgs | ForEach-Object { '"' + $_.Replace('\','/') + '"' }
Set-Content "$output/run.args" $quoted -Encoding utf8
& "$jdk/java.exe" "@$output/run.args"
if ($LASTEXITCODE -ne 0) { throw 'Hint arrow oracle execution failed.' }
[ordered]@{method='Original HintArrowEncoder1152 cases with zero-initialized padding';oracleSha256=(Get-FileHash "$serverRoot/reference/HintArrowOracle.java").Hash;runtimeManifestSha256=(Get-FileHash "$serverRoot/integration/resource-manifest.json").Hash;fixtureSha256=(Get-FileHash "$serverRoot/crates/protocol/tests/fixtures/hint-arrow.expected").Hash} | ConvertTo-Json | Set-Content "$output/provenance.json"
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-protocol','--test','rsprot_hint_arrow')


