$ErrorActionPreference = 'Stop'
$serverRoot = Split-Path $PSScriptRoot -Parent
. "$PSScriptRoot/resources.ps1"
Assert-ServerBuildCapacity
$output = "$serverRoot/work/js5-queues-oracle"
New-Item -ItemType Directory -Force -Path $output | Out-Null
$transport = @(Get-ChildItem 'C:/Users/backup/.gradle/caches/modules-2/files-2.1/io.netty/netty-transport/4.2.2.Final' -Recurse -Filter '*.jar')
if ($transport.Count -ne 1) { throw 'Expected cached Netty transport 4.2.2.' }
Copy-Item -LiteralPath $transport[0].FullName -Destination (Join-Path "$serverRoot/work/rsprot" $transport[0].Name)
foreach ($dependency in @('net.rsprot/haproxy/1.0.0-ALPHA-20260630','io.netty/netty-codec-haproxy/4.2.2.Final')) {
    $jars = @(Get-ChildItem "C:/Users/backup/.gradle/caches/modules-2/files-2.1/$dependency" -Recurse -Filter '*.jar')
    if ($jars.Count -ne 1) { throw "Expected cached $dependency jar." }
    Copy-Item -LiteralPath $jars[0].FullName -Destination (Join-Path "$serverRoot/work/rsprot" $jars[0].Name)
}
$classpath = "$serverRoot/work/rsprot/*"
$jdk = 'C:/Program Files/Eclipse Adoptium/jdk-21.0.12.101-hotspot/bin'
& "$jdk/javac.exe" '-J-Xmx256m' '-J-XX:ActiveProcessorCount=2' '-cp' $classpath '-d' $output "$serverRoot/reference/Js5Oracle.java" "$serverRoot/reference/Js5QueueOracle.java"
if ($LASTEXITCODE -ne 0) { throw 'JS5 queue oracle compilation failed.' }
& "$jdk/java.exe" '-Xmx128m' '-XX:ActiveProcessorCount=2' '-cp' "$output;$classpath" Js5QueueOracle "$serverRoot/crates/server/tests/fixtures/js5-queues.expected"
if ($LASTEXITCODE -ne 0) { throw 'JS5 queue oracle execution failed.' }
& "$serverRoot/build.ps1" -Action test -CargoArgs @('-p','gielinor-server','--test','rsprot_js5_queues')
