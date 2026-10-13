$ErrorActionPreference='Stop'
$serverRoot=Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$jdk='C:/Program Files/Eclipse Adoptium/jdk-21.0.12.101-hotspot/bin'
$oracle="$serverRoot/work/reconnect-oracle"
$fixture="$serverRoot/reference/protocol/reconnect-fixture"
New-Item -ItemType Directory -Force -Path $oracle | Out-Null
$classpath=(Get-Content "$serverRoot/work/integration/kotlin-runtime-classpath.txt" -Raw).Trim()
$compile=@('-cp',$classpath,'-d',$oracle,"$serverRoot/reference/LoginOracle.java")
$compile | ForEach-Object {'"'+$_.Replace('\','/')+'"'} | Set-Content "$oracle/compile.args"
& "$jdk/javac.exe" '-J-Xmx256m' '-J-XX:ActiveProcessorCount=2' "@$oracle/compile.args"
if ($LASTEXITCODE -ne 0) {throw 'Reconnect oracle compilation failed'}
$run=@('-Xmx128m','-XX:ActiveProcessorCount=2','-cp',"$oracle;$classpath",'LoginOracle',$fixture,'reconnect')
$run | ForEach-Object {'"'+$_.Replace('\','/')+'"'} | Set-Content "$oracle/run.args"
& "$jdk/java.exe" "@$oracle/run.args"
if ($LASTEXITCODE -ne 0) {throw 'Reconnect oracle execution failed'}
$hashes=@{}
Get-ChildItem -LiteralPath $fixture -File | ForEach-Object {$hashes[$_.Name]=(Get-FileHash -LiteralPath $_.FullName).Hash}
[ordered]@{method='Original RSProt GameReconnectDecoder: RSA, XTEA, header, previous session key, body and23 CRCs';oracleSha256=(Get-FileHash "$serverRoot/reference/LoginOracle.java").Hash;runtimeManifestSha256=(Get-FileHash "$serverRoot/integration/resource-manifest.json").Hash;fixtures=$hashes} | ConvertTo-Json -Depth 4 | Set-Content "$oracle/provenance.json"
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-protocol','--test','reconnect_reference','--test','session_reference','--test','login_reference')
