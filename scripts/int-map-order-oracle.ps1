$ErrorActionPreference='Stop'
$serverRoot=Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$root="$serverRoot/work/int-map-order-oracle"
New-Item -ItemType Directory -Force -Path "$root/classes" | Out-Null
$classpath=(Get-Content "$serverRoot/work/integration/kotlin-runtime-classpath.txt" -Raw).Trim()
$jdk='C:/Program Files/Eclipse Adoptium/jdk-21.0.12.101-hotspot/bin'
$arguments=@('-cp',$classpath,'-d',"$root/classes","$serverRoot/reference/IntMapOrderOracle.java")
$arguments | ForEach-Object {'"'+$_.Replace('\','/')+'"'} | Set-Content "$root/javac.args"
& "$jdk/javac.exe" '-J-Xmx128m' '-J-XX:ActiveProcessorCount=2' "@$root/javac.args"
if ($LASTEXITCODE -ne 0) {throw 'Int map reference oracle compilation failed'}
& "$jdk/java.exe" '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' "$root/classes;$classpath" IntMapOrderOracle "$root/int-map-order.expected" "$root/int-map-copy.expected"
if ($LASTEXITCODE -ne 0) {throw 'Int map reference oracle execution failed'}
Get-Content "$root/int-map-order.expected"
Copy-Item "$root/int-map-order.expected" "$serverRoot/crates/cache/tests/fixtures/int-map-order.expected"
Copy-Item "$root/int-map-copy.expected" "$serverRoot/crates/cache/tests/fixtures/int-map-copy.expected"
[ordered]@{method='Original fastutil Int2ObjectOpenHashMap values iterator: default constructor and LinkedHashMap copy';oracleSha256=(Get-FileHash "$serverRoot/reference/IntMapOrderOracle.java").Hash;runtimeManifestSha256=(Get-FileHash "$serverRoot/integration/resource-manifest.json").Hash;fixtureSha256=(Get-FileHash "$root/int-map-order.expected").Hash;copyFixtureSha256=(Get-FileHash "$root/int-map-copy.expected").Hash} | ConvertTo-Json | Set-Content "$root/provenance.json"

