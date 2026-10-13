$ErrorActionPreference='Stop'
$serverRoot=Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$root="$serverRoot/work/disconnect-oracle"
New-Item -ItemType Directory -Force -Path "$root/classes" | Out-Null
$classpath=(Get-Content "$serverRoot/work/integration/kotlin-runtime-classpath.txt" -Raw).Trim()
$jdk='C:/Program Files/Eclipse Adoptium/jdk-21.0.12.101-hotspot/bin'
$arguments=@('-cp',$classpath,'-d',"$root/classes","$serverRoot/reference/DisconnectOracle.java")
$arguments | ForEach-Object {'"'+$_.Replace('\','/')+'"'} | Set-Content "$root/javac.args"
& "$jdk/javac.exe" '-J-Xmx128m' '-J-XX:ActiveProcessorCount=2' "@$root/javac.args"
if ($LASTEXITCODE -ne 0) {throw 'Disconnect reference oracle compilation failed'}
& "$jdk/java.exe" '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' "$root/classes;$classpath" DisconnectOracle "$root/disconnect.expected"
if ($LASTEXITCODE -ne 0) {throw 'Disconnect reference oracle execution failed'}
Get-Content "$root/disconnect.expected"
Copy-Item "$root/disconnect.expected" "$serverRoot/crates/world/tests/fixtures/disconnect.expected"
[ordered]@{method='Original Kotlin PlayerLogoutProcess.processClientDisconnect invoked by reflection';oracleSha256=(Get-FileHash "$serverRoot/reference/DisconnectOracle.java").Hash;runtimeManifestSha256=(Get-FileHash "$serverRoot/integration/resource-manifest.json").Hash;fixtureSha256=(Get-FileHash "$root/disconnect.expected").Hash} | ConvertTo-Json | Set-Content "$root/provenance.json"
